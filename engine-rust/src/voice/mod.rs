//! Voice track: Rust owns process/timeout/cache; TTS + clone run in the Python sidecar
//! (`sidecar/voice/voice_sidecar.py`, root `.venv`), which wraps the existing OmniVoice code.
use crate::engine::executor::{NodeCtx, NodeExecutor, NodeOutput};
use crate::workspace;
use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::Semaphore;

pub const MAX_CHARS: usize = 400;
/// OmniVoice is CPU/GPU heavy: one synthesis at a time (also makes cache dedup race-free).
static TTS_LOCK: Lazy<Semaphore> = Lazy::new(|| Semaphore::new(1));

pub fn vault_dir() -> PathBuf {
    workspace::data_root().join("shared").join("voice_vault")
}

fn python() -> PathBuf {
    std::env::var("VOICE_PYTHON").map(PathBuf::from).unwrap_or_else(|_| workspace::repo_root().join(".venv/bin/python"))
}

fn sidecar_script() -> PathBuf {
    std::env::var("VOICE_SIDECAR").map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sidecar/voice/voice_sidecar.py"))
}

/// Spawn the sidecar, feed JSON on stdin, parse the last stdout line. Killed on timeout/drop.
pub async fn run_sidecar(cmd: &str, req: Value, timeout: Duration) -> Result<Value> {
    let mut child = Command::new(python())
        .arg(sidecar_script()).arg(cmd)
        .env("PYTHONUTF8", "1")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn().context("không khởi động được sidecar voice (python/.venv?)")?;
    child.stdin.take().unwrap().write_all(req.to_string().as_bytes()).await?;
    let out = tokio::time::timeout(timeout, child.wait_with_output()).await
        .map_err(|_| anyhow!("sidecar voice quá thời gian chờ ({}s)", timeout.as_secs()))??;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let last = stdout.lines().last().unwrap_or("");
    let v: Value = serde_json::from_str(last).map_err(|_| {
        let err = String::from_utf8_lossy(&out.stderr);
        anyhow!("sidecar voice không trả JSON (mã {:?}): {}", out.status.code(), err.chars().rev().take(300).collect::<String>().chars().rev().collect::<String>())
    })?;
    if v["ok"] != json!(true) {
        bail!("{}", v["error"].as_str().unwrap_or("sidecar lỗi"));
    }
    Ok(v)
}

#[derive(Debug, Clone)]
pub struct Say {
    pub path: PathBuf,
    pub duration_sec: f64,
    pub words: Value,
    pub cached: bool,
}

pub fn cache_key(text: &str, voice: &str, tempo: f64) -> String {
    // "v1" = mastering/engine revision; bump to invalidate old cache.
    let raw = json!({"v": voice, "t": text, "tempo": (tempo * 1000.0).round() / 1000.0, "rev": "v1"}).to_string();
    Sha256::digest(raw.as_bytes()).iter().take(12).map(|b| format!("{b:02x}")).collect()
}

/// Synthesize `text` with `voice` into `dir/<hash>.mp3` (+ `<hash>.json` meta). Cache hit skips the sidecar.
pub async fn synthesize(text: &str, voice: &str, tempo: f64, dir: &Path) -> Result<Say> {
    let text = text.trim();
    if text.is_empty() { bail!("Thiếu nội dung cần đọc"); }
    if text.chars().count() > MAX_CHARS { bail!("Câu quá dài (tối đa {MAX_CHARS} ký tự)"); }
    if !crate::routes::is_valid_slug(voice) || !vault_dir().join(voice).is_dir() {
        bail!("Giọng '{voice}' không có trong voice_vault");
    }
    let key = cache_key(text, voice, tempo);
    let (mp3, meta) = (dir.join(format!("{key}.mp3")), dir.join(format!("{key}.json")));
    let hit = || -> Option<Say> {
        let m: Value = serde_json::from_str(&std::fs::read_to_string(&meta).ok()?).ok()?;
        mp3.is_file().then(|| Say { path: mp3.clone(), duration_sec: m["durationSec"].as_f64().unwrap_or(0.0), words: m["words"].clone(), cached: true })
    };
    if let Some(s) = hit() { return Ok(s); }
    let _g = TTS_LOCK.acquire().await?;
    if let Some(s) = hit() { return Ok(s); }
    std::fs::create_dir_all(dir)?;
    let timeout = Duration::from_secs((12 * text.chars().count() as u64).clamp(600, 3600));
    let r = run_sidecar("say", json!({
        "vault": vault_dir(), "voiceId": voice, "text": text, "tempo": tempo, "out": mp3,
    }), timeout).await.map_err(|e| anyhow!("Không đọc được bằng giọng '{voice}': {e}"))?;
    let duration_sec = r["durationSec"].as_f64().unwrap_or(0.0);
    std::fs::write(&meta, serde_json::to_vec(&json!({"text": text, "voiceId": voice, "tempo": tempo, "durationSec": duration_sec, "words": r["words"]}))?)?;
    Ok(Say { path: mp3, duration_sec, words: r["words"].clone(), cached: false })
}

// ---------------------------------------------------------------- voice vault

fn read_json(p: &Path) -> Value {
    std::fs::read_to_string(p).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null)
}

/// One vault entry in the same shape as Python `voice_routes._view`.
pub fn voice_view(d: &Path) -> Value {
    let (p, v) = (read_json(&d.join("profile.json")), read_json(&d.join("voice.json")));
    let id = d.file_name().unwrap().to_string_lossy().to_string();
    let pick = |k: &str, vk: &str| p.get(k).or_else(|| v.get(vk)).cloned().unwrap_or(json!(""));
    let sample = p["files"]["cloned_sample"].as_str().unwrap_or("cloned_default_sample.mp3");
    let has_sample = d.join(sample).exists();
    let status = v["status"].as_str().or(p["status"].as_str()).unwrap_or("ready");
    let ready = (d.join("omnivoice_prompt.pt").exists() || d.join("reference_clean_slice.wav").exists()) && status != "todo_clone";
    json!({
        "id": id, "name": p.get("speaker_name").or_else(|| v.get("display_name")).cloned().unwrap_or(json!(id)),
        "dialect": pick("dialect", "dialect"), "engine": p.get("engine").or_else(|| v.get("engine")).cloned().unwrap_or(json!("omnivoice")),
        "durationSec": p.get("duration"), "transcript": pick("transcript", "transcript"), "createdAt": pick("created_at", "created_at"),
        "hasSample": has_sample, "sampleUrl": has_sample.then(|| format!("/voices/{id}/sample")),
        "aliases": v.get("aliases").cloned().unwrap_or(json!([])),
        "ready": ready, "status": if ready { status } else { "todo_clone" },
        "statusNote": if ready { "Sẵn sàng" } else { "Chưa nạp mẫu giọng clone" },
    })
}

pub fn list_voices() -> Vec<Value> {
    let Ok(rd) = std::fs::read_dir(vault_dir()) else { return vec![] };
    let mut dirs: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| {
        let n = p.file_name().unwrap().to_string_lossy();
        p.is_dir() && !n.starts_with('.') && !n.starts_with('_') && (p.join("profile.json").exists() || p.join("voice.json").exists())
    }).collect();
    dirs.sort();
    dirs.iter().map(|d| voice_view(d)).collect()
}

pub fn slugify(s: &str) -> String {
    let re = regex::Regex::new(r"[^a-z0-9_-]+").unwrap();
    re.replace_all(&s.to_lowercase(), "-").trim_matches('-').to_string()
}

/// Port of `character_profile.resolve_voice_id` (simplified): declared id, id/name variants, then vault aliases.
/// Err = readable message (no silent fallback to another voice).
pub fn resolve_voice_id(declared: Option<&str>, char_id: &str, aliases: &[String]) -> Result<String> {
    let voices = list_voices();
    let mut cands: Vec<String> = vec![];
    for x in [declared.unwrap_or(""), char_id].iter().filter(|x| !x.is_empty()) {
        let base = regex::Regex::new(r"[_-]v\d+$").unwrap().replace(x, "").to_string();
        for v in [x.to_string(), base] {
            for w in [v.clone(), v.replace('_', "-"), v.replace('-', "_")] {
                if !cands.contains(&w) { cands.push(w); }
            }
        }
    }
    let by_id = |id: &str| voices.iter().find(|v| v["id"] == json!(id));
    for c in &cands {
        if let Some(v) = by_id(c) {
            return if v["ready"] == json!(true) { Ok(c.clone()) } else { Err(anyhow!("giọng '{c}' chưa được nạp mẫu clone (todo_clone)")) };
        }
    }
    let keys: Vec<String> = cands.iter().cloned().chain(aliases.iter().map(|a| slugify(a))).collect();
    for v in &voices {
        let va: Vec<String> = v["aliases"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(slugify)).collect()).unwrap_or_default();
        if v["ready"] == json!(true) && keys.iter().any(|k| va.contains(k)) {
            return Ok(v["id"].as_str().unwrap().to_string());
        }
    }
    Err(anyhow!("nhân vật '{char_id}' chưa có giọng dùng được trong kho giọng (khai báo: {})", declared.unwrap_or("không có")))
}

/// Characters of a project → (voice_id declared, aliases). Reads project.json `characters`.
fn project_characters(project: &str) -> HashMap<String, (Option<String>, Vec<String>)> {
    let p = read_json(&workspace::data_root().join("projects").join(project).join("project.json"));
    p["characters"].as_object().map(|m| m.iter().map(|(id, c)| {
        let mut al: Vec<String> = c["aliases"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        if let Some(n) = c["name"].as_str() { al.push(n.to_string()); }
        (id.clone(), (c["voice_id"].as_str().or(c["voiceId"].as_str()).map(String::from), al))
    }).collect()).unwrap_or_default()
}

/// Dialogue: lines `[{characterId|speaker, text, id?}]` → one clip per line with the character's own voice.
/// Any unresolved character fails the whole call up front (before any audio is made).
pub async fn synthesize_dialogue(project: &str, lines: &[Value], tempo: f64, dir: &Path, url_prefix: &str) -> Result<Vec<Value>> {
    let chars = project_characters(project);
    let mut plan = vec![];
    let mut errors = vec![];
    for (i, l) in lines.iter().enumerate() {
        let text = l["text"].as_str().unwrap_or("").trim();
        if text.is_empty() { continue; }
        let who = l["characterId"].as_str().or(l["speaker"].as_str()).unwrap_or("");
        let key = slugify(&who.replace('_', "-"));
        let entry = chars.iter().find(|(id, (_, al))| id.as_str() == who || slugify(&id.replace('_', "-")) == key || al.iter().any(|a| slugify(a) == key));
        let (cid, declared, aliases) = match entry { Some((id, (d, a))) => (id.clone(), d.clone(), a.clone()), None => (who.to_string(), None, vec![who.to_string()]) };
        match resolve_voice_id(declared.as_deref(), &cid, &aliases) {
            Ok(v) => plan.push((i, cid, v, text.to_string())),
            Err(e) => errors.push(format!("dòng {} ({who}): {e}", i + 1)),
        }
    }
    if !errors.is_empty() { bail!("{}", errors.join("; ")); }
    let mut clips = vec![];
    for (i, cid, voice, text) in plan {
        let s = synthesize(&text, &voice, tempo, dir).await?;
        let name = s.path.file_name().unwrap().to_string_lossy();
        clips.push(json!({
            "lineId": lines[i]["id"], "order": lines[i]["order"].as_i64().unwrap_or(i as i64), "characterId": cid, "voiceId": voice,
            "text": text, "audioUrl": format!("{url_prefix}/{name}"), "durationSec": s.duration_sec, "words": s.words, "cached": s.cached,
        }));
    }
    Ok(clips)
}

// ---------------------------------------------------------------- clone

/// Clone a voice into the vault. `source` is a local audio/video file path.
pub async fn clone_voice(voice_id: &str, source: &Path, dialect: &str, name: Option<&str>, transcript: Option<&str>) -> Result<Value> {
    let r = run_sidecar("clone", json!({
        "vault": vault_dir(), "voiceId": voice_id, "source": source, "dialect": dialect, "name": name, "transcript": transcript,
    }), Duration::from_secs(1800)).await.map_err(|e| anyhow!("Lỗi nhân bản giọng OmniVoice: {e}"))?;
    Ok(json!({
        "status": "cloned", "voiceId": voice_id, "name": r["name"], "dialect": dialect, "durationSec": r["durationSec"],
        "transcript": r["transcript"], "sampleUrl": format!("/voices/{voice_id}/sample"), "vaultPath": format!("data/shared/voice_vault/{voice_id}/"),
    }))
}

// ---------------------------------------------------------------- node executor

pub struct VoiceExecutor;

#[async_trait]
impl NodeExecutor for VoiceExecutor {
    fn node_types(&self) -> &'static [&'static str] { &["voice_tts", "voice_clone"] }

    async fn execute(&self, ctx: &NodeCtx) -> Result<NodeOutput> {
        let p = |k: &str| ctx.params.get(k).and_then(|v| v.as_str()).map(str::trim).filter(|s| !s.is_empty()).map(String::from);
        let up = |k: &str| ctx.inputs.values().find_map(|o| o.get(k).cloned().or_else(|| o.get("outputs").and_then(|x| x.get(k)).cloned()));
        if ctx.node_type == "voice_clone" {
            let id = slugify(&p("voiceId").ok_or_else(|| anyhow!("voice_clone: thiếu voiceId"))?);
            if ctx.params["consentConfirmed"] != json!(true) { bail!("voice_clone: phải xác nhận quyền sử dụng giọng (consentConfirmed)"); }
            let src = p("sourceFile").ok_or_else(|| anyhow!("voice_clone: thiếu sourceFile"))?;
            let src = workspace::project_path(&ctx.project_id, &[], false)?.join(src.trim_start_matches('/'));
            if !src.is_file() { bail!("voice_clone: không thấy tệp nguồn {}", src.display()); }
            let r = clone_voice(&id, &src, p("dialect").as_deref().unwrap_or("south"), None, p("transcript").as_deref()).await?;
            return Ok(NodeOutput { preview_type: "audio".into(), preview_url: r["sampleUrl"].as_str().map(String::from), summary: format!("Đã nhân bản giọng {id}"), cost_usd: 0.0, outputs: r, artifacts: vec![] });
        }
        let tempo = ctx.params["tempo"].as_f64().unwrap_or(1.05);
        let dir = workspace::project_path(&ctx.project_id, &["output", "voice"], true)?;
        let prefix = format!("/data/projects/{}/files/output/voice", ctx.project_id);
        // dialogue lines from upstream script node, else single text
        if let Some(Value::Array(lines)) = up("lines") {
            let clips = synthesize_dialogue(&ctx.project_id, &lines, tempo, &dir, &prefix).await?;
            let total: f64 = clips.iter().map(|c| c["durationSec"].as_f64().unwrap_or(0.0)).sum();
            let first = clips.first().and_then(|c| c["audioUrl"].as_str()).map(String::from);
            return Ok(NodeOutput { preview_type: "audio".into(), preview_url: first, summary: format!("Đọc {} câu hội thoại ({total:.1}s)", clips.len()), cost_usd: 0.0, outputs: json!({"voice": clips}), artifacts: vec![] });
        }
        let text = p("text").or_else(|| up("text").and_then(|v| v.as_str().map(String::from))).ok_or_else(|| anyhow!("voice_tts: thiếu text"))?;
        let voice = p("voiceId").or_else(|| up("voiceId").and_then(|v| v.as_str().map(String::from))).ok_or_else(|| anyhow!("voice_tts: nhân vật chưa có giọng (voiceId)"))?;
        let s = synthesize(&text, &voice, tempo, &dir).await?;
        let url = format!("{prefix}/{}", s.path.file_name().unwrap().to_string_lossy());
        Ok(NodeOutput { preview_type: "audio".into(), preview_url: Some(url.clone()), summary: format!("Đã đọc giọng {voice} ({:.1}s)", s.duration_sec), cost_usd: 0.0,
            outputs: json!({"audioUrl": url, "durationSec": s.duration_sec, "words": s.words}), artifacts: vec![s.path] })
    }
}

pub fn register(r: &mut crate::engine::executor::ExecutorRegistry) {
    r.register(std::sync::Arc::new(VoiceExecutor));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cache_key_is_stable_and_param_sensitive() {
        assert_eq!(cache_key("a", "v", 1.05), cache_key("a", "v", 1.05));
        assert_ne!(cache_key("a", "v", 1.05), cache_key("a", "v", 1.1));
        assert_ne!(cache_key("a", "v", 1.05), cache_key("b", "v", 1.05));
    }
    #[test]
    fn slug() { assert_eq!(slugify("Sóc Bi!"), "s-c-bi"); }

    /// Real end-to-end (slow, needs OmniVoice + ffprobe): VOICE_E2E=1 cargo test e2e -- --nocapture, cwd = repo root.
    #[tokio::test]
    async fn e2e_vietnamese_soc_nho() {
        if std::env::var("VOICE_E2E").is_err() { return; }
        let dir = tempfile::tempdir().unwrap();
        let s = synthesize("Xin chào các bạn nhỏ, hôm nay mình học về cầu vồng nhé.", "soc-nho", 1.05, dir.path()).await.unwrap();
        let out = std::process::Command::new("ffprobe").args(["-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0"]).arg(&s.path).output().unwrap();
        let d: f64 = String::from_utf8_lossy(&out.stdout).trim().parse().unwrap();
        assert!(d > 0.0 && s.duration_sec > 0.0, "duration {d}");
        let again = synthesize("Xin chào các bạn nhỏ, hôm nay mình học về cầu vồng nhé.", "soc-nho", 1.05, dir.path()).await.unwrap();
        assert!(again.cached);
    }
}
