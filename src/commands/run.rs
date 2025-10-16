use crate::host::Host;
use crate::log::Log;
use crate::pkg_config::get_variables;
use crate::ws_config::{Job, JobId, Step, WorkspaceConfig};
use anyhow::anyhow;
use cargo_metadata::{Metadata, Package};
use clap::ArgAction;
use clap::Parser;
use console::{Term, style};
use core::error::Error;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::str::FromStr;

#[derive(Parser, Debug, Default, Clone)]
pub struct RunArgs {
    /// Names of the jobs to run
    pub jobs: Vec<String>,

    /// Show steps to execute without executing them
    #[arg(short = 'n', long, action = ArgAction::SetTrue)]
    pub dry_run: bool,

    /// Package) to run jobs on (see `cargo help pkgid`)
    #[arg(short = 'p', long, value_name = "SPEC")]
    pub package: Vec<String>,

    /// Define a variable using the syntax VAR=VAL.
    #[arg(short = 'v', long, value_parser = parse_key_val::<String, String>)]
    pub variable: Vec<(String, String)>,

    /// Send log output to the specified file.
    #[arg(short = 'l', long, value_name = "FILE")]
    pub log_file: Option<PathBuf>,

    /// Number of log files to retain (default: 16).
    #[arg(long, default_value_t = 16, value_name = "COUNT")]
    pub log_file_retention_count: usize,
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
    let pos = s.find('=').ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

pub fn run_jobs<H: Host>(args: &RunArgs, host: &mut H, ws_config: &WorkspaceConfig, metadata: &Metadata) -> anyhow::Result<()> {
    let jobs = select_jobs(args, ws_config)?;
    let packages = select_packages(args, metadata)?;

    // after this point, thia code takes care of error reporting itself
    host.fail_silently();

    let mut global_vars = HashMap::new();
    for (key, value) in host.vars() {
        if cfg!(windows) {
            if let Some(ws_key) = ws_config.variables().iter().find(|v| v.eq_ignore_ascii_case(&key)) {
                _ = global_vars.insert(ws_key.clone(), value);
            }
        } else if ws_config.variables().contains(&key) {
            _ = global_vars.insert(key, value);
        }
    }

    for (key, value) in &args.variable {
        _ = global_vars.insert(key.clone(), value.clone());
    }

    let log_prefix = if args.dry_run { "dry-run" } else { "run" };
    let mut log = Log::new(
        ws_config.directory(),
        log_prefix,
        Option::from(&args.log_file),
        args.log_file_retention_count,
    )?;
    let term = Term::stdout();

    for job_id in jobs {
        let job = ws_config
            .jobs()
            .get_job(job_id)
            .ok_or_else(|| anyhow::anyhow!("job '{job_id}' not found"))?;

        let job_name = job.name().unwrap_or(job_id.as_str());

        log.info(format!("JOB: `{job_name}`"));
        term.write_str(&format!("{job_name}:"))?;

        let continue_on_error = if let Some(coe) = job.continue_on_error() {
            coe.evaluate(&global_vars)?
        } else {
            false
        };

        let result = run_job(args, host, ws_config, &packages, &global_vars, &mut log, &term, job_name, job);

        term.clear_line()?;
        if result.is_ok() {
            term.write_line(&format!("{job_name}: Done"))?;
        } else if continue_on_error {
            term.write_line(&format!("{job_name}: {}", style("Failed (ignored)").yellow()))?;
        } else {
            return result;
        }
    }

    Ok(())
}

#[expect(clippy::too_many_arguments, reason = "Necessary for job execution")]
fn run_job<H: Host>(
    args: &RunArgs,
    host: &mut H,
    ws_config: &WorkspaceConfig,
    packages: &[&Package],
    global_vars: &HashMap<String, String>,
    log: &mut Log,
    term: &Term,
    job_name: &str,
    job: &Job,
) -> anyhow::Result<()> {
    for step in job.steps() {
        log.info(format!("  STEP: `{}`", step.name()));

        let mut packages_to_process = HashSet::new();
        for pkg in packages {
            let mut vars = global_vars.clone();
            if let Some(pkg_vars) = get_variables(pkg) {
                for (key, value) in pkg_vars {
                    _ = vars.insert(key.clone(), value.clone());
                }
            }

            if let Some(cond) = job.cond()
                && !cond.evaluate(&vars)?
            {
                log.info(format!("    PACKAGE: `{}`, skipped due to job-level condition match", pkg.name));
                continue;
            }

            if let Some(cond) = &step.cond()
                && !cond.evaluate(&vars)?
            {
                log.info(format!("    PACKAGE: `{}`, skipped due to step-level condition match", pkg.name));
                continue;
            }

            _ = packages_to_process.insert(pkg);
        }

        if packages_to_process.len() != packages.len() || step.per_package() {
            for pkg in packages_to_process {
                let mut vars = global_vars.clone();
                if step.per_package()
                    && let Some(pkg_vars) = get_variables(pkg)
                {
                    for (key, value) in &pkg_vars {
                        _ = vars.insert(key.clone(), value.clone());
                    }
                }

                let continue_on_error = if let Some(coe) = step.continue_on_error() {
                    coe.evaluate(&vars)?
                } else {
                    false
                };

                term.clear_line()?;

                log.info(format!("    PACKAGE: `{}`", pkg.name));
                term.write_str(&format!("{job_name}: {}, for '{}'", step.name(), pkg.name))?;

                if args.dry_run {
                    continue;
                }

                let output = run_step(
                    host,
                    step,
                    pkg.manifest_path.parent().expect("Should have a valid parent").as_std_path(),
                    &vars,
                );

                let msg = match &output {
                    Ok(output) => {
                        if output.status.success() {
                            None // process succeeded
                        } else {
                            Some(format!("Step `{}` in package {} failed: {}", step.name(), pkg.name, &output.status))
                        }
                    }

                    Err(e) => Some(format!("Unable to start step `{}` in package {}: {e}", step.name(), pkg.name,)),
                };

                handle_failure(host, log, term, continue_on_error, &output, msg, job_name)?;
            }
        } else {
            let continue_on_error = if let Some(coe) = step.continue_on_error() {
                coe.evaluate(global_vars)?
            } else {
                false
            };

            term.clear_line()?;
            term.write_str(&format!("{job_name}: {}", step.name()))?;

            if args.dry_run {
                continue;
            }

            let output = run_step(host, step, ws_config.directory(), global_vars);

            let msg = match &output {
                Ok(output) => {
                    if output.status.success() {
                        None // process succeeded
                    } else {
                        Some(format!("Step `{}` failed: {}", step.name(), &output.status))
                    }
                }

                Err(e) => Some(format!("Unable to start step `{}`: {e}", step.name())),
            };

            handle_failure(host, log, term, continue_on_error, &output, msg, job_name)?;
        }
    }

    Ok(())
}

fn run_step<H: Host>(host: &H, step: &Step, directory: &Path, _variables: &HashMap<String, String>) -> std::io::Result<Output> {
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

    host.spawn(&mut cmd)?.wait_with_output()
}

fn handle_failure<H: Host>(
    host: &mut H,
    log: &mut Log,
    term: &Term,
    continue_on_error: bool,
    output: &std::io::Result<Output>,
    msg: Option<String>,
    job_name: &str,
) -> anyhow::Result<()> {
    if let Some(msg) = msg {
        if continue_on_error {
            term.write_line(&format!("...{}", style("Failed (ignored)").yellow()))?;
            host.println(&msg);
            log.warn(&msg);
        } else {
            term.clear_line()?;
            term.write_line(&format!("{}: {}", job_name, style("Failed").red()))?;
            host.eprintln(&msg);
            log.error(&msg);
        }

        host.println("---");

        if let Ok(output) = &output {
            log_output(host, log, output, continue_on_error);
        }

        host.println("---");

        if continue_on_error { Ok(()) } else { Err(anyhow::anyhow!(msg)) }
    } else {
        Ok(())
    }
}

type LogFunction = Box<dyn Fn(&mut Log, &str)>;
type PrintFunction<H> = Box<dyn Fn(&mut H, &str)>;

fn log_output<H: Host>(host: &mut H, log: &mut Log, output: &Output, continue_on_error: bool) {
    let (log_fn, print_fn): (LogFunction, PrintFunction<H>) = if continue_on_error {
        (Box::new(|l, s| l.warn(s)), Box::new(|h, s| h.println(s)))
    } else {
        (Box::new(|l, s| l.error(s)), Box::new(|h, s| h.eprintln(s)))
    };

    let has_stdout = !output.stdout.is_empty();
    let has_stderr = !output.stderr.is_empty();

    if has_stdout {
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let styled_stdout = style(stdout_str.as_ref()).italic().to_string();
        if has_stderr {
            print_fn(host, &format!("stdout:\n{styled_stdout}"));
            log_fn(log, &format!("stdout:\n{stdout_str}"));
        } else {
            print_fn(host, &styled_stdout);
            log_fn(log, &stdout_str);
        }
    }

    if has_stderr {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        let styled_stderr = style(stderr_str.as_ref()).italic().to_string();
        if has_stdout {
            print_fn(host, &format!("stderr:\n{styled_stderr}"));
            log_fn(log, &format!("stderr:\n{stderr_str}"));
        } else {
            print_fn(host, &styled_stderr);
            log_fn(log, &stderr_str);
        }
    }
}

fn select_jobs<'a>(args: &RunArgs, ws_config: &'a WorkspaceConfig) -> anyhow::Result<Vec<&'a JobId>> {
    if ws_config.jobs().is_empty() {
        return Err(anyhow!("no jobs are defined in configuration"));
    }

    let mut jobs_to_run = if args.jobs.is_empty() {
        if let Some(default_jobs) = ws_config.default_jobs() {
            if default_jobs.is_empty() {
                return Err(anyhow!("no default jobs are defined"));
            }

            default_jobs.iter().collect()
        } else {
            ws_config.jobs().iter().map(|(job_id, _)| job_id).collect()
        }
    } else {
        let mut unknown_jobs = Vec::new();
        let mut jobs_to_run = HashSet::new();

        for job_name in &args.jobs {
            if let Some((job_id, _)) = ws_config.jobs().iter().find(|(id, _)| id.to_string() == *job_name) {
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
        let extras = ws_config.jobs().get_transitive_needs(job_id);
        for extra in extras {
            _ = needs.insert(extra);
        }
    }

    for need in needs {
        _ = jobs_to_run.insert(need);
    }

    Ok(ws_config.jobs().topological_sort(&jobs_to_run))
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
