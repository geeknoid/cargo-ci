use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use toml::Value;

/// Configuration for a single package.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct PackageConfig {
    /// Optional array of modifier strings controlling what steps of each jobs execute.
    #[serde(default)]
    pub modifiers: Option<Vec<String>>,
}

impl PackageConfig {
    /// Load package configuration from a Cargo.toml manifest file.
    ///
    /// This reads the `[package.metadata.ci]` or `[workspace.metadata.ci]` section
    /// and extracts the package-level configuration.
    pub fn load(manifest_path: &Path) -> Result<Self> {
        let text = fs::read_to_string(manifest_path).with_context(|| format!("Reading manifest {}", manifest_path.display()))?;
        let value: Value = text.parse::<Value>().context("Parsing Cargo.toml")?;

        // Try to find the ci metadata section
        let ci_metadata = value
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("ci"))
            .or_else(|| {
                let w = value.get("workspace")?;
                let m = w.get("metadata")?;
                m.get("ci")
            });

        // If no ci metadata section exists, return default configuration
        let Some(ci_metadata) = ci_metadata else {
            return Ok(Self::default());
        };

        // Extract modifiers if present
        let modifiers = ci_metadata
            .get("modifiers")
            .and_then(|m| m.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());

        Ok(Self { modifiers })
    }

    /// Get the modifiers for this package, if any.
    #[must_use]
    pub fn modifiers(&self) -> Option<&[String]> {
        self.modifiers.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_project() {
        let project = PackageConfig::default();
        assert!(project.modifiers.is_none());
    }

    #[test]
    fn test_project_with_modifiers() {
        let project = PackageConfig {
            modifiers: Some(vec!["nightly".to_string(), "experimental".to_string()]),
        };
        assert_eq!(project.modifiers().unwrap(), &["nightly", "experimental"]);
    }
}
