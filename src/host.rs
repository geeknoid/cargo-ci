use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, ExitStatus};

/// Represents the result of executing a command.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub exit_code: Option<i32>,
}

impl ExecutionResult {
    pub fn from_status(status: ExitStatus) -> Self {
        Self {
            success: status.success(),
            exit_code: status.code(),
        }
    }

    /// Create a successful execution result.
    #[must_use]
    #[cfg(test)]
    pub const fn success() -> Self {
        Self {
            success: true,
            exit_code: Some(0),
        }
    }
}

/// Abstract the execution of commands and outputting to the console.
///
/// This trait allows for mocking command execution in tests.
pub trait Host: Send + Sync {
    /// Execute a command line in the given directory with the specified environment variables.
    fn execute(&self, cmdline: &str, working_dir: &Path, env_vars: &[(&str, &str)]) -> Result<ExecutionResult>;

    /// Write a line to stdout.
    fn println(&self, message: &str);

    /// Write formatted output to stdout.
    fn println_fmt(&self, args: core::fmt::Arguments<'_>);

    /// Write a line to stderr.
    fn eprintln(&self, message: &str);

    /// Write formatted output to stderr.
    fn eprintln_fmt(&self, args: core::fmt::Arguments<'_>);
}

/// Default host that runs real OS commands.
#[derive(Debug, Clone, Default)]
pub struct RealHost;

impl Host for RealHost {
    fn execute(&self, cmdline: &str, working_dir: &Path, env_vars: &[(&str, &str)]) -> Result<ExecutionResult> {
        let mut command = if cfg!(windows) {
            let mut c = Command::new("cmd");
            _ = c.arg("/C").arg(cmdline);
            c
        } else {
            let mut c = Command::new("sh");
            _ = c.arg("-c").arg(cmdline);
            c
        };

        let status = command
            .current_dir(working_dir)
            .envs(env_vars.iter().copied())
            .status()
            .with_context(|| format!("Failed to spawn command '{cmdline}' in directory {}", working_dir.display()))?;

        Ok(ExecutionResult::from_status(status))
    }

    #[expect(clippy::print_stdout, reason = "Real host outputs to stdout")]
    fn println(&self, message: &str) {
        println!("{message}");
    }

    #[expect(clippy::print_stdout, reason = "Real host outputs to stdout")]
    fn println_fmt(&self, args: core::fmt::Arguments<'_>) {
        println!("{args}");
    }

    #[expect(clippy::print_stderr, reason = "Real host outputs to stderr")]
    fn eprintln(&self, message: &str) {
        eprintln!("{message}");
    }

    #[expect(clippy::print_stderr, reason = "Real host outputs to stderr")]
    fn eprintln_fmt(&self, args: core::fmt::Arguments<'_>) {
        eprintln!("{args}");
    }
}

#[cfg(test)]
mod test_support {
    use super::*;
    use anyhow::anyhow;
    use std::sync::{Arc, Mutex};

    /// A mock host for testing that captures commands and output.
    #[derive(Debug, Clone)]
    #[allow(dead_code, reason = "Used in other integration tests")]
    pub struct FakeHost {
        /// The command lines that have been executed.
        pub commands: Arc<Mutex<Vec<String>>>,

        /// The lines that have been written to stdout.
        pub stdout: Arc<Mutex<Vec<String>>>,

        /// The lines that have been written to stderr.
        pub stderr: Arc<Mutex<Vec<String>>>,

        execution_result: Arc<Mutex<ExecutionResult>>,
    }

    impl Default for FakeHost {
        fn default() -> Self {
            Self {
                commands: Arc::default(),
                stdout: Arc::default(),
                stderr: Arc::default(),
                execution_result: Arc::new(Mutex::new(ExecutionResult::success())),
            }
        }
    }

    #[allow(dead_code, reason = "Used in other integration tests")]
    impl FakeHost {
        /// Create a new `FakeHost`.
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }

        /// Set the result that `execute` will return.
        pub fn set_execution_result(&self, result: ExecutionResult) {
            let mut execution_result = self.execution_result.lock().expect("should be able to lock");
            *execution_result = result;
        }
    }

    impl Host for FakeHost {
        fn execute(&self, cmdline: &str, _working_dir: &Path, _env_vars: &[(&str, &str)]) -> Result<ExecutionResult> {
            self.commands
                .lock()
                .map_err(|e| anyhow!("Failed to lock commands: {e}"))?
                .push(cmdline.to_string());
            let result = self
                .execution_result
                .lock()
                .map_err(|e| anyhow!("Failed to lock execution_result: {e}"))?
                .clone();
            Ok(result)
        }

        fn println(&self, message: &str) {
            self.stdout.lock().expect("should be able to lock").push(message.to_string());
        }

        fn println_fmt(&self, args: core::fmt::Arguments<'_>) {
            self.stdout.lock().expect("should be able to lock").push(format!("{args}"));
        }

        fn eprintln(&self, message: &str) {
            self.stderr.lock().expect("should be able to lock").push(message.to_string());
        }

        fn eprintln_fmt(&self, args: core::fmt::Arguments<'_>) {
            self.stderr.lock().expect("should be able to lock").push(format!("{args}"));
        }
    }
}
