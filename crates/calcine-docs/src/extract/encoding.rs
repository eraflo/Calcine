//! Text files, whatever their encoding.

/// Text in any common encoding: UTF-8 (with or without a BOM), UTF-16 with
/// a BOM, or Windows-1252 for older files.
pub(super) fn decode(bytes: &[u8]) -> String {
    if let Some((encoding, _)) = encoding_rs::Encoding::for_bom(bytes) {
        return encoding.decode_with_bom_removal(bytes).0.into_owned();
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_owned(),
        Err(_) => encoding_rs::WINDOWS_1252.decode(bytes).0.into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_in_any_common_encoding() {
        assert_eq!(decode("héllo".as_bytes()), "héllo");
        assert_eq!(decode(b"\xEF\xBB\xBFbom"), "bom");
        assert_eq!(decode(b"\xFF\xFEh\x00i\x00"), "hi");
        // Windows-1252: é is 0xE9.
        assert_eq!(decode(b"caf\xE9"), "café");
    }
}
