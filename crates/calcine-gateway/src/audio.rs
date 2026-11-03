//! Audio for transcription: read WAV files, find where someone speaks, and
//! cut long recordings into pieces a model takes in one go.
//!
//! Models that listen (Gemma 4, Gemma 3n) take 30 s at most, and invent
//! sentences when given silence: only speech is sent, in pieces of
//! [`MAX_PIECE_SECONDS`] cut where it's quietest.

/// What models expect: 16 kHz mono.
pub const SAMPLE_RATE: u32 = 16_000;

/// Longest piece sent at once, under the models' 30 s limit.
pub const MAX_PIECE_SECONDS: usize = 28;

/// Loudness is measured over 20 ms frames.
const FRAME: usize = SAMPLE_RATE as usize / 50;

/// Less speech than this (in frames, 0.2 s) is noise, not words.
const MIN_SPEECH_FRAMES: usize = 10;

/// Silence kept around speech so first and last words aren't clipped (0.3 s).
const MARGIN_FRAMES: usize = 15;

/// Quieter than this (RMS of samples in [-1, 1]) is never speech.
const SILENCE_FLOOR: f32 = 0.008;

/// Louder than this is always speech, even when the whole recording is loud.
const SPEECH_CEILING: f32 = 0.03;

/// Mono samples in [-1, 1] at [`SAMPLE_RATE`].
#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    pub samples: Vec<f32>,
}

#[cfg(test)]
impl Audio {
    pub fn seconds(&self) -> f32 {
        #[allow(clippy::cast_precision_loss)]
        let seconds = self.samples.len() as f32 / SAMPLE_RATE as f32;
        seconds
    }
}

/// Read a WAV file: 16-bit PCM or 32-bit float, any channel count and sample
/// rate. Channels are mixed down and the rate converted to [`SAMPLE_RATE`].
pub fn decode_wav(bytes: &[u8]) -> Result<Audio, String> {
    const NOT_WAV: &str = "the audio isn't a WAV file: send WAV (PCM) audio";
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(NOT_WAV.into());
    }
    let mut format = None;
    let mut data = None;
    let mut at = 12;
    while at + 8 <= bytes.len() {
        let id = &bytes[at..at + 4];
        let size = u32::from_le_bytes([bytes[at + 4], bytes[at + 5], bytes[at + 6], bytes[at + 7]])
            as usize;
        let body = &bytes[at + 8..(at + 8 + size).min(bytes.len())];
        match id {
            b"fmt " if body.len() >= 16 => format = Some(body),
            b"data" => data = Some(body),
            _ => {}
        }
        // Chunks are padded to an even size.
        at += 8 + size + (size & 1);
    }
    let (Some(format), Some(data)) = (format, data) else {
        return Err(NOT_WAV.into());
    };
    let u16_at = |i: usize| u16::from_le_bytes([format[i], format[i + 1]]);
    let mut tag = u16_at(0);
    let channels = usize::from(u16_at(2).max(1));
    let rate = u32::from_le_bytes([format[4], format[5], format[6], format[7]]);
    let bits = u16_at(14);
    // WAVE_FORMAT_EXTENSIBLE: the real format is the subformat's first two bytes.
    if tag == 0xFFFE && format.len() >= 26 {
        tag = u16_at(24);
    }
    let samples: Vec<f32> = match (tag, bits) {
        (1, 16) => data
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| f32::from(i16::from_le_bytes(*pair)) / 32_768.0)
            .collect(),
        (3, 32) => data
            .as_chunks::<4>()
            .0
            .iter()
            .map(|four| f32::from_le_bytes(*four))
            .collect(),
        _ => {
            return Err(format!(
                "this WAV encoding isn't supported ({bits}-bit, format {tag}): send 16-bit PCM"
            ));
        }
    };
    if rate == 0 {
        return Err(NOT_WAV.into());
    }
    #[allow(clippy::cast_precision_loss)]
    let mono: Vec<f32> = samples
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
        .collect();
    Ok(Audio {
        samples: resample(&mono, rate),
    })
}

/// Linear interpolation to [`SAMPLE_RATE`]: enough for speech.
fn resample(samples: &[f32], rate: u32) -> Vec<f32> {
    if rate == SAMPLE_RATE || samples.is_empty() {
        return samples.to_vec();
    }
    let step = f64::from(rate) / f64::from(SAMPLE_RATE);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let length = (samples.len() as f64 / step) as usize;
    (0..length)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let position = i as f64 * step;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let left = position as usize;
            let right = (left + 1).min(samples.len() - 1);
            #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
            let weight = (position - left as f64) as f32;
            samples[left] * (1.0 - weight) + samples[right] * weight
        })
        .collect()
}

/// 16-bit PCM WAV at [`SAMPLE_RATE`], mono.
pub fn encode_wav(samples: &[f32]) -> Vec<u8> {
    let data_size = u32::try_from(samples.len() * 2).unwrap_or(u32::MAX);
    let mut bytes = Vec::with_capacity(44 + samples.len() * 2);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1_u16.to_le_bytes()); // mono
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        #[allow(clippy::cast_possible_truncation)]
        let value = (sample.clamp(-1.0, 1.0) * 32_767.0) as i16;
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

/// Loudness of each 20 ms frame.
fn loudness(samples: &[f32]) -> Vec<f32> {
    samples
        .chunks(FRAME)
        .map(|frame| {
            #[allow(clippy::cast_precision_loss)]
            let mean = frame.iter().map(|s| s * s).sum::<f32>() / frame.len() as f32;
            mean.sqrt()
        })
        .collect()
}

/// The loudness above which a frame holds speech: well above the background
/// noise (the quietest tenth of the recording), within fixed bounds.
fn threshold(levels: &[f32]) -> f32 {
    let mut sorted = levels.to_vec();
    sorted.sort_by(f32::total_cmp);
    let background = sorted.get(sorted.len() / 10).copied().unwrap_or(0.0);
    (background * 3.0).clamp(SILENCE_FLOOR, SPEECH_CEILING)
}

/// How many frames are louder than `threshold`.
fn speech_count(levels: &[f32], threshold: f32) -> usize {
    levels.iter().filter(|level| **level > threshold).count()
}

/// The part with speech, with a little silence around it, or `None` when
/// nobody speaks.
pub fn speech(audio: &Audio) -> Option<&[f32]> {
    let levels = loudness(&audio.samples);
    let threshold = threshold(&levels);
    let speaking: Vec<bool> = levels.iter().map(|level| *level > threshold).collect();
    if speech_count(&levels, threshold) < MIN_SPEECH_FRAMES {
        return None;
    }
    let first = speaking.iter().position(|frame| *frame)?;
    let last = speaking.iter().rposition(|frame| *frame)?;
    let start = first.saturating_sub(MARGIN_FRAMES) * FRAME;
    let end = ((last + 1 + MARGIN_FRAMES) * FRAME).min(audio.samples.len());
    Some(&audio.samples[start..end])
}

/// Cut `samples` into pieces of at most [`MAX_PIECE_SECONDS`], so words
/// aren't split: each piece ends in the middle of the longest pause of its
/// last ten seconds, or at its quietest frame when nobody pauses.
/// Pieces without speech are left out.
pub fn pieces(samples: &[f32]) -> Vec<&[f32]> {
    let max = MAX_PIECE_SECONDS * SAMPLE_RATE as usize;
    // One threshold for the whole recording: a piece of nonstop speech
    // mustn't pass for its own background.
    let threshold = threshold(&loudness(samples));
    let search = 10 * SAMPLE_RATE as usize;
    let mut pieces = Vec::new();
    let mut start = 0;
    while start < samples.len() {
        let mut end = (start + max).min(samples.len());
        if end < samples.len() {
            let window_start = end - search;
            let levels = loudness(&samples[window_start..end]);
            if let Some(cut) = cut_frame(&levels, threshold) {
                end = window_start + cut * FRAME + FRAME / 2;
            }
        }
        let piece = &samples[start..end];
        if speech_count(&loudness(piece), threshold) >= MIN_SPEECH_FRAMES {
            pieces.push(piece);
        }
        start = end;
    }
    pieces
}

/// Where to cut among `levels`: the middle of the longest run of quiet
/// frames, or the quietest frame when there's no pause of 0.1 s or more.
fn cut_frame(levels: &[f32], threshold: f32) -> Option<usize> {
    let mut longest = (0, 0); // (start, length)
    let mut run_start = None;
    for (index, level) in levels.iter().chain([&f32::INFINITY]).enumerate() {
        match (*level <= threshold, run_start) {
            (true, None) => run_start = Some(index),
            (false, Some(begin)) => {
                if index - begin > longest.1 {
                    longest = (begin, index - begin);
                }
                run_start = None;
            }
            _ => {}
        }
    }
    if longest.1 >= 5 {
        return Some(longest.0 + longest.1 / 2);
    }
    levels
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.total_cmp(b.1))
        .map(|(index, _)| index)
}

#[cfg(test)]
#[allow(clippy::cast_precision_loss)]
mod tests {
    use super::*;

    /// `seconds` of a 220 Hz tone at `amplitude` (speech stand-in), at `rate`.
    fn tone(seconds: f32, amplitude: f32, rate: u32) -> Vec<f32> {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let count = (seconds * rate as f32) as usize;
        #[allow(clippy::cast_precision_loss)]
        (0..count)
            .map(|i| amplitude * (i as f32 * 220.0 * std::f32::consts::TAU / rate as f32).sin())
            .collect()
    }

    fn silence(seconds: f32) -> Vec<f32> {
        tone(seconds, 0.0, SAMPLE_RATE)
    }

    #[test]
    fn reads_what_it_writes() {
        let samples = tone(0.5, 0.5, SAMPLE_RATE);
        let audio = decode_wav(&encode_wav(&samples)).unwrap();
        assert_eq!(audio.samples.len(), samples.len());
        assert!((audio.samples[100] - samples[100]).abs() < 1e-3);
        assert!((audio.seconds() - 0.5).abs() < 1e-3);
    }

    #[test]
    fn mixes_down_and_converts_the_rate() {
        // 1 s of stereo 16-bit PCM at 48 kHz.
        let left = tone(1.0, 0.4, 48_000);
        let mut wav = encode_wav(&[]);
        let frames: Vec<u8> = left
            .iter()
            .flat_map(|sample| {
                #[allow(clippy::cast_possible_truncation)]
                let value = (sample * 32_767.0) as i16;
                [value.to_le_bytes(), value.to_le_bytes()].concat()
            })
            .collect();
        // Rewrite the header: 2 channels, 48 kHz.
        wav[22..24].copy_from_slice(&2_u16.to_le_bytes());
        wav[24..28].copy_from_slice(&48_000_u32.to_le_bytes());
        wav[32..34].copy_from_slice(&4_u16.to_le_bytes());
        wav[40..44].copy_from_slice(&u32::try_from(frames.len()).unwrap().to_le_bytes());
        wav.extend_from_slice(&frames);
        let audio = decode_wav(&wav).unwrap();
        assert!((audio.seconds() - 1.0).abs() < 0.01);
        assert!(audio.samples.iter().any(|sample| sample.abs() > 0.3));
    }

    #[test]
    fn refuses_what_isnt_wav() {
        assert!(decode_wav(b"ID3\x04 not a wav file").is_err());
        let mut eight_bit = encode_wav(&silence(0.1));
        eight_bit[34..36].copy_from_slice(&8_u16.to_le_bytes());
        assert!(decode_wav(&eight_bit).unwrap_err().contains("16-bit PCM"));
    }

    #[test]
    fn finds_speech_and_ignores_silence() {
        let quiet = Audio {
            samples: silence(5.0),
        };
        assert_eq!(speech(&quiet), None);

        // A faint hum is background, not speech.
        let hum = Audio {
            samples: tone(5.0, 0.005, SAMPLE_RATE),
        };
        assert_eq!(speech(&hum), None);

        let samples = [silence(2.0), tone(1.0, 0.3, SAMPLE_RATE), silence(2.0)].concat();
        let audio = Audio { samples };
        let spoken = speech(&audio).unwrap();
        // One second of speech and 0.3 s on each side.
        assert!((spoken.len() as f32 / SAMPLE_RATE as f32 - 1.6).abs() < 0.05);
    }

    #[test]
    fn cuts_long_recordings_where_it_is_quiet() {
        // 25 s of speech, a pause, 25 s more: one cut, in the pause.
        let samples = [
            tone(25.0, 0.3, SAMPLE_RATE),
            silence(1.0),
            tone(25.0, 0.3, SAMPLE_RATE),
        ]
        .concat();
        let pieces = pieces(&samples);
        assert_eq!(pieces.len(), 2);
        let first = pieces[0].len() as f32 / SAMPLE_RATE as f32;
        assert!((25.0..26.0).contains(&first), "cut at {first} s");
        assert!(
            pieces
                .iter()
                .all(|piece| piece.len() <= MAX_PIECE_SECONDS * SAMPLE_RATE as usize)
        );

        // A long pause wins over a short breath nearer the limit:
        // 15 s, a 4 s pause, 9.8 s, a 0.2 s breath, 20 s.
        let samples = [
            tone(15.0, 0.3, SAMPLE_RATE),
            silence(4.0),
            tone(9.8, 0.3, SAMPLE_RATE),
            silence(0.2),
            tone(20.0, 0.3, SAMPLE_RATE),
        ]
        .concat();
        let first = super::pieces(&samples)[0].len() as f32 / SAMPLE_RATE as f32;
        assert!((15.0..19.0).contains(&first), "cut at {first} s");

        // Short recordings stay whole; silent pieces are dropped.
        assert_eq!(pieces_of(&tone(10.0, 0.3, SAMPLE_RATE)), 1);
        assert_eq!(
            pieces_of(&[tone(10.0, 0.3, SAMPLE_RATE), silence(40.0)].concat()),
            1
        );
    }

    fn pieces_of(samples: &[f32]) -> usize {
        pieces(samples).len()
    }
}
