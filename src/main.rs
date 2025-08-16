use clap::Parser;
use dott::cli::{Cli, Commands};
use dott::error::DottResult;
use std::process;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

async fn run() -> DottResult<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { repo } => {
            println!("Init command - repo: {:?}", repo);
            Ok(())
        }
        Commands::Install { target } => {
            println!("Install command - target: {:?}", target);
            Ok(())
        }
        Commands::Status { quiet } => {
            println!("Status command - quiet: {}", quiet);
            Ok(())
        }
        Commands::Sync { force } => {
            println!("Sync command - force: {}", force);
            Ok(())
        }
        Commands::Symlinks { action } => {
            println!("Symlinks command - action: {:?}", action);
            Ok(())
        }
        Commands::Config { repo, edit } => {
            println!("Config command - repo: {}, edit: {}", repo, edit);
            Ok(())
        }
    }
}
