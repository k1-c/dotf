use crate::cli::{MessageFormatter, Spinner, UiComponents};
use crate::core::filesystem::RealFileSystem;
use crate::error::DotfResult;
use crate::services::ConfigService;
use crate::utils::ConsolePrompt;

pub async fn handle_config(repo: bool, edit: bool) -> DotfResult<()> {
    let filesystem = RealFileSystem::new();
    let prompt = ConsolePrompt::new();
    let config_service = ConfigService::new(filesystem, prompt);
    let formatter = MessageFormatter::new();
    let ui = UiComponents::new();

    if repo {
        // Show repository configuration
        let spinner = Spinner::new("Loading repository configuration...");
        match config_service.show_repository_config().await {
            Ok(content) => {
                spinner.finish_and_clear();
                println!(
                    "{}",
                    formatter.section("Repository Configuration (dotf.toml)")
                );
                println!("{}", content);
            }
            Err(e) => {
                spinner.finish_with_error(&format!("Failed to load configuration: {}", e));
                return Err(e);
            }
        }
    } else if edit {
        // Edit local settings
        let spinner = Spinner::new("Opening settings editor...");
        match config_service.edit_settings().await {
            Ok(_) => {
                spinner.finish_with_success("Settings editor completed");
            }
            Err(e) => {
                spinner.finish_with_error(&format!("Failed to edit settings: {}", e));
                return Err(e);
            }
        }
    } else {
        // Show configuration summary
        let spinner = Spinner::new("Loading configuration summary...");
        match config_service.show_config_summary().await {
            Ok(summary) => {
                spinner.finish_and_clear();

                println!(
                    "{}",
                    ui.config_summary(
                        summary.is_valid,
                        summary.symlinks_count,
                        summary.scripts_count,
                        &summary.platforms_supported,
                        &summary.errors,
                        &summary.warnings,
                    )
                );
            }
            Err(e) => {
                spinner.finish_with_error(&format!("Failed to get configuration summary: {}", e));
                return Err(e);
            }
        }
    }

    Ok(())
}
