//! Simulate CI jobs locally by running configured commands across crates in a Cargo workspace.
//!
//! # Installation
//!
//! ```sh
//! cargo install cargo-ci
//! ```
//!
//! Then invoke via Cargo's plugin mechanism:
//!
//! ```sh
//! cargo ci --list
//! ```
//!
//! # Configuration
//!
//! Jobs can be defined in two ways:
//!
//! ## Option 1: Using `ci.toml` (Recommended)
//!
//! Create a `ci.toml` file at the root of your workspace with a simplified structure:
//!
//! ```toml
//! [jobs.build]
//! steps = ["cargo build --all --all-targets"]
//!
//! [jobs.test]
//! needs = ["build"]
//! steps = ["cargo test --all --all-features"]
//!
//! [jobs.deploy]
//! needs = ["test"]
//! steps = ["./deploy.sh"]
//!
//! [jobs.lints]
//! steps = [
//!   "cargo fmt -- --check",
//!   "cargo clippy --all -- -D warnings"
//! ]
//! ```
//!
//! ## Option 2: Using `Cargo.toml` metadata
//!
//! Alternatively, jobs can be defined in `Cargo.toml` under either `[package.metadata.ci.jobs]`
//! (single crate) or `[workspace.metadata.ci.jobs]` (workspace root):
//!
//! ```toml
//! [workspace.metadata.ci.jobs.build]
//! steps = ["cargo build --all --all-targets"]
//!
//! [workspace.metadata.ci.jobs.test]
//! needs = ["build"]
//! steps = ["cargo test --all --all-features"]
//!
//! [workspace.metadata.ci.jobs.lints]
//! steps = [
//!   "cargo fmt -- --check",
//!   "cargo clippy --all -- -D warnings"
//! ]
//! ```
//!
//! **Note**: If `ci.toml` exists, it will be used instead of the `Cargo.toml` configuration.
//! Package-level settings (like modifiers) must always be defined in a package's `Cargo.toml`.
//!
//! # Job Dependencies
//!
//! Jobs can declare dependencies on other jobs using the `needs` field. When you run a job that
//! has dependencies, cargo-ci automatically executes all required jobs first in the correct order.
//!
//! ## Basic Usage
//!
//! ```toml
//! [jobs.build]
//! steps = ["cargo build"]
//!
//! [jobs.test]
//! needs = ["build"]
//! steps = ["cargo test"]
//!
//! [jobs.deploy]
//! needs = ["test"]
//! steps = ["./deploy.sh"]
//! ```
//!
//! Running `cargo ci deploy` will execute: **build** → **test** → **deploy**
//!
//! Running `cargo ci test` will execute: **build** → **test** (deploy is not included)
//!
//! ## Multiple Dependencies
//!
//! A job can depend on multiple other jobs:
//!
//! ```toml
//! [jobs.build]
//! steps = ["cargo build"]
//!
//! [jobs.lint]
//! steps = ["cargo clippy"]
//!
//! [jobs.docs]
//! steps = ["cargo doc"]
//!
//! [jobs.verify]
//! needs = ["build", "lint", "docs"]
//! steps = ["echo 'All checks passed'"]
//! ```
//!
//! Running `cargo ci verify` ensures build, lint, and docs all complete successfully first.
//!
//! ## Transitive Dependencies
//!
//! Dependencies are resolved recursively. If job A needs B, and B needs C, running A
//! will automatically execute C first, then B, then A:
//!
//! ```toml
//! [jobs.compile]
//! steps = ["cargo build"]
//!
//! [jobs.test]
//! needs = ["compile"]
//! steps = ["cargo test"]
//!
//! [jobs.integration]
//! needs = ["test"]
//! steps = ["cargo test --test integration"]
//! ```
//!
//! Running `cargo ci integration` executes: **compile** → **test** → **integration**
//!
//! ## Complex Dependency Graphs
//!
//! You can create sophisticated dependency graphs with multiple branches:
//!
//! ```toml
//! [jobs.build]
//! steps = ["cargo build"]
//!
//! [jobs.unit_test]
//! needs = ["build"]
//! steps = ["cargo test --lib"]
//!
//! [jobs.integration_test]
//! needs = ["build"]
//! steps = ["cargo test --test '*'"]
//!
//! [jobs.lint]
//! steps = ["cargo clippy"]
//!
//! [jobs.format_check]
//! steps = ["cargo fmt -- --check"]
//!
//! [jobs.full_ci]
//! needs = ["unit_test", "integration_test", "lint", "format_check"]
//! steps = ["echo 'All CI checks passed'"]
//! ```
//!
//! Running `cargo ci full_ci` will execute all dependencies in the correct order, respecting
//! the constraint that both `unit_test` and `integration_test` require build to complete first.
//!
//! ## Automatic Dependency Resolution
//!
//! - **Topological ordering**: Jobs are automatically sorted to respect all dependencies
//! - **Recursive resolution**: Transitive dependencies are included automatically
//! - **Cycle detection**: Circular dependencies are detected and reported as errors
//! - **Validation**: Missing dependencies (non-existent jobs) cause immediate errors
//! - **Failure propagation**: If a dependency fails, dependent jobs won't run (unless `--keep-going`)
//!
//! ## Viewing Dependencies
//!
//! Use `--list` to see job dependencies:
//!
//! ```sh
//! cargo ci --list
//! ```
//!
//! Output will show:
//! ```text
//! Defined jobs:
//!   build: 1 step(s)
//!     - cargo build
//!   test: 1 step(s)
//!     needs: build
//!     - cargo test
//!   deploy: 1 step(s)
//!     needs: test
//!     - ./deploy.sh
//! ```
//!
//! ## Running All Jobs with Dependencies
//!
//! When using `--all-jobs`, all jobs are executed in topologically sorted order:
//!
//! ```sh
//! cargo ci --all-jobs
//! ```
//!
//! This ensures every job runs only after all its dependencies have completed successfully.
//!
//! # Usage
//!
//! List defined jobs:
//!
//! ```sh
//! cargo ci --list
//! ```
//!
//! Run a single job:
//!
//! ```sh
//! cargo ci build
//! ```
//!
//! Run all jobs sequentially (alphabetical order):
//!
//! ```sh
//! cargo ci --all-jobs
//! ```
//!
//! Dry-run (show what would execute without running steps):
//!
//! ```sh
//! cargo ci build --dry-run
//! ```
//!
//! Keep going on failures (continue other commands / packages):
//!
//! ```sh
//! cargo ci build --keep-going
//! ```
//!
//! Run commands sequentially across packages (instead of parallel):
//!
//! ```sh
//! cargo ci build --sequential
//! ```
//!
//! # Workspace Behavior
//!
//! In a workspace with multiple member crates, each command in a job is executed once per member crate (current working directory set to that crate's directory). In a single-crate project, commands run only once in the root.
//!
//! # Custom Display Names
//!
//! By default, cargo-ci displays the full command line in status updates. For long or complex commands, you can provide a custom display name at the job level or step level:
//!
//! ## Job-level custom name
//!
//! ```toml
//! [workspace.metadata.ci.jobs.complex_build]
//! name = "Building with custom flags"
//! steps = ["cargo build --release --all-features --target x86_64-unknown-linux-gnu"]
//! ```
//!
//! When running this job, instead of showing the entire command, it will display:
//! ```text
//! [complex_build] (1/1) Building with custom flags
//! ```
//!
//! ## Step-level custom name
//!
//! For finer control, you can specify a custom name for individual steps using the extended step format:
//!
//! ```toml
//! [workspace.metadata.ci.jobs.multi_step]
//! steps = [
//!   { command = "cargo build --release", name = "Release Build" },
//!   { command = "cargo test --all-features", name = "Full Test Suite" },
//!   "cargo clippy"  # Simple format still supported
//! ]
//! ```
//!
//! This makes the output cleaner and more readable, especially for jobs with very long command lines or when using shell scripts with many arguments.
//!
//! # Modifiers
//!
//! Modifiers allow you to conditionally execute steps based on package-specific modifiers. This is useful for running different steps depending on the build environment, feature set, or development phase.
//!
//! ## Defining Package Modifiers
//!
//! Define modifiers in the `[package.metadata.ci]` or `[workspace.metadata.ci]` section:
//!
//! ```toml
//! [package.metadata.ci]
//! modifiers = ["nightly", "experimental"]
//! ```
//!
//! ## Using Modifiers in Steps
//!
//! Steps can include a `modifiers` field with a boolean expression that determines whether the step should execute:
//!
//! ```toml
//! [workspace.metadata.ci.jobs.conditional_build]
//! steps = [
//!   "cargo build",  # Always runs
//!   { command = "cargo build --features experimental", modifiers = "experimental" },
//!   { command = "cargo +nightly build", modifiers = "nightly" },
//!   { command = "cargo test --release", modifiers = "nightly & experimental" }
//! ]
//! ```
//!
//! ## Modifier Expression Syntax
//!
//! Modifier expressions support boolean logic:
//!
//! - **Identifier**: `nightly`, `experimental`, etc. - evaluates to `true` if the modifier is defined
//! - **Logical AND**: `&` - both operands must be true
//! - **Logical OR**: `|` - at least one operand must be true
//! - **Logical NOT**: `!` - negates the following expression
//! - **Grouping**: `(` and `)` - controls evaluation order
//!
//! ## Examples
//!
//! ```toml
//! # Run only if "nightly" modifier is defined
//! { command = "cargo +nightly build", modifiers = "nightly" }
//!
//! # Run if both "nightly" AND "experimental" are defined
//! { command = "cargo test --all-features", modifiers = "nightly & experimental" }
//!
//! # Run if either "nightly" OR "stable" is defined
//! { command = "cargo build", modifiers = "nightly | stable" }
//!
//! # Run if "nightly" is NOT defined
//! { command = "cargo build --no-unstable", modifiers = "!nightly" }
//!
//! # Complex expression with grouping
//! { command = "cargo test", modifiers = "(nightly | beta) & !windows" }
//! ```
//!
//! ## Operator Precedence
//!
//! Operators are evaluated in the following order (highest to lowest precedence):
//! 1. `!` (NOT)
//! 2. `&` (AND)
//! 3. `|` (OR)
//!
//! Use parentheses to override the default precedence:
//! - `a | b & c` is evaluated as `a | (b & c)`
//! - `(a | b) & c` forces the OR to be evaluated first
//!
//! ## Behavior
//!
//! - Steps without a `modifiers` field always execute
//! - Steps with a `modifiers` field are skipped if the expression evaluates to false
//! - When a step is skipped, it's indicated in the output: `(skipped due to modifiers)`
//! - Invalid modifier expressions cause the job to fail with an error message
//!
//! ## Use Cases
//!
//! Modifiers are useful for:
//!
//! - **Development phases**: Different commands for alpha, beta, release candidates, and production
//! - **Platform-specific builds**: Run commands only on specific operating systems
//! - **Feature flags**: Test experimental features separately from stable ones
//! - **Toolchain versions**: Use nightly-only features when the nightly modifier is set
//! - **CI environments**: Different behavior for local development vs. CI servers
//!
//! # Environment Variables
//!
//! When executing steps, cargo-ci sets the following environment variables:
//!
//! - `CI_JOB`: Name of the current job being executed
//! - `CI_PACKAGE_NAME`: Name of the package in which the step is running
//!
//! These can be used in your scripts to customize behavior based on the job context.
//!

mod args;
mod host;
mod job_definition;
mod job_result;
mod jobs;
mod modifier_expression;
mod package_config;
mod step;

use anyhow::{Context, Result, anyhow};
use cargo_metadata::MetadataCommand;
use clap::Parser;
use std::collections::HashMap;
use std::sync::Arc;

use crate::job_definition::JobDefinition;
use args::Args;
use host::{Host, RealHost};
use job_result::print_summary;
use jobs::Jobs;
use package_config::PackageConfig;

fn main() {
    let args = Args::parse();
    let host: Arc<dyn Host> = Arc::new(RealHost);

    if let Err(e) = real_main(args, &host) {
        host.eprintln_fmt(format_args!("cargo-ci error: {e}"));
        std::process::exit(1);
    }
}

fn real_main(args: Args, host: &Arc<dyn Host>) -> Result<()> {
    let metadata = MetadataCommand::new().exec().context("Unable to obtain cargo metadata")?;
    let root_manifest = &metadata.workspace_root.join("Cargo.toml");
    let jobs = Jobs::load(root_manifest.as_std_path(), Arc::clone(host))?;

    if args.list {
        jobs.print_list();
        return Ok(());
    }

    if jobs.is_empty() && !args.list {
        return Err(anyhow!("No jobs are defined. Please add jobs to your `ci.toml` or `Cargo.toml`."));
    }

    let job_names_to_run = if args.all_jobs {
        let mut v: Vec<_> = jobs.iter().map(|(k, _)| k.clone()).collect();
        v.sort();
        v
    } else if !args.jobs.is_empty() {
        for name in &args.jobs {
            _ = jobs
                .get(name)
                .ok_or_else(|| anyhow!("Unknown job '{name}'. Use --list to view available jobs."))?;
        }
        args.jobs
    } else {
        jobs.print_list();
        return Err(anyhow!("No job specified. Use --all-jobs or provide a job name."));
    };

    // Validate dependencies for all jobs
    jobs.validate_dependencies(&job_names_to_run)?;

    // Sort jobs topologically to respect dependencies
    let sorted_names = jobs.topological_sort(&job_names_to_run)?;
    let ordered_jobs: Vec<(String, JobDefinition)> = sorted_names
        .into_iter()
        .map(|name| {
            let job = jobs.get(&name).unwrap().clone();
            (name, job)
        })
        .collect();

    let all_workspace_pkgs: Vec<_> = metadata.workspace_packages();

    // Filter packages based on --package option
    let selected_packages: Vec<_> = if args.package.is_empty() {
        // No filter specified, use all workspace packages
        all_workspace_pkgs
    } else {
        // Filter packages by name using iterator combinators
        args.package
            .iter()
            .map(|pkg_spec| {
                all_workspace_pkgs
                    .iter()
                    .find(|pkg| pkg.name == *pkg_spec)
                    .copied()
                    .ok_or_else(|| anyhow!("Package '{pkg_spec}' not found in workspace"))
            })
            .collect::<Result<Vec<_>>>()?
    };

    // Load package configuration for each selected package
    let package_configs: HashMap<String, PackageConfig> = selected_packages
        .iter()
        .map(|pkg| {
            let manifest_path = pkg.manifest_path.as_std_path();
            let package_config =
                PackageConfig::load(manifest_path).with_context(|| format!("Loading project configuration for {}", pkg.name))?;
            Ok((pkg.id.to_string(), package_config))
        })
        .collect::<Result<HashMap<_, _>>>()?;

    let mut results = Vec::new();
    for (job_name, job_cfg) in ordered_jobs {
        host.println_fmt(format_args!("== Job: {job_name} =="));
        match jobs.run_job(
            &job_name,
            &job_cfg,
            &selected_packages,
            &package_configs,
            args.dry_run,
            args.keep_going,
            !args.sequential, // Parallel by default, sequential only if flag is set
        ) {
            Ok(r) => results.push(r),
            Err(e) => {
                if job_cfg.continue_on_error {
                    host.eprintln_fmt(format_args!("Job '{job_name}' failed but continuing due to continue_on_error: {e}"));
                    // Create a failed result for summary
                    results.push(job_result::JobResult::new(
                        job_name.clone(),
                        selected_packages.len(),
                        0,
                        1,
                        args.dry_run,
                    ));
                } else {
                    print_summary(&results, host.as_ref());
                    return Err(e.context(format!("Job '{job_name}' failed")));
                }
            }
        }
    }

    print_summary(&results, host.as_ref());

    // Exit with error if any job had failures
    let total_failures: usize = results.iter().map(|r| r.failures).sum();
    if total_failures > 0 {
        return Err(anyhow!("Jobs completed with {total_failures} failure(s)"));
    }

    Ok(())
}
