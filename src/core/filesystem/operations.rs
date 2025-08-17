use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::{DotfError, DotfResult};
use crate::traits::filesystem::FileSystem;

#[derive(Clone)]
pub struct RealFileSystem;

impl Default for RealFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RealFileSystem {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileSystem for RealFileSystem {
    async fn exists(&self, path: &str) -> DotfResult<bool> {
        Ok(fs::metadata(path).await.is_ok())
    }

    async fn create_dir_all(&self, path: &str) -> DotfResult<()> {
        fs::create_dir_all(path).await.map_err(DotfError::Io)?;
        Ok(())
    }

    async fn create_symlink(&self, source: &str, target: &str) -> DotfResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(target).parent() {
            if !self.exists(&parent.to_string_lossy()).await? {
                self.create_dir_all(&parent.to_string_lossy()).await?;
            }
        }

        #[cfg(unix)]
        {
            tokio::fs::symlink(source, target)
                .await
                .map_err(DotfError::Io)?;
        }

        #[cfg(windows)]
        {
            // On Windows, we need to check if source is a directory or file
            let source_metadata = fs::metadata(source).await.map_err(|e| DotfError::Io(e))?;

            if source_metadata.is_dir() {
                tokio::fs::symlink_dir(source, target)
                    .await
                    .map_err(|e| DotfError::Io(e))?;
            } else {
                tokio::fs::symlink_file(source, target)
                    .await
                    .map_err(|e| DotfError::Io(e))?;
            }
        }

        Ok(())
    }

    async fn remove_file(&self, path: &str) -> DotfResult<()> {
        let metadata = fs::symlink_metadata(path).await.map_err(DotfError::Io)?;

        if metadata.is_dir() {
            fs::remove_dir_all(path).await.map_err(DotfError::Io)?;
        } else {
            fs::remove_file(path).await.map_err(DotfError::Io)?;
        }

        Ok(())
    }

    async fn remove_dir(&self, path: &str) -> DotfResult<()> {
        fs::remove_dir_all(path).await.map_err(DotfError::Io)?;
        Ok(())
    }

    async fn copy_file(&self, source: &str, target: &str) -> DotfResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(target).parent() {
            if !self.exists(&parent.to_string_lossy()).await? {
                self.create_dir_all(&parent.to_string_lossy()).await?;
            }
        }

        fs::copy(source, target).await.map_err(DotfError::Io)?;
        Ok(())
    }

    async fn read_to_string(&self, path: &str) -> DotfResult<String> {
        fs::read_to_string(path).await.map_err(DotfError::Io)
    }

    async fn write(&self, path: &str, content: &str) -> DotfResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !self.exists(&parent.to_string_lossy()).await? {
                self.create_dir_all(&parent.to_string_lossy()).await?;
            }
        }

        let mut file = fs::File::create(path).await.map_err(DotfError::Io)?;

        file.write_all(content.as_bytes())
            .await
            .map_err(DotfError::Io)?;

        file.flush().await.map_err(DotfError::Io)?;

        Ok(())
    }

    async fn is_symlink(&self, path: &str) -> DotfResult<bool> {
        let metadata = fs::symlink_metadata(path).await.map_err(DotfError::Io)?;

        Ok(metadata.file_type().is_symlink())
    }

    async fn read_link(&self, path: &str) -> DotfResult<PathBuf> {
        fs::read_link(path).await.map_err(DotfError::Io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_real_filesystem_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let fs = RealFileSystem::new();

        let test_file = temp_dir.path().join("test.txt");
        let test_file_str = test_file.to_string_lossy();

        // Test file creation and reading
        fs.write(&test_file_str, "Hello, world!").await.unwrap();
        assert!(fs.exists(&test_file_str).await.unwrap());

        let content = fs.read_to_string(&test_file_str).await.unwrap();
        assert_eq!(content, "Hello, world!");

        // Test file removal
        fs.remove_file(&test_file_str).await.unwrap();
        assert!(!fs.exists(&test_file_str).await.unwrap());
    }

    #[tokio::test]
    async fn test_real_filesystem_directory_operations() {
        let temp_dir = TempDir::new().unwrap();
        let fs = RealFileSystem::new();

        let test_dir = temp_dir.path().join("nested").join("dir");
        let test_dir_str = test_dir.to_string_lossy();

        // Test directory creation
        fs.create_dir_all(&test_dir_str).await.unwrap();
        assert!(fs.exists(&test_dir_str).await.unwrap());

        // Test directory removal
        fs.remove_dir(&test_dir_str).await.unwrap();
        assert!(!fs.exists(&test_dir_str).await.unwrap());
    }

    #[tokio::test]
    async fn test_real_filesystem_symlink_operations() {
        let temp_dir = TempDir::new().unwrap();
        let fs = RealFileSystem::new();

        let source_file = temp_dir.path().join("source.txt");
        let target_link = temp_dir.path().join("target_link.txt");

        let source_str = source_file.to_string_lossy();
        let target_str = target_link.to_string_lossy();

        // Create source file
        fs.write(&source_str, "Source content").await.unwrap();

        // Create symlink
        fs.create_symlink(&source_str, &target_str).await.unwrap();

        assert!(fs.exists(&target_str).await.unwrap());
        assert!(fs.is_symlink(&target_str).await.unwrap());

        // Test reading symlink target
        let link_target = fs.read_link(&target_str).await.unwrap();
        assert_eq!(link_target, source_file);

        // Test reading through symlink
        let content = fs.read_to_string(&target_str).await.unwrap();
        assert_eq!(content, "Source content");
    }

    #[tokio::test]
    async fn test_real_filesystem_copy_file() {
        let temp_dir = TempDir::new().unwrap();
        let fs = RealFileSystem::new();

        let source_file = temp_dir.path().join("source.txt");
        let dest_file = temp_dir.path().join("dest.txt");

        let source_str = source_file.to_string_lossy();
        let dest_str = dest_file.to_string_lossy();

        // Create source file
        fs.write(&source_str, "Content to copy").await.unwrap();

        // Copy file
        fs.copy_file(&source_str, &dest_str).await.unwrap();

        // Verify both files exist with same content
        assert!(fs.exists(&source_str).await.unwrap());
        assert!(fs.exists(&dest_str).await.unwrap());

        let source_content = fs.read_to_string(&source_str).await.unwrap();
        let dest_content = fs.read_to_string(&dest_str).await.unwrap();
        assert_eq!(source_content, dest_content);
        assert_eq!(dest_content, "Content to copy");
    }

    #[tokio::test]
    async fn test_real_filesystem_nested_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let fs = RealFileSystem::new();

        let nested_file = temp_dir.path().join("deep").join("nested").join("file.txt");
        let nested_file_str = nested_file.to_string_lossy();

        // Write to nested file (should create directories automatically)
        fs.write(&nested_file_str, "Nested content").await.unwrap();

        assert!(fs.exists(&nested_file_str).await.unwrap());

        let content = fs.read_to_string(&nested_file_str).await.unwrap();
        assert_eq!(content, "Nested content");
    }

    #[tokio::test]
    async fn test_real_filesystem_error_handling() {
        let fs = RealFileSystem::new();

        // Test reading non-existent file
        let result = fs.read_to_string("/nonexistent/file.txt").await;
        assert!(result.is_err());

        // Test creating symlink in non-existent directory (this should fail)
        let result = fs
            .create_symlink("/tmp/source.txt", "/nonexistent/dir/link.txt")
            .await;
        assert!(result.is_err());
    }
}
