use crate::color_modes::ColorModes;
use crate::host::Host;
use crate::log::Log;
use console::{StyledObject, Term, style};
use core::cell::RefCell;
use std::path::Path;
use std::process::{Command, ExitStatus, Output};

struct InnerOutputter {
    term: Term,
    activity: String,
    cmdline: String,
}

pub struct Outputter<'a, H> {
    host: &'a H,
    log: &'a Log,
    inner: RefCell<InnerOutputter>,
    color: ColorModes,
}

impl<'a, H: Host> Outputter<'a, H> {
    pub fn new(host: &'a H, log: &'a Log, color: ColorModes) -> Self {
        Self {
            host,
            log,
            inner: RefCell::new(InnerOutputter {
                term: Term::stdout(),
                activity: String::new(),
                cmdline: String::new(),
            }),
            color,
        }
    }

    pub fn start_activity(&self, activity: impl AsRef<str>) {
        let mut inner = self.inner.borrow_mut();
        inner.activity = activity.as_ref().into();

        if inner.term.is_term() {
            _ = inner.term.hide_cursor();
        }
    }

    pub fn complete_activity(&self, final_message: impl AsRef<str>) {
        let mut inner = self.inner.borrow_mut();
        _ = inner.term.clear_line();
        _ = inner.term.write_line(&format!("{}: {}", inner.activity, final_message.as_ref()));
        inner.activity = String::new();
    }

    pub fn run_command(&self, cmd: &Command) {
        let mut inner = self.inner.borrow_mut();
        inner.cmdline = format!("{}> {cmd:?}", cmd.get_current_dir().unwrap_or_else(|| Path::new("?")).display());

        self.log.info(format!("Running command: {}", inner.cmdline));
    }

    pub fn command_error(&self, failure_message: impl AsRef<str>, status: Option<ExitStatus>, output: Option<&Output>, fatal: bool) {
        let failure_msg = failure_message.as_ref();
        let inner = self.inner.borrow();

        let tail = status.map_or_else(String::new, |status| {
            let code = status.code().unwrap_or(-1);
            if fatal {
                format!(" (error code {code})")
            } else {
                format!(" (error code {code}, ignored)")
            }
        });

        let styled_message = if fatal { self.red(failure_msg) } else { self.yellow(failure_msg) };

        if inner.term.is_term() {
            _ = inner.term.write_line(&format!(" -> {styled_message}{tail}"));
        } else {
            let print_message = format!("{styled_message}{tail}");
            if fatal {
                self.host.eprintln(&print_message);
            } else {
                self.host.println(&print_message);
            }
        }

        let log_message = format!("{failure_msg}{tail}");
        if fatal {
            self.log.error(&log_message);
        } else {
            self.log.warn(&log_message);
        }

        let print_fn: &dyn Fn(&str) = if fatal {
            &|s: &str| self.host.eprintln(s)
        } else {
            &|s: &str| self.host.println(s)
        };

        let log_fn: &dyn Fn(&str) = if fatal {
            &|s: &str| self.log.error(s)
        } else {
            &|s: &str| self.log.warn(s)
        };

        print_fn("--- command-line used");
        print_fn(&inner.cmdline);

        if let Some(output) = output {
            if !output.stdout.is_empty() {
                let stdout_str = String::from_utf8_lossy(&output.stdout);
                print_fn("--- captured stdout");
                log_fn("--- captured stdout");

                let styled_stdout = style(stdout_str.trim()).italic().to_string();
                print_fn(&styled_stdout);

                for line in stdout_str.lines() {
                    log_fn(line);
                }
            }

            if !output.stderr.is_empty() {
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                print_fn("--- captured stderr");
                log_fn("--- captured stderr");

                let styled_stderr = style(stderr_str.trim()).italic().to_string();
                print_fn(&styled_stderr);

                for line in stderr_str.lines() {
                    log_fn(line);
                }
            }
        }

        print_fn("--- end");
        log_fn("--- end");
    }

    pub fn message(&self, message: impl AsRef<str>) {
        let inner = self.inner.borrow();
        let formatted = format!("{}: {}", inner.activity, message.as_ref());

        if inner.term.is_term() {
            _ = inner.term.clear_line();
            _ = inner.term.write_str(&formatted);
        } else {
            self.host.println(&formatted);
        }

        self.log.info(&formatted);
    }

    fn should_use_color(&self) -> bool {
        match self.color {
            ColorModes::Always => true,
            ColorModes::Never => false,
            ColorModes::Auto => self.inner.borrow().term.is_term(),
        }
    }

    fn red<D>(&self, data: D) -> StyledObject<D> {
        if self.should_use_color() { style(data).red() } else { style(data) }
    }

    fn yellow<D>(&self, data: D) -> StyledObject<D> {
        if self.should_use_color() {
            style(data).yellow()
        } else {
            style(data)
        }
    }
}

impl Drop for InnerOutputter {
    fn drop(&mut self) {
        if self.term.is_term() {
            let _ = self.term.show_cursor();
        }
    }
}
