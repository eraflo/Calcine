//! The text of a file, page by page when the format has pages.
//!
//! - `pdf`: `pdf-extract`, or `lopdf` when it fails. `pdf-extract` can panic
//!   on unusual files, so the app runs [`extract`] in a child process.
//! - `office/`: Word, PowerPoint and OpenDocument files, zip archives of XML
//! - `html`: web pages without their tags
//! - `encoding`: everything else is text in some encoding

mod encoding;
mod entities;
mod html;
mod office;
mod pdf;

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::kind::{DocumentKind, kind_of};
use crate::{Error, MAX_FILE_BYTES, Result};

/// A document's text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Extracted {
    pub kind: DocumentKind,
    /// One entry per page or slide; a single one for formats without pages.
    pub pages: Vec<String>,
    /// Whether `pages` are real pages (or slides) the user can look up.
    pub paged: bool,
}

/// How PDFs are read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PdfReader {
    /// `pdf-extract`: fonts decoded, words and lines laid out properly.
    #[default]
    Detailed,
    /// `lopdf`'s plainer text, for the files `pdf-extract` fails on.
    Simple,
}

/// Read the text of the file at `path`.
pub fn extract(path: &Path) -> Result<Extracted> {
    extract_with(path, PdfReader::Detailed)
}

/// Read the text of the file at `path`, PDFs with `reader`.
pub fn extract_with(path: &Path, reader: PdfReader) -> Result<Extracted> {
    let kind = kind_of(path).ok_or_else(|| {
        Error::Unsupported(
            path.extension()
                .and_then(|extension| extension.to_str())
                .map_or_else(|| "These".to_owned(), |extension| format!(".{extension}")),
        )
    })?;
    let size = std::fs::metadata(path)?.len();
    if size > MAX_FILE_BYTES {
        return Err(Error::TooLarge(format!("{} MB", size / 1024 / 1024)));
    }
    let bytes = std::fs::read(path)?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let (pages, numbered) = match (kind, extension.as_str()) {
        (DocumentKind::Pdf, _) if reader == PdfReader::Simple => (pdf::simple(&bytes)?, true),
        (DocumentKind::Pdf, _) => (pdf::detailed(&bytes)?, true),
        (DocumentKind::Word, "odt") => office::odt(&bytes)?,
        (DocumentKind::Word, _) => office::docx(&bytes)?,
        (DocumentKind::Slides, "odp") => office::odp(&bytes)?,
        (DocumentKind::Slides, _) => (office::pptx(&bytes)?, true),
        (DocumentKind::Web, _) => (vec![html::readable_text(&encoding::decode(&bytes))], false),
        _ => (vec![encoding::decode(&bytes)], false),
    };
    if pages.iter().all(|page| page.trim().is_empty()) {
        return Err(Error::NoText);
    }
    Ok(Extracted {
        kind,
        pages,
        paged: numbered,
    })
}
