use serde::{Deserialize, Serialize};
use crate::error::DottResult;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub repository_url: String,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub initialized_at: chrono::DateTime<chrono::Utc>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            repository_url: String::new(),
            last_sync: None,
            initialized_at: chrono::Utc::now(),
        }
    }
}

impl Settings {
    pub fn new(repository_url: &str) -> Self {
        Self {
            repository_url: repository_url.to_string(),
            last_sync: None,
            initialized_at: chrono::Utc::now(),
        }
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
        assert!(settings.repository_url.is_empty());
        assert!(settings.last_sync.is_none());
    }
    
    #[test]
    fn test_settings_new() {
        let settings = Settings::new("https://github.com/user/dotfiles.git");
        assert_eq!(settings.repository_url, "https://github.com/user/dotfiles.git");
        assert!(settings.last_sync.is_none());
    }
    
    #[test]
    fn test_settings_serialization() {
        let settings = Settings::new("https://github.com/user/dotfiles.git");
        let json = settings.to_json().unwrap();
        let deserialized = Settings::from_json(&json).unwrap();
        
        assert_eq!(settings.repository_url, deserialized.repository_url);
        assert_eq!(settings.last_sync, deserialized.last_sync);
    }
}