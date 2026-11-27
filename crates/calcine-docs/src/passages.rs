//! A document's text, cleaned and cut into passages of a few hundred
//! tokens: small enough that several fit next to a conversation in a 4096
//! token window, large enough to keep a thought together.

use serde::{Deserialize, Serialize};

use crate::extract::{DocumentKind, Extracted};

/// Passages aim for this many characters (about 200 tokens)…
const TARGET_CHARS: usize = 800;
/// …and a paragraph longer than this is cut at a sentence.
const MAX_CHARS: usize = 1200;

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

/// Normalized whitespace, words split across lines rejoined, no control
/// characters.
fn clean(text: &str, kind: DocumentKind) -> String {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\u{00A0}' | '\u{2007}' | '\u{202F}' => out.push(' '),
            // Soft hyphens and zero-width characters.
            '\u{00AD}' | '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' => {}
            '\u{000C}' => out.push('\n'),
            '\n' | '\t' => out.push(character),
            other if other.is_control() => {}
            other => out.push(other),
        }
    }
    if kind == DocumentKind::Pdf {
        out = dehyphenate(&out);
    }
    out
}

/// `exam-\nple` → `example`, as PDFs break words at the end of a line.
fn dehyphenate(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0;
    while index < chars.len() {
        let hyphen_break = chars[index] == '-'
            && index > 0
            && chars[index - 1].is_alphabetic()
            && chars.get(index + 1) == Some(&'\n')
            && chars.get(index + 2).is_some_and(|next| next.is_lowercase());
        if hyphen_break {
            index += 2;
        } else {
            out.push(chars[index]);
            index += 1;
        }
    }
    out
}

/// Whether a line starts a list item, which must stay on its own line.
fn starts_item(line: &str) -> bool {
    let line = line.trim_start();
    if line.starts_with(['•', '◦', '▪', '-', '*', '–', '—', '·', '>', '#', '|']) {
        return true;
    }
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    digits > 0 && digits <= 3 && line[digits..].starts_with(['.', ')'])
}

/// Paragraphs: blocks between blank lines. Hard-wrapped lines are joined
/// back unless `keep_lines` (code and data), lists and tables stay as they
/// are.
fn paragraphs(text: &str, keep_lines: bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let flush = |current: &mut String, out: &mut Vec<String>| {
        let paragraph = current.trim_end().to_owned();
        if !paragraph.trim().is_empty() {
            out.push(paragraph);
        }
        current.clear();
    };
    for line in text.split('\n') {
        let line = if keep_lines {
            line.trim_end()
        } else {
            line.trim()
        };
        if line.is_empty() {
            flush(&mut current, &mut out);
            continue;
        }
        let line = if keep_lines {
            line.to_owned()
        } else {
            collapse_spaces(line)
        };
        if current.is_empty() {
            current = line;
        } else if keep_lines || starts_item(&line) || line.contains('\t') {
            current.push('\n');
            current.push_str(&line);
        } else {
            current.push(' ');
            current.push_str(&line);
        }
    }
    flush(&mut current, &mut out);
    out
}

fn collapse_spaces(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut previous_space = false;
    for character in line.chars() {
        let space = character == ' ';
        if !(space && previous_space) {
            out.push(character);
        }
        previous_space = space;
    }
    out
}

/// Group paragraphs into passages near [`TARGET_CHARS`], cutting the long
/// ones at a sentence.
fn pack(paragraphs: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for paragraph in paragraphs {
        for piece in split_long(paragraph) {
            if !current.is_empty() && current.len() + piece.len() + 2 > TARGET_CHARS {
                out.push(std::mem::take(&mut current));
            }
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(&piece);
        }
    }
    if !current.trim().is_empty() {
        out.push(current);
    }
    out
}

/// A paragraph in pieces of at most [`MAX_CHARS`]: at a sentence end when
/// there's one, else at a line break or a space.
fn split_long(paragraph: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = paragraph.trim();
    while rest.len() > MAX_CHARS {
        let window = floor_char_boundary(rest, MAX_CHARS);
        let head = &rest[..window];
        let cut = [". ", "? ", "! ", ".\n", "\n", "; ", ", ", " "]
            .iter()
            .find_map(|separator| {
                head.rfind(separator)
                    .filter(|&at| at > TARGET_CHARS / 2)
                    .map(|at| at + separator.len())
            })
            .unwrap_or(window);
        out.push(rest[..cut].trim().to_owned());
        rest = rest[cut..].trim_start();
    }
    if !rest.is_empty() {
        out.push(rest.to_owned());
    }
    out
}

fn floor_char_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
