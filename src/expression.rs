use anyhow::{Context, anyhow};
use evalexpr::{ContextWithMutableVariables, HashMapContext, Node, Value, build_operator_tree};
use serde::{Deserialize, Deserializer, de};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Expression {
    tree: Node,
}

impl<'de> Deserialize<'de> for Expression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expr = String::deserialize(deserializer)?;
        Self::new(expr).map_err(de::Error::custom)
    }
}

impl Expression {
    pub fn new(expr: impl AsRef<str>) -> anyhow::Result<Self> {
        let tree = build_operator_tree(expr.as_ref()).with_context(|| format!("Failed to parse expression: {}", expr.as_ref()))?;
        Ok(Self { tree })
    }

    pub fn evaluate(&self, variables: &HashMap<String, String>) -> anyhow::Result<bool> {
        let mut context = HashMapContext::new();

        for (k, v) in variables {
            context
                .set_value(k.clone(), Value::String(v.clone()))
                .with_context(|| format!("Failed to set variable: {k}"))?;
        }

        let result = self.tree.eval_with_context(&context).context("Failed to evaluate expression")?;
        match result {
            Value::Boolean(b) => Ok(b),
            _ => Err(anyhow!("Expression did not evaluate to a boolean, got: {result:?}")),
        }
    }
}
