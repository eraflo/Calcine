use calcine_core::ErrorKind;
use serde::Serialize;
use specta::Type;

/// Error shape returned by every command, typed on the frontend.
#[derive(Debug, Serialize, Type)]
pub struct ApiError {
    pub kind: ErrorKind,
    pub message: String,
}

impl From<calcine_core::Error> for ApiError {
    fn from(err: calcine_core::Error) -> Self {
        Self {
            kind: err.kind(),
            message: err.to_string(),
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
