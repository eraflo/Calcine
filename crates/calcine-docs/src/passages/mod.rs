//! A document's text, cleaned and cut into passages of a few hundred
//! tokens: small enough that several fit next to a conversation in a 4096
//! token window, large enough to keep a thought together.
//!
//! - `clean`: whitespace and invisible characters, words split across lines
//! - `paragraphs`: hard-wrapped lines joined back, lists kept
//! - `pack`: paragraphs grouped into passages of the right size

mod clean;
mod pack;
mod paragraphs;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::extract::Extracted;
use clean::clean;
use pack::pack;
use paragraphs::{collapse_spaces, paragraphs};

/// A piece of a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Passage {
    /// Page (or slide) number, from 1, when the document has pages.
    pub page: Option<u32>,
    pub text: String,
}

/// Cut a document into passages. Passages never span two pages, so each
/// one can name its page.
pub fn passages(document: &Extracted) -> Vec<Passage> {
    let keep_lines = document.kind.keeps_lines();
    let mut out = Vec::new();
    for (index, page) in document.pages.iter().enumerate() {
        let number = document
            .paged
            .then(|| u32::try_from(index + 1).unwrap_or(u32::MAX));
        let cleaned = clean(page, document.kind);
        let found = if document.kind.lines_are_paragraphs() {
            cleaned
                .split('\n')
                .map(|line| collapse_spaces(line.trim()))
                .filter(|line| !line.is_empty())
                .collect()
        } else {
            paragraphs(&cleaned, keep_lines)
        };
        for text in pack(&found) {
            out.push(Passage { page: number, text });
        }
    }
    out
}
