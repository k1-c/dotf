use serde::{Deserialize, Serialize};
use crate::error::DottResult;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub repository: Repository,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub initialized_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Repository {
    pub remote: String,
    pub branch: Option<String>,
    pub local: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            repository: Repository::default(),
            last_sync: None,
            initialized_at: chrono::Utc::now(),
        }
    }
}

impl Default for Repository {
    fn default() -> Self {
        Self {
            remote: String::new(),
            branch: None,
            local: None,
        }
    }
}

impl Settings {
    pub fn new(repository_url: &str) -> Self {
        Self {
            repository: Repository {
                remote: repository_url.to_string(),
                branch: None,
                local: None,
            },
            last_sync: None,
            initialized_at: chrono::Utc::now(),
        }
    }
    
    pub fn new_with_details(repository_url: &str, branch: Option<String>, local_path: Option<String>) -> Self {
        Self {
            repository: Repository {
                remote: repository_url.to_string(),
                branch,
                local: local_path,
            },
            last_sync: None,
            initialized_at: chrono::Utc::now(),
        }
    }
    
    pub fn from_toml(toml: &str) -> DottResult<Self> {
        toml::from_str(toml).map_err(|e| e.into())
    }
    
    pub fn to_toml(&self) -> DottResult<String> {
        toml::to_string_pretty(self).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert!(settings.repository.remote.is_empty());
        assert!(settings.last_sync.is_none());
    }
    
    #[test]
    fn test_settings_new() {
        let settings = Settings::new("https://github.com/user/dotfiles.git");
        assert_eq!(settings.repository.remote, "https://github.com/user/dotfiles.git");
        assert!(settings.last_sync.is_none());
    }
    
    #[test]
    fn test_settings_serialization() {
        let settings = Settings::new_with_details(
            "https://github.com/user/dotfiles.git",
            Some("main".to_string()),
            Some("/home/user/dotfiles".to_string())
        );
        let toml = settings.to_toml().unwrap();
        let deserialized = Settings::from_toml(&toml).unwrap();
        
        assert_eq!(settings.repository.remote, deserialized.repository.remote);
        assert_eq!(settings.repository.branch, deserialized.repository.branch);
        assert_eq!(settings.repository.local, deserialized.repository.local);
        assert_eq!(settings.last_sync, deserialized.last_sync);
    }
}