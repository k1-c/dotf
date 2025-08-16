use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::DottResult;
use crate::traits::filesystem::FileSystem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub original_path: String,
    pub backup_path: String,
    pub created_at: DateTime<Utc>,
    pub file_type: BackupFileType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupFileType {
    File,
    Directory,
    Symlink { target: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub entries: HashMap<String, BackupEntry>,
}

#[derive(Debug)]
pub struct RestoreResult {
    pub restored_count: usize,
    pub failed_restorations: Vec<RestoreError>,
}

#[derive(Debug)]
pub struct RestoreError {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub original_path: String,
    pub backup_path: String,
    pub created_at: DateTime<Utc>,
    pub file_type: BackupFileType,
    pub size_estimate: String,
}

impl BackupManifest {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

pub struct BackupManager<F> {
    filesystem: F,
}

impl<F: FileSystem> BackupManager<F> {
    pub fn new(filesystem: F) -> Self {
        Self { filesystem }
    }

    pub async fn backup_file(&self, file_path: &str) -> DottResult<BackupEntry> {
        let timestamp = Utc::now();
        let backup_filename = format!(
            "{}_{}",
            Path::new(file_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            timestamp.format("%Y%m%d_%H%M%S")
        );

        let backup_path = format!("{}/{}", self.filesystem.dott_backup_path(), backup_filename);

        // Ensure backup directory exists
        self.filesystem
            .create_dir_all(&self.filesystem.dott_backup_path())
            .await?;

        let file_type = if self.filesystem.is_symlink(file_path).await? {
            let target = self.filesystem.read_link(file_path).await?;
            BackupFileType::Symlink {
                target: target.to_string_lossy().to_string(),
            }
        } else {
            BackupFileType::File
        };

        // Copy the file to backup location
        self.filesystem.copy_file(file_path, &backup_path).await?;

        let entry = BackupEntry {
            original_path: file_path.to_string(),
            backup_path,
            created_at: timestamp,
            file_type,
        };

        Ok(entry)
    }

    pub async fn restore_from_backup(&self, backup_entry: &BackupEntry) -> DottResult<()> {
        match &backup_entry.file_type {
            BackupFileType::File => {
                self.filesystem
                    .copy_file(&backup_entry.backup_path, &backup_entry.original_path)
                    .await?;
            }
            BackupFileType::Symlink { target } => {
                self.filesystem
                    .create_symlink(target, &backup_entry.original_path)
                    .await?;
            }
            BackupFileType::Directory => {
                self.filesystem
                    .create_dir_all(&backup_entry.original_path)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn load_manifest(&self) -> DottResult<BackupManifest> {
        let manifest_path = format!("{}/manifest.json", self.filesystem.dott_backup_path());

        if self.filesystem.exists(&manifest_path).await? {
            let content = self.filesystem.read_to_string(&manifest_path).await?;
            let manifest: BackupManifest = serde_json::from_str(&content).map_err(|e| {
                crate::error::DottError::Config(format!("Failed to parse backup manifest: {}", e))
            })?;
            Ok(manifest)
        } else {
            Ok(BackupManifest::new())
        }
    }

    pub async fn save_manifest(&self, manifest: &BackupManifest) -> DottResult<()> {
        let manifest_path = format!("{}/manifest.json", self.filesystem.dott_backup_path());

        // Ensure backup directory exists
        self.filesystem
            .create_dir_all(&self.filesystem.dott_backup_path())
            .await?;

        let content = serde_json::to_string_pretty(manifest).map_err(|e| {
            crate::error::DottError::Config(format!("Failed to serialize backup manifest: {}", e))
        })?;

        self.filesystem.write(&manifest_path, &content).await?;
        Ok(())
    }

    pub async fn add_backup_entry(&self, entry: BackupEntry) -> DottResult<()> {
        let mut manifest = self.load_manifest().await?;
        manifest.entries.insert(entry.original_path.clone(), entry);
        self.save_manifest(&manifest).await?;
        Ok(())
    }

    pub async fn get_backup_entry(&self, original_path: &str) -> DottResult<Option<BackupEntry>> {
        let manifest = self.load_manifest().await?;
        Ok(manifest.entries.get(original_path).cloned())
    }

    pub async fn remove_backup_entry(&self, original_path: &str) -> DottResult<()> {
        let mut manifest = self.load_manifest().await?;
        if let Some(entry) = manifest.entries.remove(original_path) {
            // Remove the backup file
            self.filesystem.remove_file(&entry.backup_path).await?;
        }
        self.save_manifest(&manifest).await?;
        Ok(())
    }

    pub async fn cleanup_old_backups(&self, days: u64) -> DottResult<()> {
        let mut manifest = self.load_manifest().await?;
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);

        let mut to_remove = Vec::new();
        for (path, entry) in &manifest.entries {
            if entry.created_at < cutoff {
                to_remove.push(path.clone());
            }
        }

        for path in to_remove {
            if let Some(entry) = manifest.entries.remove(&path) {
                self.filesystem.remove_file(&entry.backup_path).await?;
            }
        }

        self.save_manifest(&manifest).await?;
        Ok(())
    }

    pub async fn restore_specific_backup(&self, original_path: &str) -> DottResult<()> {
        let entry = self.get_backup_entry(original_path).await?;

        match entry {
            Some(backup_entry) => {
                // Remove current file/symlink if it exists
                if self.filesystem.exists(original_path).await? {
                    self.filesystem.remove_file(original_path).await?;
                }

                // Restore from backup
                self.restore_from_backup(&backup_entry).await?;

                // Remove the backup entry from manifest
                self.remove_backup_entry(original_path).await?;

                Ok(())
            }
            None => Err(crate::error::DottError::Operation(format!(
                "No backup found for: {}",
                original_path
            ))),
        }
    }

    pub async fn restore_all_backups(&self) -> DottResult<RestoreResult> {
        let manifest = self.load_manifest().await?;

        if manifest.entries.is_empty() {
            return Ok(RestoreResult {
                restored_count: 0,
                failed_restorations: Vec::new(),
            });
        }

        let mut restored_count = 0;
        let mut failed_restorations = Vec::new();

        // Process each backup entry
        for (original_path, entry) in &manifest.entries {
            match self
                .restore_specific_file_from_entry(original_path, entry)
                .await
            {
                Ok(_) => {
                    restored_count += 1;
                }
                Err(e) => {
                    failed_restorations.push(RestoreError {
                        path: original_path.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        // Clear the manifest if all restorations were successful
        if failed_restorations.is_empty() {
            let empty_manifest = BackupManifest {
                entries: HashMap::new(),
            };
            self.save_manifest(&empty_manifest).await?;
        }

        Ok(RestoreResult {
            restored_count,
            failed_restorations,
        })
    }

    async fn restore_specific_file_from_entry(
        &self,
        original_path: &str,
        entry: &BackupEntry,
    ) -> DottResult<()> {
        // Remove current file/symlink if it exists
        if self.filesystem.exists(original_path).await? {
            self.filesystem.remove_file(original_path).await?;
        }

        // Restore from backup
        self.restore_from_backup(entry).await?;

        Ok(())
    }

    pub async fn list_backups(&self) -> DottResult<Vec<BackupInfo>> {
        let manifest = self.load_manifest().await?;

        let mut backups: Vec<BackupInfo> = manifest
            .entries
            .iter()
            .map(|(path, entry)| BackupInfo {
                original_path: path.clone(),
                backup_path: entry.backup_path.clone(),
                created_at: entry.created_at,
                file_type: entry.file_type.clone(),
                size_estimate: "Unknown".to_string(), // We could add actual size calculation
            })
            .collect();

        // Sort by creation date (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::filesystem::tests::MockFileSystem;

    #[tokio::test]
    async fn test_backup_file() {
        let fs = MockFileSystem::new();
        fs.add_file("/home/user/.vimrc", "set number");

        let backup_manager = BackupManager::new(fs.clone());
        let entry = backup_manager
            .backup_file("/home/user/.vimrc")
            .await
            .unwrap();

        assert_eq!(entry.original_path, "/home/user/.vimrc");
        assert!(entry.backup_path.contains(".vimrc"));
        assert!(matches!(entry.file_type, BackupFileType::File));
    }

    #[tokio::test]
    async fn test_backup_symlink() {
        let fs = MockFileSystem::new();
        fs.create_symlink("/home/user/.dotfiles/.vimrc", "/home/user/.vimrc")
            .await
            .unwrap();

        let backup_manager = BackupManager::new(fs.clone());
        let entry = backup_manager
            .backup_file("/home/user/.vimrc")
            .await
            .unwrap();

        assert_eq!(entry.original_path, "/home/user/.vimrc");
        if let BackupFileType::Symlink { target } = &entry.file_type {
            assert_eq!(target, "/home/user/.dotfiles/.vimrc");
        } else {
            panic!("Expected symlink backup type");
        }
    }

    #[tokio::test]
    async fn test_restore_backup() {
        let fs = MockFileSystem::new();
        fs.add_file("/home/user/.vimrc", "set number");

        let backup_manager = BackupManager::new(fs.clone());
        let entry = backup_manager
            .backup_file("/home/user/.vimrc")
            .await
            .unwrap();

        // Remove original file
        fs.remove_file("/home/user/.vimrc").await.unwrap();
        assert!(!fs.exists("/home/user/.vimrc").await.unwrap());

        // Restore from backup
        backup_manager.restore_from_backup(&entry).await.unwrap();
        assert!(fs.exists("/home/user/.vimrc").await.unwrap());
        assert_eq!(
            fs.read_to_string("/home/user/.vimrc").await.unwrap(),
            "set number"
        );
    }

    #[tokio::test]
    async fn test_manifest_operations() {
        let fs = MockFileSystem::new();
        let backup_manager = BackupManager::new(fs.clone());

        let entry = BackupEntry {
            original_path: "/home/user/.vimrc".to_string(),
            backup_path: "/home/user/.dott/backups/.vimrc_20240101_120000".to_string(),
            created_at: Utc::now(),
            file_type: BackupFileType::File,
        };

        // Add entry to manifest
        backup_manager
            .add_backup_entry(entry.clone())
            .await
            .unwrap();

        // Retrieve entry
        let retrieved = backup_manager
            .get_backup_entry("/home/user/.vimrc")
            .await
            .unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().original_path, entry.original_path);

        // Remove entry
        backup_manager
            .remove_backup_entry("/home/user/.vimrc")
            .await
            .unwrap();
        let retrieved = backup_manager
            .get_backup_entry("/home/user/.vimrc")
            .await
            .unwrap();
        assert!(retrieved.is_none());
    }
}
