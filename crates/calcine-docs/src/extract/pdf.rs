//! PDF text, page by page.

use crate::{Error, Result};

/// With `pdf-extract`: fonts decoded, words and lines laid out properly.
pub(super) fn detailed(bytes: &[u8]) -> Result<Vec<String>> {
    pdf_extract::extract_text_from_mem_by_pages(bytes).map_err(|err| {
        let message = err.to_string();
        if message.to_ascii_lowercase().contains("encrypt") {
            Error::Protected
        } else {
            Error::Read(message)
        }
    })
}

/// With `lopdf`: plainer, for the files `pdf-extract` fails on.
pub(super) fn simple(bytes: &[u8]) -> Result<Vec<String>> {
    let document = lopdf::Document::load_mem(bytes).map_err(|err| Error::Read(err.to_string()))?;
    if document.is_encrypted() {
        return Err(Error::Protected);
    }
    Ok(document
        .get_pages()
        .keys()
        .map(|&number| document.extract_text(&[number]).unwrap_or_default())
        .collect())
}
