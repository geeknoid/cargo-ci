use crate::commands::{InstallArgs, ListJobArgs, RunArgs};
use clap::{Parser, Subcommand, arg};
use std::path::PathBuf;

/// The app's command-line arguments.
#[derive(Parser, Debug)]
#[command(name = "cargo-ci", bin_name = "cargo", version, about = "Local CI workflows for Rust developers")]
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
    pub command: Option<Commands>,

    /// Path to Cargo.toml.
    #[arg(long, value_name = "PATH", global = true, default_value = "Cargo.toml")]
    pub manifest_path: PathBuf,

    /// Path to configuration file [default: one of ci.[toml|yml|yaml|json] ].
    #[arg(long, short = 'c', value_name = "PATH", global = true)]
    pub config: Option<PathBuf>,

    /// Flattened `RunArgs` for when no subcommand is specified
    #[command(flatten)]
    #[expect(clippy::struct_field_names, reason = "Necessary for flattening RunArgs")]
    pub run_args: RunArgs,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Runs a set of jobs.
    Run(RunArgs),

    /// Lists all the jobs defined in configuration.
    ListJobs(ListJobArgs),

    /// Installs or updates the tools defined in configuration.
    Install(InstallArgs),
}

impl Args {
    /// Get the command, defaulting to Run with flattened args if not specified
    pub fn get_command(&self) -> Commands {
        self.command.clone().unwrap_or_else(|| Commands::Run(self.run_args.clone()))
    }
}
