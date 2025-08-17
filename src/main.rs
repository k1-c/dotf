use clap::Parser;
use dotf::cli::args::{InstallTarget, SymlinksAction};
use dotf::cli::{
    commands::schema, BackupEntry, Cli, Commands, InstallAnimation, InterruptionContext,
    InterruptionHandler, MessageFormatter, OperationResult, OperationStatus, Spinner,
    SymlinkDetail, UiComponents,
};
use dotf::core::{
    filesystem::RealFileSystem, repository::GitRepository, scripts::SystemScriptExecutor,
};
use dotf::error::{DotfError, DotfResult};
use dotf::services::{
    ConfigService, EnhancedInitService, InstallService, StatusService, SyncService,
};
use dotf::traits::{filesystem::FileSystem, prompt::Prompt};
use dotf::utils::ConsolePrompt;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let formatter = MessageFormatter::new();

    if let Err(err) = run().await {
        eprintln!("{}", formatter.error(&format!("Error: {}", err)));
        process::exit(1);
    }
}

async fn run() -> DotfResult<()> {
    let cli = Cli::parse();
    let ui = UiComponents::new();
    let formatter = MessageFormatter::new();

    match cli.command {
        Commands::Init { repo } => {
            // Create interruption handler for graceful cancellation
            let interruption_handler = InterruptionHandler::new();
            let interrupted = interruption_handler.setup_handlers().await;

            // Create enhanced init service for animations
            let repository = GitRepository::new();
            let filesystem = RealFileSystem::new();
            let prompt = ConsolePrompt::new();
            let enhanced_init_service = EnhancedInitService::new(repository, filesystem, prompt);

            // Create animation handler
            let animation = InstallAnimation::new();

            // Show welcome banner
            let version = env!("CARGO_PKG_VERSION");
            animation.show_welcome(version).await;

            // Run initialization with animated progress and interruption handling
            let init_future = enhanced_init_service.init_with_progress(repo, |stage| {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        animation.show_stage(stage).await;
                    })
                });
            });

            // Make the operation cancellable
            tokio::select! {
                result = init_future => {
                    match result {
                        Ok(repo_url) => {
                            // Show completion animation
                            animation.show_completion(&repo_url).await;
                        }
                        Err(DotfError::UserCancellation) => {
                            // User pressed Ctrl+C during prompt, show cancellation message
                            interruption_handler.show_interruption_message(InterruptionContext::Initialization);
                            std::process::exit(130);
                        }
                        Err(e) => {
                            println!("\n{}", formatter.error(&format!("Initialization failed: {}", e)));
                            return Err(e);
                        }
                    }
                }
                _ = wait_for_interruption(interrupted.clone()) => {
                    interruption_handler.show_interruption_message(InterruptionContext::Initialization);
                    std::process::exit(130); // Standard exit code for SIGINT
                }
            }
        }
        Commands::Install { target } => {
            let install_service = create_install_service();
            match target {
                InstallTarget::Deps => {
                    let spinner = Spinner::new("Installing dependencies...");
                    match install_service.install_dependencies().await {
                        Ok(_) => {
                            spinner.finish_with_success("Dependencies installed successfully!")
                        }
                        Err(e) => {
                            spinner.finish_with_error(&format!(
                                "Dependencies installation failed: {}",
                                e
                            ));
                            return Err(e);
                        }
                    }
                }
                InstallTarget::Config => {
                    let spinner = Spinner::new("Installing configuration...");
                    match install_service.install_config().await {
                        Ok(_) => {
                            spinner.finish_with_success("Configuration installed successfully!")
                        }
                        Err(e) => {
                            spinner.finish_with_error(&format!(
                                "Configuration installation failed: {}",
                                e
                            ));
                            return Err(e);
                        }
                    }
                }
                InstallTarget::Custom { name } => {
                    let spinner = Spinner::new(&format!("Running custom script: {}", name));
                    match install_service.install_custom(&name).await {
                        Ok(_) => spinner.finish_with_success(&format!(
                            "Custom script '{}' completed successfully!",
                            name
                        )),
                        Err(e) => {
                            spinner.finish_with_error(&format!(
                                "Custom script '{}' failed: {}",
                                name, e
                            ));
                            return Err(e);
                        }
                    }
                }
            }
        }
        Commands::Status { quiet } => {
            let status_service = create_status_service();
            let spinner = Spinner::new("Checking status...");

            let status = match status_service.get_status().await {
                Ok(status) => {
                    spinner.finish_and_clear();
                    status
                }
                Err(e) => {
                    spinner.finish_with_error(&format!("Failed to get status: {}", e));
                    return Err(e);
                }
            };

            if quiet {
                // Just show basic status without details
                if status.initialized {
                    println!("{}", formatter.success("Initialized"));
                    if let Some(repo) = status.repository {
                        if !repo.status.is_clean {
                            println!(
                                "{}",
                                formatter.warning("Repository has uncommitted changes")
                            );
                        }
                        if repo.status.behind_count > 0 {
                            println!(
                                "{}",
                                formatter
                                    .info(&format!("{} commits behind", repo.status.behind_count))
                            );
                        }
                        if repo.status.ahead_count > 0 {
                            println!(
                                "{}",
                                formatter
                                    .info(&format!("{} commits ahead", repo.status.ahead_count))
                            );
                        }
                    }

                    let issues = status.symlinks.missing
                        + status.symlinks.broken
                        + status.symlinks.conflicts
                        + status.symlinks.invalid_targets;
                    if issues > 0 {
                        println!(
                            "{}",
                            formatter.warning(&format!("{} symlink issues", issues))
                        );
                    } else {
                        println!("{}", formatter.success("All symlinks OK"));
                    }
                } else {
                    println!("{}", formatter.error("Not initialized"));
                }
            } else {
                // Show detailed status with beautiful formatting
                if !status.initialized {
                    println!("{}", formatter.error("Dotf is not initialized"));
                    println!(
                        "{}",
                        formatter.info("Run 'dotf init --repo <repository>' to get started")
                    );
                    return Ok(());
                }

                // Repository status
                if let Some(repo) = status.repository {
                    println!(
                        "{}",
                        ui.repository_status(
                            repo.status.is_clean,
                            repo.status.behind_count,
                            repo.status.ahead_count,
                            &repo.status.current_branch,
                        )
                    );
                }

                // Symlinks status
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

                // Detailed symlinks if there are any
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
        Commands::Sync { force } => {
            let filesystem = RealFileSystem::new();
            let repository = GitRepository::new();
            let sync_service = SyncService::new(repository, filesystem);

            let spinner = Spinner::new("Syncing with remote repository...");

            match sync_service.sync(force).await {
                Ok(result) => {
                    if result.commits_pulled > 0 {
                        spinner.finish_with_success(&format!(
                            "Pulled {} commits on branch '{}'",
                            result.commits_pulled, result.current_branch
                        ));
                    } else {
                        spinner.finish_with_success(&format!(
                            "Repository is up to date on branch '{}'",
                            result.current_branch
                        ));
                    }

                    if result.had_uncommitted_changes {
                        println!(
                            "{}",
                            formatter.warning("Repository had uncommitted changes (forced sync)")
                        );
                    }

                    if !result.is_clean_after {
                        println!(
                            "{}",
                            formatter
                                .warning("Repository still has uncommitted changes after sync")
                        );
                    }
                }
                Err(e) => {
                    spinner.finish_with_error(&format!("Sync failed: {}", e));
                    return Err(e);
                }
            }
        }
        Commands::Symlinks { action } => {
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
                                spinner.finish_with_error(&format!(
                                    "Failed to load backup list: {}",
                                    e
                                ));
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
                                        ui.operation_results(
                                            "Failed Restorations",
                                            &operation_results
                                        )
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
                                spinner
                                    .finish_with_success(&format!("Restored backup for: {}", path));
                            }
                            Err(e) => {
                                spinner.finish_with_error(&format!(
                                    "Restore failed for {}: {}",
                                    path, e
                                ));
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
        }
        Commands::Config { repo, edit } => {
            let filesystem = RealFileSystem::new();
            let prompt = ConsolePrompt::new();
            let config_service = ConfigService::new(filesystem, prompt);

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
                        spinner.finish_with_error(&format!(
                            "Failed to get configuration summary: {}",
                            e
                        ));
                        return Err(e);
                    }
                }
            }
        }
        Commands::Schema { action } => {
            schema::handle_schema(action).await?;
        }
    }

    Ok(())
}

fn create_install_service() -> InstallService<RealFileSystem, SystemScriptExecutor, ConsolePrompt> {
    let filesystem = RealFileSystem::new();
    let script_executor = SystemScriptExecutor::new();
    let prompt = ConsolePrompt::new();

    InstallService::new(filesystem, script_executor, prompt)
}

fn create_status_service() -> StatusService<GitRepository, RealFileSystem> {
    let repository = GitRepository::new();
    let filesystem = RealFileSystem::new();

    StatusService::new(repository, filesystem)
}

/// Wait for interruption signal
async fn wait_for_interruption(interrupted: Arc<AtomicBool>) {
    while !interrupted.load(Ordering::SeqCst) {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
