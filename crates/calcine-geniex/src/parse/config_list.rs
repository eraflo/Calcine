//! `geniex config list` — one `key: value` per line, unset keys blank.

use std::collections::BTreeMap;

pub fn parse(output: &str) -> BTreeMap<String, String> {
    output
        .lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .filter(|(key, _)| !key.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::fixture;

    #[test]
    fn parses_real_output() {
        let config = parse(&fixture("config_list.txt"));
        assert_eq!(
            config.get("chipset").map(String::as_str),
            Some("Snapdragon X Elite CRD")
        );
    }

    #[test]
    fn keeps_unset_keys_as_empty_values() {
        let config = parse("chipset:\n");
        assert_eq!(config.get("chipset").map(String::as_str), Some(""));
    }
}
