use super::prelude::*;

pub(crate) async fn serve_voice_sample(
    AxumPath(voice_id): AxumPath<String>,
) -> impl IntoResponse {
    if !is_valid_slug(&voice_id) {
        return Err((StatusCode::BAD_REQUEST, "Invalid voice id"));
    }
    let vault_root = workspace::data_root().join("shared").join("voice_vault").join(&voice_id);
    let sample_mp3 = vault_root.join("cloned_default_sample.mp3");
    let fallback_mp3 = vault_root.join("source_original_preview.mp3");
    let sample_wav = vault_root.join("reference_clean_slice.wav");
    
    let target = if sample_mp3.exists() {
        sample_mp3
    } else if fallback_mp3.exists() {
        fallback_mp3
    } else if sample_wav.exists() {
        sample_wav
    } else {
        return Err((StatusCode::NOT_FOUND, "Sample audio not found"));
    };

    match fs::read(&target) {
        Ok(bytes) => {
            let mime = guess_mime(&target);
            Ok((
                [(axum::http::header::CONTENT_TYPE, mime)],
                bytes,
            ))
        }
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to read audio")),
    }
}

pub(crate) async fn serve_voice_file(
    AxumPath((voice_id, filename)): AxumPath<(String, String)>,
) -> impl IntoResponse {
    if !is_valid_slug(&voice_id) {
        return Err((StatusCode::BAD_REQUEST, "Invalid voice id"));
    }
    let vault_root = workspace::data_root().join("shared").join("voice_vault").join(&voice_id);
    let canonical_vault = match vault_root.canonicalize() {
        Ok(v) => v,
        Err(_) => return Err((StatusCode::NOT_FOUND, "Voice vault not found")),
    };
    let target = canonical_vault.join(&filename);
    let canonical_target = match target.canonicalize() {
        Ok(t) => t,
        Err(_) => return Err((StatusCode::NOT_FOUND, "Voice file not found")),
    };
    if !canonical_target.starts_with(&canonical_vault) || !canonical_target.is_file() {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    match fs::read(&canonical_target) {
        Ok(bytes) => {
            let mime = guess_mime(&canonical_target);
            Ok((
                [(axum::http::header::CONTENT_TYPE, mime)],
                bytes,
            ))
        }
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file")),
    }
}

use super::voice_impl as vc;
use base64::Engine as _;

type ApiErr = (StatusCode, Json<Value>);
fn err(code: StatusCode, msg: impl ToString) -> ApiErr { (code, Json(json!({ "detail": msg.to_string() }))) }
fn synth_err(e: anyhow::Error) -> ApiErr {
    let m = e.to_string();
    err(if m.starts_with("Thiếu") || m.starts_with("Câu quá") || m.contains("voice_vault") || m.contains("giọng dùng được") { StatusCode::BAD_REQUEST } else { StatusCode::BAD_GATEWAY }, m)
}

#[utoipa::path(get, path = "/voices", responses((status = 200, description = "Voices in voice vault", body = inline(Value))))]
pub(crate) async fn get_voices_list() -> Json<Value> {
    Json(json!(vc::list_voices()))
}

#[utoipa::path(get, path = "/voice/speakers", responses((status = 200, description = "Speakers read from voice_vault", body = inline(Value))))]
pub(crate) async fn get_speakers() -> Json<Value> {
    let speakers: Vec<Value> = vc::list_voices().into_iter().map(|v| {
        let vj = fs::read_to_string(vc::vault_dir().join(v["id"].as_str().unwrap_or("")).join("voice.json")).ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok()).unwrap_or(Value::Null);
        json!({ "id": v["id"], "name": v["name"], "gender": vj["gender"], "dialect": v["dialect"], "ready": v["ready"], "status": v["status"] })
    }).collect();
    Json(json!({ "speakers": speakers }))
}

/// Shared by /voice/say and /projects/:p/voice/say. Body: {text, voiceId, tempo?} or {lines:[{characterId,text}], tempo?} (project only).
async fn say_impl(project: Option<String>, body: Value) -> Result<Json<Value>, ApiErr> {
    let tempo = body["tempo"].as_f64().unwrap_or(1.05).clamp(0.8, 1.4);
    if let Some(p) = project.as_deref() {
        if !is_valid_slug(p) { return Err(err(StatusCode::BAD_REQUEST, "Invalid project id")); }
    }
    if let (Some(lines), Some(p)) = (body["lines"].as_array(), project.as_deref()) {
        let dir = workspace::project_path(p, &["output", "voice"], true).map_err(|e| err(StatusCode::NOT_FOUND, e))?;
        let clips = vc::synthesize_dialogue(p, lines, tempo, &dir, &format!("/data/projects/{p}/files/output/voice")).await.map_err(synth_err)?;
        return Ok(Json(json!({ "clips": clips })));
    }
    let text = body["text"].as_str().unwrap_or("").trim().to_string();
    let voice = body["voiceId"].as_str().or(body["voice_id"].as_str()).unwrap_or("").trim().to_string();
    if text.is_empty() { return Err(err(StatusCode::BAD_REQUEST, "Thiếu nội dung cần đọc")); }
    if voice.is_empty() { return Err(err(StatusCode::BAD_REQUEST, "Nhân vật chưa có giọng (voiceId)")); }
    let (dir, prefix) = match project.as_deref() {
        Some(p) => (workspace::project_path(p, &["output", "voice"], true).map_err(|e| err(StatusCode::NOT_FOUND, e))?, format!("/data/projects/{p}/files/output/voice")),
        None => (vc::vault_dir().join(&voice).join("generated"), format!("/voices/{voice}/generated")),
    };
    let s = vc::synthesize(&text, &voice, tempo, &dir).await.map_err(synth_err)?;
    let url = format!("{prefix}/{}", s.path.file_name().unwrap().to_string_lossy());
    Ok(Json(json!({ "voiceId": voice, "provider": "omnivoice", "audioUrl": url, "duration": s.duration_sec, "durationSec": s.duration_sec, "words": s.words, "cached": s.cached, "text": text })))
}

#[utoipa::path(post, path = "/voice/say", request_body = inline(Value), responses((status = 200, description = "Real TTS: url + duration + word timestamps", body = inline(Value))))]
pub(crate) async fn synthesize_voice(Json(body): Json<Value>) -> Result<Json<Value>, ApiErr> { say_impl(None, body).await }

pub(crate) async fn synthesize_project_voice(AxumPath(p): AxumPath<String>, Json(body): Json<Value>) -> Result<Json<Value>, ApiErr> { say_impl(Some(p), body).await }

/// Body: {voiceId, consentConfirmed, dialect?, audioData(base64)+filename | sourceFile, transcript?, characterId?}
async fn clone_impl(project: Option<String>, body: Value) -> Result<Json<Value>, ApiErr> {
    if body["consentConfirmed"] != json!(true) {
        return Err(err(StatusCode::BAD_REQUEST, "Phải xác nhận quyền sử dụng giọng nói (chỉ giọng của chính bạn hoặc người đã đồng ý)."));
    }
    let project = project.or_else(|| body["projectId"].as_str().map(String::from));
    let id = vc::slugify(body["voiceId"].as_str().unwrap_or(""));
    if id.len() < 2 { return Err(err(StatusCode::BAD_REQUEST, "Mã giọng không hợp lệ (tối thiểu 2 ký tự chữ thường, số, dấu gạch).")); }
    let dialect = body["dialect"].as_str().filter(|d| ["north", "south", "central"].contains(d)).unwrap_or("south");
    let uploads = vc::vault_dir().join(".uploads");
    let mut tmp: Option<PathBuf> = None;
    let src = if let Some(b64) = body["audioData"].as_str() {
        let ext = Path::new(body["filename"].as_str().unwrap_or("x.mp3")).extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()).filter(|e| ["mp3", "wav", "m4a", "mp4", "mov", "webm"].contains(&e.as_str())).unwrap_or("mp3".into());
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).map_err(|e| err(StatusCode::BAD_REQUEST, format!("Dữ liệu âm thanh không hợp lệ: {e}")))?;
        fs::create_dir_all(&uploads).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        let f = uploads.join(format!("upload_{}.{ext}", uuid::Uuid::new_v4().simple()));
        fs::write(&f, bytes).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        tmp = Some(f.clone());
        f
    } else if let Some(sf) = body["sourceFile"].as_str() {
        let rel = sf.trim_start_matches('/');
        let mut c = vec![workspace::repo_root().join(rel), vc::vault_dir().join(rel), vc::vault_dir().join(&id).join(rel)];
        if let Some(p) = project.as_deref().filter(|p| is_valid_slug(p)) { c.push(workspace::data_root().join("projects").join(p).join(rel)); }
        // reject path escapes: sourceFile must stay inside the repo
        c.into_iter().filter(|p| !sf.contains("..")).find(|p| p.is_file()).ok_or_else(|| err(StatusCode::BAD_REQUEST, "Không tìm thấy tệp âm thanh nguồn."))?
    } else {
        return Err(err(StatusCode::BAD_REQUEST, "Thiếu audioData hoặc sourceFile (.mp3, .wav, .m4a, .mp4)."));
    };
    let r = vc::clone_voice(&id, &src, dialect, None, body["transcript"].as_str()).await;
    if let Some(t) = tmp { let _ = fs::remove_file(t); }
    let r = r.map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if let (Some(p), Some(cid)) = (project.as_deref().filter(|p| is_valid_slug(p)), body["characterId"].as_str()) {
        let pj = workspace::data_root().join("projects").join(p).join("project.json");
        if let Some(mut d) = fs::read_to_string(&pj).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
            if let Some(c) = d["characters"].get_mut(cid) {
                c["voice_id"] = json!(id);
                c["sample_audio_url"] = json!(format!("/voices/{id}/sample"));
                let _ = fs::write(&pj, serde_json::to_string_pretty(&d).unwrap_or_default() + "\n");
            }
        }
    }
    Ok(Json(r))
}

#[utoipa::path(post, path = "/voice/clone", request_body = inline(Value), responses((status = 200, description = "Clone voice into voice_vault", body = inline(Value))))]
pub(crate) async fn clone_voice(Json(body): Json<Value>) -> Result<Json<Value>, ApiErr> { clone_impl(None, body).await }

pub(crate) async fn clone_project_voice(AxumPath(p): AxumPath<String>, Json(body): Json<Value>) -> Result<Json<Value>, ApiErr> { clone_impl(Some(p), body).await }

/// Real sample audio for a vault voice: reads the profile greeting/transcript and synthesizes (cached).
pub(crate) async fn voice_samples(AxumPath(id): AxumPath<String>) -> Result<Json<Value>, ApiErr> {
    if !is_valid_slug(&id) { return Err(err(StatusCode::BAD_REQUEST, "Invalid voice id")); }
    let d = vc::vault_dir().join(&id);
    if !d.is_dir() { return Err(err(StatusCode::NOT_FOUND, "Voice vault not found")); }
    let p: Value = fs::read_to_string(d.join("profile.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null);
    let texts: Vec<String> = [p["default_greeting"].as_str(), p["transcript"].as_str()].into_iter().flatten().filter(|t| !t.trim().is_empty()).map(String::from).collect();
    if texts.is_empty() { return Err(err(StatusCode::NOT_FOUND, "Giọng chưa có câu mẫu")); }
    let mut out = vec![];
    for t in texts {
        let s = vc::synthesize(&t, &id, 1.05, &d.join("generated")).await.map_err(synth_err)?;
        out.push(json!({ "text": t, "url": format!("/voices/{id}/generated/{}", s.path.file_name().unwrap().to_string_lossy()), "durationSec": s.duration_sec, "words": s.words }));
    }
    Ok(Json(json!({ "voiceId": id, "samples": out })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/data/voices/{voice_id}/sample", get(serve_voice_sample))
        .route("/data/voices/{voice_id}/samples", get(voice_samples))
        .route("/data/voices/{voice_id}/{*filename}", get(serve_voice_file))
        .route("/voices/{voice_id}/{*filename}", get(serve_voice_file))
        .route("/projects/{project_id}/voice/say", post(synthesize_project_voice))
        .route("/projects/{project_id}/voice/clone", post(clone_project_voice))
        .route("/voices", get(get_voices_list))
        .route("/voices/clone", post(clone_voice))
        .route("/voices/{voice_id}/sample", get(serve_voice_sample))
        .route("/api/voice/preview", get(serve_voice_sample))
        .route("/voice/speakers", get(get_speakers))
        .route("/voice/clone", post(clone_voice))
        .route("/voice/say", post(synthesize_voice))
}
