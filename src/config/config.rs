use crate::config::Tools;
use crate::config::{JobId, Jobs};
use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
#[serde(try_from = "RawConfig")]
pub struct Config {
    tools: Tools,
    jobs: Jobs,
    passthrough_env_variables: HashSet<String>,
    default_jobs: HashSet<JobId>,
    variables: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    tools: Tools,

    #[serde(default)]
    jobs: Jobs,

    #[serde(default)]
    default_jobs: HashSet<JobId>,

    #[serde(default)]
    passthrough_env_variables: HashSet<String>,

    #[serde(default)]
    passthrough_env_variables_windows: HashSet<String>,

    #[serde(default)]
    passthrough_env_variables_linux: HashSet<String>,

    #[serde(default)]
    passthrough_env_variables_macos: HashSet<String>,

    #[serde(default)]
    variables: HashMap<String, String>,
}

impl TryFrom<RawConfig> for Config {
    type Error = anyhow::Error;

    fn try_from(raw_config: RawConfig) -> Result<Self, Self::Error> {
        for job_id in &raw_config.default_jobs {
            if raw_config.jobs.get_job(job_id).is_none() {
                return Err(anyhow!("default job '{job_id}' is not defined in the [jobs] section"));
            }
        }

        let mut passthrough_env_variables = raw_config.passthrough_env_variables;
        if cfg!(target_os = "windows") {
            passthrough_env_variables.extend(raw_config.passthrough_env_variables_windows);
        } else if cfg!(target_os = "linux") {
            passthrough_env_variables.extend(raw_config.passthrough_env_variables_linux);
        } else if cfg!(target_os = "macos") {
            passthrough_env_variables.extend(raw_config.passthrough_env_variables_macos);
        }

        Ok(Self {
            tools: raw_config.tools,
            jobs: raw_config.jobs,
            passthrough_env_variables,
            default_jobs: raw_config.default_jobs,
            variables: raw_config.variables,
        })
    }
}

impl Config {
    pub fn load(workspace_root: &Path, config_path: Option<&PathBuf>) -> Result<Self> {
        let (ci_path, text) = Self::read_config(workspace_root, config_path)?;

        let extension = ci_path.extension().and_then(|s| s.to_str()).unwrap_or("");
        match extension {
            "toml" => toml::from_str(&text).map_err(Into::into),
            "yml" | "yaml" => serde_yaml::from_str(&text).map_err(Into::into),
            "json" => serde_json::from_str(&text).map_err(Into::into),
            _ => Err(anyhow!("unsupported configuration file extension: {extension}")),
        }
    }

    #[expect(clippy::similar_names, reason = "Yep, indeed")]
    fn read_config(workspace_root: &Path, config_path: Option<&PathBuf>) -> Result<(PathBuf, String)> {
        let path = if let Some(path) = config_path {
            path.clone()
        } else {
            let yml = workspace_root.join("ci.yml");
            let yaml = workspace_root.join("ci.yaml");
            let json = workspace_root.join("ci.json");
            let toml = workspace_root.join("ci.toml");

            if toml.exists() {
                toml
            } else if yml.exists() {
                yml
            } else if yaml.exists() {
                yaml
            } else if json.exists() {
                json
            } else {
                return Err(anyhow!(
                    "no configuration file found (looked for ci.toml, ci.yml, ci.yaml, and ci.json)"
                ));
            }
        };

        let text = fs::read_to_string(&path).with_context(|| format!("Reading cargo-ci configuration from {}", path.display()))?;
        Ok((path, text))
    }

    #[must_use]
    pub const fn tools(&self) -> &Tools {
        &self.tools
    }

    #[must_use]
    pub const fn jobs(&self) -> &Jobs {
        &self.jobs
    }

    #[must_use]
    pub const fn passthrough_env_variables(&self) -> &HashSet<String> {
        &self.passthrough_env_variables
    }

    #[must_use]
    pub const fn default_jobs(&self) -> &HashSet<JobId> {
        &self.default_jobs
    }

    pub fn variables(&self) -> impl Iterator<Item = (&str, &str)> {
        self.variables.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
