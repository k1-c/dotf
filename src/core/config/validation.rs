use crate::core::config::DotfConfig;
use crate::error::{DotfError, DotfResult};

pub fn validate_config(config: &DotfConfig) -> DotfResult<()> {
    // Validate symlinks
    for (source, target) in &config.symlinks {
        if source.is_empty() || target.is_empty() {
            return Err(DotfError::Validation(
                "Symlink source and target cannot be empty".to_string(),
            ));
        }

        // Check for dangerous paths
        if target == "/" || target == "~" {
            return Err(DotfError::Validation(format!(
                "Dangerous symlink target: {}",
                target
            )));
        }
    }

    // Validate scripts
    if let Some(macos_script) = &config.scripts.deps.macos {
        if macos_script.is_empty() {
            return Err(DotfError::Validation(
                "macOS dependency script path cannot be empty".to_string(),
            ));
        }
    }

    if let Some(linux_script) = &config.scripts.deps.linux {
        if linux_script.is_empty() {
            return Err(DotfError::Validation(
                "Linux dependency script path cannot be empty".to_string(),
            ));
        }
    }

    for (name, script) in &config.scripts.custom {
        if name.is_empty() || script.is_empty() {
            return Err(DotfError::Validation(
                "Custom script name and path cannot be empty".to_string(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::dotf_config::{PlatformConfig, ScriptsConfig};
    use std::collections::HashMap;

    fn create_valid_config() -> DotfConfig {
        DotfConfig {
            symlinks: HashMap::new(),
            scripts: ScriptsConfig::default(),
            platform: PlatformConfig::default(),
        }
    }

    #[test]
    fn test_valid_config() {
        let config = create_valid_config();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_empty_symlink_paths() {
        let mut config = create_valid_config();
        config.symlinks.insert("".to_string(), "target".to_string());

        let result = validate_config(&config);
        assert!(result.is_err());
        if let Err(DotfError::Validation(msg)) = result {
            assert!(msg.contains("cannot be empty"));
        } else {
            panic!("Expected validation error");
        }
    }

    #[test]
    fn test_dangerous_symlink_target() {
        let mut config = create_valid_config();
        config
            .symlinks
            .insert("source".to_string(), "/".to_string());

        let result = validate_config(&config);
        assert!(result.is_err());
        if let Err(DotfError::Validation(msg)) = result {
            assert!(msg.contains("Dangerous symlink target"));
        } else {
            panic!("Expected validation error");
        }
    }

    #[test]
    fn test_valid_symlinks() {
        let mut config = create_valid_config();
        config
            .symlinks
            .insert("nvim".to_string(), "~/.config/nvim".to_string());
        config
            .symlinks
            .insert("zshrc".to_string(), "~/.zshrc".to_string());

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_empty_script_paths() {
        let mut config = create_valid_config();
        config.scripts.deps.macos = Some("".to_string());

        let result = validate_config(&config);
        assert!(result.is_err());
        if let Err(DotfError::Validation(msg)) = result {
            assert!(msg.contains("macOS dependency script"));
        } else {
            panic!("Expected validation error");
        }
    }

    #[test]
    fn test_valid_scripts() {
        let mut config = create_valid_config();
        config.scripts.deps.macos = Some("scripts/install-macos.sh".to_string());
        config.scripts.deps.linux = Some("scripts/install-linux.sh".to_string());
        config
            .scripts
            .custom
            .insert("vim-plugins".to_string(), "scripts/vim.sh".to_string());

        assert!(validate_config(&config).is_ok());
    }
}
