//! `geniex model list [--all]` — a box-drawing table:
//!
//! ```text
//! Qualcomm AI Hub models for Snapdragon X Elite CRD (use --all to see every model):
//!
//! ┌──────────────────────┬──────┐
//! │ NAME                 │ TYPE │
//! ├──────────────────────┼──────┤
//! │ qualcomm/Qwen3-4B    │  llm │
//! └──────────────────────┴──────┘
//! ```
//!
//! `--all` adds a `CHIPSETS` column with comma-separated chipset slugs.

use calcine_core::model::HubModel;
use calcine_core::{Error, Result};

const CELL_SEPARATOR: char = '│';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubCatalog {
    /// Chipset the list was filtered for, when not `--all`.
    pub chipset: Option<String>,
    pub models: Vec<HubModel>,
}

pub fn parse(output: &str) -> Result<HubCatalog> {
    let chipset = output.lines().find_map(|line| {
        let rest = line.trim().strip_prefix("Qualcomm AI Hub models for ")?;
        let end = rest.find(" (").or_else(|| rest.find(':'))?;
        Some(rest[..end].trim().to_owned())
    });

    let mut saw_header = false;
    let mut models = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if !line.starts_with(CELL_SEPARATOR) {
            continue;
        }
        let cells: Vec<&str> = line
            .trim_matches(CELL_SEPARATOR)
            .split(CELL_SEPARATOR)
            .map(str::trim)
            .collect();
        if cells.first() == Some(&"NAME") {
            saw_header = true;
            continue;
        }
        let [name, model_type, rest @ ..] = cells.as_slice() else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        models.push(HubModel {
            name: (*name).to_owned(),
            model_type: super::model_type(model_type),
            chipsets: rest
                .first()
                .map(|chips| {
                    chips
                        .split(',')
                        .map(str::trim)
                        .filter(|c| !c.is_empty())
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
        });
    }

    if !saw_header {
        return Err(Error::Parse("no model table in `geniex model list`".into()));
    }
    Ok(HubCatalog { chipset, models })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::fixture;
    use calcine_core::model::ModelType;

    #[test]
    fn parses_device_filtered_catalog() {
        let catalog = parse(&fixture("model_list.txt")).unwrap();
        assert_eq!(catalog.chipset.as_deref(), Some("Snapdragon X Elite CRD"));
        assert!(
            catalog.models.len() >= 20,
            "got {} models",
            catalog.models.len()
        );
        let qwen = catalog
            .models
            .iter()
            .find(|m| m.name == "qualcomm/Qwen3-4B")
            .unwrap();
        assert_eq!(qwen.model_type, ModelType::Llm);
        assert_eq!(qwen.chipsets, Vec::<String>::new());
        assert!(
            catalog
                .models
                .iter()
                .any(|m| m.model_type == ModelType::Vlm)
        );
    }

    #[test]
    fn parses_full_catalog_with_chipsets() {
        let catalog = parse(&fixture("model_list_all.txt")).unwrap();
        assert_eq!(catalog.chipset, None);
        let gemma = catalog
            .models
            .iter()
            .find(|m| m.name == "qualcomm/Gemma-4-E4B-it")
            .unwrap();
        assert_eq!(gemma.model_type, ModelType::Vlm);
        assert!(gemma.chipsets.iter().any(|c| c == "x-elite"));
        assert!(gemma.chipsets.iter().all(|c| !c.contains(' ')));
    }

    #[test]
    fn output_without_table_is_an_error() {
        assert!(parse("Error: failed to reach Qualcomm AI Hub").is_err());
    }
}
