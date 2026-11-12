//! Transcript: existing WebVTT first, then a faster-whisper Python sidecar (WHISPER_SIDECAR overrides the script path).
use crate::intelligence::util::*;
use regex::Regex;
use serde_json::{json, Value};
use std::path::Path;

fn to_sec(t: &str) -> f64 {
    let p: Vec<&str> = t.trim().split(':').collect();
    match p.len() {
        3 => p[0].parse::<f64>().unwrap_or(0.0) * 3600.0 + p[1].parse::<f64>().unwrap_or(0.0) * 60.0 + p[2].parse::<f64>().unwrap_or(0.0),
        2 => p[0].parse::<f64>().unwrap_or(0.0) * 60.0 + p[1].parse::<f64>().unwrap_or(0.0),
        _ => t.trim().parse().unwrap_or(0.0),
    }
}

pub fn parse_vtt(text: &str) -> Vec<Value> {
    let tag = Regex::new(r"<[^>]+>").unwrap();
    let mut out = vec![];
    for block in Regex::new(r"\n\s*\n").unwrap().split(text.replace("\r\n", "\n").trim()) {
        let lines: Vec<&str> = block.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        if lines.is_empty() || lines[0].contains("WEBVTT") { continue; }
        let Some(ti) = lines.iter().position(|l| l.contains("-->")) else { continue };
        let parts: Vec<&str> = lines[ti].split("-->").collect();
        if parts.len() != 2 { continue; }
        let (Some(a), Some(b)) = (parts[0].split_whitespace().next(), parts[1].split_whitespace().next()) else { continue };
        let t = tag.replace_all(&lines[ti + 1..].join(" "), "").trim().to_string();
        if !t.is_empty() {
            out.push(json!({"start": (to_sec(a) * 100.0).round() / 100.0, "end": (to_sec(b) * 100.0).round() / 100.0, "text": t, "words": []}));
        }
    }
    out
}

pub async fn run_asr(pid: &str, source_id: &str, language: &str, model: Option<&str>) -> ApiResult<Vec<Value>> {
    let dir = pp(pid, &["raw", safe_id(source_id).map_err(|e| ApiError::bad(e.to_string()))?, "x"], false).parent().unwrap().to_path_buf();
    let out_json = dir.join("transcript.orig.json");
    if let Ok(rd) = std::fs::read_dir(&dir) {
        let mut vtts: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "vtt")).collect();
        vtts.sort();
        for v in vtts {
            let segs = parse_vtt(&std::fs::read_to_string(&v).unwrap_or_default());
            if !segs.is_empty() { write_json(&out_json, &segs)?; return Ok(segs); }
        }
    }
    let audio = dir.join("audio.wav");
    if !audio.exists() { return Err(ApiError::nf(format!("Audio file not found: {}", audio.display()))); }
    let segs = whisper(&audio, language, model).await.unwrap_or_else(|e| {
        tracing::warn!("whisper sidecar unavailable: {e}");
        vec![json!({"start": 0.0, "end": 5.0, "text": "Nội dung video nguồn", "words": [], "placeholder": true})]
    });
    write_json(&out_json, &segs)?;
    Ok(segs)
}

async fn whisper(audio: &Path, lang: &str, model: Option<&str>) -> anyhow::Result<Vec<Value>> {
    let script = std::env::var("WHISPER_SIDECAR").map(std::path::PathBuf::from)
        .unwrap_or_else(|_| crate::workspace::repo_root().join("engine-rust/src/sources/whisper_sidecar.py"));
    let model = model.map(String::from).or_else(|| std::env::var("WHISPER_MODEL").ok()).unwrap_or_else(|| "base".into());
    let o = tokio::process::Command::new(std::env::var("PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(&script).arg(audio).arg(lang).arg(model).output().await?;
    if !o.status.success() { anyhow::bail!("{}", String::from_utf8_lossy(&o.stderr).lines().last().unwrap_or("sidecar failed")); }
    Ok(serde_json::from_slice(&o.stdout)?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn vtt() {
        let v = super::parse_vtt("WEBVTT\n\n00:00:01.000 --> 00:00:03.500\n<c>Xin chào</c>\n\n01:02.000 --> 01:04.000 align:start\nHai");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0]["text"], "Xin chào");
        assert_eq!(v[1]["start"], 62.0);
    }
}
