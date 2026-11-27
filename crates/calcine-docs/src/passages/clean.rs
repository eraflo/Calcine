//! Text ready to cut: whitespace normalized, invisible characters gone.

use crate::kind::DocumentKind;

/// Normalized whitespace, words split across lines rejoined, no control
/// characters.
pub(super) fn clean(text: &str, kind: DocumentKind) -> String {
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
