use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dotf")]
#[command(about = "A modern dotfile management tool")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "k1-c")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize dotf with a remote repository
    Init {
        /// Repository URL
        #[arg(long)]
        repo: Option<String>,
    },
    /// Install various components
    Install {
        #[command(subcommand)]
        target: InstallTarget,
    },
    /// Show repository sync status
    Status {
        /// Show minimal status output
        #[arg(long)]
        quiet: bool,
    },
    /// Sync with remote repository
    Sync {
        /// Force sync (override local changes)
        #[arg(long)]
        force: bool,
    },
    /// Manage symlinks
    Symlinks {
        #[command(subcommand)]
        action: Option<SymlinksAction>,
    },
    /// View and edit dotf configuration
    Config {
        /// Show repository configuration (dotf.toml)
        #[arg(long)]
        repo: bool,
        /// Edit local settings (settings.json)
        #[arg(long)]
        edit: bool,
    },
    /// Manage dotf.toml schema
    Schema {
        #[command(subcommand)]
        action: SchemaAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum InstallTarget {
    /// Install system dependencies
    Deps,
    /// Install configuration symlinks
    Config,
    /// Run custom installation script
    Custom {
        /// Name of the custom script
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum SymlinksAction {
    /// Restore files from backup
    Restore {
        /// List available backups
        #[arg(long)]
        list: bool,
        /// Restore all backed up files
        #[arg(long)]
        all: bool,
        /// Specific file path to restore
        filepath: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SchemaAction {
    /// Generate dotf.toml template file
    Init,
    /// Validate dotf.toml syntax and structure
    Test {
        /// Validation target file path (default: ./dotf.toml)
        #[arg(long, short)]
        file: Option<String>,
        /// Continue execution even if validation errors are found
        #[arg(long)]
        ignore_errors: bool,
        /// Show only errors and warnings
        #[arg(long)]
        quiet: bool,
    },
}
