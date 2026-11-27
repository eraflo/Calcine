//! Long documents: the passages that match the message, or a skim.

use std::fmt::Write as _;

use super::format::{escape, excerpt, page_word};
use super::spread::spread;
use super::{
    Candidate, ContextMode, DocumentContext, MATCHED_SHARE, PREAMBLE_TOKENS, Source,
    WRAPPER_TOKENS, count,
};
use crate::search::Index;

/// The passages that fit in `budget`: best matches first, then passages
/// spread over each document when the message matches little.
pub(super) fn pick(
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
