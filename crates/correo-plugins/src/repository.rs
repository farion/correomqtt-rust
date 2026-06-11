use crate::{bundled_plugin_manifests, PluginManifest};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

pub const PLUGIN_REPOSITORY_FORMAT_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginRepositoryDefinition {
    pub repository_format_version: u16,
    pub id: String,
    pub name: String,
    pub plugins: Vec<PluginRepositoryEntry>,
}

impl PluginRepositoryDefinition {
    pub fn from_bundled_plugins(id: impl Into<String>, name: impl Into<String>) -> Self {
        let mut plugins = bundled_plugin_manifests()
            .into_iter()
            .map(PluginRepositoryEntry::bundled)
            .collect::<Vec<_>>();
        plugins.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));

        Self {
            repository_format_version: PLUGIN_REPOSITORY_FORMAT_VERSION,
            id: id.into(),
            name: name.into(),
            plugins,
        }
    }

    pub fn validate(&self) -> Result<(), PluginRepositoryError> {
        if self.repository_format_version != PLUGIN_REPOSITORY_FORMAT_VERSION {
            return Err(PluginRepositoryError::UnsupportedFormatVersion {
                found: self.repository_format_version,
            });
        }

        for entry in &self.plugins {
            entry.manifest.validate()?;
            entry.install_source.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginRepositoryEntry {
    pub manifest: PluginManifest,
    pub install_source: PluginInstallSource,
}

impl PluginRepositoryEntry {
    pub fn bundled(manifest: PluginManifest) -> Self {
        let plugin_id = manifest.id.clone();
        Self {
            manifest,
            install_source: PluginInstallSource::Bundled { plugin_id },
        }
    }

    pub fn local_package(
        manifest: PluginManifest,
        relative_path: impl Into<PathBuf>,
    ) -> Result<Self, PluginRepositoryError> {
        let relative_path = relative_path.into();
        ensure_safe_relative_path(&relative_path)?;
        Ok(Self {
            manifest,
            install_source: PluginInstallSource::LocalPackage {
                path: slash_path(&relative_path),
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PluginInstallSource {
    Bundled { plugin_id: String },
    LocalPackage { path: String },
    Archive { url: String, sha256: String },
}

impl PluginInstallSource {
    pub fn validate(&self) -> Result<(), PluginRepositoryError> {
        match self {
            Self::Bundled { plugin_id } => {
                if plugin_id.trim().is_empty() {
                    Err(PluginRepositoryError::EmptyBundledPluginId)
                } else {
                    Ok(())
                }
            }
            Self::LocalPackage { path } => ensure_safe_relative_path(Path::new(path)),
            Self::Archive { url, sha256 } => {
                if url.trim().is_empty() {
                    Err(PluginRepositoryError::EmptyArchiveUrl)
                } else if sha256.trim().is_empty() {
                    Err(PluginRepositoryError::EmptyArchiveChecksum)
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum PluginRepositoryError {
    #[error("unsupported plugin repository format version {found}; supported version is 1")]
    UnsupportedFormatVersion { found: u16 },
    #[error("plugin repository bundled source has an empty plugin id")]
    EmptyBundledPluginId,
    #[error("plugin repository archive source has an empty URL")]
    EmptyArchiveUrl,
    #[error("plugin repository archive source has an empty SHA-256 checksum")]
    EmptyArchiveChecksum,
    #[error("plugin repository package path must be relative, safe, and non-empty: {path}")]
    UnsafePackagePath { path: PathBuf },
    #[error(transparent)]
    Manifest(#[from] crate::ManifestError),
}

fn ensure_safe_relative_path(path: &Path) -> Result<(), PluginRepositoryError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::Prefix(_) | Component::RootDir | Component::ParentDir
            )
        })
    {
        Err(PluginRepositoryError::UnsafePackagePath {
            path: path.to_path_buf(),
        })
    } else {
        Ok(())
    }
}

fn slash_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
