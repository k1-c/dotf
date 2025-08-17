use clap::Parser;
use dotf::cli::{
    commands::{
        handle_config, handle_init, handle_install, handle_schema, handle_status, handle_symlinks,
        handle_sync,
    },
    Cli, Commands, MessageFormatter,
};
use dotf::error::DotfResult;
use std::process;

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

    match cli.command {
        Commands::Init { repo } => {
            handle_init(repo).await?;
        }
        Commands::Install { target } => {
            handle_install(target).await?;
        }
        Commands::Status { quiet } => {
            handle_status(quiet).await?;
        }
        Commands::Sync { force } => {
            handle_sync(force).await?;
        }
        Commands::Symlinks { action } => {
            handle_symlinks(action).await?;
        }
        Commands::Config { repo, edit } => {
            handle_config(repo, edit).await?;
        }
        Commands::Schema { action } => {
            handle_schema(action).await?;
        }
    }

    Ok(())
}
