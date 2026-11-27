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
