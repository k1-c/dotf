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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepositoryStatus {
    pub is_clean: bool,
    pub ahead_count: usize,
    pub behind_count: usize,
    pub current_branch: String,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    
    #[derive(Clone)]
    pub struct MockRepository {
        pub validate_calls: Arc<Mutex<Vec<String>>>,
        pub clone_calls: Arc<Mutex<Vec<(String, String)>>>,
        pub pull_calls: Arc<Mutex<Vec<String>>>,
        pub should_fail_validate: Arc<Mutex<bool>>,
        pub config_response: Arc<Mutex<Option<DottConfig>>>,
        pub status_response: Arc<Mutex<Option<RepositoryStatus>>>,
        pub remote_url_response: Arc<Mutex<Option<String>>>,
    }
    
    impl MockRepository {
        pub fn new() -> Self {
            Self {
                validate_calls: Arc::new(Mutex::new(Vec::new())),
                clone_calls: Arc::new(Mutex::new(Vec::new())),
                pull_calls: Arc::new(Mutex::new(Vec::new())),
                should_fail_validate: Arc::new(Mutex::new(false)),
                config_response: Arc::new(Mutex::new(None)),
                status_response: Arc::new(Mutex::new(None)),
                remote_url_response: Arc::new(Mutex::new(None)),
            }
        }
        
        pub fn set_fail_validate(&mut self, should_fail: bool) {
            *self.should_fail_validate.lock().unwrap() = should_fail;
        }
        
        pub fn set_config_response(&mut self, config: DottConfig) {
            *self.config_response.lock().unwrap() = Some(config);
        }
        
        pub fn set_status_response(&mut self, status: RepositoryStatus) {
            *self.status_response.lock().unwrap() = Some(status);
        }
        
        pub fn set_remote_url(&mut self, url: String) {
            *self.remote_url_response.lock().unwrap() = Some(url);
        }
        
        pub fn get_validate_calls(&self) -> Vec<String> {
            self.validate_calls.lock().unwrap().clone()
        }
        
        pub fn get_clone_calls(&self) -> Vec<(String, String)> {
            self.clone_calls.lock().unwrap().clone()
        }
        
        pub fn get_pull_calls(&self) -> Vec<String> {
            self.pull_calls.lock().unwrap().clone()
        }
    }
    
    #[async_trait]
    impl Repository for MockRepository {
        async fn validate_remote(&self, url: &str) -> DottResult<()> {
            self.validate_calls.lock().unwrap().push(url.to_string());
            
            if *self.should_fail_validate.lock().unwrap() {
                return Err(crate::error::DottError::Repository("Mock validation failure".to_string()));
            }
            
            Ok(())
        }
        
        async fn fetch_config(&self, _url: &str) -> DottResult<DottConfig> {
            self.config_response
                .lock()
                .unwrap()
                .clone()
                .ok_or_else(|| crate::error::DottError::Config("No config response set".to_string()))
        }
        
        async fn clone(&self, url: &str, destination: &str) -> DottResult<()> {
            self.clone_calls.lock().unwrap().push((url.to_string(), destination.to_string()));
            Ok(())
        }
        
        async fn pull(&self, repo_path: &str) -> DottResult<()> {
            self.pull_calls.lock().unwrap().push(repo_path.to_string());
            Ok(())
        }
        
        async fn get_status(&self, _repo_path: &str) -> DottResult<RepositoryStatus> {
            self.status_response
                .lock()
                .unwrap()
                .clone()
                .ok_or_else(|| crate::error::DottError::Repository("No status response set".to_string()))
        }
        
        async fn get_remote_url(&self, _repo_path: &str) -> DottResult<String> {
            self.remote_url_response
                .lock()
                .unwrap()
                .clone()
                .ok_or_else(|| crate::error::DottError::Repository("No remote URL response set".to_string()))
        }
    }
}