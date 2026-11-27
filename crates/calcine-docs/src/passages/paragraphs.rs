//! Paragraphs out of lines.

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
pub(super) fn paragraphs(text: &str, keep_lines: bool) -> Vec<String> {
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

/// Runs of spaces as one.
pub(super) fn collapse_spaces(line: &str) -> String {
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
