//! Named character references shared by XML and HTML.

/// The character of `&name;`, empty when unknown.
pub(super) fn named_entity(name: &str) -> &'static str {
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
