use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DotfConfig {
    #[serde(default)]
    pub symlinks: HashMap<String, String>,
    #[serde(default)]
    pub scripts: ScriptsConfig,
    #[serde(default)]
    pub platform: PlatformConfig,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct ScriptsConfig {
    #[serde(default)]
    pub deps: DepsScripts,
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct DepsScripts {
    pub macos: Option<String>,
    pub linux: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct PlatformConfig {
    pub macos: Option<PlatformSymlinks>,
    pub linux: Option<PlatformSymlinks>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlatformSymlinks {
    pub symlinks: HashMap<String, String>,
}
