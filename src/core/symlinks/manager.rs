use serde::{Deserialize, Serialize};
use std::path::Path;

use super::{
    backup::{BackupEntry, BackupManager},
    conflict::{ConflictInfo, ConflictResolver},
};
use crate::error::{DotfError, DotfResult};
use crate::traits::{filesystem::FileSystem, prompt::Prompt, repository::Repository};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SymlinkStatus {
    Valid,         // Symlink exists and points to correct target
    Missing,       // Symlink does not exist
    Broken,        // Symlink exists but target does not exist
    Conflict,      // File exists at target location but is not the expected symlink
    InvalidTarget, // Symlink exists but points to wrong target
    Modified,      // Symlink is valid but source file has local changes
}

#[derive(Debug, Clone)]
pub struct SymlinkInfo {
    pub source_path: String,
    pub target_path: String,
    pub status: SymlinkStatus,
    pub current_target: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SymlinkOperation {
    pub source_path: String,
    pub target_path: String,
}

pub struct SymlinkManager<F, P> {
    filesystem: F,
    #[allow(dead_code)]
    prompt: P,
    pub backup_manager: BackupManager<F>,
    conflict_resolver: ConflictResolver<F, P>,
}

impl<F: FileSystem + Clone, P: Prompt> SymlinkManager<F, P> {
    pub fn new(filesystem: F, prompt: P) -> Self {
        let backup_manager = BackupManager::new(filesystem.clone());
        let conflict_resolver = ConflictResolver::new(filesystem.clone(), prompt.clone());

        Self {
            filesystem,
            prompt,
            backup_manager,
            conflict_resolver,
        }
    }

    pub fn get_backup_manager(&self) -> &BackupManager<F> {
        &self.backup_manager
    }

    pub async fn create_symlinks(
        &self,
        operations: &[SymlinkOperation],
        interactive: bool,
    ) -> DotfResult<Vec<BackupEntry>> {
        // Check for conflicts first
        let conflicts = self.check_conflicts(operations).await?;

        let backup_entries = if conflicts.is_empty() {
            Vec::new()
        } else if interactive {
            self.conflict_resolver
                .resolve_all_conflicts_interactive(&conflicts)
                .await?
        } else {
            return Err(DotfError::Operation(format!(
                "Found {} conflict(s) but running in non-interactive mode",
                conflicts.len()
            )));
        };

        // Create all symlinks
        for operation in operations {
            // Skip if there was a conflict that wasn't resolved
            if conflicts
                .iter()
                .any(|c| c.target_path == operation.target_path)
                && !self.filesystem.exists(&operation.target_path).await?
            {
                continue;
            }

            // Only create if target doesn't exist (conflict was resolved) or no conflict existed
            if !self.filesystem.exists(&operation.target_path).await? {
                // Ensure parent directory exists
                if let Some(parent) = Path::new(&operation.target_path).parent() {
                    self.filesystem
                        .create_dir_all(&parent.to_string_lossy())
                        .await?;
                }

                self.filesystem
                    .create_symlink(&operation.source_path, &operation.target_path)
                    .await?;
            }
        }

        Ok(backup_entries)
    }

    pub async fn check_conflicts(
        &self,
        operations: &[SymlinkOperation],
    ) -> DotfResult<Vec<ConflictInfo>> {
        let mut conflicts = Vec::new();

        for operation in operations {
            if let Some(conflict) = self
                .conflict_resolver
                .check_conflict(&operation.source_path, &operation.target_path)
                .await?
            {
                conflicts.push(conflict);
            }
        }

        Ok(conflicts)
    }

    pub async fn get_symlink_status(
        &self,
        operations: &[SymlinkOperation],
    ) -> DotfResult<Vec<SymlinkInfo>> {
        let mut statuses = Vec::new();

        for operation in operations {
            let status = self.get_single_symlink_status(operation).await?;
            statuses.push(status);
        }

        Ok(statuses)
    }

    pub async fn get_single_symlink_status(
        &self,
        operation: &SymlinkOperation,
    ) -> DotfResult<SymlinkInfo> {
        let target_exists = self.filesystem.exists(&operation.target_path).await?;

        if !target_exists {
            return Ok(SymlinkInfo {
                source_path: operation.source_path.clone(),
                target_path: operation.target_path.clone(),
                status: SymlinkStatus::Missing,
                current_target: None,
            });
        }

        let is_symlink = self.filesystem.is_symlink(&operation.target_path).await?;

        if !is_symlink {
            return Ok(SymlinkInfo {
                source_path: operation.source_path.clone(),
                target_path: operation.target_path.clone(),
                status: SymlinkStatus::Conflict,
                current_target: None,
            });
        }

        let current_target = self.filesystem.read_link(&operation.target_path).await?;
        let current_target_str = current_target.to_string_lossy().to_string();

        // Check if source exists
        let source_exists = self.filesystem.exists(&operation.source_path).await?;
        if !source_exists {
            return Ok(SymlinkInfo {
                source_path: operation.source_path.clone(),
                target_path: operation.target_path.clone(),
                status: SymlinkStatus::Broken,
                current_target: Some(current_target_str),
            });
        }

        // Check if symlink points to the correct target
        if current_target_str == operation.source_path {
            Ok(SymlinkInfo {
                source_path: operation.source_path.clone(),
                target_path: operation.target_path.clone(),
                status: SymlinkStatus::Valid,
                current_target: Some(current_target_str),
            })
        } else {
            Ok(SymlinkInfo {
                source_path: operation.source_path.clone(),
                target_path: operation.target_path.clone(),
                status: SymlinkStatus::InvalidTarget,
                current_target: Some(current_target_str),
            })
        }
    }

    pub async fn remove_symlinks(&self, operations: &[SymlinkOperation]) -> DotfResult<()> {
        for operation in operations {
            let status = self.get_single_symlink_status(operation).await?;

            match status.status {
                SymlinkStatus::Valid
                | SymlinkStatus::Broken
                | SymlinkStatus::InvalidTarget
                | SymlinkStatus::Modified => {
                    self.filesystem.remove_file(&operation.target_path).await?;
                }
                SymlinkStatus::Missing => {
                    // Already doesn't exist, nothing to do
                }
                SymlinkStatus::Conflict => {
                    return Err(DotfError::Operation(format!(
                        "Cannot remove '{}': not a symlink",
                        operation.target_path
                    )));
                }
            }
        }

        Ok(())
    }

    pub async fn repair_symlinks(
        &self,
        operations: &[SymlinkOperation],
    ) -> DotfResult<Vec<BackupEntry>> {
        let mut backup_entries = Vec::new();

        for operation in operations {
            let status = self.get_single_symlink_status(operation).await?;

            match status.status {
                SymlinkStatus::Valid | SymlinkStatus::Modified => {
                    // Nothing to repair for Valid or Modified symlinks
                    continue;
                }
                SymlinkStatus::Missing => {
                    // Create the symlink
                    if let Some(parent) = Path::new(&operation.target_path).parent() {
                        self.filesystem
                            .create_dir_all(&parent.to_string_lossy())
                            .await?;
                    }
                    self.filesystem
                        .create_symlink(&operation.source_path, &operation.target_path)
                        .await?;
                }
                SymlinkStatus::Broken | SymlinkStatus::InvalidTarget => {
                    // Remove and recreate
                    self.filesystem.remove_file(&operation.target_path).await?;
                    self.filesystem
                        .create_symlink(&operation.source_path, &operation.target_path)
                        .await?;
                }
                SymlinkStatus::Conflict => {
                    // Handle as conflict
                    if let Some(conflict) = self
                        .conflict_resolver
                        .check_conflict(&operation.source_path, &operation.target_path)
                        .await?
                    {
                        if let Some(backup_entry) = self
                            .conflict_resolver
                            .resolve_conflict_interactive(&conflict)
                            .await?
                        {
                            backup_entries.push(backup_entry);
                        }

                        // Create symlink if target was cleared
                        if !self.filesystem.exists(&operation.target_path).await? {
                            if let Some(parent) = Path::new(&operation.target_path).parent() {
                                self.filesystem
                                    .create_dir_all(&parent.to_string_lossy())
                                    .await?;
                            }
                            self.filesystem
                                .create_symlink(&operation.source_path, &operation.target_path)
                                .await?;
                        }
                    }
                }
            }
        }

        Ok(backup_entries)
    }

    pub async fn validate_sources(
        &self,
        operations: &[SymlinkOperation],
    ) -> DotfResult<Vec<String>> {
        let mut missing_sources = Vec::new();

        for operation in operations {
            if !self.filesystem.exists(&operation.source_path).await? {
                missing_sources.push(operation.source_path.clone());
            }
        }

        Ok(missing_sources)
    }

    pub async fn get_symlink_status_with_changes<R: Repository>(
        &self,
        operations: &[SymlinkOperation],
        repository: &R,
        repo_path: &str,
    ) -> DotfResult<Vec<SymlinkInfo>> {
        let mut statuses = Vec::new();

        for operation in operations {
            let mut status = self.get_single_symlink_status(operation).await?;

            // If symlink is valid, check for local changes
            if status.status == SymlinkStatus::Valid {
                // Convert absolute source path to relative path from repo root
                let relative_source = if operation.source_path.starts_with(repo_path) {
                    operation
                        .source_path
                        .strip_prefix(repo_path)
                        .unwrap_or(&operation.source_path)
                        .trim_start_matches('/')
                } else {
                    &operation.source_path
                };

                match repository
                    .is_file_modified(repo_path, relative_source)
                    .await
                {
                    Ok(true) => {
                        status.status = SymlinkStatus::Modified;
                    }
                    Ok(false) => {
                        // Keep as Valid
                    }
                    Err(_) => {
                        // If we can't check git status, keep original status
                    }
                }
            }

            statuses.push(status);
        }

        Ok(statuses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{filesystem::tests::MockFileSystem, prompt::tests::MockPrompt};

    #[tokio::test]
    async fn test_create_symlinks_no_conflicts() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        fs.add_file("/source/.vimrc", "vim config");

        let manager = SymlinkManager::new(fs.clone(), prompt);
        let operations = vec![SymlinkOperation {
            source_path: "/source/.vimrc".to_string(),
            target_path: "/home/user/.vimrc".to_string(),
        }];

        let backups = manager.create_symlinks(&operations, true).await.unwrap();
        assert!(backups.is_empty());

        assert!(fs.exists("/home/user/.vimrc").await.unwrap());
        assert!(fs.is_symlink("/home/user/.vimrc").await.unwrap());

        let target = fs.read_link("/home/user/.vimrc").await.unwrap();
        assert_eq!(target.to_string_lossy(), "/source/.vimrc");
    }

    #[tokio::test]
    async fn test_get_symlink_status_missing() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        let manager = SymlinkManager::new(fs, prompt);
        let operation = SymlinkOperation {
            source_path: "/source/.vimrc".to_string(),
            target_path: "/home/user/.vimrc".to_string(),
        };

        let status = manager.get_single_symlink_status(&operation).await.unwrap();
        assert_eq!(status.status, SymlinkStatus::Missing);
    }

    #[tokio::test]
    async fn test_get_symlink_status_valid() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        fs.add_file("/source/.vimrc", "vim config");
        fs.create_symlink("/source/.vimrc", "/home/user/.vimrc")
            .await
            .unwrap();

        let manager = SymlinkManager::new(fs, prompt);
        let operation = SymlinkOperation {
            source_path: "/source/.vimrc".to_string(),
            target_path: "/home/user/.vimrc".to_string(),
        };

        let status = manager.get_single_symlink_status(&operation).await.unwrap();
        assert_eq!(status.status, SymlinkStatus::Valid);
        assert_eq!(status.current_target, Some("/source/.vimrc".to_string()));
    }

    #[tokio::test]
    async fn test_get_symlink_status_broken() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        fs.create_symlink("/source/.vimrc", "/home/user/.vimrc")
            .await
            .unwrap();
        // Source file doesn't exist

        let manager = SymlinkManager::new(fs, prompt);
        let operation = SymlinkOperation {
            source_path: "/source/.vimrc".to_string(),
            target_path: "/home/user/.vimrc".to_string(),
        };

        let status = manager.get_single_symlink_status(&operation).await.unwrap();
        assert_eq!(status.status, SymlinkStatus::Broken);
    }

    #[tokio::test]
    async fn test_get_symlink_status_conflict() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        fs.add_file("/home/user/.vimrc", "existing file");

        let manager = SymlinkManager::new(fs, prompt);
        let operation = SymlinkOperation {
            source_path: "/source/.vimrc".to_string(),
            target_path: "/home/user/.vimrc".to_string(),
        };

        let status = manager.get_single_symlink_status(&operation).await.unwrap();
        assert_eq!(status.status, SymlinkStatus::Conflict);
    }

    #[tokio::test]
    async fn test_get_symlink_status_invalid_target() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        fs.add_file("/source/.vimrc", "vim config");
        fs.add_file("/other/.vimrc", "other vim config");
        fs.create_symlink("/other/.vimrc", "/home/user/.vimrc")
            .await
            .unwrap();

        let manager = SymlinkManager::new(fs, prompt);
        let operation = SymlinkOperation {
            source_path: "/source/.vimrc".to_string(),
            target_path: "/home/user/.vimrc".to_string(),
        };

        let status = manager.get_single_symlink_status(&operation).await.unwrap();
        assert_eq!(status.status, SymlinkStatus::InvalidTarget);
        assert_eq!(status.current_target, Some("/other/.vimrc".to_string()));
    }

    #[tokio::test]
    async fn test_remove_symlinks() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        fs.add_file("/source/.vimrc", "vim config");
        fs.create_symlink("/source/.vimrc", "/home/user/.vimrc")
            .await
            .unwrap();

        let manager = SymlinkManager::new(fs.clone(), prompt);
        let operations = vec![SymlinkOperation {
            source_path: "/source/.vimrc".to_string(),
            target_path: "/home/user/.vimrc".to_string(),
        }];

        assert!(fs.exists("/home/user/.vimrc").await.unwrap());

        manager.remove_symlinks(&operations).await.unwrap();

        assert!(!fs.exists("/home/user/.vimrc").await.unwrap());
    }

    #[tokio::test]
    async fn test_validate_sources() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        fs.add_file("/source/.vimrc", "vim config");
        // /source/.bashrc doesn't exist

        let manager = SymlinkManager::new(fs, prompt);
        let operations = vec![
            SymlinkOperation {
                source_path: "/source/.vimrc".to_string(),
                target_path: "/home/user/.vimrc".to_string(),
            },
            SymlinkOperation {
                source_path: "/source/.bashrc".to_string(),
                target_path: "/home/user/.bashrc".to_string(),
            },
        ];

        let missing = manager.validate_sources(&operations).await.unwrap();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "/source/.bashrc");
    }
}
