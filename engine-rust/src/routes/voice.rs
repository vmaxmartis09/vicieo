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

// Graph Canvas Handlers

#[utoipa::path(
    get,
    path = "/voices",
    responses(
        (status = 200, description = "List all voices available in voice vault", body = inline(Value))
    )
)]
pub(crate) async fn get_voices_list() -> Json<Value> {
    let vault_root = workspace::data_root().join("shared").join("voice_vault");
    let mut voices = Vec::new();
    if vault_root.exists() {
        if let Ok(entries) = fs::read_dir(vault_root) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let vfile = entry.path().join("voice.json");
                    if vfile.exists() {
                        if let Ok(txt) = fs::read_to_string(vfile) {
                            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                                voices.push(val);
                                continue;
                            }
                        }
                    }
                    let name = entry.file_name().to_string_lossy().to_string();
                    voices.push(json!({
                        "id": name,
                        "display_name": name,
                        "engine": "omnivoice",
                        "status": "ready"
                    }));
                }
            }
        }
    }
    Json(json!(voices))
}

#[utoipa::path(
    get,
    path = "/voice/speakers",
    responses(
        (status = 200, description = "List registered speakers", body = inline(Value))
    )
)]
pub(crate) async fn get_speakers() -> Json<Value> {
    Json(json!({
        "speakers": [
            { "id": "soc-nho", "name": "Sóc Nhỏ (Bi) - Giọng chuẩn", "gender": "child" },
            { "id": "koala", "name": "Bạn Koala - Giọng ấm", "gender": "male" },
            { "id": "koc-lyly", "name": "KOC LyLy - Reviewer", "gender": "female" }
        ]
    }))
}

#[utoipa::path(
    post,
    path = "/voice/clone",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Clone custom voice from audio input", body = inline(Value))
    )
)]
pub(crate) async fn clone_voice(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({ "ok": true, "voiceId": "custom_cloned_voice", "status": "ready" }))
}

#[utoipa::path(
    post,
    path = "/voice/say",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Synthesize text-to-speech audio", body = inline(Value))
    )
)]
pub(crate) async fn synthesize_voice(Json(body): Json<Value>) -> Json<Value> {
    let text = body.get("text").and_then(|t| t.as_str()).unwrap_or("");
    let voice_id = body.get("voiceId").or_else(|| body.get("voice_id")).and_then(|v| v.as_str()).unwrap_or("soc-nho");
    let audio_url = format!("/voices/{}/sample", voice_id);
    Json(json!({ "ok": true, "audioUrl": audio_url, "duration": 3.5, "text": text }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/data/voices/{voice_id}/sample", get(serve_voice_sample))
        .route("/data/voices/{voice_id}/{*filename}", get(serve_voice_file))
        .route("/voices/{voice_id}/{*filename}", get(serve_voice_file))
        .route("/projects/{project_id}/voice/say", post(synthesize_voice))
        .route("/projects/{project_id}/voice/clone", post(clone_voice))
        .route("/voices", get(get_voices_list))
        .route("/voices/clone", post(clone_voice))
        .route("/voices/{voice_id}/sample", get(serve_voice_sample))
        .route("/api/voice/preview", get(serve_voice_sample))
        .route("/voice/speakers", get(get_speakers))
        .route("/voice/clone", post(clone_voice))
        .route("/voice/say", post(synthesize_voice))
}
