use crate::error::DotfResult;
use async_trait::async_trait;
use std::path::PathBuf;

#[async_trait]
pub trait FileSystem: Send + Sync {
    async fn exists(&self, path: &str) -> DotfResult<bool>;
    async fn create_dir_all(&self, path: &str) -> DotfResult<()>;
    async fn create_symlink(&self, source: &str, target: &str) -> DotfResult<()>;
    async fn remove_file(&self, path: &str) -> DotfResult<()>;
    async fn remove_dir(&self, path: &str) -> DotfResult<()>;
    async fn copy_file(&self, source: &str, target: &str) -> DotfResult<()>;
    async fn read_to_string(&self, path: &str) -> DotfResult<String>;
    async fn write(&self, path: &str, content: &str) -> DotfResult<()>;
    async fn is_symlink(&self, path: &str) -> DotfResult<bool>;
    async fn read_link(&self, path: &str) -> DotfResult<PathBuf>;

    // Dotf specific path operations
    fn dotf_directory(&self) -> String {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".dotf")
            .to_string_lossy()
            .to_string()
    }

    fn dotf_repo_path(&self) -> String {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".dotf")
            .join("repo")
            .to_string_lossy()
            .to_string()
    }

    fn dotf_settings_path(&self) -> String {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".dotf")
            .join("settings.toml")
            .to_string_lossy()
            .to_string()
    }

    fn dotf_backup_path(&self) -> String {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".dotf")
            .join("backups")
            .to_string_lossy()
            .to_string()
    }

    async fn create_dotf_directory(&self) -> DotfResult<()> {
        let dotf_dir = self.dotf_directory();
        self.create_dir_all(&dotf_dir).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct MockFileSystem {
        pub files: Arc<Mutex<HashMap<String, String>>>,
        pub directories: Arc<Mutex<Vec<String>>>,
        pub symlinks: Arc<Mutex<HashMap<String, String>>>,
    }

    impl Default for MockFileSystem {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockFileSystem {
        pub fn new() -> Self {
            Self {
                files: Arc::new(Mutex::new(HashMap::new())),
                directories: Arc::new(Mutex::new(Vec::new())),
                symlinks: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn add_file(&self, path: &str, content: &str) {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_string(), content.to_string());
        }

        pub fn add_directory(&self, path: &str) {
            self.directories.lock().unwrap().push(path.to_string());
        }

        pub fn get_symlinks(&self) -> HashMap<String, String> {
            self.symlinks.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl FileSystem for MockFileSystem {
        async fn exists(&self, path: &str) -> DotfResult<bool> {
            let files = self.files.lock().unwrap();
            let dirs = self.directories.lock().unwrap();
            let symlinks = self.symlinks.lock().unwrap();
            Ok(files.contains_key(path)
                || dirs.iter().any(|p| p == path)
                || symlinks.contains_key(path))
        }

        async fn create_dir_all(&self, path: &str) -> DotfResult<()> {
            self.directories.lock().unwrap().push(path.to_string());
            Ok(())
        }

        async fn create_symlink(&self, source: &str, target: &str) -> DotfResult<()> {
            self.symlinks
                .lock()
                .unwrap()
                .insert(target.to_string(), source.to_string());
            Ok(())
        }

        async fn remove_file(&self, path: &str) -> DotfResult<()> {
            self.files.lock().unwrap().remove(path);
            self.symlinks.lock().unwrap().remove(path);
            Ok(())
        }

        async fn remove_dir(&self, path: &str) -> DotfResult<()> {
            // Remove directory itself
            self.directories.lock().unwrap().retain(|p| p != path);

            // Remove all files and subdirectories under this path
            let path_prefix = if path.ends_with('/') {
                path.to_string()
            } else {
                format!("{}/", path)
            };

            self.files
                .lock()
                .unwrap()
                .retain(|file_path, _| !file_path.starts_with(&path_prefix) && file_path != path);

            self.directories
                .lock()
                .unwrap()
                .retain(|dir_path| !dir_path.starts_with(&path_prefix) && dir_path != path);

            self.symlinks
                .lock()
                .unwrap()
                .retain(|link_path, _| !link_path.starts_with(&path_prefix) && link_path != path);

            Ok(())
        }

        async fn copy_file(&self, source: &str, target: &str) -> DotfResult<()> {
            let content = {
                let files = self.files.lock().unwrap();
                files.get(source).cloned()
            };
            if let Some(content) = content {
                self.files
                    .lock()
                    .unwrap()
                    .insert(target.to_string(), content);
            }
            Ok(())
        }

        async fn read_to_string(&self, path: &str) -> DotfResult<String> {
            self.files
                .lock()
                .unwrap()
                .get(path)
                .cloned()
                .ok_or_else(|| {
                    crate::error::DotfError::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "File not found",
                    ))
                })
        }

        async fn write(&self, path: &str, content: &str) -> DotfResult<()> {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_string(), content.to_string());
            Ok(())
        }

        async fn is_symlink(&self, path: &str) -> DotfResult<bool> {
            Ok(self.symlinks.lock().unwrap().contains_key(path))
        }

        async fn read_link(&self, path: &str) -> DotfResult<PathBuf> {
            self.symlinks
                .lock()
                .unwrap()
                .get(path)
                .map(PathBuf::from)
                .ok_or_else(|| {
                    crate::error::DotfError::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Symlink not found",
                    ))
                })
        }
    }
}

#[cfg(test)]
mod filesystem_tests {
    use super::tests::MockFileSystem;
    use super::*;

    #[tokio::test]
    async fn test_mock_filesystem_file_operations() {
        let fs = MockFileSystem::new();

        // Test file creation and reading
        fs.write("test.txt", "Hello, world!").await.unwrap();
        assert!(fs.exists("test.txt").await.unwrap());
        assert_eq!(
            fs.read_to_string("test.txt").await.unwrap(),
            "Hello, world!"
        );

        // Test file removal
        fs.remove_file("test.txt").await.unwrap();
        assert!(!fs.exists("test.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_mock_filesystem_directory_operations() {
        let fs = MockFileSystem::new();

        // Test directory creation
        fs.create_dir_all("/test/dir").await.unwrap();
        assert!(fs.exists("/test/dir").await.unwrap());

        // Test directory removal
        fs.remove_dir("/test/dir").await.unwrap();
        assert!(!fs.exists("/test/dir").await.unwrap());
    }

    #[tokio::test]
    async fn test_mock_filesystem_symlink_operations() {
        let fs = MockFileSystem::new();

        // Test symlink creation
        fs.create_symlink("/source/file", "/target/link")
            .await
            .unwrap();
        assert!(fs.exists("/target/link").await.unwrap());
        assert!(fs.is_symlink("/target/link").await.unwrap());

        // Test reading symlink
        let link_target = fs.read_link("/target/link").await.unwrap();
        assert_eq!(link_target, PathBuf::from("/source/file"));
    }

    #[tokio::test]
    async fn test_mock_filesystem_copy_file() {
        let fs = MockFileSystem::new();

        // Create source file
        fs.write("source.txt", "Content to copy").await.unwrap();

        // Copy file
        fs.copy_file("source.txt", "dest.txt").await.unwrap();

        // Verify both files exist with same content
        assert!(fs.exists("source.txt").await.unwrap());
        assert!(fs.exists("dest.txt").await.unwrap());
        assert_eq!(
            fs.read_to_string("source.txt").await.unwrap(),
            fs.read_to_string("dest.txt").await.unwrap()
        );
    }

    #[tokio::test]
    async fn test_dotf_paths() {
        let fs = MockFileSystem::new();

        // Test that dotf paths are properly formatted
        assert!(fs.dotf_directory().ends_with(".dotf"));
        assert!(fs.dotf_repo_path().ends_with(".dotf/repo"));
        assert!(fs.dotf_settings_path().ends_with(".dotf/settings.toml"));
        assert!(fs.dotf_backup_path().ends_with(".dotf/backups"));
    }
}
