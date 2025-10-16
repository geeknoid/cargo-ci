use crate::commands::{InstallArgs, ListJobArgs, RunArgs};
use clap::{Parser, Subcommand, arg};
use std::path::PathBuf;

/// The app's command-line arguments.
#[derive(Parser, Debug)]
#[command(
    name = "cargo",
    bin_name = "cargo",
    version,
    about = "Simulate CI jobs locally by running configured commands across crates in a Cargo workspace"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CargoSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum CargoSubcommand {
    /// Simulate CI jobs locally by running configured commands across crates in a Cargo workspace
    Ci(Args),
}

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to Cargo.toml.
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Runs CI jobs.
    Run(RunArgs),

    /// Lists all defined CI jobs.
    ListJobs(ListJobArgs),

    /// Installs required tools for the CI jobs.
    Install(InstallArgs),
}
