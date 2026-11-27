use super::pack::MAX_CHARS;
use super::*;
use crate::kind::DocumentKind;

fn document(kind: DocumentKind, pages: &[&str], numbered: bool) -> Extracted {
    Extracted {
        kind,
        pages: pages.iter().map(|page| (*page).to_owned()).collect(),
        paged: numbered,
    }
}

#[test]
fn pdf_lines_are_joined_and_words_rejoined() {
    let passages = passages(&document(
        DocumentKind::Pdf,
        &["The quick brown fox jumps over the la-\nzy dog.\nIt was   fast.\n\nNew paragraph."],
        true,
    ));
    assert_eq!(passages.len(), 1);
    assert_eq!(passages[0].page, Some(1));
    assert_eq!(
        passages[0].text,
        "The quick brown fox jumps over the lazy dog. It was fast.\n\nNew paragraph."
    );
}

#[test]
fn lists_keep_their_lines() {
    let passages = passages(&document(
        DocumentKind::Text,
        &["Steps:\n1. Open\n2. Close\n- done"],
        false,
    ));
    assert_eq!(passages[0].page, None);
    assert_eq!(passages[0].text, "Steps:\n1. Open\n2. Close\n- done");
}

#[test]
fn word_titles_stay_apart() {
    let passages = passages(&document(
        DocumentKind::Word,
        &["7. Budget
The budget   plans more.

Last line."],
        false,
    ));
    assert_eq!(
        passages[0].text,
        "7. Budget\n\nThe budget plans more.\n\nLast line."
    );
}

#[test]
fn code_keeps_its_indentation() {
    let passages = passages(&document(
        DocumentKind::Code,
        &["fn main() {\n    println!(\"hi\");\n}\n"],
        false,
    ));
    assert_eq!(passages[0].text, "fn main() {\n    println!(\"hi\");\n}");
}

#[test]
fn passages_stay_on_their_page_and_near_the_target_size() {
    let sentence = "This sentence is about forty characters. ";
    let long_page = sentence.repeat(60);
    let passages = passages(&document(
        DocumentKind::Pdf,
        &[&long_page, "Short second page."],
        true,
    ));
    let first_page: Vec<_> = passages.iter().filter(|p| p.page == Some(1)).collect();
    assert!(first_page.len() >= 2, "a long page is cut");
    assert!(first_page.iter().all(|p| p.text.len() <= MAX_CHARS));
    assert!(
        first_page.iter().all(|p| p.text.ends_with('.')),
        "cut at sentences"
    );
    assert_eq!(passages.last().unwrap().page, Some(2));
    assert_eq!(passages.last().unwrap().text, "Short second page.");
}

#[test]
fn small_paragraphs_are_grouped() {
    let text = (0..20)
        .map(|index| format!("Paragraph number {index}."))
        .collect::<Vec<_>>()
        .join("\n\n");
    let passages = passages(&document(DocumentKind::Text, &[&text], false));
    assert!(passages.len() < 5, "{}", passages.len());
    assert!(
        passages[0]
            .text
            .contains("Paragraph number 0.\n\nParagraph number 1.")
    );
}

#[test]
fn unicode_is_cut_safely() {
    let text = "é".repeat(3000);
    let passages = passages(&document(DocumentKind::Text, &[&text], false));
    let joined: String = passages.iter().map(|p| p.text.as_str()).collect();
    assert_eq!(joined.chars().count(), 3000);
}

#[test]
fn invisible_characters_are_dropped() {
    let passages = passages(&document(
        DocumentKind::Text,
        &["soft\u{00AD}hyphen non\u{00A0}breaking\u{200B}"],
        false,
    ));
    assert_eq!(passages[0].text, "softhyphen non breaking");
}
