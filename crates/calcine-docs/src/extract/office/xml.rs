//! One XML part of an Office file, read into pages.

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use super::rules::XmlRules;
use crate::extract::entities::named_entity;
use crate::{Error, Result};

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
pub(super) fn read_pages(xml: &str, rules: &XmlRules) -> Result<Vec<String>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::office::rules::{DOCX, ODT};

    #[test]
    fn word_paragraphs_tables_and_page_breaks() {
        let xml = r#"<w:document xmlns:w="w"><w:body>
            <w:p><w:r><w:t>Hello</w:t><w:tab/><w:t xml:space="preserve"> world &amp; co</w:t></w:r></w:p>
            <w:tbl><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
            <w:p><w:r><w:br w:type="page"/><w:t>Second page</w:t></w:r></w:p>
            <w:p><w:r><w:instrText>PAGEREF _Toc1</w:instrText><w:t>end</w:t></w:r></w:p>
        </w:body></w:document>"#;
        let pages = read_pages(xml, &DOCX).unwrap();
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
        let pages = read_pages(xml, &ODT).unwrap();
        assert_eq!(pages.len(), 1);
        assert!(pages[0].contains("Title\n"));
        assert!(pages[0].contains("One two\tthree\n"), "{:?}", pages[0]);
        assert!(!pages[0].contains("hidden"));
    }
}
