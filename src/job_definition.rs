use anyhow::{Result, anyhow};
use serde::Deserialize;

use crate::step::Step;

/// Configuration for a CI job with a list of steps to execute.
#[derive(Debug, Deserialize, Clone)]
pub struct JobDefinition {
    /// The job identifier (table name from TOML)
    #[serde(skip)]
    pub id: String,

    /// Optional custom display name for the job
    #[serde(default)]
    pub name: Option<String>,

    /// Steps to run for this job
    pub steps: Vec<Step>,

    /// Names of other jobs that must run before this job
    #[serde(default)]
    pub needs: Vec<String>,

    /// Whether to continue running subsequent jobs even if this job fails
    #[serde(default)]
    pub continue_on_error: bool,
}

impl JobDefinition {
    /// Validate that the job definition meets requirements.
    pub fn validate(&self) -> Result<()> {
        if self.steps.is_empty() {
            return Err(anyhow!("Job '{}' has no steps defined", self.id));
        }

        Ok(())
    }

    /// Get this job's steps.
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Get the display name for this job.
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_basic_job() {
        let toml_str = r#"
            steps = ["cargo build", "cargo test"]
        "#;
        let job: JobDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(job.steps.len(), 2);
        assert_eq!(job.steps[0].command(), "cargo build");
        assert_eq!(job.steps[1].command(), "cargo test");
        assert!(job.name.is_none());
        assert!(job.needs.is_empty());
        assert!(!job.continue_on_error);
    }

    #[test]
    fn test_deserialize_job_with_name() {
        let toml_str = r#"
            name = "Custom Build"
            steps = ["cargo build"]
        "#;
        let job: JobDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(job.name, Some("Custom Build".to_string()));
        assert_eq!(job.steps.len(), 1);
        assert!(job.needs.is_empty());
        assert!(!job.continue_on_error);
    }

    #[test]
    fn test_deserialize_job_with_needs() {
        let toml_str = r#"
            needs = ["previous_job"]
            steps = ["cargo build"]
        "#;
        let job: JobDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(job.needs, vec!["previous_job".to_string()]);
    }

    #[test]
    fn test_validate_empty_job_fails() {
        let job = JobDefinition {
            id: "test_job".to_string(),
            name: None,
            steps: vec![],
            needs: vec![],
            continue_on_error: false,
        };
        let result = job.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no steps"));
    }

    #[test]
    fn test_validate_with_steps_succeeds() {
        let job = JobDefinition {
            id: "test_job".to_string(),
            name: None,
            steps: vec![Step::from("cargo build")],
            needs: vec![],
            continue_on_error: false,
        };
        assert!(job.validate().is_ok());
    }

    #[test]
    fn test_steps_returns_all_steps() {
        let job = JobDefinition {
            id: "test_job".to_string(),
            name: None,
            steps: vec![Step::from("cargo build"), Step::from("cargo test")],
            needs: vec![],
            continue_on_error: false,
        };
        let steps = job.steps();
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].command(), "cargo build");
        assert_eq!(steps[1].command(), "cargo test");
    }

    #[test]
    fn test_id_field_skipped_during_deserialization() {
        let toml_str = r#"
            steps = ["cargo build"]
        "#;
        let job: JobDefinition = toml::from_str(toml_str).unwrap();
        // id should be empty string since it's skipped during deserialization
        assert_eq!(job.id, "");
    }

    #[test]
    fn test_display_name_uses_custom_name() {
        let job = JobDefinition {
            id: "build".to_string(),
            name: Some("Custom Build Name".to_string()),
            steps: vec![Step::from("cargo build")],
            needs: vec![],
            continue_on_error: false,
        };
        assert_eq!(job.display_name(), "Custom Build Name");
    }

    #[test]
    fn test_display_name_fallback_to_id() {
        let job = JobDefinition {
            id: "build".to_string(),
            name: None,
            steps: vec![Step::from("cargo build")],
            needs: vec![],
            continue_on_error: false,
        };
        assert_eq!(job.display_name(), "build");
    }
}
