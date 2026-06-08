use crate::capabilities::{CapabilityGrants, HookKind};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use thiserror::Error;

pub const SUPPORTED_MANIFEST_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub manifest_version: u16,
    pub id: String,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub provider: String,
    pub license: String,
    pub compatible_correomqtt: VersionReq,
    pub capabilities: CapabilityGrants,
    pub entrypoints: Vec<PluginEntrypoint>,
    #[serde(default)]
    pub config_schema: Option<ConfigSchemaMetadata>,
}

impl PluginManifest {
    pub fn from_toml_str(input: &str) -> Result<Self, ManifestError> {
        let manifest = toml::from_str::<Self>(input)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.manifest_version != SUPPORTED_MANIFEST_VERSION {
            return Err(ManifestError::UnsupportedManifestVersion {
                found: self.manifest_version,
            });
        }

        ensure_non_empty("id", &self.id)?;
        ensure_non_empty("name", &self.name)?;
        ensure_non_empty("description", &self.description)?;
        ensure_non_empty("provider", &self.provider)?;
        ensure_non_empty("license", &self.license)?;

        let mut seen = BTreeSet::new();
        for entrypoint in &self.entrypoints {
            ensure_non_empty("entrypoints.export", &entrypoint.export)?;
            if !self.capabilities.grants_hook(entrypoint.hook) {
                return Err(ManifestError::EntrypointCapabilityMissing {
                    hook: entrypoint.hook,
                });
            }
            if !seen.insert(entrypoint.hook) {
                return Err(ManifestError::DuplicateEntrypoint {
                    hook: entrypoint.hook,
                });
            }
        }

        Ok(())
    }

    pub fn entrypoint_for(&self, hook: HookKind) -> Option<&PluginEntrypoint> {
        self.entrypoints
            .iter()
            .find(|entrypoint| entrypoint.hook == hook)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginEntrypoint {
    pub hook: HookKind,
    pub export: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigSchemaMetadata {
    pub schema_version: u16,
    #[serde(default)]
    pub document: Value,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("invalid plugin manifest TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("unsupported plugin manifest version {found}; supported version is 1")]
    UnsupportedManifestVersion { found: u16 },
    #[error("plugin manifest field is empty: {0}")]
    EmptyField(&'static str),
    #[error("entrypoint for {hook:?} requires a matching hook capability grant")]
    EntrypointCapabilityMissing { hook: HookKind },
    #[error("duplicate entrypoint for hook {hook:?}")]
    DuplicateEntrypoint { hook: HookKind },
}

fn ensure_non_empty(field: &'static str, value: &str) -> Result<(), ManifestError> {
    if value.trim().is_empty() {
        Err(ManifestError::EmptyField(field))
    } else {
        Ok(())
    }
}
