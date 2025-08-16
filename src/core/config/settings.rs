use serde::{Deserialize, Serialize};
use crate::error::DottResult;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub repository: RepositorySettings,
    pub backup: BackupSettings,
    pub sync: SyncSettings,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepositorySettings {
    pub url: String,
    pub path: String,
    pub branch: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BackupSettings {
    pub enabled: bool,
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncSettings {
    pub auto_check: bool,
    pub merge_strategy: MergeStrategy,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MergeStrategy {
    Rebase,
    Merge,
    Reset,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            repository: RepositorySettings {
                url: String::new(),
                path: dirs::home_dir()
                    .unwrap_or_default()
                    .join(".dott")
                    .join("repo")
                    .to_string_lossy()
                    .to_string(),
                branch: "main".to_string(),
            },
            backup: BackupSettings {
                enabled: true,
                path: dirs::home_dir()
                    .unwrap_or_default()
                    .join(".dott")
                    .join("backups")
                    .to_string_lossy()
                    .to_string(),
            },
            sync: SyncSettings {
                auto_check: true,
                merge_strategy: MergeStrategy::Rebase,
            },
        }
    }
}

impl Settings {
    pub fn new(repository_url: &str) -> Self {
        let mut settings = Self::default();
        settings.repository.url = repository_url.to_string();
        settings
    }
    
    pub fn from_json(json: &str) -> DottResult<Self> {
        serde_json::from_str(json).map_err(|e| e.into())
    }
    
    pub fn to_json(&self) -> DottResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert!(settings.backup.enabled);
        assert_eq!(settings.repository.branch, "main");
        assert!(matches!(settings.sync.merge_strategy, MergeStrategy::Rebase));
    }
    
    #[test]
    fn test_settings_new() {
        let settings = Settings::new("https://github.com/user/dotfiles.git");
        assert_eq!(settings.repository.url, "https://github.com/user/dotfiles.git");
        assert!(settings.backup.enabled);
    }
    
    #[test]
    fn test_settings_serialization() {
        let settings = Settings::new("https://github.com/user/dotfiles.git");
        let json = settings.to_json().unwrap();
        let deserialized = Settings::from_json(&json).unwrap();
        
        assert_eq!(settings.repository.url, deserialized.repository.url);
        assert_eq!(settings.backup.enabled, deserialized.backup.enabled);
    }
    
    #[test]
    fn test_merge_strategy_serialization() {
        let json = r#"{"rebase": "rebase"}"#;
        assert!(json.contains("rebase"));
        
        let settings = Settings::default();
        let json = settings.to_json().unwrap();
        assert!(json.contains("\"merge_strategy\": \"rebase\""));
    }
}