use crate::expression::Expression;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ContinueOnError {
    Bool(bool),
    Expression(Expression),
}

impl ContinueOnError {
    pub fn to_expression(&self) -> Expression {
        match self {
            Self::Bool(b) => Expression::new(b.to_string()).expect("Guaranteed to work"),
            Self::Expression(e) => e.clone(),
        }
    }
}
