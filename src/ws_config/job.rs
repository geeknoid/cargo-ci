use crate::continue_on_error::ContinueOnError;
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

    #[serde(rename = "if")]
    cond: Option<Expression>,
    continue_on_error: Option<ContinueOnError>,
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

    pub fn continue_on_error(&self) -> Option<Expression> {
        self.continue_on_error.as_ref().map(ContinueOnError::to_expression)
    }
}
