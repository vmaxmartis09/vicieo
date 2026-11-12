//! Render pipeline shared by the job queue and the graph executors: RenderRequest → bundle → render → verify.
use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use super::hyperframes::{self, BundleSpec, RenderOpts, Scene};
use super::platforms::{get_platform, PlatformPreset};
use super::templates::{self, ComposeInput};
use super::util::{media_duration, resolve_media};
use super::verify::{verify_video, VerifyOpts};
use crate::caption::{chunk_words, parse_words, speech_span, text_on_timings, KWord};
use crate::workspace;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VoiceSpec {
    #[serde(default)]
    pub voice_id: Option<String>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub audio_path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderRequest {
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub template_id: String,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub data: Map<String, Value>,
    #[serde(default)]
    pub voice: Option<VoiceSpec>,
    #[serde(default)]
    pub source_ref: Option<String>,
    #[serde(default)]
    pub output_name: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RenderResult {
    /// platform → project-relative path (`output/<name>/<platform>.mp4`)
    pub outputs: HashMap<String, String>,
    pub engines: HashMap<String, String>,
    pub verify: HashMap<String, Value>,
    pub warnings: Vec<String>,
}

pub type Progress = Arc<dyn Fn(f64, &str) + Send + Sync>;

pub struct RenderCtx {
    pub cancel: CancellationToken,
    pub progress: Progress,
    /// scratch + logs, e.g. runs/render_queue/<job>
    pub work_dir: PathBuf,
}

fn str_of<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|k| v.get(k).and_then(|x| x.as_str()).filter(|s| !s.trim().is_empty()))
}

/// Copy every string in `v` that points to a real media file into `assets/` and rewrite it to `assets/<name>`.
fn localize(v: &mut Value, project_id: &str, assets: &Path, seen: &mut HashMap<PathBuf, String>) {
    match v {
        Value::String(s) => {
            if s.starts_with("assets/") || s.len() > 600 || s.contains('\n') {
                return;
            }
            let Some(src) = resolve_media(project_id, s) else { return };
            let Some(ext) = src.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()) else { return };
            if !["png", "jpg", "jpeg", "webp", "gif", "mp4", "mov", "webm", "mp3", "wav", "m4a", "svg"].contains(&ext.as_str()) {
                return;
            }
            let name = seen.entry(src.clone()).or_insert_with(|| format!("u{:02}_{}", seen_len(&src), src.file_name().unwrap().to_string_lossy())).clone();
            if std::fs::copy(&src, assets.join(&name)).is_ok() {
                *s = format!("assets/{name}");
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|x| localize(x, project_id, assets, seen)),
        Value::Object(o) => o.values_mut().for_each(|x| localize(x, project_id, assets, seen)),
        _ => {}
    }
}
fn seen_len(p: &Path) -> usize {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    p.hash(&mut h);
    (h.finish() % 100) as usize
}

fn scenes_from_data(project_id: &str, data: &Map<String, Value>) -> Vec<Scene> {
    let mut out = vec![];
    for sc in data.get("scenes").and_then(|s| s.as_array()).cloned().unwrap_or_default() {
        let media = str_of(&sc, &["video", "img", "image", "src", "mediaSrc", "url"]).and_then(|m| resolve_media(project_id, m));
        out.push(Scene {
            media,
            duration: sc.get("durationSec").or_else(|| sc.get("duration")).and_then(|d| d.as_f64()).unwrap_or(4.0).max(0.5),
            text: str_of(&sc, &["dialogue", "sub", "subtitle", "text"]).unwrap_or("").to_string(),
            camera: str_of(&sc, &["camera", "cameraPrompt", "shotCamera"]).unwrap_or("").to_string(),
        });
    }
    if out.is_empty() {
        let img = str_of(&Value::Object(data.clone()), &["image", "img", "video"]).and_then(|m| resolve_media(project_id, m));
        let txt = str_of(&Value::Object(data.clone()), &["dialogue", "hook", "hook_text", "title"]).unwrap_or("").to_string();
        out.push(Scene { media: img, duration: 4.0, text: txt, camera: String::new() });
    }
    out
}

/// Words for karaoke: explicit timestamps → else script text spread over the voice's speech interval.
pub fn resolve_words(req: &RenderRequest, voice: Option<&Path>, warnings: &mut Vec<String>) -> Vec<KWord> {
    for k in ["words", "word_timestamps", "captions"] {
        if let Some(v) = req.data.get(k) {
            let w = parse_words(v);
            if !w.is_empty() {
                return w;
            }
        }
    }
    let Some(v) = voice else { return vec![] };
    let text = req
        .voice
        .as_ref()
        .and_then(|v| v.script.clone())
        .or_else(|| req.data.get("voice_text").and_then(|x| x.as_str()).map(String::from))
        .unwrap_or_else(|| {
            req.data.get("scenes").and_then(|s| s.as_array()).map(|a| a.iter().filter_map(|s| str_of(s, &["dialogue", "sub", "text"])).collect::<Vec<_>>().join(" ")).unwrap_or_default()
        });
    if text.trim().is_empty() {
        return vec![];
    }
    let total = media_duration(v, 0.0);
    let (t0, t1) = speech_span(v, total);
    warnings.push("Phụ đề karaoke: không có mốc từ của giọng nên chia đều theo độ dài chữ trong khoảng giọng nói thực (không phải forced-alignment).".into());
    text_on_timings(&text, t0, t1)
}

fn sanitize_name(n: &str) -> String {
    let s: String = n.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    if s.is_empty() { "render".into() } else { s }
}

/// Render one platform. Returns (out_path, engine).
#[allow(clippy::too_many_arguments)]
async fn render_one(req: &RenderRequest, preset: &PlatformPreset, voice: Option<&Path>, bgm: Option<&Path>, words: &[KWord], out: &Path, ctx: &RenderCtx, warnings: &mut Vec<String>, pct: Arc<dyn Fn(f64) + Send + Sync>) -> Result<String> {
    let bundle = ctx.work_dir.join(format!("bundle_{}", preset.id));
    let _ = std::fs::remove_dir_all(&bundle);
    std::fs::create_dir_all(bundle.join("assets"))?;
    let assets = bundle.join("assets");
    let require_hf = req.data.get("require_hyperframes").and_then(|v| v.as_bool()).unwrap_or(false);
    let quality = req.data.get("quality").and_then(|v| v.as_str()).unwrap_or("draft").to_string();

    let mut used_template = false;
    if !req.template_id.is_empty() {
        match build_template_bundle(req, preset, voice, bgm, Some(words), &bundle, ctx).await {
            Ok(()) => used_template = true,
            Err(e) => {
                if e.to_string() == "cancelled" {
                    return Err(e);
                }
                warnings.push(format!("[{}] Template '{}' không dựng được ({e}); dùng bộ ghép phân cảnh.", preset.id, req.template_id));
                let _ = std::fs::remove_dir_all(&bundle);
                std::fs::create_dir_all(&assets)?;
            }
        }
    }
    if !used_template {
        let spec = BundleSpec {
            scenes: scenes_from_data(&req.project_id, &req.data),
            voice,
            bgm,
            words: words.to_vec(),
            title_badge: req.data.get("titleBadge").or_else(|| req.data.get("title")).and_then(|v| v.as_str()).unwrap_or("SÓC NHỎ TV").to_string(),
            preset,
        };
        hyperframes::build_scenes_bundle(&spec, &bundle, &ctx.cancel).await?;
    }
    let chk = hyperframes::check(&bundle, &ctx.cancel).await;
    if chk.get("ok") == Some(&json!(false)) {
        warnings.push(format!("[{}] hyperframes check: {}", preset.id, chk.to_string().chars().take(300).collect::<String>()));
    }
    let mut o = RenderOpts::new(ctx.cancel.clone());
    o.quality = quality;
    o.require_hyperframes = require_hf;
    o.log = Some(ctx.work_dir.join("render.log"));
    o.on_progress = Some(pct);
    let outcome = hyperframes::render(&bundle, out, preset, &o).await?;
    Ok(outcome.engine)
}

async fn build_template_bundle(req: &RenderRequest, preset: &PlatformPreset, voice: Option<&Path>, bgm: Option<&Path>, words: Option<&[KWord]>, bundle: &Path, ctx: &RenderCtx) -> Result<()> {
    let meta = templates::get_template(&req.template_id)?;
    let tdir = templates::template_dir(&req.template_id)?;
    let assets = bundle.join("assets");
    if tdir.join("assets").is_dir() {
        for e in std::fs::read_dir(tdir.join("assets"))?.flatten() {
            if e.path().is_file() {
                std::fs::copy(e.path(), assets.join(e.file_name()))?;
            }
        }
    }
    // user assets: data.assets = {"product.png": "/path or project-relative"}
    if let Some(m) = req.data.get("assets").and_then(|a| a.as_object()) {
        for (name, src) in m {
            let src = src.as_str().and_then(|s| resolve_media(&req.project_id, s)).ok_or_else(|| anyhow!("Tệp asset '{name}' không tồn tại"))?;
            std::fs::copy(src, assets.join(Path::new(name).file_name().ok_or_else(|| anyhow!("tên asset không hợp lệ"))?))?;
        }
    }
    let mut data = Value::Object(req.data.clone());
    data.as_object_mut().unwrap().remove("assets");
    let mut seen = HashMap::new();
    localize(&mut data, &req.project_id, &assets, &mut seen);
    let mut data = data.as_object().cloned().unwrap_or_default();

    let has_audio = hyperframes::prepare_audio(voice, bgm, &assets, preset, &ctx.cancel).await?;
    let tmpl_src = std::fs::read_to_string(tdir.join("composition.html.j2")).unwrap_or_default();
    if let (true, Some(words)) = (tmpl_src.contains("word_chunks"), words) {
        // karaoke must follow the real voice, never the sample timings packed in the template
        data.insert("word_chunks".into(), serde_json::to_value(chunk_words(words, 5))?);
    }
    let voice_file = assets.join("voiceover.mp3");
    let (html, _dur) = templates::compose_template(&ComposeInput {
        template: &meta,
        dir: &tdir,
        data: &data,
        preset,
        voice_audio: if has_audio { Some(voice_file.as_path()) } else { None },
        has_bgm: false, // BGM already ducked into voiceover.mp3
        total_duration: data.get("duration").and_then(|d| d.as_f64()),
    })?;
    let missing: Vec<String> = regex::Regex::new(r#"src="assets/([^"]+)""#)?
        .captures_iter(&html)
        .map(|c| c[1].to_string())
        .filter(|n| !assets.join(n).is_file())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    if !missing.is_empty() {
        bail!("Template '{}' cần tệp assets chưa được cung cấp: {} (truyền qua data.assets)", req.template_id, missing.join(", "));
    }
    std::fs::write(bundle.join("index.html"), html)?;
    hyperframes::ensure_gsap(&assets)?;
    hyperframes::write_hf_json(bundle)?;
    Ok(())
}

/// Full pipeline for a request: every platform → mp4 + jpg + verify.
pub async fn render_request(req: &RenderRequest, ctx: &RenderCtx) -> Result<RenderResult> {
    if !super::util::is_valid_slug(&req.project_id) {
        bail!("project_id không hợp lệ");
    }
    if req.platforms.is_empty() {
        bail!("Cần ít nhất một platform");
    }
    let mut res = RenderResult::default();
    let voice: Option<PathBuf> = req.voice.as_ref().and_then(|v| v.audio_path.as_deref()).and_then(|p| resolve_media(&req.project_id, p));
    if voice.is_none() {
        res.warnings.push(match req.voice.as_ref().and_then(|v| v.audio_path.as_ref()) {
            Some(p) => format!("Không tìm thấy audio giọng: {p}. Video sẽ im lặng."),
            None => "Không có audio giọng (voice.audio_path): video im lặng. Giọng do Track C (voice) tạo, truyền qua voice.audio_path.".into(),
        });
    }
    let bgm = req.data.get("bgm_path").and_then(|b| b.as_str()).and_then(|b| resolve_media(&req.project_id, b));
    let words = resolve_words(req, voice.as_deref(), &mut res.warnings);
    let name = sanitize_name(&req.output_name);
    let n = req.platforms.len() as f64;
    let skip_verify = req.data.get("skip_verify").and_then(|v| v.as_bool()).unwrap_or(false);

    for (i, plat) in req.platforms.iter().enumerate() {
        if ctx.cancel.is_cancelled() {
            bail!("cancelled");
        }
        let preset = get_platform(plat)?;
        let base = i as f64 / n * 100.0;
        let span = 100.0 / n;
        (ctx.progress)(base + span * 0.05, &format!("{plat}: dựng bundle"));
        let rel = format!("output/{name}/{}.mp4", preset.id);
        let out = workspace::project_path(&req.project_id, &["output", &name, &format!("{}.mp4", preset.id)], true)?;
        let pg = ctx.progress.clone();
        let (b2, s2, pid) = (base, span, preset.id.clone());
        let pct: Arc<dyn Fn(f64) + Send + Sync> = Arc::new(move |p: f64| pg(b2 + s2 * (0.1 + 0.8 * p / 100.0), &format!("{pid}: render {p:.0}%")));
        let engine = render_one(req, &preset, voice.as_deref(), bgm.as_deref(), &words, &out, ctx, &mut res.warnings, pct).await?;
        (ctx.progress)(base + span * 0.92, &format!("{plat}: kiểm tra video"));
        let report = verify_video(
            &out,
            &VerifyOpts {
                expect_duration: voice.as_ref().map(|v| (media_duration(v, 0.0), 1.5)).filter(|(d, _)| *d > 0.0),
                expect_size: Some((preset.width & !1, preset.height & !1)),
                expect_audio: voice.is_some() && !req.data.get("allow_silent").and_then(|v| v.as_bool()).unwrap_or(false),
                target_lufs: preset.loudness_lufs,
                contact_sheet: Some(out.with_file_name(format!("{}_contact.jpg", preset.id))),
            },
        )
        .await;
        if !report.pass_all && !skip_verify {
            bail!("Kiểm tra video [{}] thất bại: {}", preset.id, report.errors.join("; "));
        }
        res.warnings.extend(report.warnings.iter().map(|w| format!("[{}] {w}", preset.id)));
        res.verify.insert(preset.id.clone(), report.to_json());
        res.engines.insert(preset.id.clone(), engine);
        res.outputs.insert(preset.id.clone(), rel);
    }
    (ctx.progress)(100.0, "hoàn tất");
    Ok(res)
}

/// Template preview: sample.data.json (+ packaged sample voice) → half-size preview.mp4 + preview.jpg in the template dir.
pub async fn render_template_preview(tid: &str, cancel: &CancellationToken) -> Result<(PathBuf, PathBuf, String)> {
    let meta = templates::get_template(tid)?;
    let tdir = templates::template_dir(tid)?;
    let sample = std::fs::read_to_string(tdir.join("sample.data.json")).map_err(|_| anyhow!("Không tìm thấy sample.data.json cho template '{tid}'"))?;
    let data = serde_json::from_str::<Value>(&sample)?.as_object().cloned().unwrap_or_default();
    let preset = get_platform(&meta.default_platform)?;
    let req = RenderRequest { project_id: workspace::active_project_id(), template_id: tid.into(), platforms: vec![preset.id.clone()], data, ..Default::default() };
    let work = std::env::temp_dir().join(format!("vicieo_preview_{tid}_{}", uuid::Uuid::new_v4().simple()));
    let bundle = work.join("bundle");
    std::fs::create_dir_all(bundle.join("assets"))?;
    let ctx = RenderCtx { cancel: cancel.clone(), progress: Arc::new(|_, _| {}), work_dir: work.clone() };
    let packaged_voice = tdir.join("assets/voiceover.mp3");
    let voice = packaged_voice.is_file().then_some(packaged_voice.as_path());
    let res: Result<(PathBuf, PathBuf, String)> = async {
        build_template_bundle(&req, &preset, voice, None, None, &bundle, &ctx).await?;
        let full = work.join("full.mp4");
        let mut o = RenderOpts::new(cancel.clone());
        o.log = Some(work.join("render.log"));
        let engine = hyperframes::render(&bundle, &full, &preset, &o).await?.engine;
        let (mp4, jpg) = (tdir.join("preview.mp4"), tdir.join("preview.jpg"));
        let tmp = tdir.join("preview.tmp.mp4");
        super::util::ffmpeg(&["-i".into(), full.to_string_lossy().into(), "-vf".into(), "scale=trunc(iw/4)*2:trunc(ih/4)*2".into(), "-c:v".into(), "libx264".into(), "-preset".into(), "veryfast".into(), "-c:a".into(), "aac".into(), "-movflags".into(), "+faststart".into(), tmp.to_string_lossy().into()], cancel, 300).await?;
        std::fs::rename(&tmp, &mp4)?;
        hyperframes::extract_thumbnail(&mp4, &jpg, 1.0);
        Ok((mp4, jpg, engine))
    }
    .await;
    let _ = std::fs::remove_dir_all(&work);
    res
}
