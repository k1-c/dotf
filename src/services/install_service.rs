use std::collections::HashMap;

use crate::core::{
    config::{DotfConfig, Settings},
    symlinks::{BackupEntry, SymlinkManager, SymlinkOperation},
};
use crate::error::{DotfError, DotfResult};
use crate::traits::{
    filesystem::FileSystem,
    prompt::Prompt,
    script_executor::{ExecutionResult, ScriptExecutor},
};

pub struct InstallService<F, S, P> {
    filesystem: F,
    script_executor: S,
    prompt: P,
    symlink_manager: SymlinkManager<F, P>,
}

impl<F: FileSystem + Clone, S: ScriptExecutor, P: Prompt> InstallService<F, S, P> {
    pub fn new(filesystem: F, script_executor: S, prompt: P) -> Self {
        let symlink_manager = SymlinkManager::new(filesystem.clone(), prompt.clone());
        Self {
            filesystem,
            script_executor,
            prompt,
            symlink_manager,
        }
    }

    pub fn get_backup_manager(&self) -> &crate::core::symlinks::backup::BackupManager<F> {
        &self.symlink_manager.backup_manager
    }

    pub async fn install_dependencies(&self) -> DotfResult<()> {
        let config = self.load_config().await?;
        let platform = self.detect_platform();

        println!("=' Installing dependencies for platform: {}", platform);

        let script_path = match platform.as_str() {
            "macos" => config.scripts.deps.macos,
            "linux" => config.scripts.deps.linux,
            _ => {
                return Err(DotfError::Platform(format!(
                    "Unsupported platform: {}",
                    platform
                )));
            }
        };

        if let Some(script) = script_path {
            let settings = self.load_settings().await?;
            let repo_path = settings
                .repository
                .local
                .clone()
                .unwrap_or_else(|| self.filesystem.dotf_repo_path());
            let full_script_path = format!("{}/{}", repo_path, script);

            if !self.filesystem.exists(&full_script_path).await? {
                return Err(DotfError::ScriptExecution(format!(
                    "Dependency script not found: {}",
                    full_script_path
                )));
            }

            self.execute_script(&full_script_path, "dependency installation")
                .await?;
            println!(" Dependencies installed successfully");
        } else {
            println!(
                "9  No dependency script configured for platform: {}",
                platform
            );
        }

        Ok(())
    }

    pub async fn install_config(&self) -> DotfResult<Vec<BackupEntry>> {
        let config = self.load_config().await?;
        let platform = self.detect_platform();

        println!("= Installing configuration symlinks");

        // Get base symlinks
        let mut symlinks = config.symlinks.clone();

        // Add platform-specific symlinks
        match platform.as_str() {
            "macos" => {
                if let Some(macos_config) = config.platform.macos {
                    symlinks.extend(macos_config.symlinks);
                }
            }
            "linux" => {
                if let Some(linux_config) = config.platform.linux {
                    symlinks.extend(linux_config.symlinks);
                }
            }
            _ => {}
        }

        if symlinks.is_empty() {
            println!("9  No symlinks configured");
            return Ok(Vec::new());
        }

        // Convert to symlink operations
        let operations = self.create_symlink_operations(&symlinks).await?;

        // Validate all source files exist
        let missing_sources = self.symlink_manager.validate_sources(&operations).await?;
        if !missing_sources.is_empty() {
            return Err(DotfError::Config(format!(
                "Missing source files: {}",
                missing_sources.join(", ")
            )));
        }

        // Create symlinks (with interactive conflict resolution)
        let backup_entries = self
            .symlink_manager
            .create_symlinks(&operations, true)
            .await?;

        println!(" Installed {} symlinks", operations.len());

        // Display the list of created symlinks
        println!("\n📋 Symlinks created:");
        let home_dir = dirs::home_dir().map(|d| d.to_string_lossy().to_string());
        for operation in &operations {
            // Format paths similar to symlinks command display
            let source_display = if let Some(ref home) = home_dir {
                operation.source_path.replace(home, "~")
            } else {
                operation.source_path.clone()
            };

            let target_display = if let Some(ref home) = home_dir {
                operation.target_path.replace(home, "~")
            } else {
                operation.target_path.clone()
            };

            println!("  {} → {}", source_display, target_display);
        }
        if !backup_entries.is_empty() {
            println!("\n=� Created {} backups", backup_entries.len());
        }

        Ok(backup_entries)
    }

    pub async fn install_custom(&self, script_name: &str) -> DotfResult<ExecutionResult> {
        let config = self.load_config().await?;

        let script_path = config.scripts.custom.get(script_name).ok_or_else(|| {
            DotfError::Config(format!("Custom script '{}' not found", script_name))
        })?;

        let settings = self.load_settings().await?;
        let repo_path = settings
            .repository
            .local
            .clone()
            .unwrap_or_else(|| self.filesystem.dotf_repo_path());
        let full_script_path = format!("{}/{}", repo_path, script_path);

        if !self.filesystem.exists(&full_script_path).await? {
            return Err(DotfError::ScriptExecution(format!(
                "Custom script file not found: {}",
                full_script_path
            )));
        }

        println!("=� Executing custom script: {}", script_name);

        let result = self
            .execute_script(
                &full_script_path,
                &format!("custom script '{}'", script_name),
            )
            .await?;

        println!(" Custom script '{}' completed successfully", script_name);

        Ok(result)
    }

    pub async fn install_all(&self) -> DotfResult<Vec<BackupEntry>> {
        println!("=� Starting complete installation");

        // 1. Install dependencies first
        if let Err(e) = self.install_dependencies().await {
            eprintln!("�  Dependency installation failed: {}", e);
            let should_continue = self
                .prompt
                .confirm(
                    "Dependency installation failed. Continue with configuration installation?",
                )
                .await?;

            if !should_continue {
                return Err(e);
            }
        }

        // 2. Install configuration symlinks
        let backup_entries = self.install_config().await?;

        // 3. Ask about custom scripts
        let config = self.load_config().await?;
        if !config.scripts.custom.is_empty() {
            println!("\n=� Available custom scripts:");
            for (name, path) in &config.scripts.custom {
                println!("  - {} ({})", name, path);
            }

            let should_run_custom = self
                .prompt
                .confirm("Would you like to run any custom scripts?")
                .await?;

            if should_run_custom {
                for script_name in config.scripts.custom.keys() {
                    let should_run = self
                        .prompt
                        .confirm(&format!("Run custom script '{}'?", script_name))
                        .await?;

                    if should_run {
                        if let Err(e) = self.install_custom(script_name).await {
                            eprintln!("�  Custom script '{}' failed: {}", script_name, e);
                        }
                    }
                }
            }
        }

        println!("<� Installation completed!");
        Ok(backup_entries)
    }

    pub async fn uninstall_config(&self) -> DotfResult<()> {
        let config = self.load_config().await?;
        let platform = self.detect_platform();

        println!("=�  Uninstalling configuration symlinks");

        // Get all symlinks (base + platform-specific)
        let mut symlinks = config.symlinks.clone();
        match platform.as_str() {
            "macos" => {
                if let Some(macos_config) = config.platform.macos {
                    symlinks.extend(macos_config.symlinks);
                }
            }
            "linux" => {
                if let Some(linux_config) = config.platform.linux {
                    symlinks.extend(linux_config.symlinks);
                }
            }
            _ => {}
        }

        if symlinks.is_empty() {
            println!("9  No symlinks to uninstall");
            return Ok(());
        }

        // Convert to symlink operations
        let operations = self.create_symlink_operations(&symlinks).await?;

        // Remove symlinks
        self.symlink_manager.remove_symlinks(&operations).await?;

        println!(" Uninstalled {} symlinks", operations.len());
        Ok(())
    }

    pub async fn repair_config(&self) -> DotfResult<Vec<BackupEntry>> {
        let config = self.load_config().await?;
        let platform = self.detect_platform();

        println!("=' Repairing configuration symlinks");

        // Get all symlinks (base + platform-specific)
        let mut symlinks = config.symlinks.clone();
        match platform.as_str() {
            "macos" => {
                if let Some(macos_config) = config.platform.macos {
                    symlinks.extend(macos_config.symlinks);
                }
            }
            "linux" => {
                if let Some(linux_config) = config.platform.linux {
                    symlinks.extend(linux_config.symlinks);
                }
            }
            _ => {}
        }

        if symlinks.is_empty() {
            println!("9  No symlinks configured");
            return Ok(Vec::new());
        }

        // Convert to symlink operations
        let operations = self.create_symlink_operations(&symlinks).await?;

        // Repair symlinks
        let backup_entries = self.symlink_manager.repair_symlinks(&operations).await?;

        println!(" Repaired symlinks");
        if !backup_entries.is_empty() {
            println!("=� Created {} backups during repair", backup_entries.len());
        }

        Ok(backup_entries)
    }

    async fn load_config(&self) -> DotfResult<DotfConfig> {
        let settings = self.load_settings().await?;
        let repo_path = settings
            .repository
            .local
            .clone()
            .unwrap_or_else(|| self.filesystem.dotf_repo_path());
        let config_path = format!("{}/dotf.toml", repo_path);

        if !self.filesystem.exists(&config_path).await? {
            return Err(DotfError::Config(
                "dotf.toml not found in repository".to_string(),
            ));
        }

        let content = self.filesystem.read_to_string(&config_path).await?;
        let config: DotfConfig = toml::from_str(&content)
            .map_err(|e| DotfError::Config(format!("Failed to parse dotf.toml: {}", e)))?;

        Ok(config)
    }

    async fn create_symlink_operations(
        &self,
        symlinks: &HashMap<String, String>,
    ) -> DotfResult<Vec<SymlinkOperation>> {
        let mut operations = Vec::new();
        let settings = self.load_settings().await?;
        let repo_path = settings
            .repository
            .local
            .clone()
            .unwrap_or_else(|| self.filesystem.dotf_repo_path());

        for (source, target) in symlinks {
            // Expand target path (handle ~)
            let expanded_target = if target.starts_with("~/") {
                let home = dirs::home_dir().ok_or_else(|| {
                    DotfError::Operation("Could not determine home directory".to_string())
                })?;
                target.replacen("~", &home.to_string_lossy(), 1)
            } else {
                target.clone()
            };

            // Create absolute source path
            let absolute_source = if source.starts_with('/') {
                source.clone()
            } else {
                format!("{}/{}", repo_path, source)
            };

            // Check if source is a directory
            if self.filesystem.exists(&absolute_source).await?
                && self.filesystem.is_dir(&absolute_source).await?
            {
                // Recursively expand directory
                let dir_operations = self
                    .expand_directory_operations(&absolute_source, &expanded_target)
                    .await?;
                operations.extend(dir_operations);
            } else {
                // Single file or doesn't exist yet
                operations.push(SymlinkOperation {
                    source_path: absolute_source,
                    target_path: expanded_target,
                });
            }
        }

        Ok(operations)
    }

    async fn expand_directory_operations(
        &self,
        source_dir: &str,
        target_dir: &str,
    ) -> DotfResult<Vec<SymlinkOperation>> {
        let mut operations = Vec::new();
        let mut dir_stack = vec![(source_dir.to_string(), target_dir.to_string())];

        while let Some((current_source, current_target)) = dir_stack.pop() {
            let entries = self.filesystem.list_entries(&current_source).await?;

            for entry in entries {
                // Calculate relative path from current_source
                let relative_path = entry
                    .path
                    .strip_prefix(&current_source)
                    .unwrap_or(&entry.path)
                    .trim_start_matches('/');

                let target_path = if relative_path.is_empty() {
                    current_target.clone()
                } else {
                    format!("{}/{}", current_target, relative_path)
                };

                if entry.is_dir && !entry.is_symlink {
                    // Add subdirectory to stack for processing
                    let sub_target = format!("{}/{}", current_target, relative_path);
                    dir_stack.push((entry.path.clone(), sub_target));
                } else if entry.is_file || entry.is_symlink {
                    // Add file or symlink to operations
                    operations.push(SymlinkOperation {
                        source_path: entry.path.clone(),
                        target_path,
                    });
                }
            }
        }

        Ok(operations)
    }

    async fn execute_script(
        &self,
        script_path: &str,
        operation: &str,
    ) -> DotfResult<ExecutionResult> {
        // Check if script exists
        if !self.filesystem.exists(script_path).await? {
            return Err(DotfError::ScriptExecution(format!(
                "Script not found: {}",
                script_path
            )));
        }

        // Check if script is executable
        if !self.script_executor.has_permission(script_path).await? {
            println!("= Making script executable: {}", script_path);
            self.script_executor.make_executable(script_path).await?;
        }

        // Execute script
        println!("�  Executing {} script: {}", operation, script_path);
        let result = self.script_executor.execute(script_path).await?;

        if !result.success {
            return Err(DotfError::ScriptExecution(format!(
                "{} failed with exit code {}: {}",
                operation, result.exit_code, result.stderr
            )));
        }

        if !result.stdout.is_empty() {
            println!("=� Script output:\n{}", result.stdout);
        }

        Ok(result)
    }

    async fn load_settings(&self) -> DotfResult<Settings> {
        let settings_path = self.filesystem.dotf_settings_path();

        if !self.filesystem.exists(&settings_path).await? {
            return Err(DotfError::NotInitialized);
        }

        let content = self.filesystem.read_to_string(&settings_path).await?;
        let settings: Settings = Settings::from_toml(&content)
            .map_err(|e| DotfError::Config(format!("Failed to parse settings: {}", e)))?;

        Ok(settings)
    }

    fn detect_platform(&self) -> String {
        #[cfg(target_os = "macos")]
        return "macos".to_string();

        #[cfg(target_os = "linux")]
        return "linux".to_string();

        #[cfg(target_os = "windows")]
        return "windows".to_string();

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        return "unknown".to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::dotf_config::{DepsScripts, PlatformConfig, ScriptsConfig};
    use crate::core::config::{settings::Repository, Settings};
    use crate::traits::{
        filesystem::tests::MockFileSystem,
        prompt::tests::MockPrompt,
        script_executor::{tests::MockScriptExecutor, ExecutionResult},
    };
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_settings_file(filesystem: &MockFileSystem) {
        let settings = Settings {
            repository: Repository {
                remote: "https://github.com/user/dotfiles".to_string(),
                branch: None,
                local: None,
            },
            last_sync: None,
            initialized_at: Utc::now(),
        };
        let settings_content = settings.to_toml().unwrap();
        filesystem.add_file(&filesystem.dotf_settings_path(), &settings_content);
    }

    fn create_test_config() -> DotfConfig {
        let mut symlinks = HashMap::new();
        symlinks.insert(".vimrc".to_string(), "~/.vimrc".to_string());
        symlinks.insert(".bashrc".to_string(), "~/.bashrc".to_string());

        let mut custom_scripts = HashMap::new();
        custom_scripts.insert("setup-vim".to_string(), "scripts/setup-vim.sh".to_string());

        DotfConfig {
            symlinks,
            scripts: ScriptsConfig {
                deps: DepsScripts {
                    macos: Some("scripts/install-deps-macos.sh".to_string()),
                    linux: Some("scripts/install-deps-linux.sh".to_string()),
                },
                custom: custom_scripts,
            },
            platform: PlatformConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_install_dependencies_success() {
        let filesystem = MockFileSystem::new();
        let script_executor = MockScriptExecutor::new();
        let prompt = MockPrompt::new();

        create_test_settings_file(&filesystem);

        // Setup config file
        let config = create_test_config();
        let config_content = toml::to_string(&config).unwrap();
        filesystem.add_file(
            &format!("{}/dotf.toml", filesystem.dotf_repo_path()),
            &config_content,
        );

        // Setup dependency script
        #[cfg(target_os = "macos")]
        let script_path = format!(
            "{}/scripts/install-deps-macos.sh",
            filesystem.dotf_repo_path()
        );
        #[cfg(target_os = "linux")]
        let script_path = format!(
            "{}/scripts/install-deps-linux.sh",
            filesystem.dotf_repo_path()
        );
        filesystem.add_file(&script_path, "#!/bin/bash\necho 'Installing dependencies'");
        script_executor.set_permission(&script_path, true);
        script_executor.set_execution_result(
            &script_path,
            ExecutionResult::success("Dependencies installed".to_string()),
        );

        let service = InstallService::new(filesystem, script_executor.clone(), prompt);
        let result = service.install_dependencies().await;

        assert!(result.is_ok());

        let executed = script_executor.get_executed_scripts();
        assert_eq!(executed.len(), 1);
        assert_eq!(executed[0].0, script_path);
    }

    #[tokio::test]
    async fn test_install_dependencies_missing_script() {
        let filesystem = MockFileSystem::new();
        let script_executor = MockScriptExecutor::new();
        let prompt = MockPrompt::new();

        create_test_settings_file(&filesystem);

        // Setup config file
        let config = create_test_config();
        let config_content = toml::to_string(&config).unwrap();
        filesystem.add_file(
            &format!("{}/dotf.toml", filesystem.dotf_repo_path()),
            &config_content,
        );

        // Don't create the script file

        let service = InstallService::new(filesystem, script_executor, prompt);
        let result = service.install_dependencies().await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DotfError::ScriptExecution(_)));
    }

    #[tokio::test]
    async fn test_install_config_success() {
        let filesystem = MockFileSystem::new();
        let script_executor = MockScriptExecutor::new();
        let prompt = MockPrompt::new();

        create_test_settings_file(&filesystem);

        // Setup config file
        let config = create_test_config();
        let config_content = toml::to_string(&config).unwrap();
        filesystem.add_file(
            &format!("{}/dotf.toml", filesystem.dotf_repo_path()),
            &config_content,
        );

        // Setup source files
        filesystem.add_file(
            &format!("{}/.vimrc", filesystem.dotf_repo_path()),
            "set number",
        );
        filesystem.add_file(
            &format!("{}/.bashrc", filesystem.dotf_repo_path()),
            "alias ll='ls -la'",
        );

        let service = InstallService::new(filesystem.clone(), script_executor, prompt);
        let result = service.install_config().await;

        assert!(result.is_ok());
        let backup_entries = result.unwrap();
        assert!(backup_entries.is_empty()); // No conflicts, so no backups

        // Check that symlinks were created (mocked)
        let home = dirs::home_dir().unwrap();
        let vimrc_target = format!("{}/.vimrc", home.to_string_lossy());
        let bashrc_target = format!("{}/.bashrc", home.to_string_lossy());

        assert!(filesystem.exists(&vimrc_target).await.unwrap());
        assert!(filesystem.exists(&bashrc_target).await.unwrap());
    }

    #[tokio::test]
    async fn test_install_config_missing_source() {
        let filesystem = MockFileSystem::new();
        let script_executor = MockScriptExecutor::new();
        let prompt = MockPrompt::new();

        create_test_settings_file(&filesystem);

        // Setup config file
        let config = create_test_config();
        let config_content = toml::to_string(&config).unwrap();
        filesystem.add_file(
            &format!("{}/dotf.toml", filesystem.dotf_repo_path()),
            &config_content,
        );

        // Only create one source file (.vimrc), missing .bashrc

        filesystem.add_file(
            &format!("{}/.vimrc", filesystem.dotf_repo_path()),
            "set number",
        );

        let service = InstallService::new(filesystem, script_executor, prompt);
        let result = service.install_config().await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DotfError::Config(_)));
    }

    #[tokio::test]
    async fn test_install_custom_success() {
        let filesystem = MockFileSystem::new();
        let script_executor = MockScriptExecutor::new();
        let prompt = MockPrompt::new();

        create_test_settings_file(&filesystem);

        // Setup config file
        let config = create_test_config();
        let config_content = toml::to_string(&config).unwrap();
        filesystem.add_file(
            &format!("{}/dotf.toml", filesystem.dotf_repo_path()),
            &config_content,
        );

        // Setup custom script
        let script_path = format!("{}/scripts/setup-vim.sh", filesystem.dotf_repo_path());
        filesystem.add_file(&script_path, "#!/bin/bash\necho 'Setting up Vim'");
        script_executor.set_permission(&script_path, true);
        script_executor.set_execution_result(
            &script_path,
            ExecutionResult::success("Vim setup complete".to_string()),
        );

        let service = InstallService::new(filesystem, script_executor.clone(), prompt);
        let result = service.install_custom("setup-vim").await;

        assert!(result.is_ok());

        let executed = script_executor.get_executed_scripts();
        assert_eq!(executed.len(), 1);
        assert_eq!(executed[0].0, script_path);
    }

    #[tokio::test]
    async fn test_install_custom_not_found() {
        let filesystem = MockFileSystem::new();
        let script_executor = MockScriptExecutor::new();
        let prompt = MockPrompt::new();

        create_test_settings_file(&filesystem);

        // Setup config file
        let config = create_test_config();
        let config_content = toml::to_string(&config).unwrap();
        filesystem.add_file(
            &format!("{}/dotf.toml", filesystem.dotf_repo_path()),
            &config_content,
        );

        let service = InstallService::new(filesystem, script_executor, prompt);
        let result = service.install_custom("nonexistent-script").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DotfError::Config(_)));
    }

    #[tokio::test]
    async fn test_uninstall_config() {
        let filesystem = MockFileSystem::new();
        let script_executor = MockScriptExecutor::new();
        let prompt = MockPrompt::new();

        create_test_settings_file(&filesystem);

        // Setup config file
        let config = create_test_config();
        let config_content = toml::to_string(&config).unwrap();
        filesystem.add_file(
            &format!("{}/dotf.toml", filesystem.dotf_repo_path()),
            &config_content,
        );

        // Create existing symlinks
        let home = dirs::home_dir().unwrap();
        let vimrc_target = format!("{}/.vimrc", home.to_string_lossy());
        let bashrc_target = format!("{}/.bashrc", home.to_string_lossy());

        filesystem
            .create_symlink(
                &format!("{}/.vimrc", filesystem.dotf_repo_path()),
                &vimrc_target,
            )
            .await
            .unwrap();
        filesystem
            .create_symlink(
                &format!("{}/.bashrc", filesystem.dotf_repo_path()),
                &bashrc_target,
            )
            .await
            .unwrap();

        let service = InstallService::new(filesystem.clone(), script_executor, prompt);
        let result = service.uninstall_config().await;

        assert!(result.is_ok());

        // Check that symlinks were removed
        assert!(!filesystem.exists(&vimrc_target).await.unwrap());
        assert!(!filesystem.exists(&bashrc_target).await.unwrap());
    }
}
