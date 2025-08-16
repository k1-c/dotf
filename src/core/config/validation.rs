use crate::core::config::DottConfig;
use crate::error::{DottError, DottResult};

pub fn validate_config(config: &DottConfig) -> DottResult<()> {
    // Validate repository info
    if config.repo.name.is_empty() {
        return Err(DottError::Validation("Repository name is required".to_string()));
    }
    
    if config.repo.version.is_empty() {
        return Err(DottError::Validation("Repository version is required".to_string()));
    }
    
    // Validate symlinks
    for (source, target) in &config.symlinks {
        if source.is_empty() || target.is_empty() {
            return Err(DottError::Validation(
                "Symlink source and target cannot be empty".to_string()
            ));
        }
        
        // Check for dangerous paths
        if target == "/" || target == "~" {
            return Err(DottError::Validation(
                format!("Dangerous symlink target: {}", target)
            ));
        }
    }
    
    // Validate scripts
    if let Some(macos_script) = &config.scripts.deps.macos {
        if macos_script.is_empty() {
            return Err(DottError::Validation(
                "macOS dependency script path cannot be empty".to_string()
            ));
        }
    }
    
    if let Some(linux_script) = &config.scripts.deps.linux {
        if linux_script.is_empty() {
            return Err(DottError::Validation(
                "Linux dependency script path cannot be empty".to_string()
            ));
        }
    }
    
    for (name, script) in &config.scripts.custom {
        if name.is_empty() || script.is_empty() {
            return Err(DottError::Validation(
                "Custom script name and path cannot be empty".to_string()
            ));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::dott_config::{RepoConfig, ScriptsConfig, PlatformConfig};
    use std::collections::HashMap;
    
    fn create_valid_config() -> DottConfig {
        DottConfig {
            repo: RepoConfig {
                name: "test-dotfiles".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Test description".to_string()),
                author: Some("Test Author".to_string()),
            },
            symlinks: HashMap::new(),
            scripts: ScriptsConfig::default(),
            platform: PlatformConfig::default(),
        }
    }
    
    #[test]
    fn test_valid_config() {
        let config = create_valid_config();
        assert!(validate_config(&config).is_ok());
    }
    
    #[test]
    fn test_missing_repo_name() {
        let mut config = create_valid_config();
        config.repo.name = String::new();
        
        let result = validate_config(&config);
        assert!(result.is_err());
        if let Err(DottError::Validation(msg)) = result {
            assert!(msg.contains("Repository name"));
        } else {
            panic!("Expected validation error");
        }
    }
    
    #[test]
    fn test_missing_repo_version() {
        let mut config = create_valid_config();
        config.repo.version = String::new();
        
        let result = validate_config(&config);
        assert!(result.is_err());
        if let Err(DottError::Validation(msg)) = result {
            assert!(msg.contains("Repository version"));
        } else {
            panic!("Expected validation error");
        }
    }
    
    #[test]
    fn test_empty_symlink_paths() {
        let mut config = create_valid_config();
        config.symlinks.insert("".to_string(), "target".to_string());
        
        let result = validate_config(&config);
        assert!(result.is_err());
        if let Err(DottError::Validation(msg)) = result {
            assert!(msg.contains("cannot be empty"));
        } else {
            panic!("Expected validation error");
        }
    }
    
    #[test]
    fn test_dangerous_symlink_target() {
        let mut config = create_valid_config();
        config.symlinks.insert("source".to_string(), "/".to_string());
        
        let result = validate_config(&config);
        assert!(result.is_err());
        if let Err(DottError::Validation(msg)) = result {
            assert!(msg.contains("Dangerous symlink target"));
        } else {
            panic!("Expected validation error");
        }
    }
    
    #[test]
    fn test_valid_symlinks() {
        let mut config = create_valid_config();
        config.symlinks.insert("nvim".to_string(), "~/.config/nvim".to_string());
        config.symlinks.insert("zshrc".to_string(), "~/.zshrc".to_string());
        
        assert!(validate_config(&config).is_ok());
    }
    
    #[test]
    fn test_empty_script_paths() {
        let mut config = create_valid_config();
        config.scripts.deps.macos = Some("".to_string());
        
        let result = validate_config(&config);
        assert!(result.is_err());
        if let Err(DottError::Validation(msg)) = result {
            assert!(msg.contains("macOS dependency script"));
        } else {
            panic!("Expected validation error");
        }
    }
    
    #[test]
    fn test_valid_scripts() {
        let mut config = create_valid_config();
        config.scripts.deps.macos = Some("scripts/install-macos.sh".to_string());
        config.scripts.deps.linux = Some("scripts/install-linux.sh".to_string());
        config.scripts.custom.insert("vim-plugins".to_string(), "scripts/vim.sh".to_string());
        
        assert!(validate_config(&config).is_ok());
    }
}