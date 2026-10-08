//! `geniex pull` progress, rendered on stderr by schollz/progressbar:
//!
//! ```text
//! downloading  98% |███████████████████ | (749/760 MB, 17 MB/s) [37s:0s]\x1b[0K\r
//! ```
//!
//! Frames are separated by `\r`, sizes use decimal units (`kB`, `MB`, `GB`) and
//! the downloaded amount omits its unit when it matches the total's.

use calcine_core::jobs::JobProgress;

/// Splits a byte stream into frames (on `\r` or `\n`).
///
/// Works on bytes so a multi-byte character (the bar's `█`) split across two
/// reads is decoded intact: `\r` and `\n` never occur inside UTF-8 sequences.
#[derive(Debug, Default)]
pub struct FrameSplitter {
    pending: Vec<u8>,
}

impl FrameSplitter {
    /// Feed a chunk of output; returns the frames it completed.
    pub fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        self.pending.extend_from_slice(chunk);
        let mut frames = Vec::new();
        while let Some(end) = self.pending.iter().position(|b| *b == b'\r' || *b == b'\n') {
            let frame: Vec<u8> = self.pending.drain(..=end).collect();
            let frame = String::from_utf8_lossy(&frame[..frame.len() - 1]);
            if !frame.trim().is_empty() {
                frames.push(frame.into_owned());
            }
        }
        frames
    }

    /// Whatever is left once the stream ends.
    pub fn finish(self) -> Option<String> {
        let rest = String::from_utf8_lossy(&self.pending);
        let rest = rest.trim();
        (!rest.is_empty()).then(|| rest.to_owned())
    }
}

/// Parse one frame. Returns `None` for anything that isn't a progress line.
pub fn parse_frame(frame: &str) -> Option<JobProgress> {
    let frame = strip_ansi(frame);
    let open = frame.rfind('(')?;
    let close = open + frame[open..].find(')')?;
    let mut parts = frame[open + 1..close].split(',').map(str::trim);

    let (done, total) = parts.next()?.split_once('/')?;
    let total = parse_size(total.trim(), None)?;
    let total_unit = total.unit;
    let done = parse_size(done.trim(), Some(total_unit))?;

    let bytes_per_second = parts
        .next()
        .and_then(|speed| speed.strip_suffix("/s"))
        .and_then(|speed| parse_size(speed.trim(), None))
        .map(|size| size.bytes);

    Some(JobProgress {
        done_bytes: done.bytes.min(total.bytes),
        total_bytes: (total.bytes > 0).then_some(total.bytes),
        bytes_per_second,
        phase: None,
    })
}

#[derive(Debug, Clone, Copy)]
struct Size {
    bytes: u64,
    unit: u64,
}

/// `760 MB`, `1.7 kB`, `0 B` — or a bare number using `default_unit`.
fn parse_size(text: &str, default_unit: Option<u64>) -> Option<Size> {
    let split = text
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(text.len());
    let (number, unit) = text.split_at(split);
    let value: f64 = number.trim().parse().ok()?;
    let unit = match unit.trim() {
        "" => default_unit?,
        unit => unit_multiplier(unit)?,
    };
    // Display rounding means values are approximate anyway.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let bytes = (value * unit as f64).round() as u64;
    Some(Size { bytes, unit })
}

fn unit_multiplier(unit: &str) -> Option<u64> {
    Some(match unit {
        "B" => 1,
        "kB" | "KB" => 1_000,
        "MB" => 1_000_000,
        "GB" => 1_000_000_000,
        "TB" => 1_000_000_000_000,
        "KiB" => 1 << 10,
        "MiB" => 1 << 20,
        "GiB" => 1 << 30,
        "TiB" => 1 << 40,
        _ => return None,
    })
}

/// Remove ANSI CSI sequences (`\x1b[0K`, colors).
fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.next() == Some('[') {
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::fixture;

    fn frames_of(output: &str) -> Vec<String> {
        let mut splitter = FrameSplitter::default();
        let mut frames = splitter.push(output.as_bytes());
        frames.extend(splitter.finish());
        frames
    }

    #[test]
    fn follows_a_real_download_to_completion() {
        let progress: Vec<JobProgress> = frames_of(&fixture("pull_stderr.txt"))
            .iter()
            .filter_map(|f| parse_frame(f))
            .collect();
        assert!(
            progress.len() > 300,
            "only {} frames parsed",
            progress.len()
        );

        let first = progress.first().unwrap();
        assert_eq!(first.done_bytes, 0);
        assert_eq!(first.total_bytes, Some(760_000_000));

        let last = progress.last().unwrap();
        assert_eq!(last.done_bytes, 760_000_000);
        assert_eq!(last.bytes_per_second, Some(20_000_000));

        assert!(
            progress
                .windows(2)
                .all(|pair| pair[1].done_bytes >= pair[0].done_bytes),
            "progress went backwards"
        );
    }

    #[test]
    fn bare_amounts_use_the_total_unit() {
        let frame = "downloading  98% |███ | (749/760 MB, 17 MB/s) [37s:0s]\x1b[0K";
        let progress = parse_frame(frame).unwrap();
        assert_eq!(progress.done_bytes, 749_000_000);
        assert_eq!(progress.bytes_per_second, Some(17_000_000));
    }

    #[test]
    fn mixed_units_and_gigabytes() {
        let progress = parse_frame("downloading 0% | | (1.7 kB/3.2 GB, 8.4 kB/s)").unwrap();
        assert_eq!(progress.done_bytes, 1_700);
        assert_eq!(progress.total_bytes, Some(3_200_000_000));
        assert_eq!(progress.bytes_per_second, Some(8_400));
    }

    #[test]
    fn first_frame_has_no_speed() {
        let progress = parse_frame("downloading   0% |  | ( 0 B/760 MB) [0s:0s]").unwrap();
        assert_eq!(progress.done_bytes, 0);
        assert_eq!(progress.bytes_per_second, None);
    }

    #[test]
    fn other_lines_are_ignored() {
        assert_eq!(
            parse_frame("   Press Ctrl+C to cancel — progress is saved, not discarded."),
            None
        );
        assert_eq!(
            parse_frame("fetching available precisions from: qualcomm/Qwen3-0.6B"),
            None
        );
    }

    #[test]
    fn splitter_handles_frames_across_chunks() {
        let mut splitter = FrameSplitter::default();
        let bar = "█".as_bytes();
        assert_eq!(splitter.push(b"downloading 1% |"), Vec::<String>::new());
        assert_eq!(splitter.push(&bar[..1]), Vec::<String>::new());
        let mut rest = bar[1..].to_vec();
        rest.extend_from_slice(b"| (1/10 MB)\rnext");
        assert_eq!(splitter.push(&rest), vec!["downloading 1% |█| (1/10 MB)"]);
        assert_eq!(splitter.finish().as_deref(), Some("next"));
    }
}
