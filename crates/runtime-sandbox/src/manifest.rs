use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("failed to read manifest: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse manifest: {0}")]
    Parse(#[from] toml::de::Error),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppManifest {
    pub app: AppSection,
    pub window: Option<WindowSection>,
    pub permissions: Option<PermissionsSection>,
    pub build: Option<BuildSection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppSection {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowSection {
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resizable: Option<bool>,
    pub decorations: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionsSection {
    pub network: Option<Vec<String>>,
    pub clipboard: Option<Vec<String>>,
    pub filesystem: Option<FilesystemSection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilesystemSection {
    #[serde(rename = "user-files")]
    pub user_files: Option<FilesystemPermission>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FilesystemPermission {
    Portal,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildSection {
    pub frontend: Option<String>,
    pub assets: Option<Vec<String>>,
}

impl AppManifest {
    pub fn from_file(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)?;
        let manifest: AppManifest = toml::from_str(&content)?;
        Ok(manifest)
    }

    pub fn has_permission(&self, feature: &str) -> bool {
        let Some(ref perms) = self.permissions else {
            return false;
        };
        match feature {
            "network" => perms.network.as_ref().is_some_and(|v| !v.is_empty()),
            "clipboard" => perms.clipboard.as_ref().is_some_and(|v| !v.is_empty()),
            "filesystem" => perms.filesystem.is_some(),
            _ => false,
        }
    }
}
