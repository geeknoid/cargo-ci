use crate::ws_config::{Tool, ToolId};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct Tools(HashMap<ToolId, Tool>);

impl Tools {
    pub fn iter(&self) -> impl Iterator<Item = (&ToolId, &Tool)> {
        self.0.iter()
    }
}
