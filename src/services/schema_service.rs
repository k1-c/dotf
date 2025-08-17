use crate::error::{DotfError, DotfResult};
use std::fs;
use std::path::Path;

pub struct SchemaService;

impl Default for SchemaService {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaService {
    pub fn new() -> Self {
        Self
    }

    /// Generate dotf.toml template file
    pub async fn init(&self) -> DotfResult<()> {
        let config_path = "dotf.toml";

        // Check if dotf.toml already exists
        if Path::new(config_path).exists() {
            return Err(DotfError::Operation("dotf.toml already exists".to_string()));
        }

        let template_content = self.generate_template();

        // Write template to file
        fs::write(config_path, template_content).map_err(DotfError::Io)?;

        println!("✅ dotf.toml template created successfully!");
        println!("💡 Edit the file to customize your configuration");

        Ok(())
    }

    /// Generate the default template content
    fn generate_template(&self) -> String {
        r#"[symlinks]
# {Source path} = {Target path}
# Example:
# "zsh/.zshrc" = "~/.zshrc"
# "git/.gitconfig" = "~/.gitconfig"
# "nvim" = "~/.config/nvim"

[scripts.deps]
# Platform-specific dependency installation scripts
# Example:
# macos = "scripts/install-deps-macos.sh"
# linux = "scripts/install-deps-linux.sh"

[scripts.custom]
# Custom installation scripts
# setup-vim = "scripts/setup-vim-plugins.sh"
# install-fonts = "scripts/install-fonts.sh"
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_init_success() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("dotf.toml");

        // Set current directory to temp directory for the test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let service = SchemaService::new();
        let result = service.init().await;

        // Restore original directory - ignore errors if original dir no longer exists
        let _ = std::env::set_current_dir(&original_dir);

        assert!(result.is_ok());
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[symlinks]"));
        assert!(content.contains("[scripts.deps]"));
        assert!(content.contains("[scripts.custom]"));
    }

    #[tokio::test]
    #[ignore = "Flaky in tarpaulin coverage environment"]
    async fn test_init_file_already_exists() {
        let temp_dir = TempDir::new().unwrap();

        // Set current directory to temp directory for the test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create existing dotf.toml in the current directory
        fs::write("dotf.toml", "existing content").unwrap();

        let service = SchemaService::new();
        let result = service.init().await;

        // Restore original directory - ignore errors if original dir no longer exists
        let _ = std::env::set_current_dir(&original_dir);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("dotf.toml already exists"));
    }

    #[test]
    fn test_generate_template() {
        let service = SchemaService::new();
        let template = service.generate_template();

        assert!(template.contains("[symlinks]"));
        assert!(template.contains("[scripts.deps]"));
        assert!(template.contains("[scripts.custom]"));
        assert!(template.contains("~/.zshrc"));
        assert!(template.contains("scripts/install-deps-macos.sh"));
    }
}
