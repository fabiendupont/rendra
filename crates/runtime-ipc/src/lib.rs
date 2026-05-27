pub mod bridge;
pub mod command;
pub mod event;

#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("invalid arguments: {0}")]
    InvalidArgs(String),
    #[error("handler error: {0}")]
    HandlerError(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
