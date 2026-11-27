//! How each Office vocabulary marks text, paragraphs, tables and pages.

/// How to read one XML vocabulary. Element names are local (no prefix).
pub(super) struct XmlRules {
    /// Text is only kept inside these elements; empty keeps it everywhere.
    pub(super) text_in: &'static [&'static str],
    /// A line break after each of these.
    pub(super) paragraphs: &'static [&'static str],
    pub(super) tabs: &'static [&'static str],
    pub(super) breaks: &'static [&'static str],
    /// A tab after each table cell, so rows stay on one line.
    pub(super) cells: &'static [&'static str],
    /// A new page starts at each of these…
    pub(super) page_starts: &'static [&'static str],
    /// …or at these, when they carry this attribute value (`w:br w:type="page"`).
    pub(super) page_breaks: &'static [(&'static str, &'static str, &'static str)],
    /// One space, for `text:s`.
    pub(super) spaces: &'static [&'static str],
    /// Text inside these is left out (comments, notes).
    pub(super) skip: &'static [&'static str],
}

pub(super) const DOCX: XmlRules = XmlRules {
    text_in: &["t"],
    paragraphs: &["p"],
    tabs: &["tab"],
    breaks: &["br", "cr"],
    cells: &["tc"],
    // Where Word last broke pages when the file was saved.
    page_starts: &["lastRenderedPageBreak"],
    page_breaks: &[("br", "type", "page")],
    spaces: &[],
    skip: &[],
};

pub(super) const ODT: XmlRules = XmlRules {
    text_in: &[],
    paragraphs: &["p", "h"],
    tabs: &["tab"],
    breaks: &["line-break"],
    cells: &["table-cell"],
    page_starts: &["soft-page-break"],
    page_breaks: &[],
    spaces: &["s"],
    skip: &["annotation", "note", "font-face-decls", "automatic-styles"],
};

pub(super) const ODP: XmlRules = XmlRules {
    text_in: &[],
    paragraphs: &["p", "h"],
    tabs: &["tab"],
    breaks: &["line-break"],
    cells: &["table-cell"],
    page_starts: &["page"],
    page_breaks: &[],
    spaces: &["s"],
    skip: &["notes", "annotation", "font-face-decls", "automatic-styles"],
};

pub(super) const PPTX: XmlRules = XmlRules {
    text_in: &["t"],
    paragraphs: &["p"],
    tabs: &[],
    breaks: &["br"],
    cells: &["tc"],
    page_starts: &[],
    page_breaks: &[],
    spaces: &[],
    skip: &[],
};
