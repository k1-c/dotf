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
    init_service::InitService,
    install_service::InstallService,
    status_service::StatusService,
};
use dott::traits::filesystem::FileSystem;
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
        Commands::Sync { force: _ } => {
            return Err(DottError::Operation("Sync command not yet implemented".to_string()));
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
                        return Err(DottError::Operation("Restore all not yet implemented".to_string()));
                    } else if let Some(_path) = filepath {
                        return Err(DottError::Operation("Restore specific file not yet implemented".to_string()));
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
            
            if repo {
                // Show repository configuration
                let config_path = format!("{}/dott.toml", filesystem.dott_repo_path());
                if filesystem.exists(&config_path).await? {
                    let content = filesystem.read_to_string(&config_path).await?;
                    println!("📋 Repository Configuration (dott.toml):");
                    println!("{}", content);
                } else {
                    println!("❌ Repository configuration not found");
                    println!("   Expected at: {}", config_path);
                }
            } else if edit {
                // Edit local settings
                return Err(DottError::Operation("Edit settings not yet implemented".to_string()));
            } else {
                // Show both configurations
                let status_service = create_status_service();
                let config_status = status_service.get_config_status().await?;
                
                println!("⚙️  Configuration Status:");
                if config_status.valid {
                    println!("   Status: ✅ Valid");
                } else {
                    println!("   Status: ❌ Invalid");
                }
                println!("   Repository: {} v{}", config_status.repo_name, config_status.repo_version);
                println!("   Path: {}", config_status.path);
                println!("   Symlinks: {}", config_status.symlinks_count);
                println!("   Custom scripts: {}", config_status.custom_scripts_count);
                
                if !config_status.errors.is_empty() {
                    println!("   Errors:");
                    for error in &config_status.errors {
                        println!("     - {}", error);
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
