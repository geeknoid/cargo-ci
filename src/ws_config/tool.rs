use cargo_metadata::semver::Version;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
pub enum Tool {
    Simple(Version),

    Extended {
        version: Version,
        index: Option<String>,
        registry: Option<String>,
        git: Option<String>,
        branch: Option<String>,
        tag: Option<String>,
        rev: Option<String>,
        path: Option<String>,
        root: Option<String>,
    },
}

impl Tool {
    pub const fn version(&self) -> &Version {
        match self {
            Self::Simple(ver) => ver,
            Self::Extended { version, .. } => version,
        }
    }

    pub const fn index(&self) -> Option<&String> {
        match self {
            Self::Simple(_) => None,
            Self::Extended { index, .. } => index.as_ref(),
        }
    }

    pub const fn registry(&self) -> Option<&String> {
        match self {
            Self::Simple(_) => None,
            Self::Extended { registry, .. } => registry.as_ref(),
        }
    }

    pub const fn git(&self) -> Option<&String> {
        match self {
            Self::Simple(_) => None,
            Self::Extended { git, .. } => git.as_ref(),
        }
    }

    pub const fn branch(&self) -> Option<&String> {
        match self {
            Self::Simple(_) => None,
            Self::Extended { branch, .. } => branch.as_ref(),
        }
    }

    pub const fn tag(&self) -> Option<&String> {
        match self {
            Self::Simple(_) => None,
            Self::Extended { tag, .. } => tag.as_ref(),
        }
    }

    pub const fn rev(&self) -> Option<&String> {
        match self {
            Self::Simple(_) => None,
            Self::Extended { rev, .. } => rev.as_ref(),
        }
    }

    pub const fn path(&self) -> Option<&String> {
        match self {
            Self::Simple(_) => None,
            Self::Extended { path, .. } => path.as_ref(),
        }
    }

    pub const fn root(&self) -> Option<&String> {
        match self {
            Self::Simple(_) => None,
            Self::Extended { root, .. } => root.as_ref(),
        }
    }
}
