//! What kind of document a file is, from its extension.

use std::path::Path;

use serde::{Deserialize, Serialize};
use specta::Type;

/// What a document is, for its icon and how its pages are named.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum DocumentKind {
    Pdf,
    /// Word and OpenDocument text.
    Word,
    /// PowerPoint and OpenDocument presentations: one page per slide.
    Slides,
    Web,
    /// CSV, JSON, XML, YAML…
    Data,
    Code,
    Text,
}

impl DocumentKind {
    /// Lines matter in these: they aren't joined back into paragraphs.
    pub(crate) fn keeps_lines(self) -> bool {
        matches!(self, Self::Data | Self::Code)
    }

    /// Each line is a paragraph of its own (titles stay apart from the text
    /// that follows); in PDFs and text files, lines are often hard-wrapped.
    pub(crate) fn lines_are_paragraphs(self) -> bool {
        matches!(self, Self::Word | Self::Slides | Self::Web)
    }
}

/// Extensions Calcine reads, by kind (lowercase, without the dot).
pub const SUPPORTED_EXTENSIONS: &[(DocumentKind, &[&str])] = &[
    (DocumentKind::Pdf, &["pdf"]),
    (DocumentKind::Word, &["docx", "odt"]),
    (DocumentKind::Slides, &["pptx", "odp"]),
    (DocumentKind::Web, &["html", "htm", "xhtml"]),
    (
        DocumentKind::Data,
        &[
            "csv", "tsv", "json", "jsonl", "xml", "yaml", "yml", "toml", "ini",
        ],
    ),
    (
        DocumentKind::Code,
        &[
            "rs", "py", "js", "mjs", "ts", "tsx", "jsx", "java", "kt", "c", "h", "cpp", "hpp",
            "cc", "cs", "go", "rb", "php", "swift", "sql", "sh", "ps1", "bat", "css", "scss",
            "lua", "r", "dart", "scala", "vue", "svelte",
        ],
    ),
    (
        DocumentKind::Text,
        &["txt", "md", "markdown", "rst", "log", "tex", "srt", "vtt"],
    ),
];

/// The kind of a file, from its extension; `None` when Calcine can't read it.
pub fn kind_of(path: &Path) -> Option<DocumentKind> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    SUPPORTED_EXTENSIONS
        .iter()
        .find(|(_, extensions)| extensions.contains(&extension.as_str()))
        .map(|(kind, _)| *kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_come_from_the_extension() {
        assert_eq!(kind_of(Path::new("Report.PDF")), Some(DocumentKind::Pdf));
        assert_eq!(kind_of(Path::new("notes.md")), Some(DocumentKind::Text));
        assert_eq!(kind_of(Path::new("main.rs")), Some(DocumentKind::Code));
        assert_eq!(kind_of(Path::new("deck.pptx")), Some(DocumentKind::Slides));
        assert_eq!(kind_of(Path::new("photo.jpg")), None);
        assert_eq!(kind_of(Path::new("README")), None);
    }
}
