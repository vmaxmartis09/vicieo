use anyhow::{bail, Result};
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

