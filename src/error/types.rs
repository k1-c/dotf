use thiserror::Error;

pub type DotfResult<T> = Result<T, DotfError>;

#[derive(Error, Debug)]
pub enum DotfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("Script execution failed: {0}")]
    ScriptExecution(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Symlink error: {0}")]
    Symlink(String),

    #[error("User cancelled operation")]
    UserCancelled,

    #[error("User cancellation")]
    UserCancellation,

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Not initialized: Please run 'dotf init' first")]
    NotInitialized,

    #[error("Operation error: {0}")]
    Operation(String),

    #[error("Platform error: {0}")]
    Platform(String),
}

impl From<toml::de::Error> for DotfError {
    fn from(err: toml::de::Error) -> Self {
        DotfError::Serialization(err.to_string())
    }
}

impl From<toml::ser::Error> for DotfError {
    fn from(err: toml::ser::Error) -> Self {
        DotfError::Serialization(err.to_string())
    }
}

impl From<serde_json::Error> for DotfError {
    fn from(err: serde_json::Error) -> Self {
        DotfError::Serialization(err.to_string())
    }
}

impl From<git2::Error> for DotfError {
    fn from(err: git2::Error) -> Self {
        DotfError::Git(err.to_string())
    }
}

impl From<reqwest::Error> for DotfError {
    fn from(err: reqwest::Error) -> Self {
        DotfError::Network(err.to_string())
    }
}
