use crate::error::{DottError, DottResult};
use crate::traits::prompt::Prompt;
use async_trait::async_trait;
use dialoguer::{Confirm, Input, Select};

#[derive(Clone)]
pub struct ConsolePrompt;

impl Default for ConsolePrompt {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsolePrompt {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Prompt for ConsolePrompt {
    async fn input(&self, message: &str, default: Option<&str>) -> DottResult<String> {
        let message = message.to_string();
        let default = default.map(|s| s.to_string());

        let result = tokio::task::spawn_blocking(move || {
            let mut input = Input::<String>::new().with_prompt(&message);

            if let Some(default_value) = default {
                input = input.default(default_value);
            }

            input.interact()
        })
        .await
        .map_err(|e| DottError::Operation(format!("Task join error: {}", e)))?
        .map_err(|e| DottError::Operation(format!("Input error: {}", e)))?;

        Ok(result)
    }

    async fn confirm(&self, message: &str) -> DottResult<bool> {
        let message = message.to_string();

        let result =
            tokio::task::spawn_blocking(move || Confirm::new().with_prompt(&message).interact())
                .await
                .map_err(|e| DottError::Operation(format!("Task join error: {}", e)))?
                .map_err(|e| DottError::Operation(format!("Confirm error: {}", e)))?;

        Ok(result)
    }

    async fn select(&self, message: &str, options: &[(&str, &str)]) -> DottResult<usize> {
        let items: Vec<String> = options
            .iter()
            .map(|(label, description)| {
                if description.is_empty() {
                    label.to_string()
                } else {
                    format!("{} - {}", label, description)
                }
            })
            .collect();

        let message = message.to_string();
        let result = tokio::task::spawn_blocking(move || {
            Select::new().with_prompt(&message).items(&items).interact()
        })
        .await
        .map_err(|e| DottError::Operation(format!("Task join error: {}", e)))?
        .map_err(|e| DottError::Operation(format!("Select error: {}", e)))?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests are integration tests that require manual interaction
    // They are disabled by default to avoid blocking CI/CD pipelines
    // To test manually, run: cargo test test_console_prompt --ignored

    #[tokio::test]
    #[ignore]
    async fn test_console_prompt_input() {
        let prompt = ConsolePrompt::new();

        println!("Please enter 'test' when prompted:");
        let result = prompt
            .input("Enter test value:", Some("default"))
            .await
            .unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_console_prompt_confirm() {
        let prompt = ConsolePrompt::new();

        println!("Please answer 'yes' when prompted:");
        let result = prompt.confirm("Do you want to continue?").await.unwrap();
        println!("Result: {}", result);
    }

    #[tokio::test]
    #[ignore]
    async fn test_console_prompt_select() {
        let prompt = ConsolePrompt::new();

        let options = vec![
            ("Option 1", "First option"),
            ("Option 2", "Second option"),
            ("Option 3", "Third option"),
        ];

        println!("Please select option 2 (index 1):");
        let result = prompt.select("Choose an option:", &options).await.unwrap();
        println!("Selected index: {}", result);
    }

    // Unit tests for error handling can be added here
    // but require mocking the dialoguer components
}
