use async_trait::async_trait;
use crate::core::config::DottConfig;
use crate::error::DottResult;

#[async_trait]
pub trait Repository {
    async fn validate_remote(&self, url: &str) -> DottResult<()>;
    async fn fetch_config(&self, url: &str) -> DottResult<DottConfig>;
    async fn clone(&self, url: &str, destination: &str) -> DottResult<()>;
    async fn pull(&self, repo_path: &str) -> DottResult<()>;
    async fn get_status(&self, repo_path: &str) -> DottResult<RepositoryStatus>;
    async fn get_remote_url(&self, repo_path: &str) -> DottResult<String>;
}

#[derive(Debug)]
pub struct RepositoryStatus {
    pub is_clean: bool,
    pub ahead_count: usize,
    pub behind_count: usize,
    pub current_branch: String,
}