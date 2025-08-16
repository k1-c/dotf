use async_trait::async_trait;
use crate::error::DottResult;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ExecutionResult {
    pub fn success(stdout: String) -> Self {
        Self {
            success: true,
            exit_code: 0,
            stdout,
            stderr: String::new(),
        }
    }
    
    pub fn failure(exit_code: i32, stderr: String) -> Self {
        Self {
            success: false,
            exit_code,
            stdout: String::new(),
            stderr,
        }
    }
}

#[async_trait]
pub trait ScriptExecutor: Send + Sync {
    async fn execute(&self, script_path: &str) -> DottResult<ExecutionResult>;
    async fn execute_with_args(&self, script_path: &str, args: &[String]) -> DottResult<ExecutionResult>;
    async fn has_permission(&self, script_path: &str) -> DottResult<bool>;
    async fn make_executable(&self, script_path: &str) -> DottResult<()>;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    
    #[derive(Clone)]
    pub struct MockScriptExecutor {
        pub execution_results: Arc<Mutex<HashMap<String, ExecutionResult>>>,
        pub permissions: Arc<Mutex<HashMap<String, bool>>>,
        pub executed_scripts: Arc<Mutex<Vec<(String, Vec<String>)>>>,
    }
    
    impl MockScriptExecutor {
        pub fn new() -> Self {
            Self {
                execution_results: Arc::new(Mutex::new(HashMap::new())),
                permissions: Arc::new(Mutex::new(HashMap::new())),
                executed_scripts: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        pub fn set_execution_result(&self, script_path: &str, result: ExecutionResult) {
            self.execution_results
                .lock()
                .unwrap()
                .insert(script_path.to_string(), result);
        }
        
        pub fn set_permission(&self, script_path: &str, has_permission: bool) {
            self.permissions
                .lock()
                .unwrap()
                .insert(script_path.to_string(), has_permission);
        }
        
        pub fn get_executed_scripts(&self) -> Vec<(String, Vec<String>)> {
            self.executed_scripts.lock().unwrap().clone()
        }
    }
    
    #[async_trait]
    impl ScriptExecutor for MockScriptExecutor {
        async fn execute(&self, script_path: &str) -> DottResult<ExecutionResult> {
            self.executed_scripts
                .lock()
                .unwrap()
                .push((script_path.to_string(), Vec::new()));
            
            self.execution_results
                .lock()
                .unwrap()
                .get(script_path)
                .cloned()
                .ok_or_else(|| crate::error::DottError::ScriptExecution(
                    format!("Script not found: {}", script_path)
                ))
        }
        
        async fn execute_with_args(&self, script_path: &str, args: &[String]) -> DottResult<ExecutionResult> {
            self.executed_scripts
                .lock()
                .unwrap()
                .push((script_path.to_string(), args.to_vec()));
            
            self.execution_results
                .lock()
                .unwrap()
                .get(script_path)
                .cloned()
                .ok_or_else(|| crate::error::DottError::ScriptExecution(
                    format!("Script not found: {}", script_path)
                ))
        }
        
        async fn has_permission(&self, script_path: &str) -> DottResult<bool> {
            Ok(self.permissions
                .lock()
                .unwrap()
                .get(script_path)
                .copied()
                .unwrap_or(false))
        }
        
        async fn make_executable(&self, script_path: &str) -> DottResult<()> {
            self.permissions
                .lock()
                .unwrap()
                .insert(script_path.to_string(), true);
            Ok(())
        }
    }
}

#[cfg(test)]
mod script_executor_tests {
    use super::tests::MockScriptExecutor;
    use super::*;
    
    #[tokio::test]
    async fn test_mock_script_executor_success() {
        let executor = MockScriptExecutor::new();
        executor.set_execution_result(
            "install.sh",
            ExecutionResult::success("Installation complete".to_string())
        );
        
        let result = executor.execute("install.sh").await.unwrap();
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "Installation complete");
        assert!(result.stderr.is_empty());
    }
    
    #[tokio::test]
    async fn test_mock_script_executor_failure() {
        let executor = MockScriptExecutor::new();
        executor.set_execution_result(
            "bad-script.sh",
            ExecutionResult::failure(1, "Command not found".to_string())
        );
        
        let result = executor.execute("bad-script.sh").await.unwrap();
        assert!(!result.success);
        assert_eq!(result.exit_code, 1);
        assert!(result.stdout.is_empty());
        assert_eq!(result.stderr, "Command not found");
    }
    
    #[tokio::test]
    async fn test_mock_script_executor_with_args() {
        let executor = MockScriptExecutor::new();
        executor.set_execution_result(
            "script.sh",
            ExecutionResult::success("Args processed".to_string())
        );
        
        let args = vec!["--verbose".to_string(), "--output=test".to_string()];
        let result = executor.execute_with_args("script.sh", &args).await.unwrap();
        assert!(result.success);
        
        let executed = executor.get_executed_scripts();
        assert_eq!(executed.len(), 1);
        assert_eq!(executed[0].0, "script.sh");
        assert_eq!(executed[0].1, args);
    }
    
    #[tokio::test]
    async fn test_mock_script_executor_permissions() {
        let executor = MockScriptExecutor::new();
        
        // Initially no permission
        assert!(!executor.has_permission("script.sh").await.unwrap());
        
        // Set permission
        executor.set_permission("script.sh", true);
        assert!(executor.has_permission("script.sh").await.unwrap());
        
        // Make executable
        executor.make_executable("new-script.sh").await.unwrap();
        assert!(executor.has_permission("new-script.sh").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_mock_script_executor_missing_script() {
        let executor = MockScriptExecutor::new();
        
        let result = executor.execute("nonexistent.sh").await;
        assert!(result.is_err());
        if let Err(crate::error::DottError::ScriptExecution(msg)) = result {
            assert!(msg.contains("Script not found"));
        } else {
            panic!("Expected ScriptExecution error");
        }
    }
    
    #[tokio::test]
    async fn test_mock_script_executor_tracks_executions() {
        let executor = MockScriptExecutor::new();
        executor.set_execution_result("script1.sh", ExecutionResult::success("".to_string()));
        executor.set_execution_result("script2.sh", ExecutionResult::success("".to_string()));
        
        executor.execute("script1.sh").await.unwrap();
        executor.execute_with_args("script2.sh", &["arg1".to_string()]).await.unwrap();
        executor.execute("script1.sh").await.unwrap();
        
        let executed = executor.get_executed_scripts();
        assert_eq!(executed.len(), 3);
        assert_eq!(executed[0].0, "script1.sh");
        assert_eq!(executed[1].0, "script2.sh");
        assert_eq!(executed[2].0, "script1.sh");
    }
}