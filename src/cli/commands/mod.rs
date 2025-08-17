pub mod config;
pub mod init;
pub mod install;
pub mod schema;
pub mod status;
pub mod symlinks;
pub mod sync;

// Re-export command handlers for easy access
pub use config::handle_config;
pub use init::handle_init;
pub use install::handle_install;
pub use schema::handle_schema;
pub use status::handle_status;
pub use symlinks::handle_symlinks;
pub use sync::handle_sync;
