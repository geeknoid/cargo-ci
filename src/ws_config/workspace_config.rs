use crate::ws_config::Tools;
use crate::ws_config::{JobId, Jobs};
use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
#[serde(try_from = "RawWorkspaceConfig")]
pub struct WorkspaceConfig {
    tools: Tools,
    jobs: Jobs,
    passthrough_env_variables: HashSet<String>,
    default_jobs: Option<HashSet<JobId>>,
    directory: PathBuf,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWorkspaceConfig {
    #[serde(default)]
    tools: Tools,

    #[serde(default)]
    jobs: Jobs,

    #[serde(default)]
    default_jobs: Option<HashSet<JobId>>,

    #[serde(default)]
    passthrough_env_variables: HashSet<String>,

    #[serde(default)]
    passthrough_env_variables_windows: HashSet<String>,

    #[serde(default)]
    passthrough_env_variables_linux: HashSet<String>,

    #[serde(default)]
    passthrough_env_variables_macos: HashSet<String>,
}

impl TryFrom<RawWorkspaceConfig> for WorkspaceConfig {
    type Error = anyhow::Error;

    fn try_from(raw_config: RawWorkspaceConfig) -> Result<Self, Self::Error> {
        if let Some(default_jobs) = &raw_config.default_jobs {
            for job_id in default_jobs {
                if raw_config.jobs.get_job(job_id).is_none() {
                    return Err(anyhow!("default job '{job_id}' is not defined in the [jobs] section"));
                }
            }
        }

        let mut vars = raw_config.passthrough_env_variables;
        if cfg!(target_os = "windows") {
            vars.extend(raw_config.passthrough_env_variables_windows);
        } else if cfg!(target_os = "linux") {
            vars.extend(raw_config.passthrough_env_variables_linux);
        } else if cfg!(target_os = "macos") {
            vars.extend(raw_config.passthrough_env_variables_macos);
        }

        Ok(Self {
            tools: raw_config.tools,
            jobs: raw_config.jobs,
            passthrough_env_variables: vars,
            default_jobs: raw_config.default_jobs,
            directory: PathBuf::new(),
        })
    }
}

impl WorkspaceConfig {
    pub fn load(workspace_root: &Path, config_path: Option<&PathBuf>) -> Result<Self> {
        let (ci_path, text) = Self::read_config(workspace_root, config_path)?;

        let mut config: Self = Self::deserialize(&ci_path, &text).with_context(|| format!("Unable to parse {}", ci_path.display()))?;
        config.directory = workspace_root.to_path_buf();

        Ok(config)
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

    fn deserialize(path: &Path, text: &str) -> Result<Self> {
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        match extension {
            "toml" => toml::from_str(text).map_err(Into::into),
            "yml" | "yaml" => serde_yaml::from_str(text).map_err(Into::into),
            "json" => serde_json::from_str(text).map_err(Into::into),
            _ => Err(anyhow!("unsupported configuration file extension: {extension}")),
        }
    }

    pub const fn tools(&self) -> &Tools {
        &self.tools
    }

    pub const fn jobs(&self) -> &Jobs {
        &self.jobs
    }

    pub const fn variables(&self) -> &HashSet<String> {
        &self.passthrough_env_variables
    }

    pub const fn default_jobs(&self) -> Option<&HashSet<JobId>> {
        self.default_jobs.as_ref()
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }
}
