use anyhow::{Context, anyhow};
use evalexpr::{ContextWithMutableVariables, HashMapContext, Node, Value, build_operator_tree};
use serde::{Deserialize, Deserializer, de};

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

    pub fn evaluate(&self, variables: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>) -> anyhow::Result<bool> {
        let mut context = HashMapContext::new();

        for (k, v) in variables {
            context
                .set_value(k.as_ref().to_string(), Value::String(v.as_ref().to_string()))
                .with_context(|| format!("unable to set variable: {}", k.as_ref()))?;
        }

        let result = self.tree.eval_with_context(&context).context("Failed to evaluate expression")?;
        match result {
            Value::Boolean(b) => Ok(b),
            _ => Err(anyhow!("expression did not evaluate to a boolean, got: '{result}'")),
        }
    }
}
