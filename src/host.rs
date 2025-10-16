use std::process::{Child, Command};

/// Abstract the host environment to enable testing
pub trait Host: Send + Sync {
    /// Spawn the given command
    fn spawn(&self, cmd: &mut Command) -> std::io::Result<Child>;

    /// Gets all environment variables as key-value pairs
    fn vars(&self) -> impl Iterator<Item = (String, String)>;

    /// Write formatted output to stdout.
    fn println_fmt(&self, args: core::fmt::Arguments<'_>);

    /// Write formatted output to stderr.
    fn eprintln_fmt(&self, args: core::fmt::Arguments<'_>);

    /// Write a line to stdout.
    fn println(&self, message: impl AsRef<str>) {
        self.println_fmt(format_args!("{}", message.as_ref()));
    }

    /// Write a line to stderr.
    fn eprintln(&self, message: impl AsRef<str>) {
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
    fn println_fmt(&self, args: core::fmt::Arguments<'_>) {
        println!("{args}");
    }

    #[expect(clippy::print_stderr, reason = "Real host outputs to stderr")]
    fn eprintln_fmt(&self, args: core::fmt::Arguments<'_>) {
        eprintln!("{args}");
    }

    fn fail_silently(&mut self) {
        self.fail_silently = true;
    }

    fn should_fail_silently(&self) -> bool {
        self.fail_silently
    }
}
