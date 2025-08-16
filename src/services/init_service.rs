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

        // Show success message
        println!("✅ Successfully initialized dott with repository: {}", url);
        println!("📁 Dott directory created at: {}", self.filesystem.dott_directory());
        println!("🔗 Repository cloned to: {}", repo_path);
        
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
                println!("❌ Repository URL cannot be empty. Please try again.");
                continue;
            }

            // Basic URL validation
            if !url.contains("://") && !url.starts_with("git@") {
                println!("❌ Invalid URL format. Please provide a valid git repository URL.");
                continue;
            }

            return Ok(url);
        }
    }

    fn validate_config(&self, config: &DottConfig) -> DottResult<()> {
        // Validate repo config
        if config.repo.name.trim().is_empty() {
            return Err(DottError::Config("Repository name cannot be empty".to_string()));
        }

        if config.repo.version.trim().is_empty() {
            return Err(DottError::Config("Repository version cannot be empty".to_string()));
        }

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