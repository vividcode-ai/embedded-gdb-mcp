use thiserror::Error;

#[derive(Error, Debug)]
pub enum GdbError {
    #[error("No active GDB session with ID: {0}")]
    SessionNotFound(String),

    #[error("GDB session is not ready")]
    GdbNotReady,

    #[error("GDB process error: {0}")]
    GdbProcessError(String),

    #[error("GDB command timed out")]
    GdbTimeout,

    #[error("GDB stdin is not available")]
    GdbStdinNotAvailable,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, GdbError>;
