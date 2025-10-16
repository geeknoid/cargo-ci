use std::process::{Child, Command};

/// Abstract the host environment to enable testing
pub trait Host: Send + Sync {
    /// Spawn the given command
    fn spawn(&self, cmd: &mut Command) -> std::io::Result<Child>;

    /// Gets all environment variables as key-value pairs
    fn vars(&self) -> impl Iterator<Item = (String, String)>;

    /// Write formatted output to stdout.
    fn print_fmt(&mut self, args: core::fmt::Arguments<'_>);

    /// Write formatted output to stderr.
    fn eprint_fmt(&mut self, args: core::fmt::Arguments<'_>);

    /// Write formatted output to stdout.
    fn println_fmt(&mut self, args: core::fmt::Arguments<'_>);

    /// Write formatted output to stderr.
    fn eprintln_fmt(&mut self, args: core::fmt::Arguments<'_>);

    /// Write some text to stdout.
    fn print(&mut self, message: impl AsRef<str>) {
        self.print_fmt(format_args!("{}", message.as_ref()));
    }

    /// Write some text to stderr.
    #[expect(dead_code, reason = "TODO")]
    fn eprint(&mut self, message: impl AsRef<str>) {
        self.eprint_fmt(format_args!("{}", message.as_ref()));
    }

    /// Write a line to stdout.
    fn println(&mut self, message: impl AsRef<str>) {
        self.println_fmt(format_args!("{}", message.as_ref()));
    }

    /// Write a line to stderr.
    fn eprintln(&mut self, message: impl AsRef<str>) {
        self.eprintln_fmt(format_args!("{}", message.as_ref()));
    }

    /// Prevent the host from outputting an error on termination
    fn fail_silently(&mut self);

    /// Check if the host is set to fail silently
    fn should_fail_silently(&self) -> bool;
}

/// Default host that runs real OS commands.
#[derive(Debug, Clone, Default)]
pub struct RealHost {
    fail_silently: bool,
}

impl RealHost {
    /// Create a new `RealHost`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Host for RealHost {
    fn spawn(&self, cmd: &mut Command) -> std::io::Result<Child> {
        cmd.spawn()
    }

    fn vars(&self) -> impl Iterator<Item = (String, String)> {
        std::env::vars_os().map(|(k, v)| (k.to_string_lossy().into_owned(), v.to_string_lossy().into_owned()))
    }

    #[expect(clippy::print_stdout, reason = "Real host outputs to stdout")]
    fn print_fmt(&mut self, args: core::fmt::Arguments<'_>) {
        print!("{args}");
    }

    #[expect(clippy::print_stderr, reason = "Real host outputs to stderr")]
    fn eprint_fmt(&mut self, args: core::fmt::Arguments<'_>) {
        eprint!("{args}");
    }

    #[expect(clippy::print_stdout, reason = "Real host outputs to stdout")]
    fn println_fmt(&mut self, args: core::fmt::Arguments<'_>) {
        println!("{args}");
    }

    #[expect(clippy::print_stderr, reason = "Real host outputs to stderr")]
    fn eprintln_fmt(&mut self, args: core::fmt::Arguments<'_>) {
        eprintln!("{args}");
    }

    fn fail_silently(&mut self) {
        self.fail_silently = true;
    }

    fn should_fail_silently(&self) -> bool {
        self.fail_silently
    }
}

#[cfg(test)]
mod test_support {
    use super::*;
    use std::collections::HashMap;

    /// A mock host for testing that captures commands and output.
    #[derive(Debug, Clone, Default)]
    #[allow(dead_code, reason = "Used in other integration tests")]
    pub struct FakeHost {
        pub commands: Vec<String>,
        pub stdout: Vec<String>,
        pub stderr: Vec<String>,
        pub fail_silently: bool,

        variables: HashMap<String, String>,
    }

    #[allow(dead_code, reason = "Used in other integration tests")]
    impl FakeHost {
        /// Create a new `FakeHost`.
        #[must_use]
        pub fn new(_variables: HashMap<String, String>) -> Self {
            Self::default()
        }
    }

    impl Host for FakeHost {
        fn spawn(&self, _cmd: &mut Command) -> std::io::Result<Child> {
            todo!()
        }

        fn vars(&self) -> impl Iterator<Item = (String, String)> {
            self.variables.clone().into_iter()
        }

        fn print_fmt(&mut self, args: core::fmt::Arguments<'_>) {
            self.stdout.push(format!("{args}"));
        }

        fn eprint_fmt(&mut self, args: core::fmt::Arguments<'_>) {
            self.stderr.push(format!("{args}"));
        }

        fn println_fmt(&mut self, args: core::fmt::Arguments<'_>) {
            self.stdout.push(format!("{args}"));
        }

        fn eprintln_fmt(&mut self, args: core::fmt::Arguments<'_>) {
            self.stderr.push(format!("{args}"));
        }

        fn fail_silently(&mut self) {
            self.fail_silently = true;
        }

        fn should_fail_silently(&self) -> bool {
            self.fail_silently
        }
    }
}
