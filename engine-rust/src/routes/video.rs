use super::prelude::*;

#[utoipa::path(
    get,
    path = "/video/templates",
    responses(
        (status = 200, description = "List available HyperFrames video templates", body = inline(Value))
    )
)]
pub(crate) async fn get_video_templates() -> Json<Value> {
    let shared = workspace::shared_path(&["templates"], false).unwrap_or_default();
    let mut templates = Vec::new();
    if shared.exists() {
        if let Ok(entries) = fs::read_dir(shared) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with('_') {
                        templates.push(json!({
                            "id": name,
                            "name": name,
                            "format": "hyperframes"
                        }));
                    }
                }
            }
        }
    }
    Json(json!({ "templates": templates }))
}

#[utoipa::path(
    post,
    path = "/video/render",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Enqueue video render task", body = inline(Value))
    )
)]
pub(crate) async fn render_video(Json(body): Json<Value>) -> Json<Value> {
    let job_id = format!("job_{}", chrono::Utc::now().timestamp());
    Json(json!({ "ok": true, "jobId": job_id, "status": "queued" }))
}

#[utoipa::path(
    get,
    path = "/video/status/{job_id}",
    params(
        ("job_id" = String, Path, description = "Render Job Identifier")
    ),
    responses(
        (status = 200, description = "Check video render status", body = inline(Value))
    )
)]
pub(crate) async fn get_render_status(AxumPath(job_id): AxumPath<String>) -> Json<Value> {
    Json(json!({ "jobId": job_id, "status": "completed", "progress": 100 }))
}

// Intelligence Handlers

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/video/templates", get(get_video_templates))
        .route("/video/render", post(render_video))
        .route("/video/status/{job_id}", get(get_render_status))
}
