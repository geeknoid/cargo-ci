use crate::color_modes::ColorModes;
use crate::config::{Config, Job, JobId, Step};
use crate::host::Host;
use crate::log::Log;
use crate::outputter::Outputter;
use crate::pkg_data::variables;
use anyhow::anyhow;
use cargo_metadata::{Metadata, Package};
use clap::ArgAction;
use clap::Parser;
use core::error::Error;
use core::str::FromStr;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Parser, Debug, Default, Clone)]
pub struct RunArgs {
    /// Names of the jobs to run
    jobs: Vec<String>,

    /// Show steps to execute without executing them
    #[arg(short = 'n', long, action = ArgAction::SetTrue)]
    dry_run: bool,

    /// Package to run jobs on (see `cargo help pkgid`)
    #[arg(short = 'p', long, value_name = "SPEC")]
    package: Vec<String>,

    /// Define a variable.
    #[arg(short = 'v', long, value_parser = parse_key_val::<String, String>, value_name = "VAR=VALUE")]
    variable: Vec<(String, String)>,

    /// Send log output to the specified file.
    #[arg(short = 'l', long, value_name = "FILE")]
    log_file: Option<PathBuf>,

    /// Number of log files to retain (default: 16).
    #[arg(long, default_value_t = 16, value_name = "COUNT")]
    log_file_retention_count: usize,

    /// Colorize output.
    #[arg(long, value_name = "WHEN", default_value_t = ColorModes::Auto, value_enum)]
    color: ColorModes,
}

impl RunArgs {
    /// Returns an iterator over the variables defined in the command line arguments.
    pub fn variables(&self) -> impl Iterator<Item = (&str, &str)> {
        self.variable.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

/// Parse a single key-value pair
#[expect(clippy::string_slice, reason = "Necessary for parsing KEY=VALUE")]
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s.find('=').ok_or_else(|| format!("invalid KEY=value: no '=' found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

pub fn run_jobs<H: Host>(args: &RunArgs, host: &mut H, cfg: &Config, metadata: &Metadata) -> anyhow::Result<()> {
    let jobs = select_jobs(args, cfg)?;
    let packages = select_packages(args, metadata)?;

    let mut env_vars = HashMap::new();
    for (key, value) in host.vars() {
        if cfg!(windows) {
            if cfg.passthrough_env_variables().iter().any(|v| v.eq_ignore_ascii_case(&key)) {
                _ = env_vars.insert(key, value);
            }
        } else if cfg.passthrough_env_variables().contains(&key) {
            _ = env_vars.insert(key, value);
        }
    }

    let log_prefix = if args.dry_run { "dry-run" } else { "run" };
    let log = Log::new(
        metadata.target_directory.as_std_path(),
        log_prefix,
        args.log_file.as_deref(),
        args.log_file_retention_count,
    )?;

    // after this point, thia code takes care of error reporting itself
    host.fail_silently();

    let outputter = Outputter::new(host, &log, args.color);

    let env_vars = || env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str()));

    for job_id in jobs {
        let job = cfg.jobs().get_job(job_id).expect("job not found");
        let job_name = job.name().unwrap_or(job_id.as_str());

        outputter.start_activity(job_name);

        // we evaluate that up here even when there is no error, so that the expression gets validated eagerly
        let continue_on_error = job
            .continue_on_error()
            .evaluate(env_vars().chain(cfg.variables()).chain(args.variables()))?;

        let result = run_job(args, host, metadata, &packages, &env_vars, &outputter, cfg, job);

        if result.is_ok() {
            outputter.complete_activity(format!("ran {0} step(s)", job.steps().len()));
        } else if continue_on_error {
            outputter.complete_activity("failed, but ignored");
        } else {
            outputter.complete_activity("failed");
            return result;
        }
    }

    Ok(())
}

#[expect(clippy::too_many_lines, reason = "Necessary for job execution")]
#[expect(clippy::too_many_arguments, reason = "Necessary for job execution")]
fn run_job<'a, H: Host, F, I>(
    args: &'a RunArgs,
    host: &H,
    metadata: &Metadata,
    packages: &'a [&Package],
    env_vars: &'a F,
    outputter: &Outputter<H>,
    cfg: &'a Config,
    job: &'a Job,
) -> anyhow::Result<()>
where
    F: Fn() -> I,
    I: Iterator<Item = (&'a str, &'a str)> + Clone,
{
    for step in job.steps() {
        let mut packages_to_process = HashSet::new();
        for pkg in packages {
            if !job
                .conditional()
                .evaluate(env_vars().chain(cfg.variables()).chain(variables(pkg)).chain(args.variables()))?
            {
                outputter.message(format!("Package '{}' skipped due to job-level condition", pkg.name));
                continue;
            }

            if !step.conditional().evaluate(
                env_vars()
                    .chain(cfg.variables())
                    .chain(job.variables())
                    .chain(variables(pkg))
                    .chain(args.variables()),
            )? {
                outputter.message(format!("Package '{}' skipped due to step-level condition", pkg.name));
                continue;
            }

            _ = packages_to_process.insert(pkg);
        }

        if packages_to_process.len() != packages.len() || step.per_package() {
            for pkg in packages_to_process {
                // we evaluate that up here even when there is no error, so that the expression gets validated eagerly
                let continue_on_error = if step.per_package() {
                    step.continue_on_error().evaluate(
                        env_vars()
                            .chain(cfg.variables())
                            .chain(job.variables())
                            .chain(variables(pkg))
                            .chain(args.variables()),
                    )?
                } else {
                    step.continue_on_error()
                        .evaluate(env_vars().chain(cfg.variables()).chain(job.variables()).chain(args.variables()))?
                };

                outputter.message(format!("step '{}' for package '{}'", step.name(), pkg.name));

                if args.dry_run {
                    continue;
                }

                let mut cmd = if step.per_package() {
                    make_command(
                        step,
                        pkg.manifest_path.parent().expect("should have a valid parent").as_std_path(),
                        env_vars()
                            .chain(cfg.variables())
                            .chain(job.variables())
                            .chain(variables(pkg))
                            .chain(step.variables())
                            .chain(args.variables()),
                    )
                } else {
                    make_command(
                        step,
                        pkg.manifest_path.parent().expect("should have a valid parent").as_std_path(),
                        env_vars()
                            .chain(cfg.variables())
                            .chain(job.variables())
                            .chain(step.variables())
                            .chain(args.variables()),
                    )
                };

                outputter.run_command(&cmd);

                let e = match host.spawn(&mut cmd) {
                    Ok(child) => match child.wait_with_output() {
                        Ok(output) => {
                            if output.status.success() {
                                Ok(())
                            } else {
                                outputter.command_error("unable to run step", Some(output.status), Some(&output), !continue_on_error);
                                Err(anyhow::anyhow!(format!(
                                    "unable to run step '{}' for package '{}': {}",
                                    step.name(),
                                    pkg.name,
                                    output.status
                                )))
                            }
                        }

                        Err(e) => {
                            outputter.command_error(format!("unable to wait for step: {e}"), None, None, !continue_on_error);
                            Err(anyhow::anyhow!(format!(
                                "unable to wait for step '{}' for package '{}': {e}",
                                step.name(),
                                pkg.name
                            )))
                        }
                    },

                    Err(e) => {
                        outputter.command_error(format!("unable to start step: {e}"), None, None, !continue_on_error);
                        Err(anyhow::anyhow!(format!(
                            "unable to start step '{}' for package '{}': {e}",
                            step.name(),
                            pkg.name
                        )))
                    }
                };

                if e.is_ok() || continue_on_error {
                    continue;
                }

                e?;
            }
        } else {
            // we evaluate that up here even when there is no error, so that the expression gets validated eagerly
            let continue_on_error = step
                .continue_on_error()
                .evaluate(env_vars().chain(cfg.variables()).chain(job.variables()).chain(args.variables()))?;

            outputter.message(format!("step '{}'", step.name()));

            if args.dry_run {
                continue;
            }

            let mut cmd = make_command(
                step,
                metadata.workspace_root.as_std_path(),
                env_vars()
                    .chain(cfg.variables())
                    .chain(job.variables())
                    .chain(step.variables())
                    .chain(args.variables()),
            );
            outputter.run_command(&cmd);

            let e = match host.spawn(&mut cmd) {
                Ok(child) => match child.wait_with_output() {
                    Ok(output) => {
                        if output.status.success() {
                            Ok(())
                        } else {
                            outputter.command_error("unable to run step", Some(output.status), Some(&output), !continue_on_error);
                            Err(anyhow::anyhow!(format!("unable to run step '{}': {}", step.name(), output.status)))
                        }
                    }

                    Err(e) => {
                        outputter.command_error(format!("unable to wait for step: {e}"), None, None, !continue_on_error);
                        Err(anyhow::anyhow!(format!("unable to wait for step '{}': {e}", step.name())))
                    }
                },

                Err(e) => {
                    outputter.command_error(format!("unable to start step: {e}"), None, None, !continue_on_error);
                    Err(anyhow::anyhow!(format!("unable to start step '{}': {e}", step.name())))
                }
            };

            if e.is_ok() || continue_on_error {
                continue;
            }

            e?;
        }
    }

    Ok(())
}

fn make_command<'a>(step: &Step, directory: &Path, _variables: impl Iterator<Item = (&'a str, &'a str)>) -> Command {
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        _ = c.arg("/C").arg(step.command());
        c
    } else {
        let mut c = Command::new("sh");
        _ = c.arg("-c").arg(step.command());
        c
    };

    // TODO: figure out what to do with environment variables
    _ = cmd.current_dir(directory); // .env_clear().envs(variables);
    _ = cmd.stdout(Stdio::piped());
    _ = cmd.stderr(Stdio::piped());

    cmd
}

fn select_jobs<'a>(args: &RunArgs, cfg: &'a Config) -> anyhow::Result<Vec<&'a JobId>> {
    if cfg.jobs().is_empty() {
        return Err(anyhow!("no jobs are defined in configuration"));
    }

    let mut jobs_to_run = if args.jobs.is_empty() {
        if cfg.default_jobs().is_empty() {
            cfg.jobs().iter().map(|(job_id, _)| job_id).collect()
        } else {
            cfg.default_jobs().iter().collect()
        }
    } else {
        let mut unknown_jobs = Vec::new();
        let mut jobs_to_run = HashSet::new();

        for job_name in &args.jobs {
            if let Some((job_id, _)) = cfg.jobs().iter().find(|(id, _)| id.to_string() == *job_name) {
                _ = jobs_to_run.insert(job_id);
            } else {
                unknown_jobs.push(job_name.as_str());
            }
        }

        if !unknown_jobs.is_empty() {
            return Err(anyhow!("invalid jobs specified: {}", unknown_jobs.join(", ")));
        }

        jobs_to_run
    };

    if jobs_to_run.is_empty() {
        return Err(anyhow!("no jobs to run"));
    }

    let mut needs = HashSet::new();
    for job_id in &jobs_to_run {
        let extras = cfg.jobs().get_transitive_needs(job_id);
        for extra in extras {
            _ = needs.insert(extra);
        }
    }

    for need in needs {
        _ = jobs_to_run.insert(need);
    }

    Ok(cfg.jobs().topological_sort(&jobs_to_run))
}

fn select_packages<'a>(args: &RunArgs, metadata: &'a Metadata) -> anyhow::Result<Vec<&'a Package>> {
    let mut result = Vec::new();

    if args.package.is_empty() {
        for pkg_id in metadata.workspace_default_members.iter() {
            result.push(&metadata[pkg_id]);
        }
    } else {
        for pkg_name in &args.package {
            let mut found = false;
            for pkg in &metadata.packages {
                if pkg.name == *pkg_name {
                    if !metadata.workspace_members.contains(&pkg.id) {
                        return Err(anyhow!("package '{pkg_name}' is not a member of the workspace"));
                    }

                    result.push(pkg);
                    found = true;
                    break;
                }
            }

            if !found {
                return Err(anyhow!("package '{pkg_name}' is not a member of the workspace"));
            }
        }
    }

    Ok(result)
}
