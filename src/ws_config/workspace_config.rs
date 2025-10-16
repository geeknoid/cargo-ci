use crate::ws_config::Jobs;
use crate::ws_config::Tools;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceConfig {
    #[serde(default)]
    tools: Tools,

    #[serde(default)]
    jobs: Jobs,

    #[serde(default)]
    variables: HashSet<String>,

    #[serde(skip)]
    directory: PathBuf,
}

impl WorkspaceConfig {
    pub fn load(workspace_root: &Path) -> Result<Self> {
        let t = workspace_root.join("ci.toml");
        let ci_toml = t.as_path();
        let text = fs::read_to_string(ci_toml).with_context(|| format!("Reading cargo-ci configuration from {}", ci_toml.display()))?;

        let mut config: Self = toml::from_str(&text).with_context(|| format!("Unable to parse {}", ci_toml.display()))?;
        config.directory = workspace_root.to_path_buf();

        Ok(config)
    }

    pub const fn tools(&self) -> &Tools {
        &self.tools
    }

    pub const fn jobs(&self) -> &Jobs {
        &self.jobs
    }

    pub const fn variables(&self) -> &HashSet<String> {
        &self.variables
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }
}
