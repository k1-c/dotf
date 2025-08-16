use crate::core::config::{DottConfig, Settings};
use crate::error::{DottError, DottResult};
use crate::traits::{filesystem::FileSystem, prompt::Prompt, repository::Repository};

pub struct InitService<R, F, P> {
    repository: R,
    filesystem: F,
    prompt: P,
}

impl<R: Repository, F: FileSystem, P: Prompt> InitService<R, F, P> {
    pub fn new(repository: R, filesystem: F, prompt: P) -> Self {
        Self {
            repository,
            filesystem,
            prompt,
        }
    }

    pub async fn init(&self, repo_url: Option<String>) -> DottResult<()> {
        // Get repository URL (either provided or prompt for it)
        let url = match repo_url {
            Some(url) => url,
            None => self.prompt_for_repository_url().await?,
        };

        // Validate the repository URL
        self.repository.validate_remote(&url).await
            .map_err(|e| DottError::Repository(format!("Invalid repository URL '{}': {}", url, e)))?;

        // Fetch and validate configuration
        let config = self.repository.fetch_config(&url).await
            .map_err(|e| DottError::Config(format!("Failed to fetch configuration from '{}': {}", url, e)))?;

        self.validate_config(&config)?;

        // Setup local dott directory structure
        self.setup_dott_directory().await?;

        // Clone the repository
        let repo_path = self.filesystem.dott_repo_path();
        self.repository.clone(&url, &repo_path).await?;

        // Create local settings
        let settings = Settings {
            repository_url: url.clone(),
            last_sync: None,
            initialized_at: chrono::Utc::now(),
        };

        self.save_settings(&settings).await?;

        // Success messages will be handled by the CLI layer
        
        Ok(())
    }

    pub async fn reinit(&self, repo_url: String) -> DottResult<()> {
        // Check if already initialized
        if self.is_initialized().await? {
            let should_continue = self.prompt.confirm(
                "Dott is already initialized. This will remove the existing setup. Continue?"
            ).await?;
            
            if !should_continue {
                return Ok(());
            }
            
            // Remove existing repository
            let repo_path = self.filesystem.dott_repo_path();
            if self.filesystem.exists(&repo_path).await? {
                self.filesystem.remove_dir(&repo_path).await?;
            }
        }

        self.init(Some(repo_url)).await
    }

    pub async fn is_initialized(&self) -> DottResult<bool> {
        let settings_path = self.filesystem.dott_settings_path();
        let repo_path = self.filesystem.dott_repo_path();
        
        Ok(self.filesystem.exists(&settings_path).await? && 
           self.filesystem.exists(&repo_path).await?)
    }

    pub async fn get_current_repository_url(&self) -> DottResult<Option<String>> {
        if !self.is_initialized().await? {
            return Ok(None);
        }

        let settings = self.load_settings().await?;
        Ok(Some(settings.repository_url))
    }

    pub async fn validate_current_setup(&self) -> DottResult<()> {
        if !self.is_initialized().await? {
            return Err(DottError::NotInitialized);
        }

        // Check if settings file is valid
        let _settings = self.load_settings().await?;

        // Check if repository directory exists and is valid
        let repo_path = self.filesystem.dott_repo_path();
        if !self.filesystem.exists(&repo_path).await? {
            return Err(DottError::Repository("Repository directory does not exist".to_string()));
        }

        // Try to get repository status
        self.repository.get_status(&repo_path).await?;

        Ok(())
    }

    async fn prompt_for_repository_url(&self) -> DottResult<String> {
        loop {
            let url = self.prompt.input(
                "Enter the repository URL for your dotfiles:",
                None
            ).await?;

            if url.trim().is_empty() {
                continue;
            }

            // Basic URL validation
            if !url.contains("://") && !url.starts_with("git@") {
                continue;
            }

            return Ok(url);
        }
    }

    fn validate_config(&self, config: &DottConfig) -> DottResult<()> {
        // Validate symlinks are not empty paths
        for (target, source) in &config.symlinks {
            if target.trim().is_empty() || source.trim().is_empty() {
                return Err(DottError::Config(
                    format!("Invalid symlink configuration: '{}' -> '{}'", source, target)
                ));
            }
        }

        Ok(())
    }

    async fn setup_dott_directory(&self) -> DottResult<()> {
        let dott_dir = self.filesystem.dott_directory();
        
        // Check if .dott directory already exists
        if self.filesystem.exists(&dott_dir).await? {
            let should_overwrite = self.prompt.confirm(
                &format!("Dott directory already exists at: {}. Do you want to remove it and start fresh?", dott_dir)
            ).await?;
            
            if !should_overwrite {
                return Err(DottError::Operation("Initialization cancelled by user".to_string()));
            }
            
            // Remove existing directory
            self.filesystem.remove_dir(&dott_dir).await?;
        }
        
        // Create main dott directory
        self.filesystem.create_dott_directory().await?;

        // Create subdirectories
        let backup_path = self.filesystem.dott_backup_path();
        self.filesystem.create_dir_all(&backup_path).await?;

        Ok(())
    }

    async fn save_settings(&self, settings: &Settings) -> DottResult<()> {
        let settings_path = self.filesystem.dott_settings_path();
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| DottError::Config(format!("Failed to serialize settings: {}", e)))?;
        
        self.filesystem.write(&settings_path, &content).await?;
        Ok(())
    }

    async fn load_settings(&self) -> DottResult<Settings> {
        let settings_path = self.filesystem.dott_settings_path();
        
        if !self.filesystem.exists(&settings_path).await? {
            return Err(DottError::NotInitialized);
        }

        let content = self.filesystem.read_to_string(&settings_path).await?;
        let settings: Settings = serde_json::from_str(&content)
            .map_err(|e| DottError::Config(format!("Failed to parse settings: {}", e)))?;

        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::dott_config::{ScriptsConfig, PlatformConfig};
    use crate::traits::{
        filesystem::tests::MockFileSystem,
        prompt::tests::MockPrompt,
        repository::tests::MockRepository,
    };
    use std::collections::HashMap;

    fn create_test_config() -> DottConfig {
        DottConfig {
            symlinks: HashMap::from([
                ("~/.vimrc".to_string(), ".vimrc".to_string()),
            ]),
            scripts: ScriptsConfig::default(),
            platform: PlatformConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_init_with_fresh_directory() {
        let filesystem = MockFileSystem::new();
        let mut repository = MockRepository::new();
        let prompt = MockPrompt::new();

        let config = create_test_config();
        repository.set_config_response(config);

        let service = InitService::new(repository, filesystem.clone(), prompt);
        let result = service.init(Some("https://github.com/user/dotfiles.git".to_string())).await;

        assert!(result.is_ok());
        assert!(filesystem.exists(&filesystem.dott_directory()).await.unwrap());
        assert!(filesystem.exists(&filesystem.dott_backup_path()).await.unwrap());
        assert!(filesystem.exists(&filesystem.dott_settings_path()).await.unwrap());
    }

    #[tokio::test]
    async fn test_init_with_existing_directory_user_confirms() {
        let filesystem = MockFileSystem::new();
        let mut repository = MockRepository::new();
        let prompt = MockPrompt::new();

        // Pre-create .dott directory
        filesystem.create_dott_directory().await.unwrap();
        filesystem.add_file(&format!("{}/existing_file", filesystem.dott_directory()), "content");

        let config = create_test_config();
        repository.set_config_response(config);

        // Set prompt to confirm overwrite
        prompt.set_confirm_response(true);

        let service = InitService::new(repository, filesystem.clone(), prompt);
        let result = service.init(Some("https://github.com/user/dotfiles.git".to_string())).await;

        assert!(result.is_ok());
        assert!(filesystem.exists(&filesystem.dott_directory()).await.unwrap());
        assert!(filesystem.exists(&filesystem.dott_settings_path()).await.unwrap());
        // Existing file should be gone after overwrite
        assert!(!filesystem.exists(&format!("{}/existing_file", filesystem.dott_directory())).await.unwrap());
    }

    #[tokio::test]
    async fn test_init_with_existing_directory_user_cancels() {
        let filesystem = MockFileSystem::new();
        let mut repository = MockRepository::new();
        let prompt = MockPrompt::new();

        // Pre-create .dott directory
        filesystem.create_dott_directory().await.unwrap();
        filesystem.add_file(&format!("{}/existing_file", filesystem.dott_directory()), "content");

        let config = create_test_config();
        repository.set_config_response(config);

        // Set prompt to reject overwrite
        prompt.set_confirm_response(false);

        let service = InitService::new(repository, filesystem.clone(), prompt);
        let result = service.init(Some("https://github.com/user/dotfiles.git".to_string())).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DottError::Operation(_)));
        // Existing file should still be there
        assert!(filesystem.exists(&format!("{}/existing_file", filesystem.dott_directory())).await.unwrap());
    }

    #[tokio::test]
    async fn test_reinit_removes_existing_repo() {
        let filesystem = MockFileSystem::new();
        let mut repository = MockRepository::new();
        let prompt = MockPrompt::new();

        // Setup existing initialization
        filesystem.create_dott_directory().await.unwrap();
        let settings = Settings {
            repository_url: "https://github.com/old/repo.git".to_string(),
            last_sync: None,
            initialized_at: chrono::Utc::now(),
        };
        let settings_content = serde_json::to_string_pretty(&settings).unwrap();
        filesystem.write(&filesystem.dott_settings_path(), &settings_content).await.unwrap();
        filesystem.create_dir_all(&filesystem.dott_repo_path()).await.unwrap();

        let config = create_test_config();
        repository.set_config_response(config);

        // Set prompt to confirm reinit (for reinit confirmation)
        prompt.set_confirm_response(true);
        // Set prompt to confirm overwrite (for setup_dott_directory)
        prompt.set_confirm_response(true);

        let service = InitService::new(repository, filesystem.clone(), prompt);
        let result = service.reinit("https://github.com/user/dotfiles.git".to_string()).await;

        assert!(result.is_ok());
        assert!(filesystem.exists(&filesystem.dott_directory()).await.unwrap());
        assert!(filesystem.exists(&filesystem.dott_settings_path()).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_initialized_true() {
        let filesystem = MockFileSystem::new();
        let repository = MockRepository::new();
        let prompt = MockPrompt::new();

        // Setup existing initialization
        filesystem.create_dott_directory().await.unwrap();
        let settings = Settings {
            repository_url: "https://github.com/user/dotfiles.git".to_string(),
            last_sync: None,
            initialized_at: chrono::Utc::now(),
        };
        let settings_content = serde_json::to_string_pretty(&settings).unwrap();
        filesystem.write(&filesystem.dott_settings_path(), &settings_content).await.unwrap();
        filesystem.create_dir_all(&filesystem.dott_repo_path()).await.unwrap();

        let service = InitService::new(repository, filesystem, prompt);
        let result = service.is_initialized().await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_is_initialized_false() {
        let filesystem = MockFileSystem::new();
        let repository = MockRepository::new();
        let prompt = MockPrompt::new();

        let service = InitService::new(repository, filesystem, prompt);
        let result = service.is_initialized().await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_validate_config_invalid_symlink() {
        let filesystem = MockFileSystem::new();
        let repository = MockRepository::new();
        let prompt = MockPrompt::new();

        let service = InitService::new(repository, filesystem, prompt);
        
        let invalid_config = DottConfig {
            symlinks: HashMap::from([
                ("".to_string(), ".vimrc".to_string()), // Empty target
            ]),
            scripts: ScriptsConfig::default(),
            platform: PlatformConfig::default(),
        };

        let result = service.validate_config(&invalid_config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DottError::Config(_)));
    }
}