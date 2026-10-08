//! `geniex version`.
//!
//! ```text
//! GenieX CLI Version:     v0.8.0
//! QAIRT Runtime Version:  2.45
//! LlamaCPP Runtime Hash:  9425611
//! ```

use calcine_core::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Versions {
    pub cli: String,
    pub qairt: Option<String>,
    pub llama_cpp: Option<String>,
}

pub fn parse(output: &str) -> Result<Versions> {
    let mut cli = None;
    let mut qairt = None;
    let mut llama_cpp = None;

    for line in output.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        let key = key.trim().to_ascii_lowercase();
        if key.starts_with("geniex cli") {
            cli = Some(value.to_owned());
        } else if key.starts_with("qairt") {
            qairt = Some(value.to_owned());
        } else if key.starts_with("llamacpp") || key.starts_with("llama.cpp") {
            llama_cpp = Some(value.to_owned());
        }
    }

    let cli =
        cli.ok_or_else(|| Error::Parse("no GenieX CLI version in `geniex version`".into()))?;
    Ok(Versions {
        cli,
        qairt,
        llama_cpp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::fixture;

    #[test]
    fn parses_real_output() {
        assert_eq!(
            parse(&fixture("version.txt")).unwrap(),
            Versions {
                cli: "v0.8.0".into(),
                qairt: Some("2.45".into()),
                llama_cpp: Some("9425611".into()),
            }
        );
    }

    #[test]
    fn runtimes_are_optional() {
        let versions = parse("GenieX CLI Version: v0.9.0\r\n").unwrap();
        assert_eq!(versions.cli, "v0.9.0");
        assert_eq!(versions.qairt, None);
    }

    #[test]
    fn missing_cli_version_is_an_error() {
        assert!(parse("something else entirely").is_err());
    }
}
