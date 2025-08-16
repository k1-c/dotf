use crate::core::config::{DottConfig, Settings};
use crate::error::{DottError, DottResult};
use crate::traits::{filesystem::FileSystem, prompt::Prompt};

pub struct ConfigService<F, P> {
    filesystem: F,
    prompt: P,
}

impl<F: FileSystem, P: Prompt> ConfigService<F, P> {
    pub fn new(filesystem: F, prompt: P) -> Self {
        Self {
            filesystem,
            prompt,
        }
    }

    pub async fn show_repository_config(&self) -> DottResult<String> {
        let config_path = format!("{}/dott.toml", self.filesystem.dott_repo_path());
        
        if !self.filesystem.exists(&config_path).await? {
            return Err(DottError::Config("Repository configuration file (dott.toml) not found".to_string()));
        }

        self.filesystem.read_to_string(&config_path).await
    }

    pub async fn show_settings(&self) -> DottResult<Settings> {
        let settings_path = self.filesystem.dott_settings_path();
        
        if !self.filesystem.exists(&settings_path).await? {
            return Err(DottError::Config("Settings file not found. Run 'dott init' first.".to_string()));
        }

        let content = self.filesystem.read_to_string(&settings_path).await?;
        let settings: Settings = serde_json::from_str(&content)
            .map_err(|e| DottError::Serialization(format!("Failed to parse settings: {}", e)))?;
        
        Ok(settings)
    }

    pub async fn edit_settings(&self) -> DottResult<()> {
        let settings_path = self.filesystem.dott_settings_path();
        
        if !self.filesystem.exists(&settings_path).await? {
            return Err(DottError::Config("Settings file not found. Run 'dott init' first.".to_string()));
        }

        let current_settings = self.show_settings().await?;
        
        // Interactive editing
        println!("📝 Current Settings:");
        println!("Repository URL: {}", current_settings.repository_url);
        println!("Initialized: {}", current_settings.initialized_at.format("%Y-%m-%d %H:%M:%S"));
        if let Some(last_sync) = current_settings.last_sync {
            println!("Last Sync: {}", last_sync.format("%Y-%m-%d %H:%M:%S"));
        } else {
            println!("Last Sync: Never");
        }
        println!();

        let should_edit = self.prompt.confirm("Do you want to edit the repository URL?").await?;
        
        if should_edit {
            let new_url = self.prompt.input(
                "Enter new repository URL:",
                Some(&current_settings.repository_url)
            ).await?;

            let updated_settings = Settings {
                repository_url: new_url,
                last_sync: current_settings.last_sync,
                initialized_at: current_settings.initialized_at,
            };

            let settings_content = serde_json::to_string_pretty(&updated_settings)
                .map_err(|e| DottError::Serialization(e.to_string()))?;
            
            self.filesystem.write(&settings_path, &settings_content).await?;
            
            println!("✅ Settings updated successfully!");
        } else {
            println!("📄 No changes made.");
        }

        Ok(())
    }

    pub async fn validate_config(&self) -> DottResult<ConfigValidationResult> {
        let config_path = format!("{}/dott.toml", self.filesystem.dott_repo_path());
        
        if !self.filesystem.exists(&config_path).await? {
            return Ok(ConfigValidationResult {
                is_valid: false,
                errors: vec!["Repository configuration file (dott.toml) not found".to_string()],
                warnings: vec![],
                config: None,
            });
        }

        let content = self.filesystem.read_to_string(&config_path).await?;
        
        let config: DottConfig = match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                return Ok(ConfigValidationResult {
                    is_valid: false,
                    errors: vec![format!("Failed to parse dott.toml: {}", e)],
                    warnings: vec![],
                    config: None,
                });
            }
        };

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate symlinks
        let repo_path = self.filesystem.dott_repo_path();
        
        for (target, source) in &config.symlinks {
            let source_path = format!("{}/{}", repo_path, source);
            if !self.filesystem.exists(&source_path).await? {
                warnings.push(format!("Symlink source not found: {}", source));
            }
            
            if target.contains("..") {
                errors.push(format!("Dangerous symlink target (contains '..'): {}", target));
            }
        }

        // Validate scripts
        let scripts = &config.scripts;
        
        // Check dependency scripts
        if let Some(ref macos_script) = scripts.deps.macos {
            let full_path = format!("{}/{}", repo_path, macos_script);
            if !self.filesystem.exists(&full_path).await? {
                warnings.push(format!("Dependencies script not found for macos: {}", macos_script));
            }
        }
        
        if let Some(ref linux_script) = scripts.deps.linux {
            let full_path = format!("{}/{}", repo_path, linux_script);
            if !self.filesystem.exists(&full_path).await? {
                warnings.push(format!("Dependencies script not found for linux: {}", linux_script));
            }
        }

        // Check custom scripts
        for (name, script_path) in &scripts.custom {
            let full_path = format!("{}/{}", repo_path, script_path);
            if !self.filesystem.exists(&full_path).await? {
                warnings.push(format!("Custom script '{}' not found: {}", name, script_path));
            }
        }

        Ok(ConfigValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            config: Some(config),
        })
    }

    pub async fn show_config_summary(&self) -> DottResult<ConfigSummary> {
        let validation = self.validate_config().await?;
        
        if !validation.is_valid {
            return Ok(ConfigSummary {
                is_valid: false,
                symlinks_count: 0,
                scripts_count: 0,
                platforms_supported: vec![],
                errors: validation.errors,
                warnings: validation.warnings,
            });
        }

        let config = validation.config.unwrap();
        
        let symlinks_count = config.symlinks.len();

        let mut scripts_count = config.scripts.custom.len();
        if config.scripts.deps.macos.is_some() {
            scripts_count += 1;
        }
        if config.scripts.deps.linux.is_some() {
            scripts_count += 1;
        }

        let mut platforms_supported = Vec::new();
        if config.scripts.deps.macos.is_some() {
            platforms_supported.push("macos".to_string());
        }
        if config.scripts.deps.linux.is_some() {
            platforms_supported.push("linux".to_string());
        }
        platforms_supported.sort();
        platforms_supported.dedup();

        Ok(ConfigSummary {
            is_valid: true,
            symlinks_count,
            scripts_count,
            platforms_supported,
            errors: validation.errors,
            warnings: validation.warnings,
        })
    }
}

#[derive(Debug)]
pub struct ConfigValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub config: Option<DottConfig>,
}

#[derive(Debug)]
pub struct ConfigSummary {
    pub is_valid: bool,
    pub symlinks_count: usize,
    pub scripts_count: usize,
    pub platforms_supported: Vec<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::dott_config::{ScriptsConfig, DepsScripts};
    use crate::traits::{
        filesystem::tests::MockFileSystem,
        prompt::tests::MockPrompt,
    };
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_service() -> (ConfigService<MockFileSystem, MockPrompt>, MockFileSystem, MockPrompt) {
        let filesystem = MockFileSystem::new();
        let prompt = MockPrompt::new();
        let service = ConfigService::new(filesystem.clone(), prompt.clone());
        (service, filesystem, prompt)
    }

    fn create_test_config() -> DottConfig {
        let mut symlinks = HashMap::new();
        symlinks.insert(".vimrc".to_string(), "vim/vimrc".to_string());
        symlinks.insert(".bashrc".to_string(), "bash/bashrc".to_string());

        let mut custom_scripts = HashMap::new();
        custom_scripts.insert("setup".to_string(), "scripts/setup.sh".to_string());

        DottConfig {
            symlinks,
            scripts: ScriptsConfig {
                deps: DepsScripts {
                    macos: None,
                    linux: Some("scripts/install-linux.sh".to_string()),
                },
                custom: custom_scripts,
            },
            platform: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_show_repository_config_success() {
        let (service, filesystem, _) = create_test_service();
        
        let config = create_test_config();
        let config_content = toml::to_string_pretty(&config).unwrap();
        let config_path = format!("{}/dott.toml", filesystem.dott_repo_path());
        
        filesystem.add_file(&config_path, &config_content);
        
        let result = service.show_repository_config().await.unwrap();
        assert!(result.contains("setup"));
        assert!(result.contains("install-linux"));
    }

    #[tokio::test]
    async fn test_show_repository_config_not_found() {
        let (service, _, _) = create_test_service();
        
        let result = service.show_repository_config().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_show_settings_success() {
        let (service, filesystem, _) = create_test_service();
        
        let settings = Settings {
            repository_url: "https://github.com/user/dotfiles".to_string(),
            last_sync: Some(Utc::now()),
            initialized_at: Utc::now(),
        };
        
        let settings_content = serde_json::to_string_pretty(&settings).unwrap();
        filesystem.add_file(&filesystem.dott_settings_path(), &settings_content);
        
        let result = service.show_settings().await.unwrap();
        assert_eq!(result.repository_url, "https://github.com/user/dotfiles");
    }

    #[tokio::test]
    async fn test_validate_config_success() {
        let (service, filesystem, _) = create_test_service();
        
        let config = create_test_config();
        let config_content = toml::to_string_pretty(&config).unwrap();
        let config_path = format!("{}/dott.toml", filesystem.dott_repo_path());
        
        filesystem.add_file(&config_path, &config_content);
        
        // Add source files to avoid warnings
        let repo_path = filesystem.dott_repo_path();
        filesystem.add_file(&format!("{}/vim/vimrc", repo_path), "\" vim config");
        filesystem.add_file(&format!("{}/bash/bashrc", repo_path), "# bash config");
        filesystem.add_file(&format!("{}/scripts/install-linux.sh", repo_path), "#!/bin/bash");
        filesystem.add_file(&format!("{}/scripts/setup.sh", repo_path), "#!/bin/bash");
        
        let result = service.validate_config().await.unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.config.is_some());
    }

    #[tokio::test]
    async fn test_validate_config_with_warnings() {
        let (service, filesystem, _) = create_test_service();
        
        let config = create_test_config();
        let config_content = toml::to_string_pretty(&config).unwrap();
        let config_path = format!("{}/dott.toml", filesystem.dott_repo_path());
        
        filesystem.add_file(&config_path, &config_content);
        // Don't add source files to trigger warnings
        
        let result = service.validate_config().await.unwrap();
        assert!(result.is_valid); // Still valid, just warnings
        assert!(result.errors.is_empty());
        assert!(!result.warnings.is_empty());
        assert!(result.warnings.iter().any(|w| w.contains("not found")));
    }

    #[tokio::test]
    async fn test_show_config_summary() {
        let (service, filesystem, _) = create_test_service();
        
        let config = create_test_config();
        let config_content = toml::to_string_pretty(&config).unwrap();
        let config_path = format!("{}/dott.toml", filesystem.dott_repo_path());
        
        filesystem.add_file(&config_path, &config_content);
        
        let summary = service.show_config_summary().await.unwrap();
        assert!(summary.is_valid);
        assert_eq!(summary.symlinks_count, 2);
        assert_eq!(summary.scripts_count, 2);
        assert!(summary.platforms_supported.contains(&"linux".to_string()));
    }
}