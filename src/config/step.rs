use crate::config::StepId;
use crate::expressions::{Conditional, ContinueOnError};
use core::fmt;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;

static EMPTY_VARIABLES: LazyLock<HashMap<String, String>> = LazyLock::new(HashMap::new);

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
#[expect(clippy::large_enum_variant, reason = "Size doesn't matter, this is for the UX")]
pub enum Step {
    Simple(String),

    Extended {
        command: String,
        name: Option<String>,
        id: Option<StepId>,

        #[serde(default, rename = "if")]
        conditional: Conditional,

        #[serde(default)]
        continue_on_error: ContinueOnError,

        #[serde(default)]
        per_package: bool,

        #[serde(default)]
        variables: HashMap<String, String>,
    },
}

impl Step {
    #[must_use]
    pub fn command(&self) -> &str {
        match self {
            Self::Simple(cmd) => cmd,
            Self::Extended { command: run, .. } => run,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Simple(cmd) => cmd,
            Self::Extended { command: run, name, .. } => name.as_deref().unwrap_or(run),
        }
    }

    #[must_use]
    pub const fn id(&self) -> Option<&StepId> {
        match self {
            Self::Simple(_) => None,
            Self::Extended { id, .. } => id.as_ref(),
        }
    }

    #[must_use]
    pub const fn conditional(&self) -> &Conditional {
        match self {
            Self::Simple(_) => &Conditional::Bool(true),
            Self::Extended { conditional, .. } => conditional,
        }
    }

    #[must_use]
    pub const fn continue_on_error(&self) -> &ContinueOnError {
        match self {
            Self::Simple(_) => &ContinueOnError::Bool(false),
            Self::Extended { continue_on_error, .. } => continue_on_error,
        }
    }

    #[must_use]
    pub const fn per_package(&self) -> bool {
        match self {
            Self::Simple(_) => false,
            Self::Extended { per_package, .. } => *per_package,
        }
    }

    #[must_use]
    pub fn variables(&self) -> Box<dyn Iterator<Item = (&str, &str)> + '_> {
        match self {
            Self::Simple(_) => Box::new(EMPTY_VARIABLES.iter().map(|(k, v)| (k.as_str(), v.as_str()))),
            Self::Extended { variables, .. } => Box::new(variables.iter().map(|(k, v)| (k.as_str(), v.as_str()))),
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
