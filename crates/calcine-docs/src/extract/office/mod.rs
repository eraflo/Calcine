//! Word, PowerPoint and OpenDocument files: zip archives of XML.
//!
//! - `rules`: how each vocabulary marks paragraphs, tables and pages
//! - `xml`: reading one XML part into pages

mod rules;
mod xml;

use std::io::Read;

use rules::{DOCX, ODP, ODT, PPTX, XmlRules};
use xml::read_pages;

use crate::{Error, Result};

/// Word: pages where Word last broke them, when it saved the file.
pub(super) fn docx(bytes: &[u8]) -> Result<(Vec<String>, bool)> {
    office(bytes, "word/document.xml", &DOCX)
}

/// OpenDocument text: pages where the editor broke them.
pub(super) fn odt(bytes: &[u8]) -> Result<(Vec<String>, bool)> {
    office(bytes, "content.xml", &ODT)
}

/// OpenDocument presentation: one page per slide.
pub(super) fn odp(bytes: &[u8]) -> Result<(Vec<String>, bool)> {
    office(bytes, "content.xml", &ODP)
}

/// Pages of an Office document stored as `entry` in the archive, and
/// whether page breaks were found.
fn office(bytes: &[u8], entry: &str, rules: &XmlRules) -> Result<(Vec<String>, bool)> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|err| Error::Read(format!("not a valid Office file ({err})")))?;
    let xml = read_entry(&mut archive, entry)?;
    let pages = read_pages(&xml, rules)?;
    let numbered = pages.len() > 1;
    Ok((pages, numbered))
}

/// One page per slide, in order (`slide2.xml` before `slide10.xml`).
pub(super) fn pptx(bytes: &[u8]) -> Result<Vec<String>> {
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
            Ok(read_pages(&xml, &PPTX)?.join("\n"))
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
