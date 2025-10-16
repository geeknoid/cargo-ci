use core::fmt::Display;
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct ToolId(String);

impl Display for ToolId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
