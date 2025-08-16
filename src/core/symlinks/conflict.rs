use serde::{Deserialize, Serialize};

use super::backup::{BackupEntry, BackupManager};
use crate::error::{DottError, DottResult};
use crate::traits::{filesystem::FileSystem, prompt::Prompt};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictResolution {
    Skip,
    Backup,
    Overwrite,
    Abort,
}

#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub target_path: String,
    pub source_path: String,
    pub existing_is_symlink: bool,
    pub existing_target: Option<String>,
}

pub struct ConflictResolver<F, P> {
    filesystem: F,
    prompt: P,
    backup_manager: BackupManager<F>,
}

impl<F: FileSystem + Clone, P: Prompt> ConflictResolver<F, P> {
    pub fn new(filesystem: F, prompt: P) -> Self {
        let backup_manager = BackupManager::new(filesystem.clone());
        Self {
            filesystem,
            prompt,
            backup_manager,
        }
    }

    pub async fn check_conflict(
        &self,
        source_path: &str,
        target_path: &str,
    ) -> DottResult<Option<ConflictInfo>> {
        if !self.filesystem.exists(target_path).await? {
            return Ok(None);
        }

        let existing_is_symlink = self.filesystem.is_symlink(target_path).await?;
        let existing_target = if existing_is_symlink {
            Some(
                self.filesystem
                    .read_link(target_path)
                    .await?
                    .to_string_lossy()
                    .to_string(),
            )
        } else {
            None
        };

        // If it's already a symlink pointing to the same source, no conflict
        if let Some(ref target) = existing_target {
            if target == source_path {
                return Ok(None);
            }
        }

        Ok(Some(ConflictInfo {
            target_path: target_path.to_string(),
            source_path: source_path.to_string(),
            existing_is_symlink,
            existing_target,
        }))
    }

    pub async fn resolve_conflict(
        &self,
        conflict: &ConflictInfo,
        resolution: ConflictResolution,
    ) -> DottResult<Option<BackupEntry>> {
        match resolution {
            ConflictResolution::Skip => Ok(None),
            ConflictResolution::Abort => Err(DottError::Operation(
                "Operation aborted by user".to_string(),
            )),
            ConflictResolution::Overwrite => {
                self.remove_existing(&conflict.target_path).await?;
                Ok(None)
            }
            ConflictResolution::Backup => {
                let backup_entry = self
                    .backup_manager
                    .backup_file(&conflict.target_path)
                    .await?;
                self.remove_existing(&conflict.target_path).await?;
                self.backup_manager
                    .add_backup_entry(backup_entry.clone())
                    .await?;
                Ok(Some(backup_entry))
            }
        }
    }

    pub async fn resolve_conflict_interactive(
        &self,
        conflict: &ConflictInfo,
    ) -> DottResult<Option<BackupEntry>> {
        let existing_type = if conflict.existing_is_symlink {
            format!(
                "symlink -> {}",
                conflict
                    .existing_target
                    .as_ref()
                    .unwrap_or(&"unknown".to_string())
            )
        } else {
            "file".to_string()
        };

        let message = format!(
            "Conflict detected at '{}'\nExisting: {}\nNew target: {}\n\nHow would you like to resolve this conflict?",
            conflict.target_path,
            existing_type,
            conflict.source_path
        );

        let options = vec![
            ("Skip", "Skip creating this symlink"),
            (
                "Backup",
                "Backup existing file/symlink and create new symlink",
            ),
            (
                "Overwrite",
                "Remove existing file/symlink and create new symlink",
            ),
            ("Abort", "Abort the entire operation"),
        ];

        let choice = self.prompt.select(&message, &options).await?;

        let resolution = match choice {
            0 => ConflictResolution::Skip,
            1 => ConflictResolution::Backup,
            2 => ConflictResolution::Overwrite,
            3 => ConflictResolution::Abort,
            _ => ConflictResolution::Abort,
        };

        self.resolve_conflict(conflict, resolution).await
    }

    pub async fn resolve_all_conflicts_interactive(
        &self,
        conflicts: &[ConflictInfo],
    ) -> DottResult<Vec<BackupEntry>> {
        if conflicts.is_empty() {
            return Ok(Vec::new());
        }

        let message = format!(
            "Found {} conflict(s). How would you like to resolve all conflicts?",
            conflicts.len()
        );

        let options = vec![
            ("Individual", "Resolve each conflict individually"),
            ("Skip All", "Skip all conflicting symlinks"),
            (
                "Backup All",
                "Backup all existing files and create symlinks",
            ),
            (
                "Overwrite All",
                "Overwrite all existing files with symlinks",
            ),
            ("Abort", "Abort the operation"),
        ];

        let choice = self.prompt.select(&message, &options).await?;

        match choice {
            0 => {
                // Individual resolution
                let mut backup_entries = Vec::new();
                for conflict in conflicts {
                    if let Some(entry) = self.resolve_conflict_interactive(conflict).await? {
                        backup_entries.push(entry);
                    }
                }
                Ok(backup_entries)
            }
            1 => {
                // Skip all
                Ok(Vec::new())
            }
            2 => {
                // Backup all
                let mut backup_entries = Vec::new();
                for conflict in conflicts {
                    if let Some(entry) = self
                        .resolve_conflict(conflict, ConflictResolution::Backup)
                        .await?
                    {
                        backup_entries.push(entry);
                    }
                }
                Ok(backup_entries)
            }
            3 => {
                // Overwrite all
                for conflict in conflicts {
                    self.resolve_conflict(conflict, ConflictResolution::Overwrite)
                        .await?;
                }
                Ok(Vec::new())
            }
            4 | _ => {
                // Abort
                Err(DottError::Operation(
                    "Operation aborted by user".to_string(),
                ))
            }
        }
    }

    async fn remove_existing(&self, path: &str) -> DottResult<()> {
        if self.filesystem.is_symlink(path).await? {
            self.filesystem.remove_file(path).await?;
        } else {
            self.filesystem.remove_file(path).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{filesystem::tests::MockFileSystem, prompt::tests::MockPrompt};

    #[tokio::test]
    async fn test_no_conflict_when_file_does_not_exist() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();
        let resolver = ConflictResolver::new(fs, prompt);

        let conflict = resolver
            .check_conflict("/source/.vimrc", "/home/user/.vimrc")
            .await
            .unwrap();
        assert!(conflict.is_none());
    }

    #[tokio::test]
    async fn test_no_conflict_when_symlink_points_to_same_source() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        fs.create_symlink("/source/.vimrc", "/home/user/.vimrc")
            .await
            .unwrap();

        let resolver = ConflictResolver::new(fs, prompt);
        let conflict = resolver
            .check_conflict("/source/.vimrc", "/home/user/.vimrc")
            .await
            .unwrap();
        assert!(conflict.is_none());
    }

    #[tokio::test]
    async fn test_conflict_with_existing_file() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        fs.add_file("/home/user/.vimrc", "existing content");

        let resolver = ConflictResolver::new(fs, prompt);
        let conflict = resolver
            .check_conflict("/source/.vimrc", "/home/user/.vimrc")
            .await
            .unwrap();

        assert!(conflict.is_some());
        let conflict = conflict.unwrap();
        assert_eq!(conflict.target_path, "/home/user/.vimrc");
        assert_eq!(conflict.source_path, "/source/.vimrc");
        assert!(!conflict.existing_is_symlink);
        assert!(conflict.existing_target.is_none());
    }

    #[tokio::test]
    async fn test_conflict_with_existing_symlink() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        fs.create_symlink("/other/source/.vimrc", "/home/user/.vimrc")
            .await
            .unwrap();

        let resolver = ConflictResolver::new(fs, prompt);
        let conflict = resolver
            .check_conflict("/source/.vimrc", "/home/user/.vimrc")
            .await
            .unwrap();

        assert!(conflict.is_some());
        let conflict = conflict.unwrap();
        assert_eq!(conflict.target_path, "/home/user/.vimrc");
        assert_eq!(conflict.source_path, "/source/.vimrc");
        assert!(conflict.existing_is_symlink);
        assert_eq!(
            conflict.existing_target,
            Some("/other/source/.vimrc".to_string())
        );
    }

    #[tokio::test]
    async fn test_resolve_conflict_skip() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        fs.add_file("/home/user/.vimrc", "existing content");

        let resolver = ConflictResolver::new(fs.clone(), prompt);
        let conflict = ConflictInfo {
            target_path: "/home/user/.vimrc".to_string(),
            source_path: "/source/.vimrc".to_string(),
            existing_is_symlink: false,
            existing_target: None,
        };

        let result = resolver
            .resolve_conflict(&conflict, ConflictResolution::Skip)
            .await
            .unwrap();
        assert!(result.is_none());

        // File should still exist
        assert!(fs.exists("/home/user/.vimrc").await.unwrap());
    }

    #[tokio::test]
    async fn test_resolve_conflict_overwrite() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        fs.add_file("/home/user/.vimrc", "existing content");

        let resolver = ConflictResolver::new(fs.clone(), prompt);
        let conflict = ConflictInfo {
            target_path: "/home/user/.vimrc".to_string(),
            source_path: "/source/.vimrc".to_string(),
            existing_is_symlink: false,
            existing_target: None,
        };

        let result = resolver
            .resolve_conflict(&conflict, ConflictResolution::Overwrite)
            .await
            .unwrap();
        assert!(result.is_none());

        // File should be removed
        assert!(!fs.exists("/home/user/.vimrc").await.unwrap());
    }

    #[tokio::test]
    async fn test_resolve_conflict_backup() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        fs.add_file("/home/user/.vimrc", "existing content");

        let resolver = ConflictResolver::new(fs.clone(), prompt);
        let conflict = ConflictInfo {
            target_path: "/home/user/.vimrc".to_string(),
            source_path: "/source/.vimrc".to_string(),
            existing_is_symlink: false,
            existing_target: None,
        };

        let result = resolver
            .resolve_conflict(&conflict, ConflictResolution::Backup)
            .await
            .unwrap();
        assert!(result.is_some());

        let backup_entry = result.unwrap();
        assert_eq!(backup_entry.original_path, "/home/user/.vimrc");

        // Original file should be removed
        assert!(!fs.exists("/home/user/.vimrc").await.unwrap());
    }

    #[tokio::test]
    async fn test_resolve_conflict_abort() {
        let fs = MockFileSystem::new();
        let prompt = MockPrompt::new();

        let resolver = ConflictResolver::new(fs, prompt);
        let conflict = ConflictInfo {
            target_path: "/home/user/.vimrc".to_string(),
            source_path: "/source/.vimrc".to_string(),
            existing_is_symlink: false,
            existing_target: None,
        };

        let result = resolver
            .resolve_conflict(&conflict, ConflictResolution::Abort)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DottError::Operation(_)));
    }
}
