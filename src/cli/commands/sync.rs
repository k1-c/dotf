use crate::cli::{MessageFormatter, Spinner};
use crate::core::{filesystem::RealFileSystem, repository::GitRepository};
use crate::error::DotfResult;
use crate::services::SyncService;

pub async fn handle_sync(force: bool) -> DotfResult<()> {
    let filesystem = RealFileSystem::new();
    let repository = GitRepository::new();
    let sync_service = SyncService::new(repository, filesystem);
    let formatter = MessageFormatter::new();

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
                    formatter.warning("Repository still has uncommitted changes after sync")
                );
            }
        }
        Err(e) => {
            spinner.finish_with_error(&format!("Sync failed: {}", e));
            return Err(e);
        }
    }

    Ok(())
}
