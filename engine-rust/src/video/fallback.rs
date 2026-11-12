//! ffmpeg compositor used when the HyperFrames CLI is unavailable (port of fallback_ffmpeg.py).
//!
//! Unlike the Python version it follows the bundle's real timeline (scenes.json written by
//! `hyperframes::build_scenes_bundle`, or the `<img>/<video>` tags of a template's index.html) instead of
//! globbing `scene_*` files with equal durations, and never emits a black screen when media exists.
use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;

use super::platforms::PlatformPreset;
use super::util::{ffmpeg, font_file, has_drawtext, media_duration};
use crate::caption::{chunk_words, KWord};

#[derive(Debug, Clone, Deserialize)]
pub struct PlanScene {
    pub media: Option<String>,
    pub duration: f64,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub scenes: Vec<PlanScene>,
    pub words: Vec<KWord>,
    pub source: &'static str,
}

fn is_visual(name: &str) -> bool {
    let n = name.to_lowercase();
    [".png", ".jpg", ".jpeg", ".webp", ".mp4", ".mov", ".webm"].iter().any(|e| n.ends_with(e))
}
fn is_video(name: &str) -> bool {
    let n = name.to_lowercase();
    [".mp4", ".mov", ".webm"].iter().any(|e| n.ends_with(e))
}

/// Decide what to show: scenes.json → HTML tags → scene_* files → text card.
pub fn plan_from_bundle(bundle: &Path) -> Plan {
    let assets = bundle.join("assets");
    let voice_dur = {
        let v = assets.join("voiceover.mp3");
        if v.exists() { media_duration(&v, 0.0) } else { 0.0 }
    };
    if let Ok(s) = std::fs::read_to_string(bundle.join("scenes.json")) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
            let scenes: Vec<PlanScene> = serde_json::from_value(v["scenes"].clone()).unwrap_or_default();
            if !scenes.is_empty() {
                return Plan { scenes, words: crate::caption::parse_words(&v["words"]), source: "scenes.json" };
            }
        }
    }
    let total = if voice_dur > 0.0 { voice_dur } else { 4.0 };
    if let Ok(html) = std::fs::read_to_string(bundle.join("index.html")) {
        let tag = regex::Regex::new(r#"<(?:img|video)\b[^>]*>"#).unwrap();
        let attr = |t: &str, k: &str| -> Option<String> {
            regex::Regex::new(&format!(r#"{k}="([^"]*)""#)).unwrap().captures(t).map(|c| c[1].to_string())
        };
        let mut items: Vec<(f64, String)> = vec![];
        for m in tag.find_iter(&html) {
            let t = m.as_str();
            if let Some(src) = attr(t, "src").and_then(|s| s.strip_prefix("assets/").map(|s| s.to_string())) {
                if is_visual(&src) && assets.join(&src).is_file() && !items.iter().any(|i| i.1 == src) {
                    let start = attr(t, "data-start").and_then(|s| s.parse().ok()).unwrap_or(items.len() as f64 * 1e-3);
                    items.push((start, src));
                }
            }
        }
        if !items.is_empty() {
            items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            // Items that start together are shown one after another: equal share of the total time.
            let d = total / items.len() as f64;
            let scenes: Vec<PlanScene> = items.iter().map(|i| PlanScene { media: Some(format!("assets/{}", i.1)), duration: d, text: String::new() }).collect();
            return Plan { scenes, words: vec![], source: "index.html" };
        }
    }
    let mut files: Vec<String> = std::fs::read_dir(&assets)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with("scene_") && is_visual(n))
        .collect();
    files.sort();
    if !files.is_empty() {
        let d = total / files.len() as f64;
        return Plan { scenes: files.into_iter().map(|f| PlanScene { media: Some(format!("assets/{f}")), duration: d, text: String::new() }).collect(), words: vec![], source: "scene_files" };
    }
    Plan { scenes: vec![PlanScene { media: None, duration: total, text: String::new() }], words: vec![], source: "text_card" }
}

fn esc_filter_path(p: &str) -> String {
    p.replace('\\', "/").replace(':', "\\:").replace('\'', "\\'")
}

pub async fn render_bundle(bundle: &Path, out: &Path, preset: &PlatformPreset, fps: u32, cancel: &CancellationToken) -> Result<Plan> {
    let plan = plan_from_bundle(bundle);
    render_plan(bundle, &plan, out, preset.width, preset.height, fps, &preset.audio_bitrate, cancel).await?;
    Ok(plan)
}

#[allow(clippy::too_many_arguments)]
pub async fn render_plan(bundle: &Path, plan: &Plan, out: &Path, w: u32, h: u32, fps: u32, abr: &str, cancel: &CancellationToken) -> Result<()> {
    if let Some(p) = out.parent() {
        std::fs::create_dir_all(p)?;
    }
    let (w, h) = (w & !1, h & !1);
    let voice = bundle.join("assets").join("voiceover.mp3");
    let has_voice = voice.is_file();
    let mut args: Vec<String> = vec![];
    let mut filters: Vec<String> = vec![];
    let n = plan.scenes.len();
    let total: f64 = plan.scenes.iter().map(|s| s.duration).sum();
    for (i, s) in plan.scenes.iter().enumerate() {
        let d = format!("{:.3}", s.duration.max(0.1));
        let media: Option<PathBuf> = s.media.as_ref().map(|m| bundle.join(m)).filter(|p| p.is_file());
        match &media {
            Some(p) if is_video(&p.to_string_lossy()) => args.extend(["-stream_loop".into(), "-1".into(), "-t".into(), d, "-i".into(), p.to_string_lossy().into()]),
            Some(p) => args.extend(["-loop".into(), "1".into(), "-framerate".into(), fps.to_string(), "-t".into(), d, "-i".into(), p.to_string_lossy().into()]),
            None => args.extend(["-f".into(), "lavfi".into(), "-i".into(), format!("color=c=0x111625:s={w}x{h}:r={fps}:d={d}")]),
        }
        filters.push(format!("[{i}:v]scale={w}:{h}:force_original_aspect_ratio=increase,crop={w}:{h},setsar=1,fps={fps},format=yuv420p[v{i}]"));
    }
    let cat_in: String = (0..n).map(|i| format!("[v{i}]")).collect();
    let mut last = "vcat".to_string();
    filters.push(format!("{cat_in}concat=n={n}:v=1:a=0[vcat]"));

    // Caption text only when ffmpeg has drawtext (stock Homebrew build does not).
    let mut tmp_text = vec![];
    if has_drawtext() && !plan.words.is_empty() {
        if let Some(font) = font_file() {
            for (k, c) in chunk_words(&plan.words, 5).iter().enumerate() {
                let tf = bundle.join(format!("_cap_{k}.txt"));
                std::fs::write(&tf, c.words.iter().map(|w| w.w.as_str()).collect::<Vec<_>>().join(" "))?;
                let nxt = format!("cap{k}");
                filters.push(format!(
                    "[{last}]drawtext=fontfile='{}':textfile='{}':fontsize={}:fontcolor=0xffe600:borderw=4:bordercolor=black:x=(w-text_w)/2:y=h*0.72:enable='between(t,{:.3},{:.3})'[{nxt}]",
                    esc_filter_path(&font), esc_filter_path(&tf.to_string_lossy()), h / 28, c.start, c.end
                ));
                last = nxt;
                tmp_text.push(tf);
            }
        }
    }

    if has_voice {
        args.extend(["-i".into(), voice.to_string_lossy().into()]);
    }
    args.extend(["-filter_complex".into(), filters.join(";"), "-map".into(), format!("[{last}]")]);
    if has_voice {
        args.extend(["-map".into(), format!("{n}:a"), "-c:a".into(), "aac".into(), "-b:a".into(), abr.into()]);
    }
    args.extend(["-c:v", "libx264", "-preset", "veryfast", "-pix_fmt", "yuv420p", "-movflags", "+faststart", "-t"].map(String::from));
    args.push(format!("{:.3}", total));
    args.push(out.to_string_lossy().into());
    let r = ffmpeg(&args, cancel, 900).await;
    for t in tmp_text {
        let _ = std::fs::remove_file(t);
    }
    r?;
    if std::fs::metadata(out).map(|m| m.len() == 0).unwrap_or(true) {
        bail!("ffmpeg fallback không tạo được tệp: {}", out.display());
    }
    Ok(())
}
