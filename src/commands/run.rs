use crate::host::Host;
use crate::log::Log;
use crate::pkg_config::get_variables;
use crate::ws_config::{Job, JobId, Step, WorkspaceConfig};
use cargo_metadata::Package;
use clap::ArgAction;
use clap::Parser;
use console::Term;
use core::error::Error;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
pub struct RunArgs {
    /// Names of the jobs to run
    pub jobs: Vec<String>,

    /// Run all defined jobs
    #[arg(short = 'a', long, action = ArgAction::SetTrue, conflicts_with = "jobs")]
    pub all_jobs: bool,

    /// Show steps to execute without executing them
    #[arg(short = 'n', long, action = ArgAction::SetTrue)]
    pub dry_run: bool,

    /// Continue running remaining jobs even if a step fails
    #[arg(short = 'k', long, action = ArgAction::SetTrue)]
    pub keep_going: bool,

    /// Package(s) to run jobs on (can be specified multiple times)
    #[arg(short = 'p', long = "package", value_name = "SPEC")]
    pub package: Vec<String>,

    /// Define a variable using the syntax VAR=VAL. Can be used multiple times.
    #[arg(short = 'v', value_parser = parse_key_val::<String, String>)]
    pub variable: Vec<(String, String)>,

    /// Send log output to the specified file.
    #[arg(short = 'l', long, value_name = "FILE")]
    pub log_file: Option<PathBuf>,
}

/// Parse a single key-value pair
#[expect(clippy::string_slice, reason = "Necessary for parsing KEY=VALUE")]
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: core::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: core::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s.find('=').ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

pub fn run_jobs<H: Host>(
    args: &RunArgs,
    host: &mut H,
    ws_config: &WorkspaceConfig,
    jobs: &[&JobId],
    packages: &[&Package],
) -> anyhow::Result<()> {
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
    let mut log = Log::new(ws_config.directory(), log_prefix, Option::from(&args.log_file))?;
    let term = Term::stdout();

    for (job_idx, job_id) in jobs.iter().enumerate() {
        let job = ws_config
            .jobs()
            .get_job(job_id)
            .ok_or_else(|| anyhow::anyhow!("Job {job_id:?} not found"))?;

        log.info(&format!("JOB: `{}", job.name().unwrap_or(job_id.as_str())));

        let continue_on_errors = if let Some(coe) = job.continue_on_errors() {
            coe.evaluate(&global_vars)?
        } else {
            false
        };

        let result = run_job(args, host, ws_config, &jobs, packages, &global_vars, &mut log, &term, job_idx, job);
        if result.is_err() && !continue_on_errors {
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
    jobs: &&[&JobId],
    packages: &[&Package],
    global_vars: &HashMap<String, String>,
    log: &mut Log,
    term: &Term,
    job_idx: usize,
    job: &Job,
) -> anyhow::Result<()> {
    for (step_idx, step) in job.steps().iter().enumerate() {
        term.write_line(&format!(
            "JOB {}/{}, STEP {}/{}",
            job_idx + 1,
            jobs.len(),
            step_idx + 1,
            job.steps().len()
        ))?;

        log.info(&format!("  STEP: `{}", step.name()));

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
                log.info(&format!("  Skipping package `{}` due to job-level condition match", pkg.name));
                continue;
            }

            if let Some(cond) = &step.cond()
                && !cond.evaluate(&vars)?
            {
                log.info(&format!("  Skipping package `{}` due to step-level condition match", pkg.name));
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
                    for (key, value) in pkg_vars {
                        _ = vars.insert(key.clone(), value.clone());
                    }
                }

                let continue_on_errors = if let Some(coe) = step.continue_on_errors() {
                    coe.evaluate(&vars)?
                } else {
                    false
                };

                log.info(&format!("    PACKAGE: `{}`", pkg.name));
                run_step(
                    host,
                    step,
                    pkg.manifest_path.parent().expect("Should have a valid parent").as_std_path(),
                    &vars,
                    log,
                    args.dry_run,
                    continue_on_errors,
                )?;
            }
        } else {
            let continue_on_errors = if let Some(coe) = step.continue_on_errors() {
                coe.evaluate(global_vars)?
            } else {
                false
            };

            log.info("    WORKSPACE");
            run_step(
                host,
                step,
                ws_config.directory(),
                global_vars,
                log,
                args.dry_run,
                continue_on_errors,
            )?;
        }
        term.clear_line()?;
    }
    Ok(())
}

fn run_step<H: Host>(
    host: &mut H,
    step: &Step,
    directory: &Path,
    _variables: &HashMap<String, String>,
    log: &mut Log,
    dry_run: bool,
    continue_on_errors: bool,
) -> anyhow::Result<()> {
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

    let message = format!(
        "Running step '{}' in directory '{}': '{}'",
        step.name(),
        directory.display(),
        step.command()
    );

    host.println(message.clone());

    if dry_run {
        log.info(&format!("    [DRY-RUN] Would execute: {message}"));
        return Ok(());
    }

    let output = host.spawn(&mut cmd)?.wait_with_output()?;

    if !output.status.success() {
        log.error(&format!("    Step failed with status code {}", output.status.code().unwrap_or(-1)));
        log.error(&format!("    stdout: {}", String::from_utf8_lossy(&output.stdout)));
        log.error(&format!("    stderr: {}", String::from_utf8_lossy(&output.stderr)));

        let message = format!("Failed to run step {:?} with status {:?}", step.name(), output.status.code());

        if !continue_on_errors {
            return Err(anyhow::anyhow!(message));
        }
    }

    Ok(())
}
