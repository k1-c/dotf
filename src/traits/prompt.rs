use crate::error::DotfResult;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub enum ConflictAction {
    Backup,
    Skip,
    Overwrite,
    Abort,
}

#[async_trait]
pub trait Prompt: Send + Sync + Clone {
    async fn input(&self, message: &str, default: Option<&str>) -> DotfResult<String>;
    async fn confirm(&self, message: &str) -> DotfResult<bool>;
    async fn select(&self, message: &str, options: &[(&str, &str)]) -> DotfResult<usize>;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct MockPrompt {
        pub input_responses: Arc<Mutex<VecDeque<String>>>,
        pub confirm_responses: Arc<Mutex<VecDeque<bool>>>,
        pub select_responses: Arc<Mutex<VecDeque<usize>>>,
    }

    impl Default for MockPrompt {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockPrompt {
        pub fn new() -> Self {
            Self {
                input_responses: Arc::new(Mutex::new(VecDeque::new())),
                confirm_responses: Arc::new(Mutex::new(VecDeque::new())),
                select_responses: Arc::new(Mutex::new(VecDeque::new())),
            }
        }

        pub fn set_input_response(&self, response: String) {
            self.input_responses.lock().unwrap().push_back(response);
        }

        pub fn set_confirm_response(&self, response: bool) {
            self.confirm_responses.lock().unwrap().push_back(response);
        }

        pub fn set_select_response(&self, index: usize) {
            self.select_responses.lock().unwrap().push_back(index);
        }
    }

    #[async_trait]
    impl Prompt for MockPrompt {
        async fn input(&self, _message: &str, _default: Option<&str>) -> DotfResult<String> {
            self.input_responses
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| crate::error::DotfError::UserCancelled)
        }

        async fn confirm(&self, _message: &str) -> DotfResult<bool> {
            self.confirm_responses
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| crate::error::DotfError::UserCancelled)
        }

        async fn select(&self, _message: &str, _options: &[(&str, &str)]) -> DotfResult<usize> {
            self.select_responses
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| crate::error::DotfError::UserCancelled)
        }
    }
}

#[cfg(test)]
mod prompt_tests {
    use super::tests::MockPrompt;
    use super::*;

    #[tokio::test]
    async fn test_mock_prompt_input() {
        let prompt = MockPrompt::new();
        prompt.set_input_response("test input".to_string());

        let result = prompt.input("Enter value:", None).await.unwrap();
        assert_eq!(result, "test input");
    }

    #[tokio::test]
    async fn test_mock_prompt_confirm() {
        let prompt = MockPrompt::new();
        prompt.set_confirm_response(true);
        prompt.set_confirm_response(false);

        assert!(prompt.confirm("Continue?").await.unwrap());
        assert!(!prompt.confirm("Delete file?").await.unwrap());
    }

    #[tokio::test]
    async fn test_mock_prompt_select() {
        let prompt = MockPrompt::new();
        prompt.set_select_response(1);

        let options = vec![("Option A", "First option"), ("Option B", "Second option")];

        let selection = prompt.select("Choose:", &options).await.unwrap();
        assert_eq!(selection, 1);
    }
}
