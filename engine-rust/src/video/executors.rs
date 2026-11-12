//! NodeExecutor for the video nodes: hyperframes_render, captions, word_captions, export, scene_cut, handdraw.
//! Upstream outputs (script/image/voice nodes) arrive in `ctx.inputs`; they are scanned tolerantly for media
//! URLs/paths, `words`, `scenes` and text, so this works with whatever those executors emit.
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

use super::platforms::get_platform;
use super::render::{render_request, RenderCtx, RenderRequest, VoiceSpec};
use super::util::{self, ffmpeg, media_duration, project_file_url, resolve_media};
use super::verify::{verify_video, VerifyOpts};
use crate::caption::{export_srt, export_vtt, parse_words, segments_from_words, speech_span, text_on_timings};
use crate::engine::executor::{NodeCtx, NodeExecutor, NodeOutput};
use crate::workspace;

pub struct VideoExecutor;

#[derive(Default)]
struct Found {
    audio: Vec<String>,
    images: Vec<String>,
    videos: Vec<String>,
    words: Vec<Value>,
    scenes: Vec<Value>,
    texts: Vec<String>,
}

fn visit(v: &Value, f: &mut Found, key: &str) {
    match v {
        Value::String(s) => {
            let l = s.to_lowercase();
            let l = l.split('?').next().unwrap_or("");
            if [".mp3", ".wav", ".m4a", ".aac", ".ogg"].iter().any(|e| l.ends_with(e)) || l.contains("/voices/") && l.ends_with("/sample") {
                f.audio.push(s.clone());
            } else if [".png", ".jpg", ".jpeg", ".webp"].iter().any(|e| l.ends_with(e)) {
                f.images.push(s.clone());
            } else if [".mp4", ".mov", ".webm"].iter().any(|e| l.ends_with(e)) {
                f.videos.push(s.clone());
            } else if ["text", "script", "dialogue", "voice_text", "voiceText", "summary"].contains(&key) && !s.trim().is_empty() {
                f.texts.push(s.clone());
            }
        }
        Value::Array(a) => {
            if (key == "words" || key == "word_timestamps") && !parse_words(v).is_empty() {
                f.words.extend(a.iter().cloned());
            } else if key == "scenes" && a.iter().all(|x| x.is_object()) {
                f.scenes.extend(a.iter().cloned());
            } else {
                a.iter().for_each(|x| visit(x, f, key));
            }
        }
        Value::Object(o) => o.iter().for_each(|(k, x)| visit(x, f, k)),
        _ => {}
    }
}

fn gather(ctx: &NodeCtx) -> Found {
    let mut f = Found::default();
    let mut keys: Vec<_> = ctx.inputs.keys().collect();
    keys.sort();
    for k in keys {
        visit(&ctx.inputs[k], &mut f, "");
    }
    f
}

fn p_str<'a>(ctx: &'a NodeCtx, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|k| ctx.params.get(k).and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty()))
}

fn out_name(ctx: &NodeCtx) -> String {
    let base = p_str(ctx, &["outputName", "output_name"]).map(String::from).unwrap_or_else(|| format!("{}_{}", ctx.node_id, ctx.run_id.chars().take(8).collect::<String>()));
    base.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
}

fn platforms(ctx: &NodeCtx) -> Vec<String> {
    if let Some(a) = ctx.params.get("platforms").and_then(|p| p.as_array()) {
        let v: Vec<String> = a.iter().filter_map(|x| x.as_str().map(String::from)).collect();
        if !v.is_empty() {
            return v;
        }
    }
    if let Some(p) = p_str(ctx, &["platform"]) {
        return vec![p.to_string()];
    }
    let asp = p_str(ctx, &["aspect", "aspectRatio"]).unwrap_or("9:16");
    vec![if asp.starts_with("16:9") { "youtube" } else if asp.starts_with("4:5") { "facebook" } else { "tiktok" }.to_string()]
}

fn first_media(ctx: &NodeCtx, keys: &[&str], pool: &[String]) -> Option<PathBuf> {
    p_str(ctx, keys).into_iter().chain(pool.iter().map(|s| s.as_str())).find_map(|r| resolve_media(&ctx.project_id, r))
}

#[async_trait]
impl NodeExecutor for VideoExecutor {
    fn node_types(&self) -> &'static [&'static str] {
        &["hyperframes_render", "captions", "word_captions", "export", "scene_cut", "handdraw"]
    }

    async fn execute(&self, ctx: &NodeCtx) -> Result<NodeOutput> {
        if !util::is_valid_slug(&ctx.project_id) {
            bail!("project_id không hợp lệ");
        }
        match ctx.node_type.as_str() {
            "hyperframes_render" => render_node(ctx).await,
            "captions" | "word_captions" => captions_node(ctx).await,
            "export" => export_node(ctx).await,
            "scene_cut" => scene_cut_node(ctx).await,
            "handdraw" => handdraw_node(ctx).await,
            t => Err(anyhow!("node type không thuộc video: {t}")),
        }
    }
}

fn video_out(ctx: &NodeCtx, rels: &[String], summary: String, outputs: Value) -> NodeOutput {
    let first = rels.first().cloned();
    NodeOutput {
        preview_type: "video".into(),
        preview_url: first.as_ref().map(|r| project_file_url(&ctx.project_id, r)),
        summary,
        cost_usd: 0.0,
        outputs,
        artifacts: rels.iter().filter_map(|r| workspace::project_path(&ctx.project_id, &[r], false).ok()).collect(),
    }
}

async fn render_node(ctx: &NodeCtx) -> Result<NodeOutput> {
    let f = gather(ctx);
    let pid = &ctx.project_id;
    let mut data = ctx.params.get("data").and_then(|d| d.as_object()).cloned().unwrap_or_default();
    if !data.contains_key("scenes") {
        let scenes: Vec<Value> = if let Some(s) = ctx.params.get("scenes").and_then(|s| s.as_array()).filter(|s| !s.is_empty()) {
            s.clone()
        } else if !f.scenes.is_empty() {
            // upstream script scenes + images paired by index
            f.scenes.iter().enumerate().map(|(i, s)| {
                let mut s = s.clone();
                if let Some(o) = s.as_object_mut() {
                    if !["image", "img", "video", "src"].iter().any(|k| o.contains_key(*k)) {
                        if let Some(m) = f.videos.get(i).or_else(|| f.images.get(i)) {
                            o.insert("image".into(), json!(m));
                        }
                    }
                }
                s
            }).collect()
        } else {
            let media: Vec<&String> = f.videos.iter().chain(f.images.iter()).collect();
            let per = ctx.params.get("sceneDurationSec").and_then(|d| d.as_f64()).unwrap_or(4.0);
            media.iter().map(|m| json!({"image": m, "durationSec": per})).collect()
        };
        if scenes.is_empty() && p_str(ctx, &["template_id", "templateId"]).is_none() {
            bail!("Không có cảnh/ảnh/clip đầu vào để dựng video (nối node image_gen/video_gen/script vào hyperframes_render).");
        }
        if !scenes.is_empty() {
            data.insert("scenes".into(), json!(scenes));
        }
    }
    let audio = p_str(ctx, &["voice", "audio", "audioPath", "voiceUrl"]).map(String::from).or_else(|| f.audio.first().cloned());
    if !data.contains_key("words") && !f.words.is_empty() {
        data.insert("words".into(), json!(f.words));
    }
    for (k, pk) in [("bgm_path", "bgm"), ("titleBadge", "titleBadge"), ("quality", "quality"), ("require_hyperframes", "requireHyperFrames")] {
        if let Some(v) = ctx.params.get(pk).filter(|v| !v.is_null()) {
            data.entry(k).or_insert(v.clone());
        }
    }
    let req = RenderRequest {
        project_id: pid.clone(),
        template_id: p_str(ctx, &["template_id", "templateId"]).unwrap_or("").to_string(),
        platforms: platforms(ctx),
        data,
        voice: Some(VoiceSpec { audio_path: audio, script: f.texts.first().cloned().filter(|_| ctx.params.get("karaoke").and_then(|k| k.as_bool()).unwrap_or(true)), voice_id: None }),
        source_ref: None,
        output_name: out_name(ctx),
    };
    let work = workspace::project_path(pid, &["runs", &ctx.run_id, &format!("hf_{}", ctx.node_id)], true)?;
    std::fs::create_dir_all(&work)?;
    let rc = RenderCtx { cancel: ctx.cancel.clone(), progress: Arc::new(|_, _| {}), work_dir: work };
    let r = render_request(&req, &rc).await?;
    let mut rels: Vec<String> = req.platforms.iter().filter_map(|p| r.outputs.get(p).cloned()).collect();
    rels.sort_by_key(|r| if r.contains("/youtube") || r.contains("/tiktok") { 0 } else { 1 });
    let engine = r.engines.values().next().cloned().unwrap_or_default();
    Ok(video_out(
        ctx,
        &rels,
        format!("Đã dựng {} video ({}) bằng {}", rels.len(), req.platforms.join(", "), engine),
        json!({"videoUrl": rels.first().map(|r| project_file_url(pid, r)), "outputs": r.outputs, "engines": r.engines, "verify": r.verify, "warnings": r.warnings}),
    ))
}

async fn captions_node(ctx: &NodeCtx) -> Result<NodeOutput> {
    let f = gather(ctx);
    let audio = first_media(ctx, &["audio", "voice", "audioPath"], &f.audio);
    let mut words = if !f.words.is_empty() { parse_words(&Value::Array(f.words.clone())) } else { vec![] };
    let mut notes = vec![];
    if words.is_empty() {
        let text = p_str(ctx, &["text", "script"]).map(String::from).or_else(|| (!f.texts.is_empty()).then(|| f.texts.join(" "))).unwrap_or_default();
        if text.trim().is_empty() {
            bail!("Thiếu lời thoại (text) hoặc mốc từ (words) từ node giọng.");
        }
        let (t0, t1) = match &audio {
            Some(a) => {
                let total = media_duration(a, 0.0);
                speech_span(a, total)
            }
            None => (0.0, text.split_whitespace().count() as f64 / 2.8),
        };
        words = text_on_timings(&text, t0, t1);
        notes.push("Mốc từ ước lượng theo độ dài chữ trong khoảng giọng nói thực (chưa có forced-alignment từ node voice).");
    }
    if words.is_empty() {
        bail!("Không tạo được mốc từ");
    }
    let per = ctx.params.get("wordsPerChunk").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
    let segs = segments_from_words(&words, per);
    let name = out_name(ctx);
    let dir = workspace::project_path(&ctx.project_id, &["output", "captions", &name], true)?;
    std::fs::create_dir_all(&dir)?;
    export_srt(&segs, &dir.join("captions.srt"))?;
    export_vtt(&segs, &dir.join("captions.vtt"))?;
    std::fs::write(dir.join("words.json"), serde_json::to_string_pretty(&words)?)?;
    let u = |f: &str| project_file_url(&ctx.project_id, &format!("output/captions/{name}/{f}"));
    Ok(NodeOutput {
        preview_type: "text".into(),
        preview_url: None,
        summary: format!("Phụ đề karaoke: {} từ, {} cụm", words.len(), segs.len()),
        cost_usd: 0.0,
        outputs: json!({"words": words, "chunks": segs.len(), "srtUrl": u("captions.srt"), "vttUrl": u("captions.vtt"), "wordsUrl": u("words.json"), "notes": notes}),
        artifacts: vec![dir.join("captions.srt"), dir.join("captions.vtt"), dir.join("words.json")],
    })
}

/// Fit filter for re-framing (port of platforms.fit_strategy).
pub fn fit_filter(strategy: &str, w: u32, h: u32) -> String {
    match strategy {
        "contain" => format!("scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2:color=black,setsar=1"),
        "blur-pad" => format!("split[a][b];[a]scale={w}:{h}:force_original_aspect_ratio=increase,crop={w}:{h},boxblur=20:5[bg];[b]scale={w}:{h}:force_original_aspect_ratio=decrease[fg];[bg][fg]overlay=(W-w)/2:(H-h)/2,setsar=1"),
        _ => format!("scale={w}:{h}:force_original_aspect_ratio=increase,crop={w}:{h},setsar=1"),
    }
}

async fn export_node(ctx: &NodeCtx) -> Result<NodeOutput> {
    let f = gather(ctx);
    let src = first_media(ctx, &["video", "source"], &f.videos).ok_or_else(|| anyhow!("export cần một video đầu vào (nối node hyperframes_render/video_gen)"))?;
    let strategy = p_str(ctx, &["fit"]).unwrap_or("blur-pad").to_string();
    let name = out_name(ctx);
    let (mut rels, mut verify) = (vec![], serde_json::Map::new());
    for plat in platforms(ctx) {
        let pr = get_platform(&plat)?;
        let rel = format!("output/{name}/{}.mp4", pr.id);
        let out = workspace::project_path(&ctx.project_id, &["output", &name, &format!("{}.mp4", pr.id)], true)?;
        let (w, h) = (pr.width & !1, pr.height & !1);
        let vf = fit_filter(&strategy, w, h);
        let graph = format!("[0:v]{vf},fps={},format=yuv420p[v]", pr.fps);
        let has_audio = util::probe(&src).await.map(|p| p.has_audio).unwrap_or(false);
        let mut a: Vec<String> = vec!["-i".into(), src.to_string_lossy().into(), "-filter_complex".into(), graph, "-map".into(), "[v]".into()];
        if has_audio {
            a.extend(["-map".into(), "0:a:0".into(), "-af".into(), format!("loudnorm=I={}:LRA=11:TP=-1.5", pr.loudness_lufs), "-c:a".into(), pr.audio_codec.clone(), "-b:a".into(), pr.audio_bitrate.clone()]);
        }
        if let Some(m) = pr.max_duration_sec {
            a.extend(["-t".into(), m.to_string()]);
        }
        a.extend(["-c:v".into(), pr.codec.clone(), "-b:v".into(), pr.video_bitrate.clone(), "-preset".into(), "veryfast".into(), "-movflags".into(), "+faststart".into(), out.to_string_lossy().into()]);
        ffmpeg(&a, &ctx.cancel, 1800).await?;
        super::hyperframes::extract_thumbnail(&out, &out.with_extension("jpg"), 1.0);
        let rep = verify_video(&out, &VerifyOpts { expect_size: Some((w, h)), expect_audio: has_audio, target_lufs: pr.loudness_lufs, ..Default::default() }).await;
        if !rep.pass_all {
            bail!("Kiểm tra video [{}] thất bại: {}", pr.id, rep.errors.join("; "));
        }
        verify.insert(pr.id.clone(), rep.to_json());
        rels.push(rel);
    }
    Ok(video_out(ctx, &rels.clone(), format!("Đã xuất {} bản ({})", rels.len(), platforms(ctx).join(", ")), json!({"videoUrl": rels.first().map(|r| project_file_url(&ctx.project_id, r)), "outputs": rels, "verify": verify})))
}

async fn scene_cut_node(ctx: &NodeCtx) -> Result<NodeOutput> {
    let f = gather(ctx);
    let src = first_media(ctx, &["video", "source"], &f.videos).ok_or_else(|| anyhow!("scene_cut cần video nguồn"))?;
    let total = media_duration(&src, 0.0);
    if total <= 0.0 {
        bail!("Không đọc được thời lượng video nguồn");
    }
    let thr = ctx.params.get("threshold").and_then(|v| v.as_f64()).unwrap_or(0.3);
    let min_len = ctx.params.get("minSceneSec").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let seg = ctx.params.get("segmentSec").and_then(|v| v.as_f64()).unwrap_or(5.0).max(0.5);
    let mut c = tokio::process::Command::new(util::ffmpeg_bin());
    c.args(["-hide_banner", "-nostats", "-i"]).arg(&src).args(["-an", "-vf", &format!("select='gt(scene,{thr})',showinfo"), "-f", "null", "-"]);
    let o = util::run(&mut c, &ctx.cancel, std::time::Duration::from_secs(900), None).await?;
    let mut cuts: Vec<f64> = o.stderr.lines().filter_map(|l| l.split("pts_time:").nth(1)?.split_whitespace().next()?.parse().ok()).collect();
    let method = if cuts.is_empty() {
        cuts = (1..).map(|i| i as f64 * seg).take_while(|t| *t < total - 0.5).collect();
        "fixed_segments"
    } else {
        "scene_detect"
    };
    let mut bounds = vec![0.0];
    for t in cuts {
        if t - bounds.last().unwrap() >= min_len && total - t >= min_len {
            bounds.push(t);
        }
    }
    bounds.push(total);
    let name = out_name(ctx);
    let mut clips = vec![];
    let mut rels = vec![];
    for (i, w) in bounds.windows(2).enumerate() {
        let rel = format!("output/scene_cut/{name}/clip_{:02}.mp4", i + 1);
        let out = workspace::project_path(&ctx.project_id, &["output", "scene_cut", &name, &format!("clip_{:02}.mp4", i + 1)], true)?;
        ffmpeg(&["-ss".into(), format!("{:.3}", w[0]), "-i".into(), src.to_string_lossy().into(), "-t".into(), format!("{:.3}", w[1] - w[0]), "-c:v".into(), "libx264".into(), "-preset".into(), "veryfast".into(), "-c:a".into(), "aac".into(), "-movflags".into(), "+faststart".into(), out.to_string_lossy().into()], &ctx.cancel, 600).await?;
        clips.push(json!({"index": i + 1, "start": w[0], "end": w[1], "duration": w[1] - w[0], "url": project_file_url(&ctx.project_id, &rel)}));
        rels.push(rel);
    }
    Ok(video_out(ctx, &rels, format!("Cắt {} cảnh ({method})", clips.len()), json!({"clips": clips, "method": method, "videoUrl": clips[0]["url"]})))
}

async fn handdraw_node(ctx: &NodeCtx) -> Result<NodeOutput> {
    let f = gather(ctx);
    let img = first_media(ctx, &["image", "src"], &f.images).ok_or_else(|| anyhow!("handdraw cần ảnh đầu vào (nối node image_gen/image)"))?;
    let d = ctx.params.get("durationSec").and_then(|v| v.as_f64()).unwrap_or(6.0).clamp(2.0, 60.0);
    let asp = platforms(ctx);
    let pr = get_platform(&asp[0])?;
    let (w, h, h2) = (pr.width & !1, pr.height & !1, d / 2.0);
    let name = out_name(ctx);
    let rel = format!("output/handdraw/{name}.mp4");
    let out = workspace::project_path(&ctx.project_id, &["output", "handdraw", &format!("{name}.mp4")], true)?;
    // stage 1: white page → pencil line-art wiped in left→right; stage 2: colour wiped over the line-art
    let g = format!(
        "[0:v]scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2:color=white,setsar=1,fps=30,format=yuv420p,split=2[c1][c2];\
         [c1]edgedetect=low=0.08:high=0.25,negate,format=yuv420p[line];\
         color=c=white:s={w}x{h}:r=30:d={d},format=yuv420p[white];\
         [line][white]blend=all_expr='if(lte(X,W*min(T/{h2},1)),A,B)'[s1];\
         [c2][s1]blend=all_expr='if(lte(X,W*max(0,min((T-{h2})/{h2},1))),A,B)'[v]"
    );
    ffmpeg(&["-loop".into(), "1".into(), "-framerate".into(), "30".into(), "-t".into(), format!("{d}"), "-i".into(), img.to_string_lossy().into(), "-filter_complex".into(), g, "-map".into(), "[v]".into(), "-t".into(), format!("{d}"), "-c:v".into(), "libx264".into(), "-pix_fmt".into(), "yuv420p".into(), "-movflags".into(), "+faststart".into(), out.to_string_lossy().into()], &ctx.cancel, 600).await?;
    super::hyperframes::extract_thumbnail(&out, &out.with_extension("jpg"), d * 0.9);
    Ok(video_out(ctx, std::slice::from_ref(&rel), format!("Video vẽ tay {d:.0}s từ ảnh (procedural: nét bút rồi tô màu)"), json!({"videoUrl": project_file_url(&ctx.project_id, &rel), "durationSec": d})))
}
