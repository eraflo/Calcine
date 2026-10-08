//! Precisions of a GGUF repository, named and chosen the way GenieX does.
//!
//! The rules follow GenieX's model manager (`manifest_builder.rs`,
//! BSD-3-Clause, Qualcomm) so the names shown here are the ones
//! `geniex pull repo:<precision>` accepts and the recommended one is what a
//! bare `geniex pull repo` downloads.

use std::collections::BTreeMap;

use calcine_core::models::RemotePrecision;

/// Preferred precisions, best first. Others rank after, alphabetically.
const PRIORITY: &[&str] = &["Q4_0", "Q4_K_M", "Q8_0"];

/// A file in the repository with its size in bytes.
#[derive(Debug, Clone, Copy)]
pub struct RepoFile<'a> {
    pub path: &'a str,
    pub size: u64,
}

/// What the files of a repository add up to.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Precisions {
    /// Recommended first, then by size.
    pub precisions: Vec<RemotePrecision>,
    /// The repository ships a vision projector: GenieX treats it as a VLM.
    pub has_projector: bool,
}

/// Group weight files by precision. Sharded weights (`-00001-of-00002`) add
/// up; the vision projector GenieX would pick is added to every precision.
pub fn precisions(files: &[RepoFile<'_>]) -> Precisions {
    let mut weights: BTreeMap<String, u64> = BTreeMap::new();
    let mut projectors = Vec::new();
    for file in files {
        let lower = file.path.to_ascii_lowercase();
        if std::path::Path::new(&lower)
            .extension()
            .is_none_or(|extension| extension != "gguf")
        {
            continue;
        }
        if is_projector(&lower) {
            projectors.push(*file);
        } else if !lower.contains("mtp") {
            let precision = extract(file.path).unwrap_or_else(|| "DEFAULT".to_owned());
            *weights.entry(precision).or_default() += file.size;
        }
    }

    // Like GenieX: prefer an F16 projector, then the largest.
    let projector = projectors
        .iter()
        .max_by_key(|file| {
            let f16 = matches!(extract(file.path).as_deref(), Some("F16" | "FP16"));
            (f16, file.size)
        })
        .map_or(0, |file| file.size);

    let recommended = weights.keys().min_by_key(|name| rank(name)).cloned();
    let mut precisions: Vec<RemotePrecision> = weights
        .into_iter()
        .map(|(name, size)| RemotePrecision {
            recommended: recommended.as_ref() == Some(&name),
            name,
            size_bytes: size + projector,
        })
        .collect();
    precisions.sort_by_key(|precision| (!precision.recommended, precision.size_bytes));

    Precisions {
        precisions,
        has_projector: !projectors.is_empty(),
    }
}

fn rank(name: &str) -> (usize, &str) {
    let position = PRIORITY
        .iter()
        .position(|preferred| *preferred == name)
        .unwrap_or(PRIORITY.len());
    (position, name)
}

/// `mmproj`, `mmproj-*`, `*-mmproj`, `*_mmproj_*`… (lowercase input).
fn is_projector(lower: &str) -> bool {
    let name = lower.rsplit(['/', '\\']).next().unwrap_or(lower);
    let stem = name.strip_suffix(".gguf").unwrap_or(name);
    stem == "mmproj"
        || ["mmproj-", "mmproj.", "mmproj_"]
            .iter()
            .any(|prefix| stem.starts_with(prefix))
        || stem.ends_with("-mmproj")
        || stem.ends_with("_mmproj")
        || stem.contains("-mmproj-")
        || stem.contains("_mmproj_")
}

/// The precision tag in a file name: `Q4_K_M`, `IQ4_XS`, `TQ1_0`, `MXFP4`,
/// `F16`, `BF16`… upper-cased. Tags must start a token (after `-`, `_`, `.`
/// or a path separator). With several, a preferred one wins, else the first.
pub fn extract(name: &str) -> Option<String> {
    let upper = name.to_ascii_uppercase();
    let bytes = upper.as_bytes();
    let mut found: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !starts_token(bytes, i) {
            i += 1;
            continue;
        }
        if let Some(end) = [b"MXFP" as &[u8], b"BF", b"FP", b"F", b"I"]
            .iter()
            .find_map(|prefix| scalar_tag_end(bytes, i, prefix))
        {
            found.push(&upper[i..end]);
            i = end;
            continue;
        }
        // `Q4_0`, with an optional `I` (i-quants) or `T` (ternary) prefix.
        let mut q = i;
        if matches!(bytes[q], b'I' | b'T') && bytes.get(q + 1) == Some(&b'Q') {
            q += 1;
        }
        if bytes[q] != b'Q' || !bytes.get(q + 1).is_some_and(u8::is_ascii_digit) {
            i += 1;
            continue;
        }
        let mut end = q + 2;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        end = short_suffixes_end(bytes, end);
        found.push(&upper[i..end]);
        i = end;
    }
    PRIORITY
        .iter()
        .find(|preferred| found.contains(preferred))
        .copied()
        .or_else(|| found.first().copied())
        .map(str::to_owned)
}

fn starts_token(bytes: &[u8], i: usize) -> bool {
    i == 0 || matches!(bytes[i - 1], b'-' | b'_' | b'.' | b'/' | b'\\')
}

/// `prefix` then digits then short suffixes, e.g. `F16`, `MXFP4_MOE`.
fn scalar_tag_end(bytes: &[u8], start: usize, prefix: &[u8]) -> Option<usize> {
    if !bytes[start..].starts_with(prefix) {
        return None;
    }
    let mut end = start + prefix.len();
    if !bytes.get(end).is_some_and(u8::is_ascii_digit) {
        return None;
    }
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    Some(short_suffixes_end(bytes, end))
}

/// Trailing `_K`, `_M`, `_XL`… segments of one to three letters or digits.
/// Longer words (`_FINETUNE`) aren't part of the tag.
fn short_suffixes_end(bytes: &[u8], mut end: usize) -> usize {
    while bytes.get(end) == Some(&b'_') {
        let start = end + 1;
        let mut segment_end = start;
        while bytes
            .get(segment_end)
            .is_some_and(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        {
            segment_end += 1;
        }
        if !(1..=3).contains(&(segment_end - start)) {
            break;
        }
        end = segment_end;
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_common_tags() {
        for (name, tag) in [
            ("model-Q4_K_M.gguf", Some("Q4_K_M")),
            ("model-q8_0.gguf", Some("Q8_0")),
            ("Qwen3-0.6B-UD-IQ1_M.gguf", Some("IQ1_M")),
            ("model-TQ1_0.gguf", Some("TQ1_0")),
            ("gpt-oss-20b-mxfp4-00001-of-00002.gguf", Some("MXFP4")),
            ("model-BF16.gguf", Some("BF16")),
            ("SmolVLM-256M-Instruct-f16.gguf", Some("F16")),
            ("dir/Q4_0.gguf", Some("Q4_0")),
            ("model-Q4_K_M_finetune.gguf", Some("Q4_K_M")),
            ("model.gguf", None),
            ("aQ4_0.gguf", None),
        ] {
            assert_eq!(extract(name).as_deref(), tag, "{name}");
        }
    }

    #[test]
    fn preferred_tag_wins_in_one_name() {
        assert_eq!(extract("model-F16-Q4_0.gguf").as_deref(), Some("Q4_0"));
    }

    #[test]
    fn shards_add_up_and_projector_counts_once_per_precision() {
        let files = [
            RepoFile {
                path: "BF16/m-BF16-00001-of-00002.gguf",
                size: 10,
            },
            RepoFile {
                path: "BF16/m-BF16-00002-of-00002.gguf",
                size: 5,
            },
            RepoFile {
                path: "m-Q8_0.gguf",
                size: 8,
            },
            RepoFile {
                path: "m-Q4_0.gguf",
                size: 4,
            },
            RepoFile {
                path: "mmproj-m-BF16.gguf",
                size: 3,
            },
            RepoFile {
                path: "mmproj-m-F16.gguf",
                size: 2,
            },
            RepoFile {
                path: "MTP/mtp-m-Q4_0.gguf",
                size: 1,
            },
            RepoFile {
                path: "README.md",
                size: 1,
            },
        ];
        let result = precisions(&files);
        assert!(result.has_projector);
        let summary: Vec<(&str, u64, bool)> = result
            .precisions
            .iter()
            .map(|p| (p.name.as_str(), p.size_bytes, p.recommended))
            .collect();
        // F16 projector (2) preferred over the larger BF16 one.
        assert_eq!(
            summary,
            [("Q4_0", 6, true), ("Q8_0", 10, false), ("BF16", 17, false)]
        );
    }

    #[test]
    fn falls_back_to_alphabetical_when_nothing_preferred() {
        let files = [
            RepoFile {
                path: "m-Q6_K.gguf",
                size: 6,
            },
            RepoFile {
                path: "m-Q5_K_M.gguf",
                size: 5,
            },
        ];
        let result = precisions(&files);
        assert_eq!(result.precisions[0].name, "Q5_K_M");
        assert!(result.precisions[0].recommended);
        assert!(!result.has_projector);
    }
}
