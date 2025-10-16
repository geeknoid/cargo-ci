use serde::Deserialize;

/// A step that can be either a simple string or an object with a command and optional name.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Step {
    /// Simple string command
    Simple(String),
    /// Extended form with command and optional name
    Extended {
        run: String,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        modifiers: Option<String>,
        #[serde(default)]
        continue_on_error: bool,
        #[serde(default)]
        working_directory: Option<String>,
    },
}

impl Step {
    /// Get the command line to execute
    pub fn command(&self) -> &str {
        match self {
            Self::Simple(cmd) => cmd,
            Self::Extended { run, .. } => run,
        }
    }

    /// Get the display name for this command.
    /// Returns the custom name if provided, otherwise the command line itself.
    pub fn display_name(&self) -> &str {
        match self {
            Self::Simple(cmd) => cmd,
            Self::Extended { run, name, .. } => name.as_deref().unwrap_or(run),
        }
    }

    /// Get the modifiers expression for this step, if any.
    pub fn modifiers_expr(&self) -> Option<&str> {
        match self {
            Self::Simple(_) => None,
            Self::Extended { modifiers, .. } => modifiers.as_deref(),
        }
    }

    /// Check if this step should continue on error.
    pub const fn continue_on_error(&self) -> bool {
        match self {
            Self::Simple(_) => false,
            Self::Extended { continue_on_error, .. } => *continue_on_error,
        }
    }

    /// Get the working directory for this step, if any.
    pub fn working_directory(&self) -> Option<&str> {
        match self {
            Self::Simple(_) => None,
            Self::Extended { working_directory, .. } => working_directory.as_deref(),
        }
    }
}

impl From<String> for Step {
    fn from(s: String) -> Self {
        Self::Simple(s)
    }
}

impl From<&str> for Step {
    fn from(s: &str) -> Self {
        Self::Simple(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_simple_form() {
        let cmd = Step::Simple("cargo build".to_string());
        assert_eq!(cmd.command(), "cargo build");
        assert_eq!(cmd.display_name(), "cargo build");
    }

    #[test]
    fn test_command_extended_form_with_name() {
        let cmd = Step::Extended {
            run: "cargo build --release".to_string(),
            name: Some("Build Release".to_string()),
            modifiers: None,
            continue_on_error: false,
            working_directory: None,
        };
        assert_eq!(cmd.command(), "cargo build --release");
        assert_eq!(cmd.display_name(), "Build Release");
    }

    #[test]
    fn test_command_extended_form_without_name() {
        let cmd = Step::Extended {
            run: "cargo build".to_string(),
            name: None,
            modifiers: None,
            continue_on_error: false,
            working_directory: None,
        };
        assert_eq!(cmd.command(), "cargo build");
        assert_eq!(cmd.display_name(), "cargo build");
    }

    #[test]
    fn test_command_from_string() {
        let cmd = Step::from("cargo test".to_string());
        assert_eq!(cmd.command(), "cargo test");
        assert_eq!(cmd.display_name(), "cargo test");
    }

    #[test]
    fn test_command_from_str() {
        let cmd = Step::from("cargo clippy");
        assert_eq!(cmd.command(), "cargo clippy");
        assert_eq!(cmd.display_name(), "cargo clippy");
    }

    #[test]
    fn test_command_equality() {
        let cmd1 = Step::Simple("cargo build".to_string());
        let cmd2 = Step::Simple("cargo build".to_string());
        let cmd3 = Step::Simple("cargo test".to_string());

        assert_eq!(cmd1, cmd2);
        assert_ne!(cmd1, cmd3);
    }

    #[test]
    fn test_command_clone() {
        let cmd1 = Step::Extended {
            run: "cargo build".to_string(),
            name: Some("Build".to_string()),
            modifiers: None,
            continue_on_error: false,
            working_directory: None,
        };
        let cmd2 = cmd1.clone();

        assert_eq!(cmd1, cmd2);
        assert_eq!(cmd1.command(), cmd2.command());
        assert_eq!(cmd1.display_name(), cmd2.display_name());
    }

    #[test]
    fn test_continue_on_error_simple() {
        let cmd = Step::Simple("cargo build".to_string());
        assert!(!cmd.continue_on_error());
    }

    #[test]
    fn test_continue_on_error_extended_false() {
        let cmd = Step::Extended {
            run: "cargo build".to_string(),
            name: None,
            modifiers: None,
            continue_on_error: false,
            working_directory: None,
        };
        assert!(!cmd.continue_on_error());
    }

    #[test]
    fn test_continue_on_error_extended_true() {
        let cmd = Step::Extended {
            run: "cargo build".to_string(),
            name: None,
            modifiers: None,
            continue_on_error: true,
            working_directory: None,
        };
        assert!(cmd.continue_on_error());
    }

    #[test]
    fn test_deserialize_step_with_continue_on_error() {
        let toml_str = r#"
            run = "cargo test"
            continue_on_error = true
        "#;
        let step: Step = toml::from_str(toml_str).unwrap();
        assert_eq!(step.command(), "cargo test");
        assert!(step.continue_on_error());
    }

    #[test]
    fn test_working_directory_simple() {
        let cmd = Step::Simple("cargo build".to_string());
        assert_eq!(cmd.working_directory(), None);
    }

    #[test]
    fn test_working_directory_extended_none() {
        let cmd = Step::Extended {
            run: "cargo build".to_string(),
            name: None,
            modifiers: None,
            continue_on_error: false,
            working_directory: None,
        };
        assert_eq!(cmd.working_directory(), None);
    }

    #[test]
    fn test_working_directory_extended_some() {
        let cmd = Step::Extended {
            run: "cargo build".to_string(),
            name: None,
            modifiers: None,
            continue_on_error: false,
            working_directory: Some("./subdir".to_string()),
        };
        assert_eq!(cmd.working_directory(), Some("./subdir"));
    }

    #[test]
    fn test_deserialize_step_with_working_directory() {
        let toml_str = r#"
            run = "cargo test"
            working_directory = "/tmp/test"
        "#;
        let step: Step = toml::from_str(toml_str).unwrap();
        assert_eq!(step.command(), "cargo test");
        assert_eq!(step.working_directory(), Some("/tmp/test"));
    }

    #[test]
    fn test_deserialize_step_with_all_fields() {
        let toml_str = r#"
            run = "cargo test"
            name = "Run tests"
            modifiers = "feature:test"
            continue_on_error = true
            working_directory = "./test-dir"
        "#;
        let step: Step = toml::from_str(toml_str).unwrap();
        assert_eq!(step.command(), "cargo test");
        assert_eq!(step.display_name(), "Run tests");
        assert_eq!(step.modifiers_expr(), Some("feature:test"));
        assert!(step.continue_on_error());
        assert_eq!(step.working_directory(), Some("./test-dir"));
    }
}
