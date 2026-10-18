use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForgeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid process data: {0}")]
    Json(#[from] serde_json::Error),

    #[error("a process named `{0}` already exists")]
    DuplicateProcess(String),

    #[error("process `{0}` was not found")]
    ProcessNotFound(String),

    #[error("process `{0}` has no running PID")]
    ProcessNotRunning(String),

    #[error("the command cannot be empty")]
    EmptyCommand,

    #[error("failed to signal process: {0}")]
    Signal(#[from] nix::errno::Errno),
}

pub type Result<T> = std::result::Result<T, ForgeError>;
