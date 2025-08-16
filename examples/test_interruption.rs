//! Test interruption handling manually

use dott::cli::ui::{InterruptionHandler, InterruptionContext};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("🧪 Testing interruption handling...");
    println!("Press Ctrl+C within 5 seconds to see the interruption message");
    
    let handler = InterruptionHandler::new();
    let interrupted = handler.setup_handlers().await;
    
    // Simulate a long-running operation
    let long_operation = async {
        for i in 1..=5 {
            println!("⏳ Operation step {}/5...", i);
            sleep(Duration::from_secs(1)).await;
        }
        println!("✅ Operation completed successfully!");
    };
    
    // Make it cancellable
    tokio::select! {
        _ = long_operation => {
            println!("🎉 Test completed without interruption");
        }
        _ = wait_for_interruption(interrupted) => {
            handler.show_interruption_message(InterruptionContext::Generic("Test operation".to_string()));
            std::process::exit(130);
        }
    }
}

async fn wait_for_interruption(interrupted: std::sync::Arc<std::sync::atomic::AtomicBool>) {
    use std::sync::atomic::Ordering;
    while !interrupted.load(Ordering::SeqCst) {
        sleep(Duration::from_millis(100)).await;
    }
}