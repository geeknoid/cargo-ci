use clap::{ArgAction, Parser};

/// The app's command-line arguments.
#[derive(Parser, Debug)]
#[command(
    name = "cargo-ci",
    version,
    about = "Simulate CI jobs locally by running configured commands across crates in a Cargo workspace"
)]
#[expect(clippy::struct_excessive_bools, reason = "CLI arguments are naturally boolean flags")]
pub struct Args {
    /// Names of the jobs to run
    pub jobs: Vec<String>,

    /// Run all defined jobs
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "jobs")]
    pub all_jobs: bool,

    /// List available jobs and exit
    #[arg(long, action = ArgAction::SetTrue)]
    pub list: bool,

    /// Show steps to execute without executing them
    #[arg(long, action = ArgAction::SetTrue)]
    pub dry_run: bool,

    /// Continue running remaining jobs even if a step fails
    #[arg(long, action = ArgAction::SetTrue)]
    pub keep_going: bool,

    /// Execute jobs sequentially across packages instead of in parallel
    #[arg(long, action = ArgAction::SetTrue)]
    pub sequential: bool,

    /// Package(s) to run jobs on (can be specified multiple times)
    #[arg(short = 'p', long = "package", value_name = "SPEC")]
    pub package: Vec<String>,
}
