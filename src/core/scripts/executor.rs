use async_trait::async_trait;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::error::{DottError, DottResult};
use crate::traits::script_executor::{ExecutionResult, ScriptExecutor};

pub struct SystemScriptExecutor;

impl SystemScriptExecutor {
    pub fn new() -> Self {
        Self
    }

    async fn check_and_set_permissions(&self, script_path: &str) -> DottResult<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            use tokio::fs;

            let metadata = fs::metadata(script_path)
                .await
                .map_err(|e| DottError::Io(e))?;

            let permissions = metadata.permissions();
            let mode = permissions.mode();

            // Check if executable bit is set (owner execute: 0o100)
            if mode & 0o100 == 0 {
                // Set executable permission for owner
                let new_mode = mode | 0o744; // rwxr--r--
                let new_permissions = std::fs::Permissions::from_mode(new_mode);
                fs::set_permissions(script_path, new_permissions)
                    .await
                    .map_err(|e| DottError::Io(e))?;
            }
        }

        #[cfg(windows)]
        {
            // On Windows, .bat, .cmd, .exe files are inherently executable
            // For shell scripts, we might need to run them through a shell
        }

        Ok(())
    }

    fn get_shell_command(&self) -> (&'static str, &'static str) {
        #[cfg(unix)]
        {
            // Try to use bash first, fallback to sh
            if std::path::Path::new("/bin/bash").exists() {
                ("/bin/bash", "-c")
            } else {
                ("/bin/sh", "-c")
            }
        }

        #[cfg(windows)]
        {
            ("cmd", "/C")
        }
    }

    async fn execute_command(
        &self,
        script_path: &str,
        args: &[String],
    ) -> DottResult<ExecutionResult> {
        let script_extension = std::path::Path::new(script_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let mut command = if script_extension == "sh"
            || script_extension == "bash"
            || script_path.starts_with("#!")
        {
            // Execute shell scripts through shell
            let (shell, shell_arg) = self.get_shell_command();
            let mut cmd = Command::new(shell);

            if args.is_empty() {
                cmd.arg(shell_arg).arg(script_path);
            } else {
                let command_line = format!("{} {}", script_path, args.join(" "));
                cmd.arg(shell_arg).arg(command_line);
            }
            cmd
        } else {
            // Execute directly
            let mut cmd = Command::new(script_path);
            cmd.args(args);
            cmd
        };

        // Capture both stdout and stderr
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        let mut child = command
            .spawn()
            .map_err(|e| DottError::ScriptExecution(format!("Failed to spawn process: {}", e)))?;

        // Capture output streams
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| DottError::ScriptExecution("Failed to capture stdout".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| DottError::ScriptExecution("Failed to capture stderr".to_string()))?;

        // Read output in parallel
        let stdout_handle = tokio::spawn(async move {
            let mut lines = Vec::new();
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                println!("  | {}", line);
                lines.push(line);
            }
            lines.join("\n")
        });

        let stderr_handle = tokio::spawn(async move {
            let mut lines = Vec::new();
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                eprintln!("  ! {}", line);
                lines.push(line);
            }
            lines.join("\n")
        });

        // Wait for process to complete
        let exit_status = child.wait().await.map_err(|e| {
            DottError::ScriptExecution(format!("Failed to wait for process: {}", e))
        })?;

        // Collect output
        let stdout_output = stdout_handle
            .await
            .map_err(|e| DottError::ScriptExecution(format!("Failed to read stdout: {}", e)))?;
        let stderr_output = stderr_handle
            .await
            .map_err(|e| DottError::ScriptExecution(format!("Failed to read stderr: {}", e)))?;

        let exit_code = exit_status.code().unwrap_or(-1);
        let success = exit_status.success();

        Ok(ExecutionResult {
            success,
            exit_code,
            stdout: stdout_output,
            stderr: stderr_output,
        })
    }
}

#[async_trait]
impl ScriptExecutor for SystemScriptExecutor {
    async fn execute(&self, script_path: &str) -> DottResult<ExecutionResult> {
        self.execute_with_args(script_path, &[]).await
    }

    async fn execute_with_args(
        &self,
        script_path: &str,
        args: &[String],
    ) -> DottResult<ExecutionResult> {
        // Check if script exists
        if !tokio::fs::metadata(script_path).await.is_ok() {
            return Err(DottError::ScriptExecution(format!(
                "Script not found: {}",
                script_path
            )));
        }

        // Ensure script has execute permissions
        self.check_and_set_permissions(script_path).await?;

        // Execute the script
        self.execute_command(script_path, args).await
    }

    async fn has_permission(&self, script_path: &str) -> DottResult<bool> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            use tokio::fs;

            let metadata = fs::metadata(script_path)
                .await
                .map_err(|e| DottError::Io(e))?;

            let permissions = metadata.permissions();
            let mode = permissions.mode();

            // Check if executable bit is set (owner execute: 0o100)
            Ok(mode & 0o100 != 0)
        }

        #[cfg(windows)]
        {
            // On Windows, check file extension or assume executable
            let script_extension = std::path::Path::new(script_path)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            Ok(matches!(script_extension, "exe" | "bat" | "cmd" | "ps1"))
        }
    }

    async fn make_executable(&self, script_path: &str) -> DottResult<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            use tokio::fs;

            let metadata = fs::metadata(script_path)
                .await
                .map_err(|e| DottError::Io(e))?;

            let permissions = metadata.permissions();
            let mode = permissions.mode();

            // Set executable permission for owner (rwxr--r--)
            let new_mode = mode | 0o744;
            let new_permissions = std::fs::Permissions::from_mode(new_mode);

            fs::set_permissions(script_path, new_permissions)
                .await
                .map_err(|e| DottError::Io(e))?;
        }

        #[cfg(windows)]
        {
            // On Windows, files are executable based on extension
            // No action needed for permissions
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;
    use tokio::io::AsyncWriteExt;

    async fn create_test_script(content: &str, extension: &str) -> (TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join(format!("test_script.{}", extension));

        let mut file = fs::File::create(&script_path).await.unwrap();
        file.write_all(content.as_bytes()).await.unwrap();
        file.flush().await.unwrap();

        let script_path_str = script_path.to_string_lossy().to_string();
        (temp_dir, script_path_str)
    }

    #[tokio::test]
    async fn test_system_script_executor_success() {
        let executor = SystemScriptExecutor::new();

        let script_content = r#"#!/bin/bash
echo "Hello from script"
echo "Success output"
"#;

        let (_temp_dir, script_path) = create_test_script(script_content, "sh").await;

        let result = executor.execute(&script_path).await.unwrap();

        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("Hello from script"));
        assert!(result.stdout.contains("Success output"));
        assert!(result.stderr.is_empty());
    }

    #[tokio::test]
    async fn test_system_script_executor_failure() {
        let executor = SystemScriptExecutor::new();

        let script_content = r#"#!/bin/bash
echo "This will fail"
exit 1
"#;

        let (_temp_dir, script_path) = create_test_script(script_content, "sh").await;

        let result = executor.execute(&script_path).await.unwrap();

        assert!(!result.success);
        assert_eq!(result.exit_code, 1);
        assert!(result.stdout.contains("This will fail"));
    }

    #[tokio::test]
    async fn test_system_script_executor_with_args() {
        let executor = SystemScriptExecutor::new();

        let script_content = r#"#!/bin/bash
echo "Arg 1: $1"
echo "Arg 2: $2"
echo "All args: $@"
"#;

        let (_temp_dir, script_path) = create_test_script(script_content, "sh").await;

        let args = vec!["first".to_string(), "second".to_string()];
        let result = executor
            .execute_with_args(&script_path, &args)
            .await
            .unwrap();

        assert!(result.success);
        assert!(result.stdout.contains("Arg 1: first"));
        assert!(result.stdout.contains("Arg 2: second"));
        assert!(result.stdout.contains("All args: first second"));
    }

    #[tokio::test]
    async fn test_system_script_executor_stderr() {
        let executor = SystemScriptExecutor::new();

        let script_content = r#"#!/bin/bash
echo "stdout message"
echo "stderr message" >&2
"#;

        let (_temp_dir, script_path) = create_test_script(script_content, "sh").await;

        let result = executor.execute(&script_path).await.unwrap();

        assert!(result.success);
        assert!(result.stdout.contains("stdout message"));
        assert!(result.stderr.contains("stderr message"));
    }

    #[tokio::test]
    async fn test_system_script_executor_nonexistent_script() {
        let executor = SystemScriptExecutor::new();

        let result = executor.execute("/nonexistent/script.sh").await;

        assert!(result.is_err());
        if let Err(DottError::ScriptExecution(msg)) = result {
            assert!(msg.contains("Script not found"));
        } else {
            panic!("Expected ScriptExecution error");
        }
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_system_script_executor_permissions() {
        let executor = SystemScriptExecutor::new();

        let script_content = r#"#!/bin/bash
echo "permission test"
"#;

        let (_temp_dir, script_path) = create_test_script(script_content, "sh").await;

        // Initially should not be executable
        let has_permission = executor.has_permission(&script_path).await.unwrap();
        assert!(!has_permission);

        // Make executable
        executor.make_executable(&script_path).await.unwrap();

        // Now should be executable
        let has_permission = executor.has_permission(&script_path).await.unwrap();
        assert!(has_permission);

        // Should be able to execute
        let result = executor.execute(&script_path).await.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("permission test"));
    }
}
