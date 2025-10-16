use semver::VersionReq;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct InstallInfo {
    version_req: Option<VersionReq>,

    #[serde(default)]
    bins: Vec<String>,

    #[serde(default)]
    features: Vec<String>,

    #[serde(default)]
    all_features: bool,

    #[serde(default)]
    no_default_features: bool,

    #[serde(default)]
    profile: Option<String>,

    #[serde(default)]
    target: Option<String>,

    #[serde(default)]
    rustc: Option<String>,
}

impl InstallInfo {
    #[must_use]
    pub const fn version_req(&self) -> Option<&VersionReq> {
        self.version_req.as_ref()
    }

    #[must_use]
    pub const fn bins(&self) -> &Vec<String> {
        &self.bins
    }

    #[must_use]
    pub const fn features(&self) -> &Vec<String> {
        &self.features
    }

    #[must_use]
    pub const fn all_features(&self) -> bool {
        self.all_features
    }

    #[must_use]
    pub const fn no_default_features(&self) -> bool {
        self.no_default_features
    }

    #[must_use]
    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    #[must_use]
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    #[must_use]
    pub fn rustc(&self) -> Option<&str> {
        self.rustc.as_deref()
    }
}
