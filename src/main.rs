use clap::Parser;
use dott::cli::{Cli, Commands};
use dott::cli::args::{InstallTarget, SymlinksAction};
use dott::core::{
    filesystem::RealFileSystem,
    repository::GitRepository,
    scripts::SystemScriptExecutor,
};
use dott::error::{DottError, DottResult};
use dott::services::{
    ConfigService,
    InitService,
    InstallService,
    StatusService,
    SyncService,
};
use dott::traits::prompt::Prompt;
use dott::utils::ConsolePrompt;
use std::process;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("❌ Error: {}", err);
        process::exit(1);
    }
}

async fn run() -> DottResult<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { repo } => {
            let init_service = create_init_service();
            init_service.init(repo).await?;
        }
        Commands::Install { target } => {
            let install_service = create_install_service();
            match target {
                InstallTarget::Deps => {
                    install_service.install_dependencies().await?;
                }
                InstallTarget::Config => {
                    install_service.install_config().await?;
                }
                InstallTarget::Custom { name } => {
                    install_service.install_custom(&name).await?;
                }
            }
        }
        Commands::Status { quiet } => {
            let status_service = create_status_service();
            if quiet {
                // Just show basic status without details
                let status = status_service.get_status().await?;
                if status.initialized {
                    println!("✅ Initialized");
                    if let Some(repo) = status.repository {
                        if !repo.status.is_clean {
                            println!("⚠️  Repository has uncommitted changes");
                        }
                        if repo.status.behind_count > 0 {
                            println!("⬇️  {} commits behind", repo.status.behind_count);
                        }
                        if repo.status.ahead_count > 0 {
                            println!("⬆️  {} commits ahead", repo.status.ahead_count);
                        }
                    }
                    
                    let issues = status.symlinks.missing + status.symlinks.broken + 
                               status.symlinks.conflicts + status.symlinks.invalid_targets;
                    if issues > 0 {
                        println!("⚠️  {} symlink issues", issues);
                    } else {
                        println!("✅ All symlinks OK");
                    }
                } else {
                    println!("❌ Not initialized");
                }
            } else {
                status_service.print_status().await?;
            }
        }
        Commands::Sync { force } => {
            let filesystem = RealFileSystem::new();
            let repository = GitRepository::new();
            let sync_service = SyncService::new(repository, filesystem);

            match sync_service.sync(force).await {
                Ok(result) => {
                    if result.commits_pulled > 0 {
                        println!("🔄 Pulled {} commits on branch '{}'", result.commits_pulled, result.current_branch);
                    } else {
                        println!("✅ Repository is up to date on branch '{}'", result.current_branch);
                    }
                    
                    if result.had_uncommitted_changes {
                        println!("⚠️  Repository had uncommitted changes (forced sync)");
                    }
                    
                    if !result.is_clean_after {
                        println!("⚠️  Repository still has uncommitted changes after sync");
                    }
                }
                Err(e) => {
                    println!("❌ Sync failed: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::Symlinks { action } => {
            match action {
                Some(SymlinksAction::Restore { list, all, filepath }) => {
                    if list {
                        // List available backups
                        let filesystem = RealFileSystem::new();
                        let prompt = ConsolePrompt::new();
                        let install_service = InstallService::new(
                            filesystem.clone(),
                            SystemScriptExecutor::new(),
                            prompt.clone(),
                        );
                        let backup_manager = install_service.get_backup_manager();
                        let manifest = backup_manager.load_manifest().await?;
                        
                        if manifest.entries.is_empty() {
                            println!("📦 No backups found");
                        } else {
                            println!("📦 Available backups:");
                            for (path, entry) in &manifest.entries {
                                println!("  {} -> {} ({})", 
                                        path, 
                                        entry.backup_path,
                                        entry.created_at.format("%Y-%m-%d %H:%M:%S")
                                );
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
                        
                        let confirm = prompt.confirm("⚠️  This will restore ALL backed up files, potentially overwriting current files. Continue?").await?;
                        if !confirm {
                            println!("❌ Restore cancelled");
                            return Ok(());
                        }
                        
                        match backup_manager.restore_all_backups().await {
                            Ok(result) => {
                                println!("✅ Restored {} files", result.restored_count);
                                if !result.failed_restorations.is_empty() {
                                    println!("⚠️  {} failures:", result.failed_restorations.len());
                                    for failure in &result.failed_restorations {
                                        println!("  ❌ {}: {}", failure.path, failure.error);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("❌ Restore failed: {}", e);
                                process::exit(1);
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
                        
                        match backup_manager.restore_specific_backup(&path).await {
                            Ok(_) => {
                                println!("✅ Restored backup for: {}", path);
                            }
                            Err(e) => {
                                println!("❌ Restore failed for {}: {}", path, e);
                                process::exit(1);
                            }
                        }
                    } else {
                        return Err(DottError::Operation("No restore action specified".to_string()));
                    }
                }
                None => {
                    // Show symlink status by default
                    let status_service = create_status_service();
                    let status = status_service.get_status().await?;
                    
                    if !status.initialized {
                        println!("❌ Dott is not initialized");
                        return Ok(());
                    }
                    
                    println!("🔗 Symlinks Status:");
                    println!("   Total: {}", status.symlinks.total);
                    println!("   Valid: {} ✅", status.symlinks.valid);
                    if status.symlinks.missing > 0 {
                        println!("   Missing: {} ❌", status.symlinks.missing);
                    }
                    if status.symlinks.broken > 0 {
                        println!("   Broken: {} 💥", status.symlinks.broken);
                    }
                    if status.symlinks.conflicts > 0 {
                        println!("   Conflicts: {} ⚠️", status.symlinks.conflicts);
                    }
                    if status.symlinks.invalid_targets > 0 {
                        println!("   Invalid targets: {} 🎯", status.symlinks.invalid_targets);
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
                match config_service.show_repository_config().await {
                    Ok(content) => {
                        println!("📋 Repository Configuration (dott.toml):");
                        println!("{}", content);
                    }
                    Err(e) => {
                        println!("❌ {}", e);
                        process::exit(1);
                    }
                }
            } else if edit {
                // Edit local settings
                match config_service.edit_settings().await {
                    Ok(_) => {},
                    Err(e) => {
                        println!("❌ Failed to edit settings: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                // Show configuration summary
                match config_service.show_config_summary().await {
                    Ok(summary) => {
                        println!("⚙️  Configuration Summary:");
                        if summary.is_valid {
                            println!("   Status: ✅ Valid");
                        } else {
                            println!("   Status: ❌ Invalid");
                        }
                        
                        if let Some(name) = &summary.repo_name {
                            if let Some(version) = &summary.repo_version {
                                println!("   Repository: {} v{}", name, version);
                            }
                        }
                        
                        println!("   Symlinks: {}", summary.symlinks_count);
                        println!("   Scripts: {}", summary.scripts_count);
                        
                        if !summary.platforms_supported.is_empty() {
                            println!("   Platforms: {}", summary.platforms_supported.join(", "));
                        }
                        
                        if !summary.errors.is_empty() {
                            println!("   ❌ Errors:");
                            for error in &summary.errors {
                                println!("     - {}", error);
                            }
                        }
                        
                        if !summary.warnings.is_empty() {
                            println!("   ⚠️  Warnings:");
                            for warning in &summary.warnings {
                                println!("     - {}", warning);
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to get configuration summary: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn create_init_service() -> InitService<GitRepository, RealFileSystem, ConsolePrompt> {
    let repository = GitRepository::new();
    let filesystem = RealFileSystem::new();
    let prompt = ConsolePrompt::new();
    
    InitService::new(repository, filesystem, prompt)
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
