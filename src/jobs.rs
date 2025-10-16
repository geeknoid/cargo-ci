use anyhow::{Context, Result, anyhow};
use cargo_metadata::Package;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use toml::Value;

use crate::host::Host;
use crate::job_definition::JobDefinition;
use crate::job_result::JobResult;
use crate::modifier_expression;
use crate::package_config::PackageConfig;
use crate::step::Step;

/// Context for executing jobs, encapsulating all execution parameters.
struct JobExecutionContext<'a> {
    job: &'a JobDefinition,
    packages: &'a [&'a Package],
    package_configs: &'a HashMap<String, PackageConfig>,
    dry_run: bool,
    keep_going: bool,
    host: &'a Arc<dyn Host>,
}

/// Container for CI jobs with methods to load, list, and execute them.
pub struct Jobs {
    jobs: HashMap<String, JobDefinition>,
    host: Arc<dyn Host>,
}

impl Jobs {
    /// Load jobs from `ci.toml` if it exists, otherwise fall back to `Cargo.toml` manifest file.
    ///
    /// When loading from `ci.toml`, jobs are defined under `[jobs]` section.
    /// When loading from `Cargo.toml`, jobs are defined under `[package.metadata.ci.jobs]`
    /// or `[workspace.metadata.ci.jobs]` section.
    pub fn load(manifest_path: &Path, host: Arc<dyn Host>) -> Result<Self> {
        // First, try to load from ci.toml in the same directory as Cargo.toml
        let ci_toml_path = manifest_path
            .parent()
            .ok_or_else(|| anyhow!("Invalid manifest path"))?
            .join("ci.toml");

        let jobs = if ci_toml_path.exists() {
            Self::load_from_ci_toml(&ci_toml_path)
                .with_context(|| format!("Failed to load CI configuration from {}", ci_toml_path.display()))?
        } else {
            // Try Cargo.toml, but provide a helpful error message if it fails
            Self::load_from_cargo_toml(manifest_path).map_err(|e| {
                anyhow!(
                    "No CI configuration found. You can define jobs in either:\n\
                     1. A dedicated 'ci.toml' file with a [jobs] section, or\n\
                     2. Your Cargo.toml under [package.metadata.ci.jobs] or [workspace.metadata.ci.jobs]\n\n\
                     Original error: {e}"
                )
            })?
        };

        // Validate each job
        for (name, job) in &jobs {
            job.validate().with_context(|| format!("Invalid job definition: {name}"))?;
        }

        Ok(Self { jobs, host })
    }

    /// Load jobs from a dedicated ci.toml file with simplified table structure
    fn load_from_ci_toml(ci_toml_path: &Path) -> Result<HashMap<String, JobDefinition>> {
        let text = fs::read_to_string(ci_toml_path).with_context(|| format!("Reading ci.toml at {}", ci_toml_path.display()))?;
        let value: Value = text.parse::<Value>().context("Parsing ci.toml")?;

        let jobs_table = value.get("jobs").ok_or_else(|| anyhow!("No [jobs] section found in ci.toml"))?;

        let mut jobs: HashMap<String, JobDefinition> = jobs_table
            .clone()
            .try_into()
            .context("Failed to deserialize jobs configuration from ci.toml")?;

        // Set the id field for each job
        for (name, job) in &mut jobs {
            job.id.clone_from(name);
        }

        Ok(jobs)
    }

    /// Load jobs from Cargo.toml manifest file using the metadata.ci.jobs structure
    fn load_from_cargo_toml(manifest_path: &Path) -> Result<HashMap<String, JobDefinition>> {
        let text = fs::read_to_string(manifest_path).with_context(|| format!("Reading manifest {}", manifest_path.display()))?;
        let value: Value = text.parse::<Value>().context("Parsing Cargo.toml")?;

        let jobs_table = value
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("ci"))
            .and_then(|ci| ci.get("jobs"))
            .or_else(|| {
                let workspace = value.get("workspace")?;
                let metadata = workspace.get("metadata")?;
                let ci = metadata.get("ci")?;
                ci.get("jobs")
            })
            .ok_or_else(|| anyhow!("No [package.metadata.ci.jobs] or [workspace.metadata.ci.jobs] section found in Cargo.toml"))?;

        let mut jobs: HashMap<String, JobDefinition> = jobs_table.clone().try_into().context("Failed to deserialize jobs configuration")?;

        // Set the id field for each job
        for (name, job) in &mut jobs {
            job.id.clone_from(name);
        }

        Ok(jobs)
    }

    /// Get a reference to a specific job by name
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&JobDefinition> {
        self.jobs.get(name)
    }

    /// Returns true if there are no jobs defined.
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    /// Get an iterator over all jobs
    pub fn iter(&self) -> impl Iterator<Item = (&String, &JobDefinition)> {
        self.jobs.iter()
    }

    /// Print a list of all defined jobs
    pub fn print_list(&self) {
        self.host.println("Defined jobs:");
        for (name, job) in &self.jobs {
            self.host.println_fmt(format_args!("  {name}: {} step(s)", job.steps.len()));
            if !job.needs.is_empty() {
                self.host.println_fmt(format_args!("    needs: {}", job.needs.join(", ")));
            }
            for step in &job.steps {
                self.host.println_fmt(format_args!("    - {}", step.command()));
            }
        }
    }

    /// Validate job dependencies and detect cycles
    pub fn validate_dependencies(&self, job_names: &[String]) -> Result<()> {
        for job_name in job_names {
            self.validate_job_dependencies(job_name, &mut HashSet::new())?;
        }
        Ok(())
    }

    /// Recursively validate a single job's dependencies
    fn validate_job_dependencies(&self, job_name: &str, visited: &mut HashSet<String>) -> Result<()> {
        // Check if job exists
        let job = self.get(job_name).ok_or_else(|| anyhow!("Job '{job_name}' not found"))?;

        // Check for circular dependencies
        if visited.contains(job_name) {
            return Err(anyhow!("Circular dependency detected involving job '{job_name}'"));
        }

        let _ = visited.insert(job_name.to_string());

        // Validate each dependency
        for dep in &job.needs {
            if !self.jobs.contains_key(dep) {
                return Err(anyhow!("Job '{job_name}' depends on non-existent job '{dep}'"));
            }
            self.validate_job_dependencies(dep, visited)?;
        }

        let _ = visited.remove(job_name);
        Ok(())
    }

    /// Sort jobs in topological order respecting dependencies
    pub fn topological_sort(&self, job_names: &[String]) -> Result<Vec<String>> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut in_progress = HashSet::new();

        for job_name in job_names {
            if !visited.contains(job_name.as_str()) {
                self.topological_visit(job_name, &mut visited, &mut in_progress, &mut sorted)?;
            }
        }

        Ok(sorted)
    }

    /// Helper function for topological sort using depth-first search
    fn topological_visit(
        &self,
        job_name: &str,
        visited: &mut HashSet<String>,
        in_progress: &mut HashSet<String>,
        sorted: &mut Vec<String>,
    ) -> Result<()> {
        if in_progress.contains(job_name) {
            return Err(anyhow!("Circular dependency detected involving job '{job_name}'"));
        }

        if visited.contains(job_name) {
            return Ok(());
        }

        let job = self.get(job_name).ok_or_else(|| anyhow!("Job '{job_name}' not found"))?;

        let _ = in_progress.insert(job_name.to_string());

        // Visit all dependencies first
        for dep in &job.needs {
            self.topological_visit(dep, visited, in_progress, sorted)?;
        }

        let _ = in_progress.remove(job_name);
        let _ = visited.insert(job_name.to_string());
        sorted.push(job_name.to_string());

        Ok(())
    }

    /// Run a single job across all selected packages
    #[expect(clippy::too_many_arguments, reason = "Public API - refactored internally")]
    pub fn run_job(
        &self,
        job_name: &str,
        job: &JobDefinition,
        all_packages: &[&Package],
        package_configs: &HashMap<String, PackageConfig>,
        dry_run: bool,
        keep_going: bool,
        parallel: bool,
    ) -> Result<JobResult> {
        let selected: Vec<&Package> = all_packages.to_vec();

        let ctx = JobExecutionContext {
            job,
            packages: &selected,
            package_configs,
            dry_run,
            keep_going,
            host: &self.host,
        };

        let mut failures = 0usize;
        let mut total_steps_executed = 0usize;

        if parallel {
            run_job_parallel(&ctx, &mut failures, &mut total_steps_executed)?;
        } else {
            run_job_sequential(&ctx, &mut failures, &mut total_steps_executed)?;
        }

        Ok(JobResult::new(
            job_name.to_string(),
            selected.len(),
            total_steps_executed,
            failures,
            dry_run,
        ))
    }
}

#[expect(clippy::too_many_lines, reason = "Parallel execution logic is complex but well-structured")]
fn run_job_parallel(ctx: &JobExecutionContext<'_>, failures: &mut usize, total_steps_executed: &mut usize) -> Result<()> {
    use rayon::prelude::*;
    use std::sync::Mutex;

    let job_name = ctx.job.display_name();
    let steps = ctx.job.steps();

    ctx.host.println_fmt(format_args!(
        "[{job_name}] Running job in parallel across {} package(s)",
        ctx.packages.len()
    ));

    let failure_counter = AtomicUsize::new(0);
    let step_counter = AtomicUsize::new(0);
    let failures_list = Mutex::new(Vec::new());

    ctx.packages.par_iter().for_each(|pkg| {
        // Execute all steps for this package
        for (idx, step) in steps.iter().enumerate() {
            // Check if step should be executed based on modifiers
            if let Some(expr) = step.modifiers_expr() {
                let package_config = ctx.package_configs.get(&pkg.id.to_string());
                let modifiers = package_config.and_then(|p| p.modifiers()).unwrap_or(&[]);

                match modifier_expression::evaluate(expr, modifiers) {
                    Ok(true) => { /* continue execution */ }
                    Ok(false) => {
                        // Skip this step for this package
                        continue;
                    }
                    Err(e) => {
                        let _ = failure_counter.fetch_add(1, Ordering::SeqCst);
                        let error_msg = format!("Package '{}': Failed to evaluate modifiers expression '{}': {}", pkg.name, expr, e);
                        if let Ok(mut f) = failures_list.lock() {
                            f.push(error_msg);
                        }
                        return; // Stop processing this package
                    }
                }
            }

            let _ = step_counter.fetch_add(1, Ordering::SeqCst);

            if ctx.dry_run {
                continue;
            }

            let manifest_path = Path::new(&pkg.manifest_path);
            let pkg_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));

            // Use custom working directory if specified, otherwise use package directory
            let working_dir = step.working_directory().map_or(pkg_dir, Path::new);

            let env_vars = vec![("CI_JOB", ctx.job.id.as_str()), ("CI_PACKAGE_NAME", &*pkg.name)];

            match ctx.host.execute(step.command(), working_dir, &env_vars) {
                Ok(result) => {
                    if !result.success {
                        let _ = failure_counter.fetch_add(1, Ordering::SeqCst);
                        let error_msg = if step.continue_on_error() {
                            format!(
                                "Package '{}': Step {} '{}' failed with exit code {:?} (continuing due to continue_on_error)",
                                pkg.name,
                                idx + 1,
                                step.display_name(),
                                result.exit_code
                            )
                        } else {
                            format!(
                                "Package '{}': Step {} '{}' failed with exit code {:?}",
                                pkg.name,
                                idx + 1,
                                step.display_name(),
                                result.exit_code
                            )
                        };
                        if let Ok(mut f) = failures_list.lock() {
                            f.push(error_msg);
                        }
                        if !step.continue_on_error() && !ctx.keep_going {
                            return; // Stop processing this package
                        }
                    }
                }
                Err(e) => {
                    let _ = failure_counter.fetch_add(1, Ordering::SeqCst);
                    let error_msg = if step.continue_on_error() {
                        format!(
                            "Package '{}': Step {} '{}' error: {} (continuing due to continue_on_error)",
                            pkg.name,
                            idx + 1,
                            step.display_name(),
                            e
                        )
                    } else {
                        format!("Package '{}': Step {} '{}' error: {}", pkg.name, idx + 1, step.display_name(), e)
                    };
                    if let Ok(mut f) = failures_list.lock() {
                        f.push(error_msg);
                    }
                    if !step.continue_on_error() && !ctx.keep_going {
                        return; // Stop processing this package
                    }
                }
            }
        }
    });

    let failure_count = failure_counter.load(Ordering::SeqCst);
    let steps_executed = step_counter.load(Ordering::SeqCst);
    *total_steps_executed += steps_executed;
    *failures += failure_count;

    // Report all failures after parallel execution completes
    if failure_count > 0 {
        if let Ok(failure_list) = failures_list.lock() {
            for error in failure_list.iter() {
                ctx.host.eprintln_fmt(format_args!("  {error}"));
            }
        }

        if !ctx.keep_going {
            return Err(anyhow!("Job {job_name} failed in {failure_count} package(s)"));
        }
    }

    Ok(())
}

fn run_job_sequential(ctx: &JobExecutionContext<'_>, failures: &mut usize, total_steps_executed: &mut usize) -> Result<()> {
    let job_name = ctx.job.display_name();

    for pkg in ctx.packages {
        let steps = ctx.job.steps();

        if steps.is_empty() {
            ctx.host.println_fmt(format_args!("-- Package: {} --", pkg.name));
            ctx.host.println("(no steps)");
            continue;
        }

        ctx.host.println_fmt(format_args!("-- Package: {} --", pkg.name));

        for (idx, step) in steps.iter().enumerate() {
            // Check if step should be executed based on modifiers
            if let Some(expr) = step.modifiers_expr() {
                let package = ctx.package_configs.get(&pkg.id.to_string());
                let modifiers = package.and_then(|p| p.modifiers()).unwrap_or(&[]);

                match modifier_expression::evaluate(expr, modifiers) {
                    Ok(should_execute) => {
                        if !should_execute {
                            ctx.host.println_fmt(format_args!(
                                "[{job_name}] ({}/{}) {} (skipped due to modifiers)",
                                idx + 1,
                                steps.len(),
                                step.display_name()
                            ));
                            continue;
                        }
                    }
                    Err(e) => {
                        return Err(anyhow!("Failed to evaluate modifiers expression '{expr}' for step: {e}"));
                    }
                }
            }

            let step_name = step.display_name();
            ctx.host
                .println_fmt(format_args!("[{job_name}] ({}/{}) {}", idx + 1, steps.len(), step_name));
            *total_steps_executed += 1;
            if ctx.dry_run {
                continue;
            }

            if !run_step_single(&ctx.job.id, step, pkg, ctx.package_configs, ctx.host)? {
                *failures += 1;
                if step.continue_on_error() {
                    ctx.host
                        .eprintln_fmt(format_args!("Step failed but continuing due to continue_on_error"));
                } else {
                    ctx.host.eprintln("Step failed");
                    if !ctx.keep_going {
                        return Err(anyhow!("Job {job_name} failed on package {}", pkg.name));
                    }
                }
            }
        }
    }

    Ok(())
}

fn run_step_single(
    job_name: &str,
    step: &Step,
    package: &Package,
    package_configs: &HashMap<String, PackageConfig>,
    host: &Arc<dyn Host>,
) -> Result<bool> {
    // Check if step should be executed based on modifiers
    if let Some(expr) = step.modifiers_expr() {
        let package_config = package_configs.get(&package.id.to_string());
        let modifiers = package_config.and_then(|p| p.modifiers()).unwrap_or(&[]);

        let should_execute =
            modifier_expression::evaluate(expr, modifiers).with_context(|| format!("Failed to evaluate modifiers expression '{expr}'"))?;

        if !should_execute {
            // Step should be skipped
            return Ok(true);
        }
    }

    let manifest_path = Path::new(&package.manifest_path);
    let pkg_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));

    // Use custom working directory if specified, otherwise use package directory
    let working_dir = step.working_directory().map_or(pkg_dir, Path::new);

    let env_vars = vec![("CI_JOB", job_name), ("CI_PACKAGE_NAME", &*package.name)];

    match host.execute(step.command(), working_dir, &env_vars) {
        Ok(result) => Ok(result.success),
        Err(e) => {
            host.eprintln_fmt(format_args!("Error executing command: {e}"));
            Ok(false)
        }
    }
}
