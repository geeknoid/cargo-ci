use crate::expressions::expression::Expression;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Conditional {
    Bool(bool),
    Expression(Expression),
}

impl Conditional {
    pub fn evaluate(&self, variables: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>) -> anyhow::Result<bool> {
        match self {
            Self::Bool(b) => Ok(*b),
            Self::Expression(expr) => expr.evaluate(variables),
        }
    }
}

impl Default for Conditional {
    fn default() -> Self {
        Self::Bool(true)
    }
}
