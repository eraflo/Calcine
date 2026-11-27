//! The readable text of an HTML page.

use super::entities::named_entity;

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
pub(super) fn readable_text(html: &str) -> String {
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
    fn html_without_tags_scripts_or_entities() {
        let html = "<html><head><title>x</title><style>p{}</style></head><body>\
            <h1>Café &amp; thé</h1><!-- a > b --><p>Line&nbsp;one<br>two &#233;t&#xE9;</p>\
            <script>alert('<p>')</script><table><tr><td>a</td><td>b</td></tr></table></body></html>";
        let text = readable_text(html);
        assert!(text.contains("Café & thé"), "{text:?}");
        assert!(text.contains("Line one\ntwo été"), "{text:?}");
        assert!(!text.contains("alert") && !text.contains("p{}") && !text.contains('x'));
        assert!(text.contains("\ta\tb"), "{text:?}");
    }

    #[test]
    fn broken_html_and_stray_ampersands() {
        assert_eq!(readable_text("a & b <b>bold"), "a & b bold");
        assert_eq!(decode_entities("AT&T &unknown; &#65;"), "AT&T &unknown; A");
    }
}
