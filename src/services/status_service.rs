use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::{
    config::{DotfConfig, Settings},
    symlinks::{SymlinkManager, SymlinkOperation, SymlinkStatus},
};
use crate::error::{DotfError, DotfResult};
use crate::traits::{
    filesystem::FileSystem,
    prompt::Prompt,
    repository::{Repository, RepositoryStatus},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotfStatus {
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
    pub modified: usize,
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
    async fn input(&self, _message: &str, _default: Option<&str>) -> DotfResult<String> {
        Err(DotfError::Operation(
            "Prompt not available in status service".to_string(),
        ))
    }

    async fn confirm(&self, _message: &str) -> DotfResult<bool> {
        Err(DotfError::Operation(
            "Prompt not available in status service".to_string(),
        ))
    }

    async fn select(&self, _message: &str, _options: &[(&str, &str)]) -> DotfResult<usize> {
        Err(DotfError::Operation(
            "Prompt not available in status service".to_string(),
        ))
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

    pub async fn get_status(&self) -> DotfResult<DotfStatus> {
        let initialized = self.is_initialized().await?;

        if !initialized {
            return Ok(DotfStatus {
                initialized: false,
                repository: None,
                symlinks: SymlinksStatusInfo {
                    total: 0,
                    valid: 0,
                    missing: 0,
                    broken: 0,
                    conflicts: 0,
                    invalid_targets: 0,
                    modified: 0,
                    details: Vec::new(),
                },
                config: ConfigStatusInfo {
                    valid: false,
                    path: String::new(),
                    symlinks_count: 0,
                    custom_scripts_count: 0,
                    has_platform_config: false,
                    errors: vec!["Dotf is not initialized".to_string()],
                },
            });
        }

        let repository_status = self.get_repository_status().await?;
        let config_status = self.get_config_status().await?;
        let symlinks_status = self.get_symlinks_status().await?;

        Ok(DotfStatus {
            initialized: true,
            repository: Some(repository_status),
            symlinks: symlinks_status,
            config: config_status,
        })
    }

    pub async fn get_repository_status(&self) -> DotfResult<RepositoryStatusInfo> {
        let settings = self.load_settings().await?;
        let repo_path = settings
            .repository
            .local
            .clone()
            .unwrap_or_else(|| self.filesystem.dotf_repo_path());

        let status = self.repository.get_status(&repo_path).await?;

        Ok(RepositoryStatusInfo {
            url: settings.repository.remote,
            path: repo_path,
            status,
            last_sync: settings.last_sync,
        })
    }

    pub async fn get_symlinks_status(&self) -> DotfResult<SymlinksStatusInfo> {
        let config = match self.load_config().await {
            Ok(config) => config,
            Err(_) => {
                // If config can't be loaded, return empty status
                return Ok(SymlinksStatusInfo {
                    total: 0,
                    valid: 0,
                    missing: 0,
                    broken: 0,
                    conflicts: 0,
                    invalid_targets: 0,
                    modified: 0,
                    details: Vec::new(),
                });
            }
        };

        let platform = self.detect_platform();
        let mut symlinks = config.symlinks.clone();

        // Add platform-specific symlinks
        match platform.as_str() {
            "macos" => {
                if let Some(macos_config) = config.platform.macos {
                    symlinks.extend(macos_config.symlinks);
                }
            }
            "linux" => {
                if let Some(linux_config) = config.platform.linux {
                    symlinks.extend(linux_config.symlinks);
                }
            }
            _ => {}
        }

        let operations = self.create_symlink_operations(&symlinks).await?;
        let settings = self.load_settings().await?;
        let repo_path = settings
            .repository
            .local
            .clone()
            .unwrap_or_else(|| self.filesystem.dotf_repo_path());
        let symlink_infos = self
            .symlink_manager
            .get_symlink_status_with_changes(&operations, &self.repository, &repo_path)
            .await?;

        let mut status_info = SymlinksStatusInfo {
            total: symlink_infos.len(),
            valid: 0,
            missing: 0,
            broken: 0,
            conflicts: 0,
            invalid_targets: 0,
            modified: 0,
            details: Vec::new(),
        };

        for info in symlink_infos {
            match info.status {
                SymlinkStatus::Valid => status_info.valid += 1,
                SymlinkStatus::Missing => status_info.missing += 1,
                SymlinkStatus::Broken => status_info.broken += 1,
                SymlinkStatus::Conflict => status_info.conflicts += 1,
                SymlinkStatus::InvalidTarget => status_info.invalid_targets += 1,
                SymlinkStatus::Modified => status_info.modified += 1,
            }

            status_info.details.push(SymlinkStatusDetail {
                source_path: info.source_path,
                target_path: info.target_path,
                status: info.status,
                current_target: info.current_target,
            });
        }

        Ok(status_info)
    }

    pub async fn get_config_status(&self) -> DotfResult<ConfigStatusInfo> {
        let settings = self.load_settings().await?;
        let repo_path = settings
            .repository
            .local
            .clone()
            .unwrap_or_else(|| self.filesystem.dotf_repo_path());
        let config_path = format!("{}/dotf.toml", repo_path);
        let errors = Vec::new();

        if !self.filesystem.exists(&config_path).await? {
            return Ok(ConfigStatusInfo {
                valid: false,
                path: config_path,
                symlinks_count: 0,
                custom_scripts_count: 0,
                has_platform_config: false,
                errors: vec!["Configuration file dotf.toml not found".to_string()],
            });
        }

        let config = match self.load_config().await {
            Ok(config) => config,
            Err(e) => {
                return Ok(ConfigStatusInfo {
                    valid: false,
                    path: config_path,
                    symlinks_count: 0,
                    custom_scripts_count: 0,
                    has_platform_config: false,
                    errors: vec![format!("Failed to parse configuration: {}", e)],
                });
            }
        };

        let has_platform_config =
            config.platform.macos.is_some() || config.platform.linux.is_some();

        Ok(ConfigStatusInfo {
            valid: errors.is_empty(),
            path: config_path,
            symlinks_count: config.symlinks.len(),
            custom_scripts_count: config.scripts.custom.len(),
            has_platform_config,
            errors,
        })
    }

    pub async fn print_status(&self) -> DotfResult<()> {
        let status = self.get_status().await?;

        if !status.initialized {
            println!("❌ Dotf is not initialized");
            println!("   Run 'dotf init <repository-url>' to get started");
            return Ok(());
        }

        // Repository status
        if let Some(repo) = &status.repository {
            println!("📦 Repository Status:");
            println!("   URL: {}", repo.url);
            println!("   Path: {}", repo.path);
            println!("   Branch: {}", repo.status.current_branch);
            println!(
                "   Clean: {}",
                if repo.status.is_clean { "✅" } else { "❌" }
            );

            if repo.status.ahead_count > 0 {
                println!("   Ahead: {} commits", repo.status.ahead_count);
            }
            if repo.status.behind_count > 0 {
                println!("   Behind: {} commits", repo.status.behind_count);
            }

            if let Some(last_sync) = repo.last_sync {
                println!(
                    "   Last sync: {}",
                    last_sync.format("%Y-%m-%d %H:%M:%S UTC")
                );
            } else {
                println!("   Last sync: Never");
            }
        }

        println!("✅ Status check completed");
        Ok(())
    }

    async fn is_initialized(&self) -> DotfResult<bool> {
        let settings_path = self.filesystem.dotf_settings_path();
        // For initialization check, we need to handle the case where settings might not exist yet
        let repo_path = if let Ok(settings) = self.load_settings().await {
            settings
                .repository
                .local
                .unwrap_or_else(|| self.filesystem.dotf_repo_path())
        } else {
            self.filesystem.dotf_repo_path()
        };

        Ok(self.filesystem.exists(&settings_path).await?
            && self.filesystem.exists(&repo_path).await?)
    }

    async fn load_settings(&self) -> DotfResult<Settings> {
        let settings_path = self.filesystem.dotf_settings_path();

        if !self.filesystem.exists(&settings_path).await? {
            return Err(DotfError::NotInitialized);
        }

        let content = self.filesystem.read_to_string(&settings_path).await?;
        let settings: Settings = Settings::from_toml(&content)
            .map_err(|e| DotfError::Config(format!("Failed to parse settings: {}", e)))?;

        Ok(settings)
    }

    async fn load_config(&self) -> DotfResult<DotfConfig> {
        let settings = self.load_settings().await?;
        let repo_path = settings
            .repository
            .local
            .clone()
            .unwrap_or_else(|| self.filesystem.dotf_repo_path());
        let config_path = format!("{}/dotf.toml", repo_path);

        if !self.filesystem.exists(&config_path).await? {
            return Err(DotfError::Config(
                "dotf.toml not found in repository".to_string(),
            ));
        }

        let content = self.filesystem.read_to_string(&config_path).await?;
        let config: DotfConfig = toml::from_str(&content)
            .map_err(|e| DotfError::Config(format!("Failed to parse dotf.toml: {}", e)))?;

        Ok(config)
    }

    async fn create_symlink_operations(
        &self,
        symlinks: &HashMap<String, String>,
    ) -> DotfResult<Vec<SymlinkOperation>> {
        let mut operations = Vec::new();
        let settings = self.load_settings().await?;
        let repo_path = settings
            .repository
            .local
            .clone()
            .unwrap_or_else(|| self.filesystem.dotf_repo_path());

        for (source, target) in symlinks {
            // Expand target path (handle ~)
            let expanded_target = if target.starts_with("~/") {
                let home = dirs::home_dir().ok_or_else(|| {
                    DotfError::Operation("Could not determine home directory".to_string())
                })?;
                target.replacen("~", &home.to_string_lossy(), 1)
            } else {
                target.clone()
            };

            // Create absolute source path
            let absolute_source = if source.starts_with('/') {
                source.clone()
            } else {
                format!("{}/{}", repo_path, source)
            };

            operations.push(SymlinkOperation {
                source_path: absolute_source,
                target_path: expanded_target,
            });
        }

        Ok(operations)
    }

    fn detect_platform(&self) -> String {
        #[cfg(target_os = "macos")]
        return "macos".to_string();

        #[cfg(target_os = "linux")]
        return "linux".to_string();

        #[cfg(target_os = "windows")]
        return "windows".to_string();

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        return "unknown".to_string();
    }
}
