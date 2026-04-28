use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("IO error: {0}")]
    Io(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Hi not found")]
    HiNotFound,
    
    #[error("Session not found")]
    SessionNotFound,
    
    #[error("IPC error: {0}")]
    IpcError(String),
    
    #[error("Command failed: {0}")]
    CommandFailed(String),
}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_string()
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> AppError {
        AppError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> AppError {
        AppError::Serialization(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> AppError {
        AppError::Database(err.to_string())
    }
}