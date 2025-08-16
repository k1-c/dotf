use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::core::{
    config::{DottConfig, Settings},
    symlinks::{SymlinkManager, SymlinkStatus},
};
use crate::error::{DottError, DottResult};
use crate::traits::{
    filesystem::FileSystem,
    prompt::Prompt,
    repository::{Repository, RepositoryStatus},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DottStatus {
    pub initialized: bool,
    pub repository: Option<RepositoryStatusInfo>,
    pub symlinks: SymlinksStatusInfo,
    pub config: ConfigStatusInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatusInfo {
    pub url: String,
    pub path: String,
    pub status: RepositoryStatus,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinksStatusInfo {
    pub total: usize,
    pub valid: usize,
    pub missing: usize,
    pub broken: usize,
    pub conflicts: usize,
    pub invalid_targets: usize,
    pub details: Vec<SymlinkStatusDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkStatusDetail {
    pub source_path: String,
    pub target_path: String,
    pub status: SymlinkStatus,
    pub current_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigStatusInfo {
    pub valid: bool,
    pub path: String,
    pub repo_name: String,
    pub repo_version: String,
    pub symlinks_count: usize,
    pub custom_scripts_count: usize,
    pub has_platform_config: bool,
    pub errors: Vec<String>,
}

pub struct StatusService<R, F> {
    repository: R,
    filesystem: F,
    #[allow(dead_code)]
    symlink_manager: SymlinkManager<F, ConsolePrompt>,
}

// We need a dummy prompt for the symlink manager since status checking doesn't need interactive prompts
#[derive(Clone)]
struct ConsolePrompt;

#[async_trait]
impl Prompt for ConsolePrompt {
    async fn input(&self, _message: &str, _default: Option<&str>) -> DottResult<String> {
        Err(DottError::Operation("Prompt not available in status service".to_string()))
    }

    async fn confirm(&self, _message: &str) -> DottResult<bool> {
        Err(DottError::Operation("Prompt not available in status service".to_string()))
    }

    async fn select(&self, _message: &str, _options: &[(&str, &str)]) -> DottResult<usize> {
        Err(DottError::Operation("Prompt not available in status service".to_string()))
    }
}

impl<R: Repository, F: FileSystem + Clone> StatusService<R, F> {
    pub fn new(repository: R, filesystem: F) -> Self {
        let prompt = ConsolePrompt;
        let symlink_manager = SymlinkManager::new(filesystem.clone(), prompt);
        Self {
            repository,
            filesystem,
            symlink_manager,
        }
    }

    pub async fn get_status(&self) -> DottResult<DottStatus> {
        let initialized = self.is_initialized().await?;

        if !initialized {
            return Ok(DottStatus {
                initialized: false,
                repository: None,
                symlinks: SymlinksStatusInfo {
                    total: 0,
                    valid: 0,
                    missing: 0,
                    broken: 0,
                    conflicts: 0,
                    invalid_targets: 0,
                    details: Vec::new(),
                },
                config: ConfigStatusInfo {
                    valid: false,
                    path: String::new(),
                    repo_name: String::new(),
                    repo_version: String::new(),
                    symlinks_count: 0,
                    custom_scripts_count: 0,
                    has_platform_config: false,
                    errors: vec!["Dott is not initialized".to_string()],
                },
            });
        }

        let repository_status = self.get_repository_status().await?;
        let config_status = self.get_config_status().await?;
        let symlinks_status = SymlinksStatusInfo {
            total: 0,
            valid: 0,
            missing: 0,
            broken: 0,
            conflicts: 0,
            invalid_targets: 0,
            details: Vec::new(),
        };

        Ok(DottStatus {
            initialized: true,
            repository: Some(repository_status),
            symlinks: symlinks_status,
            config: config_status,
        })
    }

    pub async fn get_repository_status(&self) -> DottResult<RepositoryStatusInfo> {
        let settings = self.load_settings().await?;
        let repo_path = self.filesystem.dott_repo_path();
        
        let status = self.repository.get_status(&repo_path).await?;

        Ok(RepositoryStatusInfo {
            url: settings.repository_url,
            path: repo_path,
            status,
            last_sync: settings.last_sync,
        })
    }

    pub async fn get_config_status(&self) -> DottResult<ConfigStatusInfo> {
        let config_path = format!("{}/dott.toml", self.filesystem.dott_repo_path());
        let errors = Vec::new();

        if !self.filesystem.exists(&config_path).await? {
            return Ok(ConfigStatusInfo {
                valid: false,
                path: config_path,
                repo_name: String::new(),
                repo_version: String::new(),
                symlinks_count: 0,
                custom_scripts_count: 0,
                has_platform_config: false,
                errors: vec!["Configuration file dott.toml not found".to_string()],
            });
        }

        let config = match self.load_config().await {
            Ok(config) => config,
            Err(e) => {
                return Ok(ConfigStatusInfo {
                    valid: false,
                    path: config_path,
                    repo_name: String::new(),
                    repo_version: String::new(),
                    symlinks_count: 0,
                    custom_scripts_count: 0,
                    has_platform_config: false,
                    errors: vec![format!("Failed to parse configuration: {}", e)],
                });
            }
        };

        let has_platform_config = config.platform.macos.is_some() || config.platform.linux.is_some();

        Ok(ConfigStatusInfo {
            valid: errors.is_empty(),
            path: config_path,
            repo_name: config.repo.name,
            repo_version: config.repo.version,
            symlinks_count: config.symlinks.len(),
            custom_scripts_count: config.scripts.custom.len(),
            has_platform_config,
            errors,
        })
    }

    pub async fn print_status(&self) -> DottResult<()> {
        let status = self.get_status().await?;

        if !status.initialized {
            println!("❌ Dott is not initialized");
            println!("   Run 'dott init <repository-url>' to get started");
            return Ok(());
        }

        // Repository status
        if let Some(repo) = &status.repository {
            println!("📦 Repository Status:");
            println!("   URL: {}", repo.url);
            println!("   Path: {}", repo.path);
            println!("   Branch: {}", repo.status.current_branch);
            println!("   Clean: {}", if repo.status.is_clean { "✅" } else { "❌" });
            
            if repo.status.ahead_count > 0 {
                println!("   Ahead: {} commits", repo.status.ahead_count);
            }
            if repo.status.behind_count > 0 {
                println!("   Behind: {} commits", repo.status.behind_count);
            }
            
            if let Some(last_sync) = repo.last_sync {
                println!("   Last sync: {}", last_sync.format("%Y-%m-%d %H:%M:%S UTC"));
            } else {
                println!("   Last sync: Never");
            }
        }

        println!("✅ Status check completed");
        Ok(())
    }

    async fn is_initialized(&self) -> DottResult<bool> {
        let settings_path = self.filesystem.dott_settings_path();
        let repo_path = self.filesystem.dott_repo_path();
        
        Ok(self.filesystem.exists(&settings_path).await? && 
           self.filesystem.exists(&repo_path).await?)
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

    async fn load_config(&self) -> DottResult<DottConfig> {
        let config_path = format!("{}/dott.toml", self.filesystem.dott_repo_path());
        
        if !self.filesystem.exists(&config_path).await? {
            return Err(DottError::Config("dott.toml not found in repository".to_string()));
        }

        let content = self.filesystem.read_to_string(&config_path).await?;
        let config: DottConfig = toml::from_str(&content)
            .map_err(|e| DottError::Config(format!("Failed to parse dott.toml: {}", e)))?;

        Ok(config)
    }
}