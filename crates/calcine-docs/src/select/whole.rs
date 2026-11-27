//! Documents short enough to be read in full.

use std::fmt::Write as _;

use super::format::{escape, excerpt, page_word};
use super::{Candidate, ContextMode, DocumentContext, Source, count};

/// Every document, in full, with its pages marked.
pub(super) fn read_whole(
    candidates: &[Candidate<'_>],
    tokens: u32,
    total: usize,
) -> DocumentContext {
    let mut text = String::from(
        "The user shared the documents below. Answer from them when they're relevant, \
         and say which page you used when it helps.\n",
    );
    let mut sources = Vec::new();
    for candidate in candidates {
        let document = candidate.document;
        let info = &document.info;
        let _ = write!(text, "\n<document name=\"{}\">\n", escape(&info.name));
        let mut page = None;
        for passage in &document.passages {
            if passage.page.is_some() && passage.page != page {
                page = passage.page;
                let _ = writeln!(
                    text,
                    "[{} {}]",
                    page_word(info.kind),
                    page.unwrap_or_default()
                );
            }
            text.push_str(&passage.text);
            text.push_str("\n\n");
        }
        text.truncate(text.trim_end().len());
        text.push_str("\n</document>\n");
        sources.push(Source {
            document_id: info.id.clone(),
            name: info.name.clone(),
            kind: info.kind,
            page: None,
            excerpt: document
                .passages
                .first()
                .map(|passage| excerpt(&passage.text))
                .unwrap_or_default(),
        });
    }
    DocumentContext {
        mode: ContextMode::Full,
        text,
        tokens,
        sources,
        passages_read: count(total),
        passages_total: count(total),
    }
}
