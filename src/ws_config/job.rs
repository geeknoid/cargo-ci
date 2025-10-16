use crate::expression::Expression;
use crate::ws_config::Step;
use crate::ws_config::job_id::JobId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    name: Option<String>,

    #[serde(default)]
    needs: Vec<JobId>,

    #[serde(default)]
    steps: Vec<Step>,

    cond: Option<Expression>,
    continue_on_errors: Option<Expression>,
}

impl Job {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub const fn needs(&self) -> &Vec<JobId> {
        &self.needs
    }

    pub const fn steps(&self) -> &Vec<Step> {
        &self.steps
    }

    pub const fn cond(&self) -> Option<&Expression> {
        self.cond.as_ref()
    }

    pub const fn continue_on_errors(&self) -> Option<&Expression> {
        self.continue_on_errors.as_ref()
    }
}
