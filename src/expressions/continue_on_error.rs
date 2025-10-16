use crate::expressions::expression::Expression;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ContinueOnError {
    Bool(bool),
    Expression(Expression),
}

impl ContinueOnError {
    pub fn evaluate(&self, variables: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>) -> anyhow::Result<bool> {
        match self {
            Self::Bool(b) => Ok(*b),
            Self::Expression(expr) => expr.evaluate(variables),
        }
    }
}

impl Default for ContinueOnError {
    fn default() -> Self {
        Self::Bool(false)
    }
}
