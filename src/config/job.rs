use crate::config::Step;
use crate::config::job_id::JobId;
use crate::expressions::{Conditional, ContinueOnError};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    name: Option<String>,
    steps: Vec<Step>,

    #[serde(default)]
    needs: HashSet<JobId>,

    #[serde(default, rename = "if")]
    conditional: Conditional,

    #[serde(default)]
    continue_on_error: ContinueOnError,

    #[serde(default)]
    variables: HashMap<String, String>,
}

impl Job {
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[must_use]
    pub const fn needs(&self) -> &HashSet<JobId> {
        &self.needs
    }

    #[must_use]
    pub const fn steps(&self) -> &Vec<Step> {
        &self.steps
    }

    #[must_use]
    pub const fn conditional(&self) -> &Conditional {
        &self.conditional
    }

    #[must_use]
    pub const fn continue_on_error(&self) -> &ContinueOnError {
        &self.continue_on_error
    }

    pub fn variables(&self) -> impl Iterator<Item = (&str, &str)> {
        self.variables.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
