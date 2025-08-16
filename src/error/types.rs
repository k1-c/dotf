use thiserror::Error;

pub type DottResult<T> = Result<T, DottError>;

#[derive(Error, Debug)]
pub enum DottError {
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
    
    #[error("Not initialized: Please run 'dott init' first")]
    NotInitialized,
    
    #[error("Operation error: {0}")]
    Operation(String),
    
    #[error("Platform error: {0}")]
    Platform(String),
}

impl From<toml::de::Error> for DottError {
    fn from(err: toml::de::Error) -> Self {
        DottError::Serialization(err.to_string())
    }
}

impl From<serde_json::Error> for DottError {
    fn from(err: serde_json::Error) -> Self {
        DottError::Serialization(err.to_string())
    }
}

impl From<git2::Error> for DottError {
    fn from(err: git2::Error) -> Self {
        DottError::Git(err.to_string())
    }
}

impl From<reqwest::Error> for DottError {
    fn from(err: reqwest::Error) -> Self {
        DottError::Network(err.to_string())
    }
}