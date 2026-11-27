//! The text of a file, page by page when the format has pages.
//!
//! PDF goes through `pdf-extract`; Word, PowerPoint and OpenDocument files
//! are zip archives of XML, read directly; HTML loses its tags; everything
//! else is text in some encoding. `pdf-extract` can panic on unusual files,
//! so the app runs [`extract`] in a child process.

use std::io::Read;
use std::path::Path;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{Error, MAX_FILE_BYTES, Result};

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

/// A document's text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Extracted {
    pub kind: DocumentKind,
    /// One entry per page or slide; a single one for formats without pages.
    pub pages: Vec<String>,
    /// Whether `pages` are real pages (or slides) the user can look up.
    pub paged: bool,
}

/// How PDFs are read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PdfReader {
    /// `pdf-extract`: fonts decoded, words and lines laid out properly.
    #[default]
    Detailed,
    /// `lopdf`'s plainer text, for the files `pdf-extract` fails on.
    Simple,
}

/// Read the text of the file at `path`.
pub fn extract(path: &Path) -> Result<Extracted> {
    extract_with(path, PdfReader::Detailed)
}

/// Read the text of the file at `path`, PDFs with `reader`.
pub fn extract_with(path: &Path, reader: PdfReader) -> Result<Extracted> {
    let kind = kind_of(path).ok_or_else(|| {
        Error::Unsupported(
            path.extension()
                .and_then(|extension| extension.to_str())
                .map_or_else(|| "These".to_owned(), |extension| format!(".{extension}")),
        )
    })?;
    let size = std::fs::metadata(path)?.len();
    if size > MAX_FILE_BYTES {
        return Err(Error::TooLarge(format!("{} MB", size / 1024 / 1024)));
    }
    let bytes = std::fs::read(path)?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let (pages, numbered) = match (kind, extension.as_str()) {
        (DocumentKind::Pdf, _) if reader == PdfReader::Simple => (pdf_simple(&bytes)?, true),
        (DocumentKind::Pdf, _) => (pdf(&bytes)?, true),
        (DocumentKind::Word, "odt") => office(&bytes, "content.xml", &ODT)?,
        (DocumentKind::Word, _) => office(&bytes, "word/document.xml", &DOCX)?,
        (DocumentKind::Slides, "odp") => office(&bytes, "content.xml", &ODP)?,
        (DocumentKind::Slides, _) => (pptx(&bytes)?, true),
        (DocumentKind::Web, _) => (vec![html_text(&decode(&bytes))], false),
        _ => (vec![decode(&bytes)], false),
    };
    if pages.iter().all(|page| page.trim().is_empty()) {
        return Err(Error::NoText);
    }
    Ok(Extracted {
        kind,
        pages,
        paged: numbered,
    })
}

fn pdf(bytes: &[u8]) -> Result<Vec<String>> {
    pdf_extract::extract_text_from_mem_by_pages(bytes).map_err(|err| {
        let message = err.to_string();
        if message.to_ascii_lowercase().contains("encrypt") {
            Error::Protected
        } else {
            Error::Read(message)
        }
    })
}

fn pdf_simple(bytes: &[u8]) -> Result<Vec<String>> {
    let document = lopdf::Document::load_mem(bytes).map_err(|err| Error::Read(err.to_string()))?;
    if document.is_encrypted() {
        return Err(Error::Protected);
    }
    Ok(document
        .get_pages()
        .keys()
        .map(|&number| document.extract_text(&[number]).unwrap_or_default())
        .collect())
}

/// Text in any common encoding: UTF-8 (with or without a BOM), UTF-16 with
/// a BOM, or Windows-1252 for older files.
pub(crate) fn decode(bytes: &[u8]) -> String {
    if let Some((encoding, _)) = encoding_rs::Encoding::for_bom(bytes) {
        return encoding.decode_with_bom_removal(bytes).0.into_owned();
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_owned(),
        Err(_) => encoding_rs::WINDOWS_1252.decode(bytes).0.into_owned(),
    }
}

// --- Office formats: XML inside a zip archive -------------------------------

/// How to read one XML vocabulary. Element names are local (no prefix).
struct XmlRules {
    /// Text is only kept inside these elements; empty keeps it everywhere.
    text_in: &'static [&'static str],
    /// A line break after each of these.
    paragraphs: &'static [&'static str],
    tabs: &'static [&'static str],
    breaks: &'static [&'static str],
    /// A tab after each table cell, so rows stay on one line.
    cells: &'static [&'static str],
    /// A new page starts at each of these…
    page_starts: &'static [&'static str],
    /// …or at these, when they carry this attribute value (`w:br w:type="page"`).
    page_breaks: &'static [(&'static str, &'static str, &'static str)],
    /// One space, for `text:s`.
    spaces: &'static [&'static str],
    /// Text inside these is left out (comments, notes).
    skip: &'static [&'static str],
}

const DOCX: XmlRules = XmlRules {
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

const ODT: XmlRules = XmlRules {
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

const ODP: XmlRules = XmlRules {
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

const PPTX: XmlRules = XmlRules {
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

/// Pages of an Office document stored as `entry` in the archive, and
/// whether page breaks were found.
fn office(bytes: &[u8], entry: &str, rules: &XmlRules) -> Result<(Vec<String>, bool)> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|err| Error::Read(format!("not a valid Office file ({err})")))?;
    let xml = read_entry(&mut archive, entry)?;
    let pages = xml_pages(&xml, rules)?;
    let numbered = pages.len() > 1;
    Ok((pages, numbered))
}

/// One page per slide, in order (`slide2.xml` before `slide10.xml`).
fn pptx(bytes: &[u8]) -> Result<Vec<String>> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|err| Error::Read(format!("not a valid PowerPoint file ({err})")))?;
    let mut slides: Vec<(u32, String)> = archive
        .file_names()
        .filter_map(|name| {
            let name = name.ok()?;
            let number = name
                .strip_prefix("ppt/slides/slide")?
                .strip_suffix(".xml")?
                .parse()
                .ok()?;
            Some((number, name.into_owned()))
        })
        .collect();
    slides.sort_unstable();
    slides
        .into_iter()
        .map(|(_, name)| {
            let xml = read_entry(&mut archive, &name)?;
            Ok(xml_pages(&xml, &PPTX)?.join("\n"))
        })
        .collect()
}

/// Largest XML part read from an archive: a zip bomb stops here.
const MAX_ENTRY_BYTES: u64 = 200 * 1024 * 1024;

fn read_entry(archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>, name: &str) -> Result<String> {
    let file = archive
        .by_name(name)
        .map_err(|err| Error::Read(format!("{name} is missing ({err})")))?;
    let mut xml = String::new();
    file.take(MAX_ENTRY_BYTES)
        .read_to_string(&mut xml)
        .map_err(|err| Error::Read(format!("{name}: {err}")))?;
    Ok(xml)
}

fn has(names: &[&str], name: &str) -> bool {
    names.contains(&name)
}

fn is_page_break(rules: &XmlRules, element: &BytesStart<'_>) -> bool {
    let name = element.local_name();
    let name = name.as_ref();
    if has(rules.page_starts, name) {
        return true;
    }
    rules
        .page_breaks
        .iter()
        .filter(|(tag, _, _)| *tag == name)
        .any(|(_, attribute, value)| {
            element.attributes().flatten().any(|candidate| {
                candidate.key.local_name().as_ref() == *attribute && candidate.value == *value
            })
        })
}

/// The text of an XML part, split into pages.
fn xml_pages(xml: &str, rules: &XmlRules) -> Result<Vec<String>> {
    let mut reader = Reader::from_str(xml);
    let mut pages = vec![String::new()];
    // Depth inside elements whose text is kept, and inside skipped ones.
    let mut text_depth = 0usize;
    let mut skip_depth = 0usize;
    let keep_everywhere = rules.text_in.is_empty();
    loop {
        let event = reader
            .read_event()
            .map_err(|err| Error::Read(format!("invalid XML ({err})")))?;
        let page = pages.last_mut().expect("there is always a page");
        match event {
            Event::Start(element) => {
                let name = element.local_name();
                let name = name.as_ref();
                if has(rules.skip, name) {
                    skip_depth += 1;
                } else if has(rules.text_in, name) {
                    text_depth += 1;
                }
                if skip_depth == 0 && is_page_break(rules, &element) && !page.trim().is_empty() {
                    pages.push(String::new());
                }
            }
            Event::Empty(element) => {
                if skip_depth > 0 {
                    continue;
                }
                let name = element.local_name();
                let name = name.as_ref();
                if is_page_break(rules, &element) {
                    if !page.trim().is_empty() {
                        pages.push(String::new());
                    }
                } else if has(rules.tabs, name) {
                    page.push('\t');
                } else if has(rules.breaks, name) || has(rules.paragraphs, name) {
                    page.push('\n');
                } else if has(rules.spaces, name) {
                    page.push(' ');
                }
            }
            Event::End(element) => {
                let name = element.local_name();
                let name = name.as_ref();
                if has(rules.skip, name) {
                    skip_depth = skip_depth.saturating_sub(1);
                } else if skip_depth == 0 {
                    if has(rules.text_in, name) {
                        text_depth = text_depth.saturating_sub(1);
                    } else if has(rules.paragraphs, name) {
                        page.push('\n');
                    } else if has(rules.cells, name) {
                        page.push('\t');
                    }
                }
            }
            Event::Text(text) if skip_depth == 0 && (keep_everywhere || text_depth > 0) => {
                page.push_str(&text.into_inner());
            }
            Event::CData(text) if skip_depth == 0 && (keep_everywhere || text_depth > 0) => {
                page.push_str(&text.into_inner());
            }
            Event::GeneralRef(reference)
                if skip_depth == 0 && (keep_everywhere || text_depth > 0) =>
            {
                if let Ok(Some(character)) = reference.resolve_char_ref() {
                    page.push(character);
                } else {
                    page.push_str(named_entity(&reference.into_inner()));
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if pages.len() > 1 && pages.last().is_some_and(|page| page.trim().is_empty()) {
        pages.pop();
    }
    Ok(pages)
}

fn named_entity(name: &str) -> &'static str {
    match name {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        "nbsp" => " ",
        _ => "",
    }
}

// --- HTML --------------------------------------------------------------------

/// Elements that start a new line.
const HTML_BLOCKS: &[&str] = &[
    "p",
    "div",
    "br",
    "li",
    "ul",
    "ol",
    "tr",
    "table",
    "section",
    "article",
    "header",
    "footer",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "blockquote",
    "pre",
    "hr",
    "dt",
    "dd",
    "main",
    "nav",
    "aside",
    "figure",
    "figcaption",
    "form",
];
/// Elements whose content isn't text to read.
const HTML_HIDDEN: &[&str] = &["script", "style", "noscript", "template", "svg", "head"];

/// The readable text of an HTML page: tags removed, entities decoded,
/// scripts and styles left out. Tolerant of broken markup.
pub(crate) fn html_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len() / 2);
    let mut rest = html;
    let mut hidden: Option<String> = None;
    while let Some(start) = rest.find('<') {
        if hidden.is_none() {
            out.push_str(&decode_entities(&rest[..start]));
        }
        rest = &rest[start..];
        // Comments may contain `>`.
        if let Some(comment) = rest.strip_prefix("<!--") {
            rest = comment.find("-->").map_or("", |end| &comment[end + 3..]);
            continue;
        }
        let Some(end) = rest.find('>') else {
            rest = "";
            break;
        };
        let tag = &rest[1..end];
        rest = &rest[end + 1..];
        let closing = tag.starts_with('/');
        let name: String = tag
            .trim_start_matches('/')
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect::<String>()
            .to_ascii_lowercase();
        if let Some(open) = &hidden {
            if closing && *open == name {
                hidden = None;
            }
            continue;
        }
        if !closing && HTML_HIDDEN.contains(&name.as_str()) && !tag.ends_with('/') {
            hidden = Some(name);
            continue;
        }
        if HTML_BLOCKS.contains(&name.as_str()) {
            out.push('\n');
        } else if !closing && (name == "td" || name == "th") {
            out.push('\t');
        }
    }
    if hidden.is_none() {
        out.push_str(&decode_entities(rest));
    }
    out
}

fn decode_entities(text: &str) -> String {
    if !text.contains('&') {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        let decoded = rest[1..]
            .find(';')
            .filter(|&end| end <= 10)
            .and_then(|end| {
                let name = &rest[1..=end];
                let character = if let Some(number) = name.strip_prefix('#') {
                    let code = match number.strip_prefix(['x', 'X']) {
                        Some(hex) => u32::from_str_radix(hex, 16).ok(),
                        None => number.parse().ok(),
                    };
                    code.and_then(char::from_u32).map(String::from)
                } else {
                    Some(named_entity(name))
                        .filter(|value| !value.is_empty())
                        .map(str::to_owned)
                };
                character.map(|character| (character, end + 2))
            });
        if let Some((character, length)) = decoded {
            out.push_str(&character);
            rest = &rest[length..];
        } else {
            out.push('&');
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    out
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

    #[test]
    fn text_in_any_common_encoding() {
        assert_eq!(decode("héllo".as_bytes()), "héllo");
        assert_eq!(decode(b"\xEF\xBB\xBFbom"), "bom");
        assert_eq!(decode(b"\xFF\xFEh\x00i\x00"), "hi");
        // Windows-1252: é is 0xE9.
        assert_eq!(decode(b"caf\xE9"), "café");
    }

    #[test]
    fn word_paragraphs_tables_and_page_breaks() {
        let xml = r#"<w:document xmlns:w="w"><w:body>
            <w:p><w:r><w:t>Hello</w:t><w:tab/><w:t xml:space="preserve"> world &amp; co</w:t></w:r></w:p>
            <w:tbl><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
            <w:p><w:r><w:br w:type="page"/><w:t>Second page</w:t></w:r></w:p>
            <w:p><w:r><w:instrText>PAGEREF _Toc1</w:instrText><w:t>end</w:t></w:r></w:p>
        </w:body></w:document>"#;
        let pages = xml_pages(xml, &DOCX).unwrap();
        assert_eq!(pages.len(), 2, "{pages:?}");
        assert!(pages[0].contains("Hello\t world & co\n"), "{:?}", pages[0]);
        assert!(pages[0].contains("A\n\tB\n\t"), "{:?}", pages[0]);
        assert!(pages[1].starts_with("Second page\n"), "{:?}", pages[1]);
        assert!(!pages[1].contains("PAGEREF"), "field codes are left out");
    }

    #[test]
    fn opendocument_spaces_and_notes() {
        let xml = r#"<office:document-content xmlns:text="t" xmlns:office="o"><office:body><office:text>
            <text:h>Title</text:h>
            <text:p>One<text:s/>two<text:tab/>three<office:annotation><text:p>hidden</text:p></office:annotation></text:p>
        </office:text></office:body></office:document-content>"#;
        let pages = xml_pages(xml, &ODT).unwrap();
        assert_eq!(pages.len(), 1);
        assert!(pages[0].contains("Title\n"));
        assert!(pages[0].contains("One two\tthree\n"), "{:?}", pages[0]);
        assert!(!pages[0].contains("hidden"));
    }

    #[test]
    fn html_without_tags_scripts_or_entities() {
        let html = "<html><head><title>x</title><style>p{}</style></head><body>\
            <h1>Café &amp; thé</h1><!-- a > b --><p>Line&nbsp;one<br>two &#233;t&#xE9;</p>\
            <script>alert('<p>')</script><table><tr><td>a</td><td>b</td></tr></table></body></html>";
        let text = html_text(html);
        assert!(text.contains("Café & thé"), "{text:?}");
        assert!(text.contains("Line one\ntwo été"), "{text:?}");
        assert!(!text.contains("alert") && !text.contains("p{}") && !text.contains('x'));
        assert!(text.contains("\ta\tb"), "{text:?}");
    }

    #[test]
    fn broken_html_and_stray_ampersands() {
        assert_eq!(html_text("a & b <b>bold"), "a & b bold");
        assert_eq!(decode_entities("AT&T &unknown; &#65;"), "AT&T &unknown; A");
    }
}
