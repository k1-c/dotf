use crate::core::config::DottConfig;
use crate::error::DottResult;
use crate::traits::repository::{Repository, RepositoryStatus};
use std::sync::Arc;

pub struct RepositoryManager<R>
where
    R: Repository,
{
    repository: Arc<R>,
}

impl<R> RepositoryManager<R>
where
    R: Repository,
{
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    pub async fn validate_and_fetch_config(&self, url: &str) -> DottResult<DottConfig> {
        // First validate the remote repository
        self.repository.validate_remote(url).await?;

        // Then fetch the configuration
        self.repository.fetch_config(url).await
    }

    pub async fn clone_repository(&self, url: &str, destination: &str) -> DottResult<()> {
        Repository::clone(&*self.repository, url, destination).await
    }

    pub async fn sync_repository(&self, repo_path: &str) -> DottResult<()> {
        self.repository.pull(repo_path).await
    }

    pub async fn get_repository_status(&self, repo_path: &str) -> DottResult<RepositoryStatus> {
        self.repository.get_status(repo_path).await
    }

    pub async fn get_remote_url(&self, repo_path: &str) -> DottResult<String> {
        self.repository.get_remote_url(repo_path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::repository::tests::MockRepository;

    #[tokio::test]
    async fn test_repository_manager_validate_and_fetch() {
        let mut mock_repo = MockRepository::new();
        mock_repo.set_config_response(DottConfig {
            symlinks: std::collections::HashMap::new(),
            scripts: crate::core::config::dott_config::ScriptsConfig::default(),
            platform: crate::core::config::dott_config::PlatformConfig::default(),
        });

        let manager = RepositoryManager::new(mock_repo);
        let config = manager
            .validate_and_fetch_config("https://github.com/test/repo.git")
            .await
            .unwrap();

        assert_eq!(config.symlinks.len(), 0);
        assert_eq!(config.scripts.custom.len(), 0);
    }

    #[tokio::test]
    async fn test_repository_manager_clone() {
        let mock_repo = MockRepository::new();
        let manager = RepositoryManager::new(Clone::clone(&mock_repo));

        manager
            .clone_repository("https://github.com/test/repo.git", "/tmp/repo")
            .await
            .unwrap();

        let clone_calls = mock_repo.get_clone_calls();
        assert_eq!(clone_calls.len(), 1);
        assert_eq!(clone_calls[0].0, "https://github.com/test/repo.git");
        assert_eq!(clone_calls[0].1, "/tmp/repo");
    }

    #[tokio::test]
    async fn test_repository_manager_status() {
        let mut mock_repo = MockRepository::new();
        mock_repo.set_status_response(RepositoryStatus {
            is_clean: true,
            ahead_count: 2,
            behind_count: 1,
            current_branch: "main".to_string(),
        });

        let manager = RepositoryManager::new(mock_repo);
        let status = manager.get_repository_status("/tmp/repo").await.unwrap();

        assert!(status.is_clean);
        assert_eq!(status.ahead_count, 2);
        assert_eq!(status.behind_count, 1);
        assert_eq!(status.current_branch, "main");
    }
}
