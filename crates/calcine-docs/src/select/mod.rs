//! What the model reads of the attached documents, within the tokens the
//! conversation leaves free.
//!
//! When everything fits, the documents are read whole (the app puts them in
//! the system prompt, where they stay the same from one message to the next
//! and GenieX reuses what it already read). Otherwise the passages that best
//! match the message are picked; questions that match no words ("summarize
//! this") get passages spread over the whole document instead.
//!
//! - `whole`: documents short enough to read in full
//! - `passages`: the passages that match, or a skim of each document
//! - `spread`: the order in which to skim a document
//! - `format`: tags, page names and excerpts

mod format;
mod passages;
mod spread;
#[cfg(test)]
mod tests;
mod whole;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::kind::DocumentKind;
use crate::store::StoredDocument;

/// Tokens for the instructions before the documents…
pub(super) const PREAMBLE_TOKENS: u32 = 60;
/// …and around each document or passage (its tag, name and page).
pub(super) const WRAPPER_TOKENS: u32 = 20;
/// Characters of a passage shown as a source under the reply.
pub(super) const EXCERPT_CHARS: usize = 320;
/// Below this share of the budget, keyword matches are topped up with
/// passages from across the documents.
pub(super) const MATCHED_SHARE: f32 = 0.5;

/// A document and the tokens each of its passages takes.
#[derive(Debug, Clone, Copy)]
pub struct Candidate<'a> {
    pub document: &'a StoredDocument,
    /// One count per passage, in order.
    pub tokens: &'a [u32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ContextMode {
    /// The documents are read whole.
    Full,
    /// Too long: only the passages that match the message are read.
    Passages,
}

/// Where an answer may come from, shown under the reply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub document_id: String,
    pub name: String,
    pub kind: DocumentKind,
    /// Page or slide, for documents that have them.
    pub page: Option<u32>,
    /// The start of the passage.
    pub excerpt: String,
}

/// The documents' share of the next request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentContext {
    pub mode: ContextMode,
    /// What the model is given: the whole documents (for the system prompt)
    /// or the chosen passages (before the user's message).
    pub text: String,
    /// Tokens `text` takes, about.
    pub tokens: u32,
    pub sources: Vec<Source>,
    /// Passages read, out of all the documents' passages.
    pub passages_read: u32,
    pub passages_total: u32,
}

/// Pick what fits in `budget` tokens. `queries` are the user's messages,
/// newest first, with their weights.
pub fn select(
    candidates: &[Candidate<'_>],
    queries: &[(&str, f32)],
    budget: u32,
) -> DocumentContext {
    let total_passages: usize = candidates.iter().map(|c| c.document.passages.len()).sum();
    let whole: u32 = PREAMBLE_TOKENS
        + candidates
            .iter()
            .map(|c| c.tokens.iter().sum::<u32>() + WRAPPER_TOKENS)
            .sum::<u32>();
    if whole <= budget {
        return whole::read_whole(candidates, whole, total_passages);
    }
    passages::pick(candidates, queries, budget, total_passages)
}

pub(super) fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
