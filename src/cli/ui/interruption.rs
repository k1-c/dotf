//! Graceful interruption handling with beautiful exit messages

use crate::cli::ui::{MessageFormatter, Theme};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;

/// Manages graceful interruption handling
pub struct InterruptionHandler {
    formatter: MessageFormatter,
    theme: Theme,
    interrupted: Arc<AtomicBool>,
}

impl InterruptionHandler {
    /// Create a new interruption handler
    pub fn new() -> Self {
        Self {
            formatter: MessageFormatter::new(),
            theme: Theme::new(),
            interrupted: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Set up signal handlers and return a handle to check for interruption
    pub async fn setup_handlers(&self) -> Arc<AtomicBool> {
        let interrupted = self.interrupted.clone();

        #[cfg(unix)]
        {
            let interrupted_clone = interrupted.clone();
            tokio::spawn(async move {
                let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
                    .expect("Failed to create SIGINT handler");
                let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("Failed to create SIGTERM handler");

                tokio::select! {
                    _ = sigint.recv() => {
                        interrupted_clone.store(true, Ordering::SeqCst);
                    }
                    _ = sigterm.recv() => {
                        interrupted_clone.store(true, Ordering::SeqCst);
                    }
                }
            });
        }

        #[cfg(windows)]
        {
            let interrupted_clone = interrupted.clone();
            tokio::spawn(async move {
                signal::ctrl_c()
                    .await
                    .expect("Failed to setup Ctrl-C handler");
                interrupted_clone.store(true, Ordering::SeqCst);
            });
        }

        interrupted
    }

    /// Check if interrupted
    pub fn is_interrupted(&self) -> bool {
        self.interrupted.load(Ordering::SeqCst)
    }

    /// Display initialization cancellation message
    pub fn show_init_cancellation(&self) {
        println!("\n"); // Add some space

        // Stylish cancellation banner
        let border = "═".repeat(50);
        println!("{}", self.theme.muted(&border));
        println!("{}", self.theme.warning("🛑 Initialization Cancelled"));
        println!("{}", self.theme.muted(&border));

        println!();
        println!(
            "{}",
            self.formatter.info("No changes were made to your system.")
        );
        println!("{}", self.theme.muted("• No directories were created"));
        println!("{}", self.theme.muted("• No files were modified"));
        println!("{}", self.theme.muted("• No repositories were cloned"));

        println!();
        println!("{}", self.formatter.info("To initialize later, run:"));
        println!(
            "  {}",
            self.theme.command("dotf init --repo <your-repository-url>")
        );

        println!();
        println!("{}", self.theme.muted("Thank you for using Dotf! 👋"));
    }

    /// Display sync cancellation message
    pub fn show_sync_cancellation(&self) {
        println!("\n");
        println!("{}", self.formatter.warning("Sync operation cancelled"));
        println!("{}", self.theme.muted("Repository state unchanged"));
    }

    /// Display install cancellation message
    pub fn show_install_cancellation(&self) {
        println!("\n");
        println!("{}", self.formatter.warning("Installation cancelled"));
        println!(
            "{}",
            self.theme.muted("Partial changes may have been applied")
        );
    }

    /// Display generic operation cancellation
    pub fn show_operation_cancellation(&self, operation: &str) {
        println!("\n");
        println!(
            "{}",
            self.formatter.warning(&format!("{} cancelled", operation))
        );
    }

    /// Show elegant "user interrupted" message for any operation
    pub fn show_interruption_message(&self, context: InterruptionContext) {
        println!(); // Ensure we're on a new line

        match context {
            InterruptionContext::Initialization => self.show_init_cancellation(),
            InterruptionContext::Sync => self.show_sync_cancellation(),
            InterruptionContext::Install => self.show_install_cancellation(),
            InterruptionContext::Generic(op) => self.show_operation_cancellation(&op),
        }
    }
}

impl Default for InterruptionHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for different types of interruptions
#[derive(Debug, Clone)]
pub enum InterruptionContext {
    Initialization,
    Sync,
    Install,
    Generic(String),
}

/// Helper function to create a cancellable future
pub async fn cancellable<F, T>(
    future: F,
    interrupted: Arc<AtomicBool>,
    handler: &InterruptionHandler,
    context: InterruptionContext,
) -> Result<T, InterruptionError>
where
    F: std::future::Future<Output = T>,
{
    tokio::select! {
        result = future => Ok(result),
        _ = wait_for_interruption(interrupted) => {
            handler.show_interruption_message(context);
            Err(InterruptionError::UserCancelled)
        }
    }
}

/// Wait for interruption signal
async fn wait_for_interruption(interrupted: Arc<AtomicBool>) {
    while !interrupted.load(Ordering::SeqCst) {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

/// Error type for interruptions
#[derive(Debug, thiserror::Error)]
pub enum InterruptionError {
    #[error("Operation cancelled by user")]
    UserCancelled,
}

/// Macro to make any operation cancellable
#[macro_export]
macro_rules! cancellable_operation {
    ($future:expr, $handler:expr, $context:expr) => {{
        let interrupted = $handler.setup_handlers().await;
        $crate::cli::ui::interruption::cancellable($future, interrupted, $handler, $context).await
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_interruption_handler_creation() {
        let handler = InterruptionHandler::new();
        assert!(!handler.is_interrupted());
    }

    #[test]
    fn test_interruption_contexts() {
        let _init_ctx = InterruptionContext::Initialization;
        let _sync_ctx = InterruptionContext::Sync;
        let _install_ctx = InterruptionContext::Install;
        let _generic_ctx = InterruptionContext::Generic("test".to_string());
    }
}
