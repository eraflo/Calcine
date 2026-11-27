//! Paragraphs grouped into passages of a few hundred tokens.

/// Passages aim for this many characters (about 200 tokens)…
const TARGET_CHARS: usize = 800;
/// …and a paragraph longer than this is cut at a sentence.
pub(super) const MAX_CHARS: usize = 1200;

/// Group paragraphs into passages near [`TARGET_CHARS`], cutting the long
/// ones at a sentence.
pub(super) fn pack(paragraphs: &[String]) -> Vec<String> {
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
