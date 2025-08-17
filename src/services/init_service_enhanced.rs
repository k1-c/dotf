//! Enhanced init service with progress callbacks for animations

use crate::cli::ui::InstallStage;
use crate::core::config::{DotfConfig, Repository as RepositoryConfig, Settings};
use crate::error::{DotfError, DotfResult};
use crate::traits::{filesystem::FileSystem, prompt::Prompt, repository::Repository};

/// Progress callback function type
pub type ProgressCallback = Box<dyn Fn(&InstallStage) + Send + Sync>;

pub struct EnhancedInitService<R, F, P> {
    repository: R,
    filesystem: F,
    prompt: P,
}

impl<R: Repository, F: FileSystem, P: Prompt> EnhancedInitService<R, F, P> {
    pub fn new(repository: R, filesystem: F, prompt: P) -> Self {
        Self {
            repository,
            filesystem,
            prompt,
        }
    }

    pub async fn init_with_progress<C>(
        &self,
        repo_url: Option<String>,
        progress_callback: C,
    ) -> DotfResult<String>
    where
        C: Fn(&InstallStage) + Send + Sync,
    {
        progress_callback(&InstallStage::Welcome);

        // Get repository URL (either provided or prompt for it)
        let url = match repo_url {
            Some(url) => {
                // URL provided, proceed to validation
                progress_callback(&InstallStage::ValidatingRepository);
                url
            }
            None => {
                // No URL provided, prompt for it first
                let prompted_url = self.prompt_for_repository_url().await?;
                // Now validate the prompted URL
                progress_callback(&InstallStage::ValidatingRepository);
                prompted_url
            }
        };

        // Validate the repository URL
        self.repository.validate_remote(&url).await.map_err(|e| {
            DotfError::Repository(format!("Invalid repository URL '{}': {}", url, e))
        })?;

        // Get default branch and prompt for branch selection
        progress_callback(&InstallStage::SelectingBranch);
        let default_branch = self
            .repository
            .get_default_branch(&url)
            .await
            .unwrap_or_else(|_| "main".to_string());

        let selected_branch = self.prompt_for_branch(&default_branch).await?;

        // Validate that the selected branch exists
        if !self
            .repository
            .branch_exists(&url, &selected_branch)
            .await?
        {
            return Err(DotfError::Repository(format!(
                "Branch '{}' does not exist in repository '{}'",
                selected_branch, url
            )));
        }

        // Fetch and validate configuration
        progress_callback(&InstallStage::FetchingConfiguration);
        let config = self
            .repository
            .fetch_config_from_branch(&url, &selected_branch)
            .await
            .map_err(|e| {
                DotfError::Config(format!(
                    "Failed to fetch configuration from '{}' branch '{}': {}",
                    url, selected_branch, e
                ))
            })?;

        self.validate_config(&config)?;

        // Setup local dotf directory structure
        progress_callback(&InstallStage::SettingUpDirectories);
        self.setup_dotf_directory().await?;

        // Clone the repository
        progress_callback(&InstallStage::CloningRepository);
        let repo_path = self.filesystem.dotf_repo_path();
        self.repository
            .clone_branch(&url, &selected_branch, &repo_path)
            .await?;

        // Create local settings
        progress_callback(&InstallStage::FinalizeSetup);
        let settings = Settings {
            repository: RepositoryConfig {
                remote: url.clone(),
                branch: Some(selected_branch),
                local: Some(repo_path.clone()),
            },
            last_sync: None,
            initialized_at: chrono::Utc::now(),
        };

        self.save_settings(&settings).await?;

        progress_callback(&InstallStage::Complete);

        Ok(url)
    }

    async fn prompt_for_branch(&self, default_branch: &str) -> DotfResult<String> {
        #[allow(clippy::never_loop)]
        loop {
            let prompt_text = format!("Enter the branch to use (default: {}): ", default_branch);
            match self.prompt.input(&prompt_text, Some(default_branch)).await {
                Ok(branch) => {
                    let branch = branch.trim();
                    if branch.is_empty() {
                        return Ok(default_branch.to_string());
                    }
                    return Ok(branch.to_string());
                }
                Err(e) => {
                    // Check if this is an interruption (Ctrl+C)
                    let error_msg = e.to_string();
                    if error_msg.contains("read interrupted") || error_msg.contains("Interrupted") {
                        return Err(DotfError::UserCancellation);
                    }
                    // Re-throw other errors
                    return Err(e);
                }
            }
        }
    }

    // Include all the original methods from InitService
    async fn prompt_for_repository_url(&self) -> DotfResult<String> {
        loop {
            match self
                .prompt
                .input("Enter the repository URL for your dotfiles:", None)
                .await
            {
                Ok(url) => {
                    if url.trim().is_empty() {
                        continue;
                    }

                    // Basic URL validation
                    if !url.contains("://") && !url.starts_with("git@") {
                        continue;
                    }

                    return Ok(url);
                }
                Err(e) => {
                    // Check if this is an interruption (Ctrl+C)
                    let error_msg = e.to_string();
                    if error_msg.contains("read interrupted") || error_msg.contains("Interrupted") {
                        return Err(DotfError::UserCancellation);
                    }
                    // Re-throw other errors
                    return Err(e);
                }
            }
        }
    }

    fn validate_config(&self, config: &DotfConfig) -> DotfResult<()> {
        // Validate symlinks are not empty paths
        for (target, source) in &config.symlinks {
            if target.trim().is_empty() || source.trim().is_empty() {
                return Err(DotfError::Config(format!(
                    "Invalid symlink configuration: '{}' -> '{}'",
                    source, target
                )));
            }
        }

        Ok(())
    }

    async fn setup_dotf_directory(&self) -> DotfResult<()> {
        let dotf_dir = self.filesystem.dotf_directory();

        // Check if .dotf directory already exists
        if self.filesystem.exists(&dotf_dir).await? {
            let should_overwrite = self.prompt.confirm(
                &format!("Dotf directory already exists at: {}. Do you want to remove it and start fresh?", dotf_dir)
            ).await?;

            if !should_overwrite {
                return Err(DotfError::Operation(
                    "Initialization cancelled by user".to_string(),
                ));
            }

            // Remove existing directory
            self.filesystem.remove_dir(&dotf_dir).await?;
        }

        // Create main dotf directory
        self.filesystem.create_dotf_directory().await?;

        // Create subdirectories
        let backup_path = self.filesystem.dotf_backup_path();
        self.filesystem.create_dir_all(&backup_path).await?;

        Ok(())
    }

    async fn save_settings(&self, settings: &Settings) -> DotfResult<()> {
        let settings_path = self.filesystem.dotf_settings_path();
        let content = settings
            .to_toml()
            .map_err(|e| DotfError::Config(format!("Failed to serialize settings: {}", e)))?;

        self.filesystem.write(&settings_path, &content).await?;
        Ok(())
    }
}
