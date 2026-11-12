//! Douyin/TikTok localization pipeline: ASR -> translate -> rewrite -> voice -> render request.
//! Voice and render are requested from this same engine (Track C / B routes); failures are recorded, not fatal.
use super::{asr::run_asr, rewrite::run_rewrite, service::get_source, translate::run_translation};
use crate::intelligence::util::*;
use serde_json::{json, Value};

fn base() -> String { format!("http://127.0.0.1:{}", std::env::var("VICIEO_PORT").ok().and_then(|p| p.parse::<u16>().ok()).unwrap_or(9000)) }

pub async fn self_post(path: &str, body: &Value) -> Option<Value> {
    let c = reqwest::Client::builder().timeout(std::time::Duration::from_secs(120)).build().ok()?;
    let r = c.post(format!("{}{path}", base())).json(body).send().await.ok()?;
    if !r.status().is_success() { return None; }
    r.json().await.ok()
}
pub async fn self_get(path: &str) -> Option<Value> {
    let r = reqwest::get(format!("{}{path}", base())).await.ok()?;
    if !r.status().is_success() { return None; }
    r.json().await.ok()
}

pub struct LocalizeReq { pub mode: String, pub style: String, pub voice_id: String, pub platforms: Vec<String>, pub template_id: Option<String> }

pub async fn run_localize_pipeline(pid: &str, source_id: &str, r: LocalizeReq) -> ApiResult<Value> {
    let source = get_source(pid, source_id).ok_or_else(|| ApiError::nf(format!("Source item not found: {source_id}")))?;
    if !["R", "L"].contains(&r.mode.as_str()) || !["faithful", "hook_boost", "character"].contains(&r.style.as_str()) { return Err(ApiError::unproc("invalid mode/style")); }
    if r.mode == "L" && source.rights == "reference_only" {
        return Err(ApiError::bad("Chế độ Localize (Mode L) giữ lại video gốc, yêu cầu rights khác 'reference_only'. Nguồn này đang là 'reference_only'. Vui lòng nâng cấp bản quyền hoặc chọn Mode R (Remake) để tạo video mới hoàn toàn an toàn."));
    }
    let dir = pp(pid, &["raw", source_id, "x"], false).parent().unwrap().to_path_buf();
    if !dir.join("transcript.orig.json").exists() { run_asr(pid, source_id, "zh", None).await?; }
    if !dir.join("transcript.vi.json").exists() { run_translation(pid, source_id).await?; }
    let lines = run_rewrite(pid, source_id, &r.style)?;

    let script = lines.iter().map(|l| s(l, "text")).collect::<Vec<_>>().join("\n");
    let voice = self_post(&format!("/projects/{pid}/voice/say"), &json!({"text": script, "voiceId": r.voice_id, "voice_id": r.voice_id, "sourceId": source_id})).await;
    let voice_path = format!("raw/{source_id}/voice.vi.mp3");
    let tmpl = r.template_id.clone().unwrap_or_else(|| if r.mode == "L" { "localized-reup-v1".into() } else { "story-cartoon-v1".into() });
    let title = if source.title.is_empty() { format!("Video {source_id}") } else { source.title.clone() };
    let req = json!({
        "project_id": pid, "template_id": tmpl, "platforms": r.platforms,
        "data": {"mode": r.mode, "source_id": source_id, "title": title, "source_author": if source.author.is_empty() { "@source".to_string() } else { format!("@{}", source.author) },
                 "character_reaction": "Sóc Nhỏ cùng các bạn theo dõi nhé!", "story_title": title,
                 "video_path": if r.mode == "L" { json!(format!("raw/{source_id}/video.mp4")) } else { Value::Null },
                 "voice_path": voice_path, "script_path": format!("content/scripts/{source_id}.{}.json", r.style), "style": r.style},
        "voice": {"voice_id": r.voice_id, "script": null, "audio_path": voice_path},
        "source_ref": source_id, "output_name": format!("loc_{source_id}_{}", r.style),
    });
    let job = self_post("/video/render", &req).await;
    let job_id = job.as_ref().and_then(|j| j.get("job_id").or_else(|| j.get("jobId"))).cloned().unwrap_or(Value::Null);
    let summary = json!({
        "source_id": source_id, "project_id": pid, "mode": r.mode, "style": r.style, "voice_id": r.voice_id, "platforms": r.platforms,
        "template_id": tmpl, "render_job_id": job_id, "voice_path": voice_path,
        "voice_ok": voice.is_some(), "status": job.as_ref().and_then(|j| j.get("status")).cloned().unwrap_or(json!("render_unavailable")),
    });
    write_json(&dir.join("localize_job.json"), &summary)?;
    Ok(summary)
}
