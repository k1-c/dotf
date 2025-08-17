use crate::core::config::DotfConfig;
use crate::error::{DotfError, DotfResult};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub line: Option<usize>,
    pub section: String,
    pub message: String,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub toml_syntax_valid: bool,
    pub symlinks_valid: bool,
    pub scripts_valid: bool,
}

impl ValidationResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            toml_syntax_valid: true,
            symlinks_valid: true,
            scripts_valid: true,
        }
    }

    pub fn with_errors(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            toml_syntax_valid: true,
            symlinks_valid: false,
            scripts_valid: false,
        }
    }
}

pub struct SchemaValidator;

impl SchemaValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate dotf.toml file
    pub async fn validate(&self, file_path: &str) -> DotfResult<ValidationResult> {
        // Check if file exists
        if !Path::new(file_path).exists() {
            return Err(DotfError::Config(format!(
                "Configuration file not found: {}",
                file_path
            )));
        }

        // Read file content
        let content = fs::read_to_string(file_path).map_err(|e| DotfError::Io(e))?;

        self.validate_content(&content).await
    }

    /// Validate TOML content
    pub async fn validate_content(&self, content: &str) -> DotfResult<ValidationResult> {
        let mut errors = Vec::new();

        // 1. Parse TOML syntax
        let config = match toml::from_str::<DotfConfig>(content) {
            Ok(config) => config,
            Err(e) => {
                errors.push(ValidationError {
                    line: None,
                    section: "TOML Syntax".to_string(),
                    message: format!("Invalid TOML syntax: {}", e),
                });
                let mut result = ValidationResult::with_errors(errors);
                result.toml_syntax_valid = false;
                return Ok(result);
            }
        };

        // 2. Validate structure
        self.validate_structure(&config, &mut errors);

        // 3. Validate symlinks
        self.validate_symlinks(&config, &mut errors).await;

        // 4. Validate scripts
        self.validate_scripts(&config, &mut errors).await;

        Ok(if errors.is_empty() {
            ValidationResult::success()
        } else {
            ValidationResult::with_errors(errors)
        })
    }

    fn validate_structure(&self, config: &DotfConfig, errors: &mut Vec<ValidationError>) {
        // Check if symlinks section exists and is not empty
        if config.symlinks.is_empty() {
            errors.push(ValidationError {
                line: None,
                section: "Structure".to_string(),
                message: "Required section [symlinks] is empty".to_string(),
            });
        }
    }

    async fn validate_symlinks(&self, config: &DotfConfig, errors: &mut Vec<ValidationError>) {
        let mut target_paths = HashSet::new();

        for (source_path, target_path) in &config.symlinks {
            // Check for empty paths
            if source_path.trim().is_empty() {
                errors.push(ValidationError {
                    line: None,
                    section: "symlinks".to_string(),
                    message: format!(
                        "Empty source path: \"{}\" = \"{}\"",
                        source_path, target_path
                    ),
                });
                continue;
            }

            if target_path.trim().is_empty() {
                errors.push(ValidationError {
                    line: None,
                    section: "symlinks".to_string(),
                    message: format!(
                        "Empty target path: \"{}\" = \"{}\"",
                        source_path, target_path
                    ),
                });
                continue;
            }

            // Check for duplicate target paths
            if target_paths.contains(target_path) {
                errors.push(ValidationError {
                    line: None,
                    section: "symlinks".to_string(),
                    message: format!("Duplicate target path: \"{}\"", target_path),
                });
            }
            target_paths.insert(target_path.clone());

            // Check if source file/directory exists
            if !source_path.starts_with('/') && !Path::new(source_path).exists() {
                errors.push(ValidationError {
                    line: None,
                    section: "symlinks".to_string(),
                    message: format!("Source path does not exist: \"{}\"", source_path),
                });
            }

            // Check for invalid characters in paths
            if target_path.contains('\0') || source_path.contains('\0') {
                errors.push(ValidationError {
                    line: None,
                    section: "symlinks".to_string(),
                    message: format!(
                        "Invalid path contains null character: \"{}\" = \"{}\"",
                        source_path, target_path
                    ),
                });
            }
        }
    }

    async fn validate_scripts(&self, config: &DotfConfig, errors: &mut Vec<ValidationError>) {
        // Validate dependency scripts
        if let Some(ref script_path) = config.scripts.deps.macos {
            if !Path::new(script_path).exists() {
                errors.push(ValidationError {
                    line: None,
                    section: "scripts.deps".to_string(),
                    message: format!("Missing script file for platform 'macos': {}", script_path),
                });
            }
        }

        if let Some(ref script_path) = config.scripts.deps.linux {
            if !Path::new(script_path).exists() {
                errors.push(ValidationError {
                    line: None,
                    section: "scripts.deps".to_string(),
                    message: format!("Missing script file for platform 'linux': {}", script_path),
                });
            }
        }

        // Validate custom scripts
        for (script_name, script_path) in &config.scripts.custom {
            if !Path::new(script_path).exists() {
                errors.push(ValidationError {
                    line: None,
                    section: "scripts.custom".to_string(),
                    message: format!("Missing script file for '{}': {}", script_name, script_path),
                });
            }
        }
    }

    /// Show validation results with proper formatting
    pub fn format_result(&self, result: &ValidationResult, quiet: bool) -> String {
        let mut output = Vec::new();

        if !quiet {
            output.push("🔍 Validating dotf.toml...".to_string());
            output.push("".to_string());
        }

        if result.is_valid {
            if !quiet {
                output.push("✅ TOML syntax: Valid".to_string());
                output.push("✅ Schema compliance: Valid".to_string());
                output.push("✅ Symlinks configuration: Valid".to_string());
                output.push("✅ Scripts configuration: Valid".to_string());
                output.push("".to_string());
                output.push("🎉 dotf.toml validation successful!".to_string());
            }
        } else {
            if !quiet {
                if result.toml_syntax_valid {
                    output.push("✅ TOML syntax: Valid".to_string());
                } else {
                    output.push("❌ TOML syntax: Invalid".to_string());
                }
                output.push("❌ Schema compliance: Issues found".to_string());
                output.push("".to_string());
            }

            output.push("🚨 Validation errors:".to_string());
            for error in &result.errors {
                let line_info = if let Some(line) = error.line {
                    format!("   Line {}: ", line)
                } else {
                    "   ".to_string()
                };
                output.push(format!(
                    "{}[{}] {}",
                    line_info, error.section, error.message
                ));
            }

            output.push("".to_string());
            output.push(format!(
                "❌ Validation failed with {} errors.",
                result.errors.len()
            ));
        }

        output.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_validate_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("dotf.toml");

        let valid_content = r#"
[symlinks]
"test/.vimrc" = "~/.vimrc"
"test/.zshrc" = "~/.zshrc"

[scripts.deps]
macos = "test/script.sh"

[scripts.custom]
setup = "test/setup.sh"
"#;

        // Create test files
        fs::create_dir_all(temp_dir.path().join("test")).unwrap();
        fs::write(temp_dir.path().join("test/.vimrc"), "").unwrap();
        fs::write(temp_dir.path().join("test/.zshrc"), "").unwrap();
        fs::write(temp_dir.path().join("test/script.sh"), "").unwrap();
        fs::write(temp_dir.path().join("test/setup.sh"), "").unwrap();

        fs::write(&config_path, valid_content).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let validator = SchemaValidator::new();
        let result = validator.validate("dotf.toml").await.unwrap();

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_invalid_toml() {
        let validator = SchemaValidator::new();
        let invalid_content = r#"
[symlinks
"test" = "invalid
"#;

        let result = validator.validate_content(invalid_content).await.unwrap();

        assert!(!result.is_valid);
        assert!(!result.toml_syntax_valid);
        assert!(!result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_empty_paths() {
        let validator = SchemaValidator::new();
        let content = r#"
[symlinks]
"" = "~/.vimrc"
"test" = ""
"#;

        let result = validator.validate_content(content).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 2);
        assert!(result.errors[0].message.contains("Empty source path"));
        assert!(result.errors[1].message.contains("Empty target path"));
    }

    #[tokio::test]
    async fn test_validate_duplicate_targets() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("test")).unwrap();
        fs::write(temp_dir.path().join("test/file1"), "").unwrap();
        fs::write(temp_dir.path().join("test/file2"), "").unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let validator = SchemaValidator::new();
        let content = r#"
[symlinks]
"test/file1" = "~/.config"
"test/file2" = "~/.config"
"#;

        let result = validator.validate_content(content).await.unwrap();

        std::env::set_current_dir(original_dir).unwrap();

        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("Duplicate target path")));
    }

    #[test]
    fn test_format_result_success() {
        let validator = SchemaValidator::new();
        let result = ValidationResult::success();

        let output = validator.format_result(&result, false);
        assert!(output.contains("🎉 dotf.toml validation successful!"));
        assert!(output.contains("✅ TOML syntax: Valid"));
    }

    #[test]
    fn test_format_result_with_errors() {
        let validator = SchemaValidator::new();
        let errors = vec![ValidationError {
            line: Some(5),
            section: "symlinks".to_string(),
            message: "Test error".to_string(),
        }];
        let result = ValidationResult::with_errors(errors);

        let output = validator.format_result(&result, false);
        assert!(output.contains("❌ Validation failed"));
        assert!(output.contains("Line 5:"));
        assert!(output.contains("Test error"));
    }

    #[test]
    fn test_format_result_quiet() {
        let validator = SchemaValidator::new();
        let result = ValidationResult::success();

        let output = validator.format_result(&result, true);
        assert!(!output.contains("🔍 Validating"));
        assert!(!output.contains("✅ TOML syntax"));
    }
}
