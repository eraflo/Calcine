use std::path::PathBuf;

use super::*;
use crate::kind::DocumentKind;

fn temp_dir(name: &str) -> PathBuf {
    let mut random = [0u8; 8];
    getrandom::fill(&mut random).unwrap();
    std::env::temp_dir().join(format!("calcine-docs-{name}-{}", hex::encode(random)))
}

fn sample() -> Extracted {
    Extracted {
        kind: DocumentKind::Pdf,
        pages: vec!["First page.".into(), "Second page.".into()],
        paged: true,
    }
}

#[test]
fn documents_round_trip() {
    let dir = temp_dir("round-trip");
    let store = DocumentStore::new(&dir);
    let info = store.add("report.pdf", 1234, &sample()).unwrap();
    assert_eq!(info.pages, Some(2));
    assert_eq!(info.passages, 2);
    assert_eq!(info.chars, 23);
    let stored = store.get(&info.id).unwrap();
    assert_eq!(stored.info, info);
    assert_eq!(stored.passages[1].page, Some(2));
    store.remove(std::slice::from_ref(&info.id));
    assert!(matches!(store.get(&info.id), Err(Error::NotFound)));
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn ids_cant_leave_the_folder() {
    let store = DocumentStore::new(temp_dir("ids"));
    assert!(matches!(store.get("../secrets"), Err(Error::NotFound)));
    assert!(matches!(store.get(&"a".repeat(31)), Err(Error::NotFound)));
}

#[test]
fn empty_documents_are_refused() {
    let store = DocumentStore::new(temp_dir("empty"));
    let empty = Extracted {
        kind: DocumentKind::Pdf,
        pages: vec!["  \n ".into()],
        paged: true,
    };
    assert!(matches!(
        store.add("scan.pdf", 1, &empty),
        Err(Error::NoText)
    ));
}

#[test]
fn retain_deletes_the_others() {
    let dir = temp_dir("retain");
    let store = DocumentStore::new(&dir);
    let kept = store.add("a.pdf", 1, &sample()).unwrap();
    let dropped = store.add("b.pdf", 1, &sample()).unwrap();
    std::fs::write(dir.join("stray.txt"), "not ours").unwrap();
    assert_eq!(store.retain(std::slice::from_ref(&kept.id)), 1);
    assert!(store.get(&kept.id).is_ok());
    assert!(matches!(store.get(&dropped.id), Err(Error::NotFound)));
    assert!(dir.join("stray.txt").exists(), "other files are left alone");
    std::fs::remove_dir_all(dir).ok();
}
