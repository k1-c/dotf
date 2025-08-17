use crate::cli::{MessageFormatter, Spinner, SymlinkDetail, UiComponents};
use crate::core::{filesystem::RealFileSystem, repository::GitRepository};
use crate::error::DotfResult;
use crate::services::StatusService;
use crate::traits::filesystem::FileSystem;

pub async fn handle_status(quiet: bool) -> DotfResult<()> {
    let status_service = create_status_service();
    let formatter = MessageFormatter::new();
    let ui = UiComponents::new();
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
                        formatter.info(&format!("{} commits behind", repo.status.behind_count))
                    );
                }
                if repo.status.ahead_count > 0 {
                    println!(
                        "{}",
                        formatter.info(&format!("{} commits ahead", repo.status.ahead_count))
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

    Ok(())
}

fn create_status_service() -> StatusService<GitRepository, RealFileSystem> {
    let repository = GitRepository::new();
    let filesystem = RealFileSystem::new();

    StatusService::new(repository, filesystem)
}
