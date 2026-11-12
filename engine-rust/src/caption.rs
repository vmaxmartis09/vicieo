use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleWord {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleSegment {
    pub id: u32,
    pub start: f64,
    pub end: f64,
    pub text: String,
    #[serde(default)]
    pub words: Vec<SubtitleWord>,
}

pub fn format_timestamp_srt(seconds: f64) -> String {
    let mut millis = (seconds.max(0.0) * 1000.0).round() as u64;
    let hours = millis / 3_600_000;
    millis %= 3_600_000;
    let mins = millis / 60_000;
    millis %= 60_000;
    let secs = millis / 1000;
    millis %= 1000;
    format!("{:02}:{:02}:{:02},{:03}", hours, mins, secs, millis)
}

pub fn format_timestamp_vtt(seconds: f64) -> String {
    let mut millis = (seconds.max(0.0) * 1000.0).round() as u64;
    let hours = millis / 3_600_000;
    millis %= 3_600_000;
    let mins = millis / 60_000;
    millis %= 60_000;
    let secs = millis / 1000;
    millis %= 1000;
    format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, millis)
}

pub fn export_srt(segments: &[SubtitleSegment], output_path: &Path) -> Result<PathBuf> {
    let mut file = File::create(output_path)?;
    for (i, seg) in segments.iter().enumerate() {
        writeln!(file, "{}", i + 1)?;
        writeln!(
            file,
            "{} --> {}",
            format_timestamp_srt(seg.start),
            format_timestamp_srt(seg.end)
        )?;
        writeln!(file, "{}\n", seg.text.trim())?;
    }
    Ok(output_path.to_path_buf())
}

pub fn export_vtt(segments: &[SubtitleSegment], output_path: &Path) -> Result<PathBuf> {
    let mut file = File::create(output_path)?;
    writeln!(file, "WEBVTT\n")?;
    for (i, seg) in segments.iter().enumerate() {
        writeln!(file, "{}", i + 1)?;
        writeln!(
            file,
            "{} --> {}",
            format_timestamp_vtt(seg.start),
            format_timestamp_vtt(seg.end)
        )?;
        writeln!(file, "{}\n", seg.text.trim())?;
    }
    Ok(output_path.to_path_buf())
}


// ---- Karaoke word captions (port of templates/runner.py: text_on_timings / chunk_words) ----

/// One timed word. JSON uses `w` (HyperFrames template format); `word` is accepted on input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KWord {
    #[serde(alias = "word", alias = "text")]
    pub w: String,
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WordChunk {
    pub start: f64,
    pub end: f64,
    pub words: Vec<KWord>,
}

/// Accepts `[{w|word,start,end}]`, or caption segments `[{words:[...]}]`.
pub fn parse_words(v: &serde_json::Value) -> Vec<KWord> {
    let Some(arr) = v.as_array() else { return vec![] };
    let mut out = vec![];
    for it in arr {
        if let Some(ws) = it.get("words").filter(|w| w.is_array()) {
            out.extend(parse_words(ws));
        } else if let Ok(w) = serde_json::from_value::<KWord>(it.clone()) {
            if !w.w.trim().is_empty() {
                out.push(KWord { w: w.w.trim().to_string(), ..w });
            }
        }
    }
    out
}

/// Put the script's words (correct spelling) on the speech interval [t0, t1], weighted by length.
pub fn text_on_timings(text: &str, t0: f64, t1: f64) -> Vec<KWord> {
    let toks: Vec<&str> = text.split_whitespace().collect();
    if toks.is_empty() || t1 <= t0 {
        return vec![];
    }
    let weights: Vec<f64> = toks.iter().map(|t| t.chars().count().max(2) as f64).collect();
    let total: f64 = weights.iter().sum();
    let mut cur = t0;
    toks.iter()
        .zip(weights)
        .map(|(t, w)| {
            let d = (t1 - t0) * w / total;
            let r = |x: f64| (x * 1000.0).round() / 1000.0;
            let k = KWord { w: t.to_string(), start: r(cur), end: r(cur + d) };
            cur += d;
            k
        })
        .collect()
}

/// Group words into caption chunks; a chunk stays until the next starts (no overlap).
pub fn chunk_words(words: &[KWord], per_chunk: usize) -> Vec<WordChunk> {
    let groups: Vec<&[KWord]> = words.chunks(per_chunk.max(1)).collect();
    groups
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let end = groups.get(i + 1).map(|n| n[0].start).unwrap_or(g[g.len() - 1].end + 0.3);
            WordChunk { start: g[0].start, end: (end * 1000.0).round() / 1000.0, words: g.to_vec() }
        })
        .collect()
}

/// Speech interval of an audio file (trims leading/trailing silence) via ffmpeg silencedetect.
pub fn speech_span(audio: &Path, total: f64) -> (f64, f64) {
    let out = std::process::Command::new("ffmpeg")
        .args(["-hide_banner", "-i"])
        .arg(audio)
        .args(["-af", "silencedetect=n=-35dB:d=0.25", "-f", "null", "-"])
        .output();
    let Ok(out) = out else { return (0.0, total) };
    let err = String::from_utf8_lossy(&out.stderr);
    let num = |l: &str, key: &str| -> Option<f64> {
        let i = l.find(key)? + key.len();
        l[i..].trim_start().split(|c: char| c.is_whitespace() || c == '|').next()?.parse().ok()
    };
    let (mut t0, mut t1) = (0.0, total);
    let mut open: Option<f64> = None;
    for l in err.lines() {
        if let Some(s) = num(l, "silence_start:") {
            open = Some(s);
        }
        if let Some(e) = num(l, "silence_end:") {
            if open.map(|s| s < 0.05).unwrap_or(false) {
                t0 = e;
            }
            open = None;
        }
    }
    if let Some(s) = open.filter(|s| *s > t0) {
        t1 = s;
    }
    (t0, t1.max(t0))
}

/// Segments (for SRT/VTT) built from word chunks.
pub fn segments_from_words(words: &[KWord], per_chunk: usize) -> Vec<SubtitleSegment> {
    chunk_words(words, per_chunk)
        .into_iter()
        .enumerate()
        .map(|(i, c)| SubtitleSegment {
            id: i as u32 + 1,
            start: c.start,
            end: c.end,
            text: c.words.iter().map(|w| w.w.as_str()).collect::<Vec<_>>().join(" "),
            words: c.words.iter().map(|w| SubtitleWord { word: w.w.clone(), start: w.start, end: w.end, score: None }).collect(),
        })
        .collect()
}
