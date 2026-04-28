use thiserror::Error;

#[derive(Debug, Error)]
pub enum HiError {
    #[error("IPC connection failed: {0}")]
    IpcConnect(String),
    #[error("Serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Target not reachable: {0}")]
    TargetNotReachable(String),
}

pub type HiResult<T> = Result<T, HiError>;
