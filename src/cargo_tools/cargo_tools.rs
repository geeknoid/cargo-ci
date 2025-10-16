use crate::cargo_tools::{InstallInfo, InstallKey};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CargoTools {
    #[serde(default)]
    installs: HashMap<InstallKey, InstallInfo>,
}

impl CargoTools {
    /// Read the `.crates2.json` file from the Cargo home directory.
    pub fn read() -> Result<Self> {
        let path = Self::crates2_path()?;
        let contents = fs::read_to_string(&path).with_context(|| format!("unable to read .crates2.json from {}", path.display()))?;
        serde_json::from_str(&contents).with_context(|| format!("Unable to parse .crates2.json from {}", path.display()))
    }

    fn crates2_path() -> Result<PathBuf> {
        let cargo_home = home::cargo_home().context("Unable to determine Cargo home directory")?;
        Ok(cargo_home.join(".crates2.json"))
    }

    pub fn installed(&self) -> impl Iterator<Item = (&InstallKey, &InstallInfo)> {
        self.installs.iter()
    }

    #[must_use]
    pub fn is_installed(&self, name: &str) -> bool {
        self.installs.keys().any(|key| key.name() == name)
    }

    #[must_use]
    pub fn get_install(&self, name: &str) -> Option<(&InstallKey, &InstallInfo)> {
        self.installs.iter().find(|(key, _)| key.name() == name)
    }
}
