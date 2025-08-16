use chrono::Utc;

use crate::core::config::Settings;
use crate::error::{DottError, DottResult};
use crate::traits::{filesystem::FileSystem, repository::Repository};

pub struct SyncService<R, F> {
    repository: R,
    filesystem: F,
}

impl<R: Repository, F: FileSystem> SyncService<R, F> {
    pub fn new(repository: R, filesystem: F) -> Self {
        Self {
            repository,
            filesystem,
        }
    }

    pub async fn sync(&self, force: bool) -> DottResult<SyncResult> {
        // Check if dott is initialized
        let settings_path = self.filesystem.dott_settings_path();
        if !self.filesystem.exists(&settings_path).await? {
            return Err(DottError::Operation("Dott not initialized. Run 'dott init' first.".to_string()));
        }

        // Load current settings
        let settings = self.load_settings().await?;
        let repo_path = self.filesystem.dott_repo_path();

        // Check if repository exists
        if !self.filesystem.exists(&repo_path).await? {
            return Err(DottError::Repository("Repository directory not found. Run 'dott init' to reinitialize.".to_string()));
        }

        // Get repository status before sync
        let status_before = self.repository.get_status(&repo_path).await?;
        
        if !status_before.is_clean && !force {
            return Err(DottError::Operation(
                "Repository has uncommitted changes. Use --force to sync anyway, or commit your changes first.".to_string()
            ));
        }

        // Perform pull (repository will use the configured branch)
        self.repository.pull(&repo_path).await?;

        // Get status after sync
        let status_after = self.repository.get_status(&repo_path).await?;

        // Update last sync timestamp
        let updated_settings = Settings {
            repository: settings.repository,
            last_sync: Some(Utc::now()),
            initialized_at: settings.initialized_at,
        };

        let settings_content = updated_settings.to_toml()
            .map_err(|e| DottError::Serialization(e.to_string()))?;
        
        self.filesystem.write(&settings_path, &settings_content).await?;

        Ok(SyncResult {
            had_uncommitted_changes: !status_before.is_clean,
            commits_pulled: if status_before.behind_count != status_after.behind_count {
                status_before.behind_count
            } else {
                0
            },
            current_branch: status_after.current_branch,
            is_clean_after: status_after.is_clean,
        })
    }

    pub async fn check_sync_status(&self) -> DottResult<SyncStatus> {
        let settings_path = self.filesystem.dott_settings_path();
        if !self.filesystem.exists(&settings_path).await? {
            return Ok(SyncStatus::NotInitialized);
        }

        let repo_path = self.filesystem.dott_repo_path();
        if !self.filesystem.exists(&repo_path).await? {
            return Ok(SyncStatus::RepositoryMissing);
        }

        let status = self.repository.get_status(&repo_path).await?;
        
        if !status.is_clean {
            return Ok(SyncStatus::HasUncommittedChanges {
                branch: status.current_branch,
                ahead: status.ahead_count,
                behind: status.behind_count,
            });
        }

        if status.behind_count > 0 {
            return Ok(SyncStatus::BehindRemote {
                branch: status.current_branch,
                behind_count: status.behind_count,
            });
        }

        if status.ahead_count > 0 {
            return Ok(SyncStatus::AheadOfRemote {
                branch: status.current_branch,
                ahead_count: status.ahead_count,
            });
        }

        let settings = self.load_settings().await?;
        Ok(SyncStatus::UpToDate {
            branch: status.current_branch,
            last_sync: settings.last_sync,
        })
    }

    async fn load_settings(&self) -> DottResult<Settings> {
        let settings_path = self.filesystem.dott_settings_path();
        let content = self.filesystem.read_to_string(&settings_path).await?;
        
        let settings: Settings = Settings::from_toml(&content)
            .map_err(|e| DottError::Serialization(format!("Failed to parse settings: {}", e)))?;
        
        Ok(settings)
    }
}

#[derive(Debug)]
pub struct SyncResult {
    pub had_uncommitted_changes: bool,
    pub commits_pulled: usize,
    pub current_branch: String,
    pub is_clean_after: bool,
}

#[derive(Debug)]
pub enum SyncStatus {
    NotInitialized,
    RepositoryMissing,
    HasUncommittedChanges {
        branch: String,
        ahead: usize,
        behind: usize,
    },
    BehindRemote {
        branch: String,
        behind_count: usize,
    },
    AheadOfRemote {
        branch: String,
        ahead_count: usize,
    },
    UpToDate {
        branch: String,
        last_sync: Option<chrono::DateTime<Utc>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{
        filesystem::tests::MockFileSystem,
        repository::{tests::MockRepository, RepositoryStatus},
    };
    use chrono::Utc;

    fn create_test_service() -> (SyncService<MockRepository, MockFileSystem>, MockRepository, MockFileSystem) {
        let mut repository = MockRepository::new();
        let filesystem = MockFileSystem::new();
        
        // Set up default responses
        repository.set_status_response(RepositoryStatus {
            is_clean: true,
            ahead_count: 0,
            behind_count: 0,
            current_branch: "main".to_string(),
        });
        
        let service = SyncService::new(Clone::clone(&repository), filesystem.clone());
        (service, repository, filesystem)
    }

    #[tokio::test]
    async fn test_sync_not_initialized() {
        let (service, _, _) = create_test_service();
        
        let result = service.sync(false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not initialized"));
    }

    #[tokio::test]
    async fn test_sync_success() {
        let (service, repository, filesystem) = create_test_service();
        
        // Set up initialized state
        let settings = Settings {
            repository: RepositoryConfig {
                remote: "https://github.com/user/dotfiles".to_string(),
                branch: None,
                local: None,
            },
            last_sync: None,
            initialized_at: Utc::now(),
        };
        
        let settings_content = settings.to_toml().unwrap();
        filesystem.add_file(&filesystem.dott_settings_path(), &settings_content);
        filesystem.add_directory(&filesystem.dott_repo_path());
        
        let result = service.sync(false).await.unwrap();
        
        assert!(!result.had_uncommitted_changes);
        assert_eq!(result.commits_pulled, 0);
        assert_eq!(result.current_branch, "main");
        assert!(result.is_clean_after);
        
        // Verify repository.pull was called
        assert_eq!(repository.get_pull_calls().len(), 1);
    }

    #[tokio::test]
    async fn test_sync_with_uncommitted_changes_without_force() {
        let (service, mut repository, filesystem) = create_test_service();
        
        // Set repository to have uncommitted changes
        repository.set_status_response(RepositoryStatus {
            is_clean: false,
            ahead_count: 1,
            behind_count: 0,
            current_branch: "main".to_string(),
        });
        
        // Set up initialized state
        let settings = Settings {
            repository: RepositoryConfig {
                remote: "https://github.com/user/dotfiles".to_string(),
                branch: None,
                local: None,
            },
            last_sync: None,
            initialized_at: Utc::now(),
        };
        
        let settings_content = settings.to_toml().unwrap();
        filesystem.add_file(&filesystem.dott_settings_path(), &settings_content);
        filesystem.add_directory(&filesystem.dott_repo_path());
        
        let result = service.sync(false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("uncommitted changes"));
    }

    #[tokio::test]
    async fn test_check_sync_status_up_to_date() {
        let (service, _, filesystem) = create_test_service();
        
        // Set up initialized state
        let settings = Settings {
            repository_url: "https://github.com/user/dotfiles".to_string(),
            branch: None,
            local_path: None,
            last_sync: Some(Utc::now()),
            initialized_at: Utc::now(),
        };
        
        let settings_content = settings.to_toml().unwrap();
        filesystem.add_file(&filesystem.dott_settings_path(), &settings_content);
        filesystem.add_directory(&filesystem.dott_repo_path());
        
        let status = service.check_sync_status().await.unwrap();
        
        match status {
            SyncStatus::UpToDate { branch, last_sync } => {
                assert_eq!(branch, "main");
                assert!(last_sync.is_some());
            }
            _ => panic!("Expected UpToDate status"),
        }
    }

    #[tokio::test]
    async fn test_check_sync_status_behind_remote() {
        let (service, mut repository, filesystem) = create_test_service();
        
        // Set repository to be behind remote
        repository.set_status_response(RepositoryStatus {
            is_clean: true,
            ahead_count: 0,
            behind_count: 3,
            current_branch: "main".to_string(),
        });
        
        // Set up initialized state
        let settings = Settings {
            repository: RepositoryConfig {
                remote: "https://github.com/user/dotfiles".to_string(),
                branch: None,
                local: None,
            },
            last_sync: None,
            initialized_at: Utc::now(),
        };
        
        let settings_content = settings.to_toml().unwrap();
        filesystem.add_file(&filesystem.dott_settings_path(), &settings_content);
        filesystem.add_directory(&filesystem.dott_repo_path());
        
        let status = service.check_sync_status().await.unwrap();
        
        match status {
            SyncStatus::BehindRemote { branch, behind_count } => {
                assert_eq!(branch, "main");
                assert_eq!(behind_count, 3);
            }
            _ => panic!("Expected BehindRemote status"),
        }
    }
}