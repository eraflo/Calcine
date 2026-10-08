use serde::Serialize;
use specta::Type;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Every failure a Calcine service can report.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("GenieX isn't installed on this machine")]
    RuntimeNotFound,

    #[error("`geniex {command}` failed: {message}")]
    Command { command: String, message: String },

    #[error("`geniex {command}` didn't finish within {seconds}s")]
    Timeout { command: String, seconds: u64 },

    #[error("couldn't read GenieX output: {0}")]
    Parse(String),

    /// The user's input can't be used as is. The message is shown verbatim.
    #[error("{0}")]
    InvalidInput(String),

    #[error("cancelled")]
    Cancelled,

    #[error("{0} isn't available yet")]
    NotImplemented(&'static str),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Stable, machine-readable error category exposed to the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    RuntimeNotFound,
    Command,
    Timeout,
    Parse,
    InvalidInput,
    Cancelled,
    NotImplemented,
    Io,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::RuntimeNotFound => ErrorKind::RuntimeNotFound,
            Self::Command { .. } => ErrorKind::Command,
            Self::Timeout { .. } => ErrorKind::Timeout,
            Self::Parse(_) => ErrorKind::Parse,
            Self::InvalidInput(_) => ErrorKind::InvalidInput,
            Self::Cancelled => ErrorKind::Cancelled,
            Self::NotImplemented(_) => ErrorKind::NotImplemented,
            Self::Io(_) => ErrorKind::Io,
        }
    }
}
