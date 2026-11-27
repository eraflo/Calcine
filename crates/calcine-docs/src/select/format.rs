//! How documents and passages are named for the model and the user.

use super::EXCERPT_CHARS;
use crate::kind::DocumentKind;

/// "page" or "slide", for the model's tags.
pub(super) fn page_word(kind: DocumentKind) -> &'static str {
    if kind == DocumentKind::Slides {
        "slide"
    } else {
        "page"
    }
}

/// Names go in an attribute: no double quotes.
pub(super) fn escape(name: &str) -> String {
    name.replace('"', "'")
}

/// The start of a passage, cut at a word.
pub(super) fn excerpt(text: &str) -> String {
    if text.chars().count() <= EXCERPT_CHARS {
        return text.to_owned();
    }
    let cut: String = text.chars().take(EXCERPT_CHARS).collect();
    let cut = cut.rfind(' ').map_or(cut.as_str(), |space| &cut[..space]);
    format!("{}…", cut.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excerpts_end_at_a_word() {
        let long = "word ".repeat(200);
        let short = excerpt(&long);
        assert!(short.ends_with("word…"));
        assert!(short.chars().count() <= EXCERPT_CHARS + 1);
    }
}
