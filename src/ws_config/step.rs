use crate::expression::Expression;
use core::fmt;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
pub enum Step {
    Simple(String),

    Extended {
        command: String,
        name: Option<String>,
        cond: Option<Expression>,
        continue_on_errors: Option<Expression>,

        #[serde(default)]
        per_package: bool,
    },
}

impl Step {
    pub fn command(&self) -> &str {
        match self {
            Self::Simple(cmd) => cmd,
            Self::Extended { command: run, .. } => run,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Simple(cmd) => cmd,
            Self::Extended { command: run, name, .. } => name.as_deref().unwrap_or(run),
        }
    }

    pub const fn cond(&self) -> Option<&Expression> {
        match self {
            Self::Simple(_) => None,
            Self::Extended { cond, .. } => cond.as_ref(),
        }
    }

    pub const fn continue_on_errors(&self) -> Option<&Expression> {
        match self {
            Self::Simple(_) => None,
            Self::Extended { continue_on_errors, .. } => continue_on_errors.as_ref(),
        }
    }

    pub const fn per_package(&self) -> bool {
        match self {
            Self::Simple(_) => false,
            Self::Extended { per_package, .. } => *per_package,
        }
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
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
