use async_trait::async_trait;
use crate::error::DottResult;

#[derive(Debug, Clone)]
pub enum ConflictAction {
    Backup,
    Skip,
    Overwrite,
    Abort,
}

#[async_trait]
pub trait Prompt: Send + Sync {
    async fn ask_repository_url(&self) -> DottResult<String>;
    async fn ask_conflict_resolution(&self, path: &str) -> DottResult<ConflictAction>;
    async fn confirm(&self, message: &str) -> DottResult<bool>;
    async fn select_option(&self, message: &str, options: &[String]) -> DottResult<usize>;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    
    #[derive(Clone)]
    pub struct MockPrompt {
        pub repository_url_response: Arc<Mutex<Option<String>>>,
        pub conflict_responses: Arc<Mutex<VecDeque<ConflictAction>>>,
        pub confirm_responses: Arc<Mutex<VecDeque<bool>>>,
        pub select_responses: Arc<Mutex<VecDeque<usize>>>,
        pub asked_repository_url: Arc<Mutex<bool>>,
    }
    
    impl MockPrompt {
        pub fn new() -> Self {
            Self {
                repository_url_response: Arc::new(Mutex::new(None)),
                conflict_responses: Arc::new(Mutex::new(VecDeque::new())),
                confirm_responses: Arc::new(Mutex::new(VecDeque::new())),
                select_responses: Arc::new(Mutex::new(VecDeque::new())),
                asked_repository_url: Arc::new(Mutex::new(false)),
            }
        }
        
        pub fn set_repository_url_response(&self, url: &str) {
            *self.repository_url_response.lock().unwrap() = Some(url.to_string());
        }
        
        pub fn add_conflict_response(&self, action: ConflictAction) {
            self.conflict_responses.lock().unwrap().push_back(action);
        }
        
        pub fn add_confirm_response(&self, response: bool) {
            self.confirm_responses.lock().unwrap().push_back(response);
        }
        
        pub fn add_select_response(&self, index: usize) {
            self.select_responses.lock().unwrap().push_back(index);
        }
        
        pub fn was_repository_url_asked(&self) -> bool {
            *self.asked_repository_url.lock().unwrap()
        }
    }
    
    #[async_trait]
    impl Prompt for MockPrompt {
        async fn ask_repository_url(&self) -> DottResult<String> {
            *self.asked_repository_url.lock().unwrap() = true;
            self.repository_url_response
                .lock()
                .unwrap()
                .clone()
                .ok_or_else(|| crate::error::DottError::UserCancelled)
        }
        
        async fn ask_conflict_resolution(&self, _path: &str) -> DottResult<ConflictAction> {
            self.conflict_responses
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| crate::error::DottError::UserCancelled)
        }
        
        async fn confirm(&self, _message: &str) -> DottResult<bool> {
            self.confirm_responses
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| crate::error::DottError::UserCancelled)
        }
        
        async fn select_option(&self, _message: &str, _options: &[String]) -> DottResult<usize> {
            self.select_responses
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| crate::error::DottError::UserCancelled)
        }
    }
}

#[cfg(test)]
mod prompt_tests {
    use super::tests::MockPrompt;
    use super::*;
    
    #[tokio::test]
    async fn test_mock_prompt_repository_url() {
        let prompt = MockPrompt::new();
        prompt.set_repository_url_response("https://github.com/user/dotfiles.git");
        
        let url = prompt.ask_repository_url().await.unwrap();
        assert_eq!(url, "https://github.com/user/dotfiles.git");
        assert!(prompt.was_repository_url_asked());
    }
    
    #[tokio::test]
    async fn test_mock_prompt_conflict_resolution() {
        let prompt = MockPrompt::new();
        prompt.add_conflict_response(ConflictAction::Backup);
        prompt.add_conflict_response(ConflictAction::Skip);
        
        let action1 = prompt.ask_conflict_resolution("file1.txt").await.unwrap();
        assert!(matches!(action1, ConflictAction::Backup));
        
        let action2 = prompt.ask_conflict_resolution("file2.txt").await.unwrap();
        assert!(matches!(action2, ConflictAction::Skip));
    }
    
    #[tokio::test]
    async fn test_mock_prompt_confirm() {
        let prompt = MockPrompt::new();
        prompt.add_confirm_response(true);
        prompt.add_confirm_response(false);
        
        assert!(prompt.confirm("Continue?").await.unwrap());
        assert!(!prompt.confirm("Delete file?").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_mock_prompt_select_option() {
        let prompt = MockPrompt::new();
        prompt.add_select_response(1);
        prompt.add_select_response(0);
        
        let options = vec!["Option A".to_string(), "Option B".to_string()];
        
        let selection1 = prompt.select_option("Choose:", &options).await.unwrap();
        assert_eq!(selection1, 1);
        
        let selection2 = prompt.select_option("Choose again:", &options).await.unwrap();
        assert_eq!(selection2, 0);
    }
    
    #[tokio::test]
    async fn test_mock_prompt_no_response_returns_error() {
        let prompt = MockPrompt::new();
        
        // Without setting any response, should return UserCancelled error
        let result = prompt.ask_repository_url().await;
        assert!(result.is_err());
        if let Err(crate::error::DottError::UserCancelled) = result {
            // Expected
        } else {
            panic!("Expected UserCancelled error");
        }
    }
}