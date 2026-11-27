//! What the model reads of the attached documents, within the tokens the
//! conversation leaves free.
//!
//! When everything fits, the documents are read whole (the app puts them in
//! the system prompt, where they stay the same from one message to the next
//! and GenieX reuses what it already read). Otherwise the passages that best
//! match the message are picked; questions that match no words ("summarize
//! this") get passages spread over the whole document instead.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::extract::DocumentKind;
use crate::search::Index;
use crate::store::StoredDocument;

/// Tokens for the instructions before the documents…
const PREAMBLE_TOKENS: u32 = 60;
/// …and around each document or passage (its tag, name and page).
const WRAPPER_TOKENS: u32 = 20;
/// Characters of a passage shown as a source under the reply.
const EXCERPT_CHARS: usize = 320;
/// Below this share of the budget, keyword matches are topped up with
/// passages from across the documents.
const MATCHED_SHARE: f32 = 0.5;

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
        return full(candidates, whole, total_passages);
    }
    passages(candidates, queries, budget, total_passages)
}

fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// "page" or "slide", for the model's tags.
fn page_word(kind: DocumentKind) -> &'static str {
    if kind == DocumentKind::Slides {
        "slide"
    } else {
        "page"
    }
}

fn escape(name: &str) -> String {
    name.replace('"', "'")
}

fn excerpt(text: &str) -> String {
    if text.chars().count() <= EXCERPT_CHARS {
        return text.to_owned();
    }
    let cut: String = text.chars().take(EXCERPT_CHARS).collect();
    let cut = cut.rfind(' ').map_or(cut.as_str(), |space| &cut[..space]);
    format!("{}…", cut.trim_end())
}

fn full(candidates: &[Candidate<'_>], tokens: u32, total: usize) -> DocumentContext {
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

fn passages(
    candidates: &[Candidate<'_>],
    queries: &[(&str, f32)],
    budget: u32,
    total: usize,
) -> DocumentContext {
    // Every passage of every document, ranked together.
    let all: Vec<(usize, usize)> = candidates
        .iter()
        .enumerate()
        .flat_map(|(document, candidate)| {
            (0..candidate.document.passages.len()).map(move |passage| (document, passage))
        })
        .collect();
    let index = Index::new(
        all.iter()
            .map(|&(d, p)| candidates[d].document.passages[p].text.as_str()),
    );
    let scores = index.scores(queries);
    let cost = |(d, p): (usize, usize)| {
        candidates[d].tokens.get(p).copied().unwrap_or(u32::MAX / 4) + WRAPPER_TOKENS
    };

    let room = budget.saturating_sub(PREAMBLE_TOKENS);
    let mut chosen: Vec<(usize, usize)> = Vec::new();
    let mut used = 0u32;
    let take = |item: (usize, usize), limit: u32, used: &mut u32, chosen: &mut Vec<_>| {
        let tokens = cost(item);
        if !chosen.contains(&item) && *used + tokens <= limit {
            *used += tokens;
            chosen.push(item);
        }
    };

    // When the message itself matches nothing ("summarize it"), earlier
    // messages may still point somewhere, but only get half the room: the
    // rest skims the documents.
    let message_matches = queries.first().is_some_and(|query| {
        index
            .scores(std::slice::from_ref(query))
            .iter()
            .any(|&score| score > 0.0)
    });
    let ranked_room = if message_matches { room } else { room / 2 };

    // Best matches first.
    let mut ranked: Vec<usize> = (0..all.len()).filter(|&i| scores[i] > 0.0).collect();
    ranked.sort_by(|&a, &b| scores[b].total_cmp(&scores[a]).then(a.cmp(&b)));
    for i in ranked {
        take(all[i], ranked_room, &mut used, &mut chosen);
    }

    // Few matches: add passages spread over each document, its start first,
    // taking turns between documents.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let matched_enough = message_matches && used as f32 >= room as f32 * MATCHED_SHARE;
    if !matched_enough {
        let spreads: Vec<Vec<usize>> = candidates
            .iter()
            .map(|candidate| spread(candidate.document.passages.len()))
            .collect();
        let longest = spreads.iter().map(Vec::len).max().unwrap_or(0);
        for step in 0..longest {
            for (document, order) in spreads.iter().enumerate() {
                if let Some(&passage) = order.get(step) {
                    take((document, passage), room, &mut used, &mut chosen);
                }
            }
        }
    }

    // Read in document order.
    chosen.sort_unstable();
    let mut text = String::from(
        "Passages from the user's documents that may help with the message below. The \
         documents are too long to read whole: if these passages don't hold the answer, \
         say so rather than guessing.\n",
    );
    let mut sources: Vec<Source> = Vec::new();
    for &(d, p) in &chosen {
        let info = &candidates[d].document.info;
        let passage = &candidates[d].document.passages[p];
        let _ = write!(text, "\n<passage document=\"{}\"", escape(&info.name));
        if let Some(page) = passage.page {
            let _ = write!(text, " {}=\"{page}\"", page_word(info.kind));
        }
        let _ = write!(text, ">\n{}\n</passage>\n", passage.text);
        let known = sources
            .iter()
            .any(|source| source.document_id == info.id && source.page == passage.page);
        if !known {
            sources.push(Source {
                document_id: info.id.clone(),
                name: info.name.clone(),
                kind: info.kind,
                page: passage.page,
                excerpt: excerpt(&passage.text),
            });
        }
    }
    DocumentContext {
        mode: ContextMode::Passages,
        text,
        tokens: used + PREAMBLE_TOKENS,
        sources,
        passages_read: count(chosen.len()),
        passages_total: count(total),
    }
}

/// Passage indexes of a document of `length` passages, in the order to add
/// them when skimming: the start, then ever finer splits of the rest.
fn spread(length: usize) -> Vec<usize> {
    let mut order = Vec::with_capacity(length);
    let mut seen = vec![false; length];
    let mut parts = 1;
    while order.len() < length {
        for part in 0..parts {
            let at = part * length / parts;
            if at < length && !seen[at] {
                seen[at] = true;
                order.push(at);
            }
        }
        if parts >= length {
            // Whatever the splits missed.
            order.extend((0..length).filter(|&at| !seen[at]));
            break;
        }
        parts *= 2;
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::passages::Passage;
    use crate::store::DocumentInfo;

    fn document(id: &str, name: &str, texts: &[&str]) -> StoredDocument {
        StoredDocument {
            info: DocumentInfo {
                id: id.to_owned(),
                name: name.to_owned(),
                kind: DocumentKind::Pdf,
                pages: Some(count(texts.len())),
                chars: 0,
                bytes: 0,
                passages: count(texts.len()),
            },
            passages: texts
                .iter()
                .enumerate()
                .map(|(index, text)| Passage {
                    page: Some(count(index + 1)),
                    text: (*text).to_owned(),
                })
                .collect(),
        }
    }

    const PASSAGES: [&str; 4] = [
        "Introduction to the yearly report of the company.",
        "The 2027 budget plans 4 million euros for research.",
        "Hiring: twelve engineers will join in the spring.",
        "Conclusion and thanks to all the teams.",
    ];

    #[test]
    fn short_documents_are_read_whole() {
        let report = document("a", "report.pdf", &PASSAGES);
        let tokens = [20, 20, 20, 20];
        let context = select(
            &[Candidate {
                document: &report,
                tokens: &tokens,
            }],
            &[("budget?", 1.0)],
            1000,
        );
        assert_eq!(context.mode, ContextMode::Full);
        assert!(
            context
                .text
                .contains("<document name=\"report.pdf\">\n[page 1]\nIntroduction")
        );
        assert!(context.text.contains("[page 4]\nConclusion"));
        assert_eq!(context.sources.len(), 1);
        assert_eq!(context.passages_read, 4);
    }

    #[test]
    fn long_documents_give_their_best_passages() {
        let report = document("a", "report.pdf", &PASSAGES);
        let tokens = [100, 100, 100, 100];
        // Room for two passages.
        let budget = PREAMBLE_TOKENS + 2 * (100 + WRAPPER_TOKENS);
        let context = select(
            &[Candidate {
                document: &report,
                tokens: &tokens,
            }],
            &[("What's the budget for research?", 1.0)],
            budget,
        );
        assert_eq!(context.mode, ContextMode::Passages);
        assert!(
            context.text.contains("page=\"2\">\nThe 2027 budget"),
            "{}",
            context.text
        );
        assert_eq!(context.sources[0].page, Some(2));
        assert!(context.tokens <= budget);
        assert_eq!(context.passages_total, 4);
    }

    #[test]
    fn summaries_skim_the_whole_document() {
        let report = document("a", "report.pdf", &PASSAGES);
        let tokens = [100, 100, 100, 100];
        let budget = PREAMBLE_TOKENS + 2 * (100 + WRAPPER_TOKENS);
        let context = select(
            &[Candidate {
                document: &report,
                tokens: &tokens,
            }],
            &[("Summarize it", 1.0)],
            budget,
        );
        // The start, then the middle.
        let pages: Vec<_> = context.sources.iter().map(|s| s.page).collect();
        assert_eq!(pages, vec![Some(1), Some(3)]);
    }

    #[test]
    fn a_summary_after_a_question_still_skims() {
        let report = document("a", "report.pdf", &PASSAGES);
        let tokens = [100, 100, 100, 100];
        let budget = PREAMBLE_TOKENS + 2 * (100 + WRAPPER_TOKENS);
        let context = select(
            &[Candidate {
                document: &report,
                tokens: &tokens,
            }],
            &[
                ("Summarize it", 1.0),
                ("How many engineers are hired?", 0.5),
            ],
            budget,
        );
        // Half for the earlier question (hiring, page 3), half from the start.
        let pages: Vec<_> = context.sources.iter().map(|s| s.page).collect();
        assert_eq!(pages, vec![Some(1), Some(3)]);
    }

    #[test]
    fn several_documents_take_turns() {
        let first = document("a", "a.pdf", &PASSAGES);
        let second = document("b", "b.pdf", &PASSAGES);
        let tokens = [100, 100, 100, 100];
        let budget = PREAMBLE_TOKENS + 2 * (100 + WRAPPER_TOKENS);
        let context = select(
            &[
                Candidate {
                    document: &first,
                    tokens: &tokens,
                },
                Candidate {
                    document: &second,
                    tokens: &tokens,
                },
            ],
            &[("overview", 1.0)],
            budget,
        );
        let names: Vec<_> = context.sources.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["a.pdf", "b.pdf"]);
    }

    #[test]
    fn nothing_fits_in_no_room() {
        let report = document("a", "report.pdf", &PASSAGES);
        let tokens = [100, 100, 100, 100];
        let context = select(
            &[Candidate {
                document: &report,
                tokens: &tokens,
            }],
            &[("budget", 1.0)],
            10,
        );
        assert_eq!(context.passages_read, 0);
        assert_eq!(context.sources, Vec::new());
    }

    #[test]
    fn spread_starts_at_the_beginning_and_covers_everything() {
        assert_eq!(spread(4), vec![0, 2, 1, 3]);
        assert_eq!(spread(1), vec![0]);
        assert_eq!(spread(0), Vec::<usize>::new());
        let mut all = spread(13);
        all.sort_unstable();
        assert_eq!(all, (0..13).collect::<Vec<_>>());
    }

    #[test]
    fn excerpts_end_at_a_word() {
        let long = "word ".repeat(200);
        let short = excerpt(&long);
        assert!(short.ends_with("word…"));
        assert!(short.chars().count() <= EXCERPT_CHARS + 1);
    }
}
