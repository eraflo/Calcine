//! Why a file couldn't be read.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0} files aren't supported")]
    Unsupported(String),
    /// Scanned PDFs are images: there's no text to read.
    #[error("no text found in this file (a scanned document?)")]
    NoText,
    #[error("this file is too large ({0})")]
    TooLarge(String),
    #[error("this file is protected by a password")]
    Protected,
    #[error("couldn't read this file: {0}")]
    Read(String),
    #[error("document not found")]
    NotFound,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Why a file couldn't be read, for a message in the user's language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DocumentProblem {
    Unsupported,
    NoText,
    TooLarge,
    Protected,
    /// Damaged, or a variant of the format Calcine can't read.
    Unreadable,
}

impl Error {
    pub fn problem(&self) -> DocumentProblem {
        match self {
            Self::Unsupported(_) => DocumentProblem::Unsupported,
            Self::NoText => DocumentProblem::NoText,
            Self::TooLarge(_) => DocumentProblem::TooLarge,
            Self::Protected => DocumentProblem::Protected,
            Self::Read(_) | Self::NotFound | Self::Io(_) => DocumentProblem::Unreadable,
        }
    }
}
