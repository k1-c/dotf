use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct DottConfig {
    pub repo: RepoConfig,
    #[serde(default)]
    pub symlinks: HashMap<String, String>,
    #[serde(default)]
    pub scripts: ScriptsConfig,
    #[serde(default)]
    pub platform: PlatformConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepoConfig {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ScriptsConfig {
    #[serde(default)]
    pub deps: DepsScripts,
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct DepsScripts {
    pub macos: Option<String>,
    pub linux: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PlatformConfig {
    pub macos: Option<PlatformSymlinks>,
    pub linux: Option<PlatformSymlinks>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlatformSymlinks {
    pub symlinks: HashMap<String, String>,
}