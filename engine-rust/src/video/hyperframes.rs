//! HyperFrames engine: bundle building (multi-scene + karaoke captions + ducked BGM), CLI render with
//! cancel, ffmpeg fallback, post-process (loudness + thumbnail). Port of hyperframes_engine.py.
use anyhow::{bail, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use super::fallback;
use super::platforms::PlatformPreset;
use super::util::{self, ffmpeg, media_duration, run, tail};
use crate::audio_mastering::AudioMastering;
use crate::caption::{chunk_words, KWord};
use crate::workspace;

pub fn find_hyperframes_bin() -> Option<PathBuf> {
    if let Ok(b) = std::env::var("HYPERFRAMES_BIN") {
        let p = PathBuf::from(b);
        if p.is_file() {
            return Some(p);
        }
    }
    let name = if cfg!(windows) { "hyperframes.cmd" } else { "hyperframes" };
    let root = workspace::repo_root();
    for base in [root.join("tool/dashboard/node_modules/.bin"), root.join("node_modules/.bin")] {
        if base.join(name).is_file() {
            return Some(base.join(name));
        }
    }
    std::env::var_os("PATH").and_then(|p| std::env::split_paths(&p).map(|d| d.join(name)).find(|c| c.is_file()))
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub media: Option<PathBuf>,
    pub duration: f64,
    pub text: String,
    pub camera: String,
}

pub struct BundleSpec<'a> {
    pub scenes: Vec<Scene>,
    pub voice: Option<&'a Path>,
    pub bgm: Option<&'a Path>,
    pub words: Vec<KWord>,
    pub title_badge: String,
    pub preset: &'a PlatformPreset,
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn camera_tween(sel: &str, start: f64, dur: f64, cam: &str) -> String {
    let c = cam.to_lowercase();
    let (from, to) = if c.contains("pull") || c.contains("zoom out") || c.contains("wide") {
        ("{ scale: 1.12, xPercent: 0, yPercent: 0 }", "{ scale: 1.0, xPercent: 0, yPercent: 0")
    } else if c.contains("pan left") || c.contains("pan-left") {
        ("{ scale: 1.12, xPercent: 3, yPercent: 0 }", "{ scale: 1.12, xPercent: -3, yPercent: 0")
    } else if c.contains("pan right") || c.contains("pan-right") {
        ("{ scale: 1.12, xPercent: -3, yPercent: 0 }", "{ scale: 1.12, xPercent: 3, yPercent: 0")
    } else if c.contains("static") || c.contains("lock") {
        ("{ scale: 1.0 }", "{ scale: 1.0")
    } else {
        ("{ scale: 1.0, xPercent: 0, yPercent: 0 }", "{ scale: 1.08, xPercent: 0, yPercent: 0")
    };
    format!("tl.fromTo(\"{sel}\", {from}, {to}, duration: {dur:.3}, ease: \"none\" }}, {start:.3});")
}

/// Build a HyperFrames bundle from explicit scenes. Returns the composition duration.
/// The voice track decides the length; scene durations are rescaled to fit it.
pub async fn build_scenes_bundle(spec: &BundleSpec<'_>, dir: &Path, cancel: &CancellationToken) -> Result<f64> {
    let assets = dir.join("assets");
    std::fs::create_dir_all(&assets)?;
    let p = spec.preset;

    let has_voice = prepare_audio(spec.voice, spec.bgm, &assets, p, cancel).await?;

    let raw_total: f64 = spec.scenes.iter().map(|s| s.duration).sum::<f64>().max(0.5);
    let total = if has_voice { media_duration(&assets.join("voiceover.mp3"), raw_total) } else { raw_total };
    let k = total / raw_total;

    let mut html_scenes = String::new();
    let mut tweens = String::new();
    let mut manifest = vec![];
    let mut t = 0.0;
    let n = spec.scenes.len();
    for (i, s) in spec.scenes.iter().enumerate() {
        let d = s.duration * k;
        let id = format!("scene_{}", i + 1);
        let mut rel = None;
        if let Some(m) = s.media.as_ref().filter(|m| m.is_file()) {
            let ext = m.extension().and_then(|e| e.to_str()).unwrap_or("png");
            let name = format!("scene_{:02}.{}", i + 1, ext);
            std::fs::copy(m, assets.join(&name))?;
            rel = Some(format!("assets/{name}"));
        }
        let is_vid = rel.as_deref().map(|r| ["mp4", "mov", "webm"].iter().any(|e| r.ends_with(e))).unwrap_or(false);
        let visual = match (&rel, is_vid) {
            (Some(r), true) => format!(r#"<video id="m_{id}" class="clip media-clip" src="{r}" data-start="{t:.3}" data-duration="{d:.3}" data-track-index="0" playsinline muted preload="auto"></video>"#),
            (Some(r), false) => format!(r#"<img id="m_{id}" class="clip media-clip" src="{r}" data-start="{t:.3}" data-duration="{d:.3}" data-track-index="0" />"#),
            (None, _) => format!(r#"<div id="m_{id}" class="clip media-clip placeholder" data-start="{t:.3}" data-duration="{d:.3}" data-track-index="0"><span>{}</span></div>"#, esc(&s.text)),
        };
        let sub = if spec.words.is_empty() && !s.text.trim().is_empty() {
            format!(
                r#"<div id="sub_{id}" class="subtitle-wrapper clip" data-start="{t:.3}" data-duration="{d:.3}" data-track-index="1"><div class="subtitle-pill"><span class="sub-text">{}</span></div></div>"#,
                esc(&s.text)
            )
        } else {
            String::new()
        };
        html_scenes.push_str(&format!("<div id=\"{id}\" class=\"scene-container\">{visual}{sub}</div>\n"));
        let sel = format!("#m_{id}");
        tweens.push_str(&format!(
            "tl.set(\"#{id}\", {{ opacity: 1 }}, {t:.3});\n{}\n",
            camera_tween(&sel, t, d, &s.camera)
        ));
        if !sub.is_empty() {
            tweens.push_str(&format!("tl.fromTo(\"#sub_{id}\", {{ y: 25, opacity: 0 }}, {{ y: 0, opacity: 1, duration: 0.35, ease: \"back.out(1.4)\" }}, {:.3});\n", t + 0.1));
        }
        tweens.push_str(&format!("tl.set(\"#{id}\", {{ opacity: 0 }}, {:.3});\n", if i + 1 == n { total } else { t + d }));
        manifest.push(json!({"media": rel, "duration": d, "text": s.text}));
        t += d;
    }

    // Karaoke captions: chunk of ≤5 words, current word highlighted at its own timestamp.
    let mut kw_html = String::new();
    let mut kw_js = String::new();
    for (ci, c) in chunk_words(&spec.words, 5).iter().enumerate() {
        let cd = (c.end - c.start).max(0.2);
        let spans: String = c.words.iter().enumerate().map(|(wi, w)| format!(r#"<span class="kw" id="kw_{ci}_{wi}">{}</span> "#, esc(&w.w))).collect();
        kw_html.push_str(&format!(
            "<div id=\"kc_{ci}\" class=\"subtitle-wrapper clip\" data-start=\"{:.3}\" data-duration=\"{:.3}\" data-track-index=\"1\"><div class=\"subtitle-pill\">{spans}</div></div>\n",
            c.start, cd
        ));
        kw_js.push_str(&format!("tl.fromTo(\"#kc_{ci}\", {{ opacity: 0, y: 18 }}, {{ opacity: 1, y: 0, duration: 0.15 }}, {:.3});\n", c.start));
        for (wi, w) in c.words.iter().enumerate() {
            kw_js.push_str(&format!("tl.to(\"#kw_{ci}_{wi}\", {{ color: \"#ffe600\", scale: 1.12, duration: 0.08 }}, {:.3});\n", w.start));
        }
    }

    let sz = &p.safe_zone;
    let sub_top = (p.height as i64 - sz.bottom as i64 - 180).max(100);
    let audio = if has_voice {
        format!(r#"<audio id="audio-voiceover" class="clip" src="assets/voiceover.mp3" data-start="0" data-duration="{total:.3}" data-track-index="3"></audio>"#)
    } else {
        String::new()
    };
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="vi"><head><meta charset="UTF-8" /><meta name="viewport" content="width={w}, height={h}, initial-scale=1.0" /><title>{badge}</title>
<script src="assets/gsap.min.js"></script>
<style>
*{{box-sizing:border-box;margin:0;padding:0}}
body,html{{width:{w}px;height:{h}px;overflow:hidden;background:#000;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,Arial,sans-serif}}
#root{{position:relative;width:{w}px;height:{h}px;overflow:hidden}}
.scene-container{{position:absolute;inset:0;opacity:0}}
.media-clip{{position:absolute;inset:0;width:100%;height:100%;object-fit:cover}}
.placeholder{{background:#111625;display:flex;align-items:center;justify-content:center;color:#fff;font-size:48px;padding:80px;text-align:center}}
.vignette{{position:absolute;inset:0;background:linear-gradient(180deg,rgba(0,0,0,.55) 0%,transparent 22%,transparent 72%,rgba(0,0,0,.8) 100%);z-index:5}}
.badge{{position:absolute;top:{top}px;left:{left}px;z-index:10;background:rgba(15,23,42,.85);border:1.5px solid rgba(255,255,255,.2);padding:10px 24px;border-radius:9999px;color:#fff;font-size:28px;font-weight:800;letter-spacing:.05em;text-transform:uppercase}}
.subtitle-wrapper{{position:absolute;top:{sub_top}px;left:0;width:100%;display:flex;justify-content:center;padding:0 {pl}px 0 {pl}px;z-index:10}}
.subtitle-pill{{background:rgba(0,0,0,.82);border:1.5px solid rgba(255,255,255,.22);border-radius:26px;padding:18px 36px;max-width:{maxw}px;text-align:center;box-shadow:0 12px 35px rgba(0,0,0,.6)}}
.sub-text,.kw{{color:#fff;font-size:{fs}px;font-weight:800;line-height:1.35;text-shadow:0 3px 12px rgba(0,0,0,.95);display:inline-block}}
.sub-text{{color:#ffe600}}
.kw{{margin:0 .14em}}
</style></head><body>
<div id="root" data-composition-id="main" data-start="0" data-duration="{total:.3}" data-width="{w}" data-height="{h}">
<div class="vignette clip" data-start="0" data-duration="{total:.3}" data-track-index="2"></div>
<div id="top-badge" class="badge clip" data-start="0" data-duration="{total:.3}" data-track-index="2">{badge}</div>
{html_scenes}{kw_html}{audio}
</div>
<script>
window.__timelines = window.__timelines || {{}};
const tl = gsap.timeline({{ paused: true }});
{tweens}{kw_js}window.__timelines["main"] = tl;
tl.seek(0);
</script></body></html>"#,
        w = p.width,
        h = p.height,
        badge = esc(&spec.title_badge),
        top = sz.top,
        left = sz.left.max(30),
        sub_top = sub_top,
        pl = sz.left.max(30),
        maxw = p.width as i64 - sz.left as i64 - sz.right as i64,
        fs = if p.width >= p.height { 48 } else { 52 },
    );
    std::fs::write(dir.join("index.html"), html)?;
    ensure_gsap(&assets)?;
    write_hf_json(dir)?;
    std::fs::write(dir.join("scenes.json"), serde_json::to_string_pretty(&json!({"scenes": manifest, "words": spec.words, "duration": total}))?)?;
    Ok(total)
}

/// Put the voice (optionally mixed with ducked BGM) at assets/voiceover.mp3. Returns whether audio exists.
pub async fn prepare_audio(voice: Option<&Path>, bgm: Option<&Path>, assets: &Path, p: &PlatformPreset, cancel: &CancellationToken) -> Result<bool> {
    std::fs::create_dir_all(assets)?;
    let Some(v) = voice.filter(|v| v.is_file()) else { return Ok(false) };
    let dest = assets.join("voiceover.mp3");
    if let Some(b) = bgm.filter(|b| b.is_file()) {
        // real ducking: BGM sidechain-compressed by the voice, mixed and normalised
        let (v, b, d) = (v.to_path_buf(), b.to_path_buf(), dest.clone());
        let lufs = p.loudness_lufs;
        tokio::task::spawn_blocking(move || AudioMastering::new(None).mix_voice_and_bgm_ducking(&v, &b, &d, lufs)).await??;
    } else {
        ffmpeg(&["-i".into(), v.to_string_lossy().into(), "-ar".into(), "44100".into(), "-b:a".into(), "192k".into(), dest.to_string_lossy().into()], cancel, 300).await?;
    }
    Ok(true)
}

/// GSAP is loaded locally (offline-safe, deterministic): copy the vendored copy shipped with the templates.
pub fn ensure_gsap(assets: &Path) -> Result<()> {
    let dest = assets.join("gsap.min.js");
    if dest.exists() {
        return Ok(());
    }
    let src = workspace::data_root().join("shared/templates/_engine/vendor/gsap.min.js");
    if src.is_file() {
        std::fs::copy(src, dest)?;
        Ok(())
    } else {
        bail!("Thiếu data/shared/templates/_engine/vendor/gsap.min.js")
    }
}

pub fn write_hf_json(dir: &Path) -> Result<()> {
    std::fs::write(
        dir.join("hyperframes.json"),
        json!({"$schema": "https://hyperframes.heygen.com/schema/hyperframes.json", "paths": {"blocks": "compositions", "components": "compositions/components", "assets": "assets"}}).to_string(),
    )?;
    Ok(())
}

/// `hyperframes check --json` (warn-only).
pub async fn check(bundle: &Path, cancel: &CancellationToken) -> Value {
    let Some(bin) = find_hyperframes_bin() else { return json!({"ok": false, "error": "HyperFrames CLI không tồn tại."}) };
    let mut c = Command::new(bin);
    c.args(["check", "--json"]).current_dir(bundle);
    match run(&mut c, cancel, Duration::from_secs(90), None).await {
        Ok(o) => {
            let ok = o.code == 0;
            match o.stdout.find('{').and_then(|i| serde_json::from_str::<Value>(&o.stdout[i..]).ok()) {
                Some(d) => json!({"ok": ok, "details": d}),
                None => json!({"ok": ok, "output": tail(&format!("{}{}", o.stdout, o.stderr), 400)}),
            }
        }
        Err(e) => json!({"ok": false, "error": e.to_string()}),
    }
}

pub struct RenderOpts {
    pub quality: String,
    pub cancel: CancellationToken,
    pub log: Option<PathBuf>,
    pub timeout_s: u64,
    pub require_hyperframes: bool,
    pub on_progress: Option<Arc<dyn Fn(f64) + Send + Sync>>,
}

impl RenderOpts {
    pub fn new(cancel: CancellationToken) -> Self {
        Self { quality: "draft".into(), cancel, log: None, timeout_s: 900, require_hyperframes: false, on_progress: None }
    }
}

#[derive(Debug, Clone)]
pub struct RenderOutcome {
    pub engine: String,
    pub cli_error: String,
}

fn append_log(log: &Option<PathBuf>, s: &str) {
    if let Some(l) = log {
        if let Some(p) = l.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(l) {
            let _ = f.write_all(s.as_bytes());
        }
    }
}

/// Render a bundle to MP4 via the HyperFrames CLI (Node/Chrome); falls back to ffmpeg unless required.
pub async fn render(bundle: &Path, out: &Path, preset: &PlatformPreset, o: &RenderOpts) -> Result<RenderOutcome> {
    if let Some(p) = out.parent() {
        std::fs::create_dir_all(p)?;
    }
    let _ = std::fs::remove_file(out);
    let mut cli_error = String::new();
    if std::env::var("HYPERFRAMES_USE_CLI").map(|v| v != "0").unwrap_or(true) {
        match find_hyperframes_bin() {
            None => cli_error = "Không tìm thấy HyperFrames CLI (đặt HYPERFRAMES_BIN hoặc cài tool/dashboard/node_modules).".into(),
            Some(bin) => {
                let comp_dur = std::fs::read_to_string(bundle.join("index.html"))
                    .ok()
                    .and_then(|h| regex::Regex::new(r#"data-duration=["']([0-9.]+)["']"#).unwrap().captures(&h).and_then(|c| c[1].parse::<f64>().ok()))
                    .unwrap_or(0.0);
                let mut workers1 = comp_dur > 60.0;
                for attempt in 0..2 {
                    let mut c = Command::new(&bin);
                    c.args(["render", "--skill", "hyperframes", "--quality", &o.quality, "--fps", &preset.fps.to_string(), "--output"]).arg(out).current_dir(bundle);
                    if workers1 {
                        c.args(["--workers", "1"]);
                    }
                    let re = regex::Regex::new(r"(\d{1,3})\s*%").unwrap();
                    let hook: Option<util::LineHook> = o.on_progress.clone().map(|cb| {
                        let f: util::LineHook = Arc::new(move |l: &str| {
                            if let Some(m) = re.captures_iter(l).last().and_then(|c| c[1].parse::<f64>().ok()) {
                                cb(m.min(100.0));
                            }
                        });
                        f
                    });
                    let r = run(&mut c, &o.cancel, Duration::from_secs(o.timeout_s), hook).await;
                    let r = match r {
                        Err(e) if e.to_string() == "cancelled" => return Err(e),
                        r => r,
                    };
                    match r {
                        Ok(out_) => {
                            append_log(&o.log, &format!("--- hyperframes render (code {}) ---\n{}\n{}\n", out_.code, out_.stdout, out_.stderr));
                            if out_.code == 0 && std::fs::metadata(out).map(|m| m.len() > 0).unwrap_or(false) {
                                post_process(out, preset);
                                return Ok(RenderOutcome { engine: "hyperframes".into(), cli_error: String::new() });
                            }
                            let msg = format!("{}{}", out_.stderr, out_.stdout);
                            cli_error = tail(&msg, 500);
                            let low = msg.to_lowercase();
                            if attempt == 0 && !workers1 && ["enospc", "space", "disk", "killed", "memory"].iter().any(|k| low.contains(k)) {
                                workers1 = true;
                                continue;
                            }
                        }
                        Err(e) => {
                            cli_error = e.to_string();
                            append_log(&o.log, &format!("hyperframes exception: {e}\n"));
                        }
                    }
                    break;
                }
            }
        }
    }
    if o.require_hyperframes {
        bail!("HyperFrames CLI render thất bại và require_hyperframes=true. Lỗi: {cli_error}");
    }
    tracing::warn!("FALLBACK_USED ffmpeg cho {} ({})", preset.id, cli_error);
    append_log(&o.log, &format!("FALLBACK_USED: ffmpeg ({cli_error})\n"));
    let plan = fallback::render_bundle(bundle, out, preset, preset.fps, &o.cancel).await?;
    append_log(&o.log, &format!("fallback plan source={} scenes={}\n", plan.source, plan.scenes.len()));
    post_process(out, preset);
    Ok(RenderOutcome { engine: "ffmpeg_fallback".into(), cli_error })
}

/// Loudness normalisation + thumbnail next to the video (best effort).
pub fn post_process(video: &Path, preset: &PlatformPreset) {
    if let Err(e) = AudioMastering::new(None).normalize_video_audio(video, preset.loudness_lufs) {
        tracing::warn!("loudnorm lỗi: {e}");
    }
    extract_thumbnail(video, &video.with_extension("jpg"), 1.0);
}

pub fn extract_thumbnail(video: &Path, jpg: &Path, at: f64) {
    let dur = media_duration(video, 0.0);
    let ts = if dur > at + 0.1 { at } else { 0.0 };
    let _ = std::process::Command::new(util::ffmpeg_bin())
        .args(["-hide_banner", "-y", "-ss", &format!("{ts:.2}"), "-i"])
        .arg(video)
        .args(["-vframes", "1", "-q:v", "2"])
        .arg(jpg)
        .output();
}
