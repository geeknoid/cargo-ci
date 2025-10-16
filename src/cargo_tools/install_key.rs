use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;

#[derive(Deserialize)]
struct RawInstallKey(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstallKey {
    name: String,
    version: Version,
    source: Option<String>,
}

impl InstallKey {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn version(&self) -> &Version {
        &self.version
    }

    #[must_use]
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }
}

impl TryFrom<RawInstallKey> for InstallKey {
    type Error = anyhow::Error;

    fn try_from(raw: RawInstallKey) -> Result<Self> {
        let raw_str = raw.0;

        // Parse the key format: "package_name version (source)"
        let mut parts = raw_str.split_whitespace();
        let name = parts.next().context("Missing package name in install key")?.to_string();
        let version_str = parts.next().context("Missing version in install key")?;
        let version = Version::parse(version_str).with_context(|| format!("invalid version '{version_str}' in install key"))?;

        let source = parts.next().map(|rest| {
            let source_parts: Vec<&str> = core::iter::once(rest).chain(parts).collect();
            let source_str = source_parts.join(" ");

            if source_str.starts_with('(') && source_str.ends_with(')') {
                source_str.strip_prefix('(').and_then(|s| s.strip_suffix(')')).unwrap_or(&source_str).to_string()
            } else {
                source_str
            }
        });

        Ok(Self { name, version, source })
    }
}

impl<'de> Deserialize<'de> for InstallKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawInstallKey::deserialize(deserializer)?;
        Self::try_from(raw).map_err(serde::de::Error::custom)
    }
}
