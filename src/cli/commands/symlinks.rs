use crate::cli::args::SymlinksAction;
use crate::cli::{
    BackupEntry, MessageFormatter, OperationResult, OperationStatus, Spinner, SymlinkDetail,
    UiComponents,
};
use crate::core::{filesystem::RealFileSystem, scripts::SystemScriptExecutor};
use crate::error::{DotfError, DotfResult};
use crate::services::{InstallService, StatusService};
use crate::traits::{filesystem::FileSystem, prompt::Prompt};
use crate::utils::ConsolePrompt;

pub async fn handle_symlinks(action: Option<SymlinksAction>) -> DotfResult<()> {
    let formatter = MessageFormatter::new();
    let ui = UiComponents::new();

    match action {
        Some(SymlinksAction::Restore {
            list,
            all,
            filepath,
        }) => {
            if list {
                // List available backups
                let spinner = Spinner::new("Loading backup list...");
                let filesystem = RealFileSystem::new();
                let prompt = ConsolePrompt::new();
                let install_service = InstallService::new(
                    filesystem.clone(),
                    SystemScriptExecutor::new(),
                    prompt.clone(),
                );
                let backup_manager = install_service.get_backup_manager();

                match backup_manager.load_manifest().await {
                    Ok(manifest) => {
                        spinner.finish_and_clear();

                        if manifest.entries.is_empty() {
                            println!("{}", formatter.info("No backups found"));
                        } else {
                            let backup_entries: Vec<BackupEntry> = manifest
                                .entries
                                .iter()
                                .map(|(path, entry)| BackupEntry {
                                    original_path: path.clone(),
                                    backup_path: entry.backup_path.clone(),
                                    created_at: entry
                                        .created_at
                                        .format("%Y-%m-%d %H:%M:%S")
                                        .to_string(),
                                })
                                .collect();

                            println!("{}", ui.backup_list(&backup_entries));
                        }
                    }
                    Err(e) => {
                        spinner.finish_with_error(&format!("Failed to load backup list: {}", e));
                        return Err(e);
                    }
                }
            } else if all {
                // Restore all backups
                let filesystem = RealFileSystem::new();
                let prompt = ConsolePrompt::new();
                let install_service = InstallService::new(
                    filesystem.clone(),
                    SystemScriptExecutor::new(),
                    prompt.clone(),
                );
                let backup_manager = install_service.get_backup_manager();

                let confirm = prompt.confirm(&formatter.question("This will restore ALL backed up files, potentially overwriting current files. Continue?")).await?;
                if !confirm {
                    println!("{}", formatter.info("Restore cancelled"));
                    return Ok(());
                }

                let spinner = Spinner::new("Restoring all backups...");
                match backup_manager.restore_all_backups().await {
                    Ok(result) => {
                        spinner.finish_with_success(&format!(
                            "Restored {} files",
                            result.restored_count
                        ));

                        if !result.failed_restorations.is_empty() {
                            println!(
                                "{}",
                                formatter.warning(&format!(
                                    "{} failures occurred:",
                                    result.failed_restorations.len()
                                ))
                            );

                            let operation_results: Vec<OperationResult> = result
                                .failed_restorations
                                .iter()
                                .map(|failure| OperationResult {
                                    operation: failure.path.clone(),
                                    status: OperationStatus::Failed,
                                    details: Some(failure.error.clone()),
                                })
                                .collect();

                            println!(
                                "{}",
                                ui.operation_results("Failed Restorations", &operation_results)
                            );
                        }
                    }
                    Err(e) => {
                        spinner.finish_with_error(&format!("Restore failed: {}", e));
                        return Err(e);
                    }
                }
            } else if let Some(path) = filepath {
                // Restore specific file
                let filesystem = RealFileSystem::new();
                let prompt = ConsolePrompt::new();
                let install_service = InstallService::new(
                    filesystem.clone(),
                    SystemScriptExecutor::new(),
                    prompt.clone(),
                );
                let backup_manager = install_service.get_backup_manager();

                let spinner = Spinner::new(&format!("Restoring backup for: {}", path));
                match backup_manager.restore_specific_backup(&path).await {
                    Ok(_) => {
                        spinner.finish_with_success(&format!("Restored backup for: {}", path));
                    }
                    Err(e) => {
                        spinner.finish_with_error(&format!("Restore failed for {}: {}", path, e));
                        return Err(e);
                    }
                }
            } else {
                return Err(DotfError::Operation(
                    "No restore action specified".to_string(),
                ));
            }
        }
        None => {
            // Show symlink status by default
            let spinner = Spinner::new("Checking symlinks...");
            let status_service = create_status_service();

            let status = match status_service.get_status().await {
                Ok(status) => {
                    spinner.finish_and_clear();
                    status
                }
                Err(e) => {
                    spinner.finish_with_error(&format!("Failed to check symlinks: {}", e));
                    return Err(e);
                }
            };

            if !status.initialized {
                println!("{}", formatter.error("Dotf is not initialized"));
                println!(
                    "{}",
                    formatter.info("Run 'dotf init --repo <repository>' to get started")
                );
                return Ok(());
            }

            // Show symlinks summary
            println!(
                "{}",
                ui.symlinks_status_summary(
                    status.symlinks.total,
                    status.symlinks.valid,
                    status.symlinks.missing,
                    status.symlinks.broken,
                    status.symlinks.conflicts,
                    status.symlinks.invalid_targets,
                    status.symlinks.modified,
                )
            );

            // Display detailed status for each symlink if any exist
            if !status.symlinks.details.is_empty() {
                let symlink_details: Vec<SymlinkDetail> = status
                    .symlinks
                    .details
                    .iter()
                    .map(|detail| SymlinkDetail {
                        status: detail.status.clone(),
                        target_path: detail.target_path.clone(),
                        source_path: detail.source_path.clone(),
                        current_target: detail.current_target.clone(),
                    })
                    .collect();

                let filesystem = RealFileSystem::new();
                let repo_path = filesystem.dotf_repo_path();
                println!("{}", ui.symlinks_status_table(&symlink_details, &repo_path));
            }
        }
    }

    Ok(())
}

fn create_status_service(
) -> StatusService<crate::core::repository::GitRepository, crate::core::filesystem::RealFileSystem>
{
    use crate::core::repository::GitRepository;

    let repository = GitRepository::new();
    let filesystem = RealFileSystem::new();

    StatusService::new(repository, filesystem)
}
