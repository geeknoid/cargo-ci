//! Local CI workflows for Rust developers.
//!
//! Modern CI systems provide powerful ways to define and run complex job workflows,
//! doing things like running various checks, builds, and tests across multiple crates in a
//! workspace. They usually require pushing code to a remote repository, and then waiting for
//! the automation to kick in before you start getting feedback. This process can be slow and
//! cumbersome, especially during development when you want quick feedback. It can also be frustrating,
//! as CI environments often differ from your local setup, leading to unexpected failures.
//!
//! `cargo-ci` addresses these challenges by allowing you to define and run CI-style job pipelines
//! directly on your local machine. This means you can get immediate feedback on your code changes
//! without the need to push to a remote repository. By simulating CI workflows locally, you
//! can catch issues early, ensure consistency across your workspace, and streamline your development process.
//!
//! You can easily use `cargo-ci` in your real CI system as well, ensuring that the local developer
//! experience matches the remote CI environment.
//!
//! # Key Features
//!
//! - **Job Orchestration**: Define jobs, which are sequences of individual steps to perform a particular task. Jobs
//!   can include building, testing, linting, and more.
//!
//! - **Variables**. Jobs and steps can execute conditionally based on variables sourced from the environment, command-line, and
//!   Cargo.toml files.
//!
//! - **Tool Installation**. Can install required Cargo tools automatically before running jobs, ensuring consistent
//!   versions across environments and development teams.
//!
//! # Getting Started
//!
//! 1.  Install `cargo-ci`:
//!
//!     ```shell
//!     cargo install cargo-ci
//!     ```
//!
//! 2.  Create a `ci.toml` file in the root of your Cargo workspace:
//!
//!     ```toml
//!     default_jobs = ["build", "test", "lint"]
//!
//!     [jobs.build]
//!     name = "Build Workspace"
//!     steps = ["cargo build --all-targets --all-features"]
//!
//!     [jobs.test]
//!     name = "Run Tests"
//!     needs = ["build"]
//!     steps = [
//!         "cargo test",
//!         "cargo test --doc"
//!     ]
//!
//!     [jobs.lint]
//!     name = "Lint Workspace"
//!     needs = ["build"]
//!     steps = [
//!         "cargo fmt -- --check",
//!         "cargo clippy -- -D warnings",
//!         "cargo audit"
//!     ]
//!
//!     [tools]
//!     cargo-audit = "0.21.2"
//!     ```
//!
//! 3.  Run your CI pipeline:
//!
//!     ```shell
//!     # Run the default jobs ("build", "test", and "lint")
//!     cargo ci
//!
//!     # Run a specific job (its dependencies will run first)
//!     cargo ci test
//!     ```
//!
//! # Terminology
//!
//! - **Job**. A job is a collection of steps that accomplish a specific task, such as building the project or running tests. Jobs can depend on
//!   other jobs and can be conditionally executed based on defined variables.
//!
//! - **Step**. A step is an individual command or action within a job. Steps are executed sequentially as part of their parent job.
//!   Think of a step as a single command-line instruction, like `cargo build` or `cargo test --doc`.
//!
//! - **Tool**. A tool is a Cargo-based utility that can be installed and used
//!   within jobs and steps, such as `cargo-audit` or `cargo-deny`.
//!
//! - **Variable**. A variable is a named value that can influence the execution of jobs
//!   and steps. Variables can be defined via environment variables, command-line arguments,
//!   or Cargo.toml metadata.
//!
//! # Command-Line Interface
//!
//! `cargo-ci` is invoked as a cargo subcommand: `cargo ci`. It supports these following subcommands:
//!
//! - `run`. Executes CI jobs (default).
//!
//! - `list-jobs`. Lists all defined CI jobs.
//!
//! - `install`. Installs or updates required tools for the CI jobs.
//!
//! If no subcommand is specified, `run` is assumed. For example, `cargo ci lint` is equivalent to `cargo ci run lint`.
//!
//! ## Global Options
//!
//! These options can be used with any subcommand.
//!
//! - `--manifest-path <PATH>`: Path to the `Cargo.toml` of the workspace. Defaults to the `Cargo.toml` in the current directory.
//!
//! - `-c, --config <PATH>`: Path to the `cargo-ci` configuration file. Defaults to any of `ci.toml`,
//!   `ci.yml`, `ci.yaml`, or `ci.json` in the workspace root.
//!
//! ## The `run` Subcommand
//!
//! This is the main workhorse subcommand: it lets you execute jobs. This is the default subcommand, so you don't
//! need to specify it explicitly if you don't want to.
//!
//! **Usage**: `cargo ci run [OPTIONS] [JOBS]...`
//!
//! - `[JOBS]...`. A space-separated list of job IDs to run. If omitted, runs the `default_jobs` defined in the configuration file.
//!   If there are no default jobs defined, all available jobs are run.
//!
//! - `-n, --dry-run`. Show the execution plan without running any commands.
//!
//! - `-p, --package <SPEC>`. Run jobs only on specified packages. This flag can be used multiple times.
//!
//! - `-v, --variable <KEY=VAL>`. Define a variable for expression evaluation. This can be used multiple times and will override variables from other sources.
//!
//! - `-l, --log-file <FILE>`. Redirect detailed log output to a specific file. By default, logs are stored in `target/logs/cargo-ci/`.
//!
//! - `--log-file-retention-count <COUNT>`. Number of log files to retain (default: 16).
//!
//! - `--color <WHEN>`. Control when to use colored output. Valid values are `auto` (default), `always`, or `never`.
//!
//! ## The `list-jobs` Subcommand
//!
//! Lists all jobs defined in configuration.
//!
//! **Usage**: `cargo ci list-jobs [OPTIONS]`
//!
//! - `-s, --show-steps`. Show the steps for each job.
//!
//! ## The `install` Subcommand
//!
//! Installs or updates the tools defined in configuration.
//!
//! **Usage**: `cargo ci install [OPTIONS]`
//!
//! - `-l, --log-file <FILE>`. Redirect installation log output to a specific file.
//!
//! - `--log-file-retention-count <COUNT>`. Number of log files to retain (default: 16).
//!
//! - `--color <WHEN>`. Control when to use colored output. Valid values are `auto` (default), `always`, or `never`.
//!
//! # Configuration File
//!
//! Jobs and steps are defined in the `cargo-ci` configuration file, normally called `ci.toml` and located at the root of
//! your workspace. You can specify a different path for the configuration file using the `--config <PATH>` option. Configuration
//! files can be in TOML, YAML, or JSON formats, although we show only TOML in this documentation.
//!
//! ## Top-Level Values
//!
//! - `default_jobs`. (Optional) An array of job IDs to run when `cargo ci run` is invoked without specific jobs. When this
//!   value is not defined, then the default behavior is to run all defined jobs.
//!
//!   ```toml
//!   default_jobs = ["build", "test"]
//!   ```
//!
//! - `passthrough_env_variables`. (Optional) An array of environment variable names to import for use in expressions. There are four versions of this field to handle
//!   platform-specific values. If none of these values are defined, then all environment variables are imported.
//!
//!   ```toml
//!   passthrough_env_variables = ["CARGO_HOME", "RUST_VERSION"]
//!   passthrough_env_variables_windows = ["ProgramFiles"]
//!   passthrough_env_variables_linux = ["USER"]
//!   passthrough_env_variables_macos = ["TMP"]
//!   ```
//!
//!   Only the environment variables listed here will be available for conditional expressions in the job definitions and will be
//!   available at runtime to the various tools invoked by `cargo-ci`. This helps ensure that only intended environment variables
//!   influence the CI process.
//!
//! ## The `[tools]` Table
//!
//! This table defines the `cargo` tools required by your jobs. These can be installed or updated using `cargo ci install`.
//! The key is the crate name, and the value can be a simple version string or a detailed table.
//!
//! ```toml
//! [tools]
//! # Simple version
//! cargo-edit = "0.12.0"
//!
//! # Extended configuration, mapping to `cargo install` options
//! cargo-nextest = { version = "0.9.62", git = "https://github.com/nextest-rs/nextest.git", rev = "..." }
//! ```
//!
//! The extended form supports `version`, `index`, `registry`, `git`, `branch`, `tag`, `rev`, and `path` fields. These
//! map directly to the corresponding [`cargo install`](https://doc.rust-lang.org/cargo/commands/cargo-install.html)
//! command-line options and provide you fine-grained control over how each tools is installed.
//!
//! ## The `[jobs.<job-id>]` Tables
//!
//! These tables let you define jobs, where each job is made up of a sequence of individual steps. The `<job-id>` is a unique identifier
//! used to run the job and define dependencies.
//!
//! ```toml
//! [jobs.test]
//! name = "Run Tests"
//! needs = ["build"]
//! if = "os == 'linux'"
//! continue_on_error = false
//! steps = [
//!   "cargo test --workspace --all-targets",
//!   { command = "cargo test --doc", per_package = true }
//! ]
//! ```
//!
//! Here are the properties you can set for each job:
//!
//! - `name`. (Optional) A human-friendly display name for the job, used in logs and output. Defaults to the job ID.
//! - `needs`. (Optional) An array of job IDs that must complete successfully before this job starts.
//! - `if`. (Optional) An expression that must evaluate to `true` for the job to run.
//! - `continue_on_error`. (Optional) A boolean or an expression. If `true`, a failure in this job will not stop the entire CI run. Defaults to `false`.
//! - `steps`. (Required) An array of steps to execute.
//! - `variables`. (Optional) A table of variables specific to this job that can be used in expressions.
//!
//! ### Steps
//!
//! A step can be a simple command string or a table for more configuration.
//!
//! - **Simple Step Form**
//!
//!     ```toml
//!     steps = ["cargo fmt --check"]
//!     ```
//!
//! - **Extended Step Form**
//!
//!   ```toml
//!     steps = [{ command = "cargo clippy", name = "Lints", id = "clippy-check", continue_on_error = true }]
//!   ```
//!
//! Here are the properties you can set in the extended form:
//!
//! - `command`: (Required) The shell command to execute for this step.
//! - `name`: (Optional) A display name for the step, used for logs and output. Defaults to the command string.
//! - `id`: (Optional) A stable identifier, used when steps depend on one another.
//! - `if`: (Optional) An expression to conditionally run this step.
//! - `continue_on_error`. (Optional) A boolean or an expression. If `true`, a failure in this step will not stop the entire job. Defaults to `false`.
//! - `per_package`: (Optional) If `true`, run this step for each selected package in the workspace. The working directory will be the package's root. Otherwise,
//!   the step runs once in the workspace root. Defaults to `false`.
//! - `variables`. (Optional) A table of variables specific to this step that can be used in expressions.
//!
//! ## The `[variables]` Table
//!
//! This table lets you define global variables that can be used in expressions throughout the configuration file. For example:
//!
//! ```toml
//! [variables]
//! FOO = "Bar"
//! ```
//!
//! ## File Formats
//!
//! `cargo-ci` supports configuration files in TOML, YAML, and JSON formats. The file extension
//! determines the format: `.toml` for TOML, `.yml` or `.yaml` for YAML, and `.json` for JSON.
//! This flexibility allows you to choose the format that best fits your project's needs.
//!
//! # Variables and Expressions
//!
//! `cargo-ci` supports conditional execution of jobs and steps using expressions. These expressions
//! can reference variables from multiple sources, allowing for flexible and dynamic CI workflows.
//! The excellent [`evalexpr`](https://docs.rs/evalexpr/latest/evalexpr/#features) crate is used to evaluate
//! `if` and `continue_on_error` expressions. Please see that crate's documentation for full details on the
//! supported expression syntax and operations.
//!
//! The variables you can reference in expressions come from many sources:
//!
//! - **Environment Variables**. When `cargo-ci` starts, it imports environment variables from the shell based on the
//!   `passthrough_env_variables` setting in `ci.toml`. Only the variables listed there will be available for use in expressions.
//!   If no such setting is defined, then all environment variables are imported.
//!
//! - **Workspace Variables**. You can define global variables in the `[variables]` table in the configuration file.
//!
//! - **Job and Step Variables**. You can define variables specific to a job or step using the `variables` property.
//!
//! - **Package Metadata**. You can define variables in a crate's `Cargo.toml` file inside the `[package.metadata.ci.variables]`
//!   table. These variables take precedence over environment variables.
//!
//!     ```toml
//!     # In a crate's Cargo.toml
//!     [package.metadata.ci.variables]
//!     SPECIAL_FLAG = "true"
//!     ```
//!
//! - **Command-Line Variables**. You can define variables directly via the command-line using the `-v, --variable <KEY=VAL>` option.
//!   These variables take precedence over all other variable sources.
//!
//! Given all these sources, it gets complicated to know which variable takes effect when and what is the precedence of selection
//! in case there are conflicting definitions. Hopefully, the following helps clarify things:
//!
//! When evaluating `Job::if`, precedence from lowest to highest is:
//!
//! - Environment variables
//! - Config variables
//! - Package metadata variables
//! - Command-line variables
//!
//! When evaluating `Job::continue_on_error`, precedence from lowest to highest is:
//!
//! - Environment variables
//! - Config variables
//! - Command-line variables
//!
//! When evaluating `Step::if`, precedence from lowest to highest is:
//!
//! - Environment variables
//! - Workspace variables
//! - Job variables
//! - Package metadata variables
//! - Command-line variables
//!
//! When evaluating `Step::continue_on_error`, precedence from lowest to highest is:
//!
//! - Environment variables
//! - Workspace variables
//! - Job variables
//! - Package metadata variables (only when `Step::per_package` is true)
//! - Command-line variables
//!
//! When executing individual steps, precedence from lowest to highest is:
//!
//! - Environment variables
//! - Workspace variables
//! - Job variables
//! - Step variables
//! - Package metadata variables (only when `Step::per_package` is true)
//! - Command-line variables
//!
//! ## Example Expression
//!
//! This step only runs on the `main` branch when the `CI` environment variable is set.
//!
//! ```toml
//! passthrough_env_variables = ["CI", "GITHUB_REF_NAME"]
//!
//! [jobs.publish]
//! steps = [
//!   { command = "cargo publish", if = "CI == 'true' && GITHUB_REF_NAME == 'main'" }
//! ]
//! ```
//!
//! # Logging
//!
//! `cargo-ci` generates detailed logs for each use of the `run` or `install` subcommands. The logs are
//! stored by default in the `target/logs/cargo-ci/` directory. The log files are named with timestamps
//! to help you identify when each run occurred. By default, the last 16 log files are retained, and older ones
//! are automatically deleted to save space.
//!
//! You can specify a custom log file using the `--log-file <FILE>` option, and you can control how many
//! log files to retain with the `--log-file-retention-count <COUNT>` option.
//!
//! # Using `cargo-ci` in Real CI Systems
//!
//! `cargo-ci` is designed to be compatible with real CI systems. You can use it in your CI pipelines
//! to ensure that the same job definitions and workflows you use locally are also executed in your
//! remote CI environment. This helps maintain consistency and reduces the chances of discrepancies
//! between local and CI runs.
//!
//! To use `cargo-ci` in a real CI system, simply include it as part of your CI configuration,
//! and run the desired jobs as you would locally. Make sure to install `cargo-ci`
//! in your CI environment before invoking it.

mod args;
//mod cargo_tools;
mod color_modes;
mod commands;
mod config;
mod expressions;
mod host;
mod log;
mod outputter;
mod pkg_data;

use crate::args::{Args, CargoSubcommand, Commands};
//use crate::cargo_tools::CargoTools;
use crate::config::Config;
use anyhow::{Context, Result};
use args::Cli;
use cargo_metadata::MetadataCommand;
use clap::Parser;
use commands::{install_tools, list_jobs, run_jobs};
use host::{Host, RealHost};

fn main() {
    let CargoSubcommand::Ci(args) = Cli::parse().command;
    let mut host = RealHost::new();

    if let Err(e) = inner_main(&args, &mut host) {
        if !host.should_fail_silently() {
            host.eprintln(format!("ERROR: {e}"));
        }
        std::process::exit(1);
    }
}

fn inner_main<H: Host>(args: &Args, host: &mut H) -> Result<()> {
    let mut cmd = MetadataCommand::new();
    _ = cmd.manifest_path(&args.manifest_path);

    let metadata = cmd.no_deps().exec().context("unable to obtain cargo metadata")?;
    let cfg = Config::load(metadata.workspace_root.as_std_path(), args.config.as_ref())?;
    //    let _tools = CargoTools::read()?;

    match args.get_command() {
        Commands::Run(ref args) => {
            run_jobs(args, host, &cfg, &metadata)?;
        }

        Commands::ListJobs(ref args) => {
            list_jobs(args, host, &cfg);
        }

        Commands::Install(ref args) => {
            install_tools(args, host, &cfg, &metadata)?;
        }
    }

    Ok(())
}
