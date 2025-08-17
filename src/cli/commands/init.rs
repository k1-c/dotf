use crate::cli::{InstallAnimation, InterruptionContext, InterruptionHandler, MessageFormatter};
use crate::core::{filesystem::RealFileSystem, repository::GitRepository};
use crate::error::{DotfError, DotfResult};
use crate::services::EnhancedInitService;
use crate::utils::ConsolePrompt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub async fn handle_init(repo: Option<String>) -> DotfResult<()> {
    let formatter = MessageFormatter::new();

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

    Ok(())
}

/// Wait for interruption signal
async fn wait_for_interruption(interrupted: Arc<AtomicBool>) {
    while !interrupted.load(Ordering::SeqCst) {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
