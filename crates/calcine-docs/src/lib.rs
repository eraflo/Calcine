//! Documents attached in Calcine's chat.
//!
//! Small models have small context windows (4096 tokens for most NPU
//! builds), so a document is rarely read whole. Calcine keeps its text cut
//! into passages and, for each message, shows the model the passages that
//! match it best, with their page, within the room the conversation leaves.
//!
//! - `extract`: the text of PDF, Office, OpenDocument, HTML and text files
//! - `passages`: cleaned text cut into passages, each with its page
//! - `search`: keyword ranking (BM25), no embedding model needed
//! - `select`: what the model is shown within a token budget
//! - `store`: extracted documents kept on disk, text only

mod extract;
mod passages;
mod search;
mod select;
mod store;

pub use extract::{
    DocumentKind, Extracted, PdfReader, SUPPORTED_EXTENSIONS, extract, extract_with, kind_of,
};
pub use passages::{Passage, passages};
pub use search::{Index, terms};
pub use select::{Candidate, ContextMode, DocumentContext, Source, select};
pub use store::{DocumentInfo, DocumentStore, StoredDocument};

/// Largest file Calcine reads.
pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;

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
