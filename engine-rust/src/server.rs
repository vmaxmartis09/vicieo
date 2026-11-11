use axum::{
    extract::{Path as AxumPath, Query},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post, put, delete},
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::workspace;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "ViciEO API Engine",
        version = "0.2.0",
        description = "Lightweight High-Performance Native AI Video Studio REST API"
    ),
    paths(
        health_check,
    )
)]
struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<String>,
}

pub async fn run_server(port: u16) -> anyhow::Result<()> {
    let (tx, _rx) = broadcast::channel(100);
    let state = AppState { tx };

    let app = Router::new()
        // Core & System Health
        .route("/health", get(health_check))
        .route("/api/v1/info", get(get_workspace_info))
        .route("/api/v1/projects", get(list_projects))
        .route("/projects", get(get_active_project_legacy))
        .route("/data/projects", get(list_projects_data))
        .route("/data/projects/{project_id}", get(get_project_detail))
        .route("/data/projects/{project_id}/characters", get(get_project_characters).post(save_project_character))
        .route("/data/projects/{project_id}/characters/{character_id}", put(save_project_character_put).delete(delete_project_character))
        .route("/data/projects/{project_id}/channels", get(get_project_channels).put(save_project_channels))
        .route("/data/projects/{project_id}/products", get(get_project_products))
        .route("/data/projects/{project_id}/library", get(get_project_library))
        .route("/data/projects/{project_id}/policy", get(get_project_policy).put(save_project_policy))
        .route("/projects/{project_id}/products", get(get_project_products))
        .route("/projects/{project_id}/graph", get(get_project_graph_by_path).post(save_project_graph_by_path))
        .route("/projects/{project_id}/publishing", get(get_project_publishing))
        .route("/projects/{project_id}/publishing/schedule", post(save_project_scheduled_post))
        .route("/projects/{project_id}/files/{*file_path}", get(serve_project_file))
        .route("/voices/{voice_id}/sample", get(serve_voice_sample))
        .route("/voices/{voice_id}/{filename}", get(serve_voice_file))
        .nest_service("/data", tower_http::services::ServeDir::new(workspace::data_root()))
        
        // Graph Canvas & State API (Port 8000/9000 compatibility)
        .route("/graph", get(get_graph).post(save_graph))
        .route("/graph/run", post(run_graph))
        .route("/graph/cancel", post(cancel_graph))
        .route("/graph/events", get(sse_events))
        .route("/graph/validate", post(validate_graph))
        .route("/graph/cost", post(estimate_cost))

        // Data Catalog & Assets
        .route("/api/data/characters", get(get_characters).post(save_character))
        .route("/api/data/products", get(get_products).post(save_product))
        .route("/api/data/assets", get(get_assets))
        .route("/api/data/library", get(get_library))

        // Social & Publishing Center
        .route("/api/social/accounts", get(get_social_accounts).post(save_social_account))
        .route("/api/social/posts", get(get_social_posts).post(create_social_post))
        .route("/api/social/schedule", get(get_schedule).post(save_schedule))
        .route("/api/social/analytics", get(get_analytics))

        // Video & HyperFrames Engine
        .route("/video/templates", get(get_video_templates))
        .route("/video/render", post(render_video))
        .route("/video/status/{job_id}", get(get_render_status))

        // Intelligence & KOC Ideas
        .route("/api/intel/ideas", post(generate_ideas))
        .route("/api/intel/script", post(generate_script))
        .route("/api/intel/compliance", post(check_compliance))

        // Voice & Audio
        .route("/voice/speakers", get(get_speakers))
        .route("/voice/clone", post(clone_voice))
        .route("/voice/say", post(synthesize_voice))

        // Swagger UI & OpenAPI Specification
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))

        .with_state(state)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 ViciEO Full Rust API Engine running on http://0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ----------------- HANDLERS IMPLEMENTATION -----------------

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Engine health status", body = inline(Value))
    )
)]
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "engine": "vicieo-rust-native-v2",
        "version": "0.2.0",
        "active_backend": "rust-axum",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_workspace_info() -> Json<Value> {
    let root = workspace::repo_root();
    let act_proj = workspace::active_project_id();
    Json(json!({
        "repo_root": root.to_string_lossy(),
        "active_project": act_proj,
        "runtime": "Rust Axum Native"
    }))
}

async fn list_projects() -> Json<Value> {
    let projects = workspace::list_projects();
    Json(json!({ "projects": projects }))
}

async fn get_active_project_legacy() -> Json<Value> {
    let act_proj = workspace::active_project_id();
    Json(json!({ "active_project": act_proj, "activeProject": act_proj }))
}

async fn list_projects_data() -> Json<Value> {
    let mut list = Vec::new();
    let projs = workspace::list_projects();
    for p in projs {
        let p_path = workspace::project_path(&p.id, &["project.json"], false).unwrap_or_default();
        if p_path.exists() {
            if let Ok(txt) = fs::read_to_string(&p_path) {
                if let Ok(mut val) = serde_json::from_str::<Value>(&txt) {
                    // Populate counts
                    let chars_count = workspace::project_path(&p.id, &["characters", "studio_characters.json"], false)
                        .ok()
                        .and_then(|cp| fs::read_to_string(cp).ok())
                        .and_then(|ctxt| serde_json::from_str::<Value>(&ctxt).ok())
                        .and_then(|cval| cval.get("characters").and_then(|c| c.as_array().map(|a| a.len())))
                        .unwrap_or(0);
                    let eps_count = workspace::project_path(&p.id, &["content", "episodes"], false)
                        .ok()
                        .and_then(|ep| fs::read_dir(ep).ok())
                        .map(|entries| entries.flatten().filter(|e| e.path().is_dir()).count())
                        .unwrap_or(0);

                    if let Some(obj) = val.as_object_mut() {
                        obj.insert("charactersCount".to_string(), json!(chars_count));
                        obj.insert("episodesCount".to_string(), json!(eps_count));
                        if !obj.contains_key("defaultAspectRatio") {
                            let ar = obj.get("default_aspect_ratio").cloned().unwrap_or(json!("9:16"));
                            obj.insert("defaultAspectRatio".to_string(), ar);
                        }
                    }
                    list.push(val);
                    continue;
                }
            }
        }
        list.push(json!({
            "id": p.id,
            "name": p.name.unwrap_or(p.id.clone()),
            "description": p.description.unwrap_or_default(),
            "charactersCount": 0,
            "episodesCount": 0,
            "defaultAspectRatio": "9:16"
        }));
    }
    Json(json!(list))
}

async fn get_project_detail(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let p_path = workspace::project_path(&project_id, &["project.json"], false).unwrap_or_default();
    if p_path.exists() {
        if let Ok(txt) = fs::read_to_string(&p_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    Json(json!({ "id": project_id }))
}

async fn get_project_characters(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let sc_path = workspace::project_path(&project_id, &["characters", "studio_characters.json"], false).unwrap_or_default();
    if sc_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sc_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                if let Some(chars) = val.get("characters") {
                    return Json(chars.clone());
                }
            }
        }
    }
    Json(json!([]))
}

async fn save_project_character(AxumPath(project_id): AxumPath<String>, Json(body): Json<Value>) -> Json<Value> {
    let sc_path = workspace::project_path(&project_id, &["characters", "studio_characters.json"], true).unwrap_or_default();
    let mut current_chars: Vec<Value> = Vec::new();
    if sc_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sc_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                if let Some(arr) = val.get("characters").and_then(|c| c.as_array()) {
                    current_chars = arr.clone();
                }
            }
        }
    }
    current_chars.push(body.clone());
    let _ = fs::write(&sc_path, serde_json::to_string_pretty(&json!({ "characters": current_chars })).unwrap_or_default());
    Json(body)
}

async fn save_project_character_put(AxumPath((project_id, char_id)): AxumPath<(String, String)>, Json(body): Json<Value>) -> Json<Value> {
    let sc_path = workspace::project_path(&project_id, &["characters", "studio_characters.json"], true).unwrap_or_default();
    let mut current_chars: Vec<Value> = Vec::new();
    if sc_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sc_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                if let Some(arr) = val.get("characters").and_then(|c| c.as_array()) {
                    current_chars = arr.clone();
                }
            }
        }
    }
    let mut found = false;
    for c in &mut current_chars {
        if c.get("id").and_then(|v| v.as_str()) == Some(&char_id) {
            *c = body.clone();
            found = true;
            break;
        }
    }
    if !found {
        current_chars.push(body.clone());
    }
    let _ = fs::write(&sc_path, serde_json::to_string_pretty(&json!({ "characters": current_chars })).unwrap_or_default());
    Json(body)
}

async fn delete_project_character(AxumPath((project_id, char_id)): AxumPath<(String, String)>) -> Json<Value> {
    let sc_path = workspace::project_path(&project_id, &["characters", "studio_characters.json"], false).unwrap_or_default();
    if sc_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sc_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                if let Some(arr) = val.get("characters").and_then(|c| c.as_array()) {
                    let next: Vec<Value> = arr.iter().filter(|c| c.get("id").and_then(|v| v.as_str()) != Some(&char_id)).cloned().collect();
                    let _ = fs::write(&sc_path, serde_json::to_string_pretty(&json!({ "characters": next })).unwrap_or_default());
                }
            }
        }
    }
    Json(json!({ "ok": true }))
}

async fn get_project_channels(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    // Try publishing/accounts.json first, then publishing/schedule.json
    let acc_path = workspace::project_path(&project_id, &["publishing", "accounts.json"], false).unwrap_or_default();
    if acc_path.exists() {
        if let Ok(txt) = fs::read_to_string(&acc_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    let sched_path = workspace::project_path(&project_id, &["publishing", "schedule.json"], false).unwrap_or_default();
    if sched_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sched_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                if let Some(chans) = val.get("channels") {
                    return Json(chans.clone());
                }
            }
        }
    }
    Json(json!([]))
}

async fn save_project_channels(AxumPath(project_id): AxumPath<String>, Json(body): Json<Value>) -> Json<Value> {
    let acc_path = workspace::project_path(&project_id, &["publishing", "accounts.json"], true).unwrap_or_default();
    let _ = fs::write(acc_path, serde_json::to_string_pretty(&body).unwrap_or_default());
    Json(body)
}

async fn get_project_products(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let prod_dir = workspace::project_path(&project_id, &["products"], false).unwrap_or_default();
    let mut products = Vec::new();
    if prod_dir.exists() {
        if let Ok(entries) = fs::read_dir(prod_dir) {
            for entry in entries.flatten() {
                if entry.file_name() == "_trash" { continue; }
                let pfile = entry.path().join("product.json");
                if pfile.exists() {
                    if let Ok(txt) = fs::read_to_string(pfile) {
                        if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                            products.push(val);
                        }
                    }
                }
            }
        }
    }
    Json(json!(products))
}

async fn get_project_publishing(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let sched_path = workspace::project_path(&project_id, &["publishing", "schedule.json"], false).unwrap_or_default();
    if sched_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sched_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    Json(json!({ "channels": [], "posts": [], "schedule": [] }))
}

async fn save_project_scheduled_post(AxumPath(project_id): AxumPath<String>, Json(body): Json<Value>) -> Json<Value> {
    let sched_path = workspace::project_path(&project_id, &["publishing", "schedule.json"], true).unwrap_or_default();
    let mut current_posts: Vec<Value> = Vec::new();
    let mut raw_val = json!({ "channels": [], "posts": [] });
    if sched_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sched_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                if let Some(arr) = val.get("posts").and_then(|p| p.as_array()) {
                    current_posts = arr.clone();
                }
                raw_val = val;
            }
        }
    }
    current_posts.push(body.clone());
    if let Some(obj) = raw_val.as_object_mut() {
        obj.insert("posts".to_string(), json!(current_posts));
    }
    let _ = fs::write(&sched_path, serde_json::to_string_pretty(&raw_val).unwrap_or_default());
    Json(json!({ "ok": true }))
}

async fn get_project_library(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let mut episodes: Vec<Value> = Vec::new();
    let scripts: Vec<Value> = Vec::new();
    let images: Vec<Value> = Vec::new();
    let videos: Vec<Value> = Vec::new();

    let ep_dir = workspace::project_path(&project_id, &["content", "episodes"], false).unwrap_or_default();
    if ep_dir.exists() {
        if let Ok(entries) = fs::read_dir(ep_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let ep_id = entry.file_name().to_string_lossy().to_string();
                    episodes.push(json!({
                        "id": ep_id,
                        "title": ep_id,
                        "videoCount": 0,
                        "imageCount": 0,
                        "scriptCount": 0
                    }));
                }
            }
        }
    }

    Json(json!({
        "episodes": episodes,
        "scripts": scripts,
        "images": images,
        "videos": videos
    }))
}

async fn get_project_policy(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let policy_path = workspace::project_path(&project_id, &["policy.json"], false).unwrap_or_default();
    if policy_path.exists() {
        if let Ok(content) = fs::read_to_string(&policy_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                return Json(val);
            }
        }
    }
    Json(json!({
        "projectId": project_id,
        "rules": []
    }))
}

async fn save_project_policy(AxumPath(project_id): AxumPath<String>, Json(body): Json<Value>) -> Json<Value> {
    let policy_path = workspace::project_path(&project_id, &["policy.json"], true).unwrap_or_default();
    let _ = fs::write(policy_path, serde_json::to_string_pretty(&body).unwrap_or_default());
    Json(body)
}

async fn get_project_graph_by_path(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let path = workspace::project_path(&project_id, &["graph.json"], false).unwrap_or_default();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                return Json(val);
            }
        }
    }
    Json(json!({ "nodes": [], "edges": [], "viewport": { "x": 0, "y": 0, "zoom": 1 } }))
}

async fn save_project_graph_by_path(AxumPath(project_id): AxumPath<String>, Json(body): Json<Value>) -> Json<Value> {
    let path = workspace::project_path(&project_id, &["graph.json"], true).unwrap_or_default();
    if let Ok(str_val) = serde_json::to_string_pretty(&body) {
        let _ = fs::write(&path, str_val);
    }
    Json(json!({ "ok": true, "saved": true, "project": project_id }))
}

use axum::response::IntoResponse;
use axum::http::StatusCode;

fn guess_mime(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "json" => "application/json",
        "md" | "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

async fn serve_project_file(
    AxumPath((project_id, file_path)): AxumPath<(String, String)>,
) -> impl IntoResponse {
    let base = workspace::project_path(&project_id, &[], false).unwrap_or_default();
    let full_path = base.join(&file_path);
    if !full_path.exists() || !full_path.is_file() {
        return Err((StatusCode::NOT_FOUND, "File not found"));
    }
    
    match fs::read(&full_path) {
        Ok(bytes) => {
            let mime = guess_mime(&full_path);
            Ok((
                [(axum::http::header::CONTENT_TYPE, mime)],
                bytes,
            ))
        }
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file")),
    }
}

async fn serve_voice_sample(
    AxumPath(voice_id): AxumPath<String>,
) -> impl IntoResponse {
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

async fn serve_voice_file(
    AxumPath((voice_id, filename)): AxumPath<(String, String)>,
) -> impl IntoResponse {
    let target = workspace::data_root().join("shared").join("voice_vault").join(&voice_id).join(&filename);
    if !target.exists() || !target.is_file() {
        return Err((StatusCode::NOT_FOUND, "Voice file not found"));
    }

    match fs::read(&target) {
        Ok(bytes) => {
            let mime = guess_mime(&target);
            Ok((
                [(axum::http::header::CONTENT_TYPE, mime)],
                bytes,
            ))
        }
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file")),
    }
}

// Graph Canvas Handlers
#[derive(Deserialize)]
struct ProjectQuery {
    project: Option<String>,
}

async fn get_graph(Query(q): Query<ProjectQuery>) -> Json<Value> {
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    let path = workspace::project_path(&proj, &["graph.json"], false).unwrap_or_default();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                return Json(val);
            }
        }
    }
    Json(json!({ "nodes": [], "edges": [], "viewport": { "x": 0, "y": 0, "zoom": 1 } }))
}

async fn save_graph(Query(q): Query<ProjectQuery>, Json(body): Json<Value>) -> Json<Value> {
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    let path = workspace::project_path(&proj, &["graph.json"], true).unwrap_or_default();
    if let Ok(str_val) = serde_json::to_string_pretty(&body) {
        let _ = fs::write(&path, str_val);
    }
    Json(json!({ "ok": true, "saved": true, "project": proj }))
}

async fn run_graph(Query(q): Query<ProjectQuery>, Json(body): Json<Value>) -> Json<Value> {
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    let run_id = format!("run_{}", chrono::Utc::now().timestamp());
    Json(json!({
        "ok": true,
        "runId": run_id,
        "project": proj,
        "status": "started",
        "nodesQueued": body.get("nodes").and_then(|n| n.as_array()).map(|a| a.len()).unwrap_or(0)
    }))
}

async fn cancel_graph(Query(q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "ok": true, "cancelled": true }))
}

async fn sse_events(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.tx.subscribe();
    let stream = async_stream::stream! {
        yield Ok(Event::default().event("init").data(json!({"connected": true}).to_string()));
        while let Ok(msg) = rx.recv().await {
            yield Ok(Event::default().data(msg));
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn validate_graph(Json(_body): Json<Value>) -> Json<Value> {
    Json(json!({ "valid": true, "errors": [] }))
}

async fn estimate_cost(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({ "totalCostUsd": 0.0, "currency": "USD", "isFreeMode": true }))
}

// Data Catalog & Character Handlers
async fn get_characters(Query(q): Query<ProjectQuery>) -> Json<Value> {
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    let char_dir = workspace::project_path(&proj, &["characters"], false).unwrap_or_default();
    let mut characters = Vec::new();
    if char_dir.exists() {
        if let Ok(entries) = fs::read_dir(char_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let meta_path = entry.path().join("profile.json");
                    if meta_path.exists() {
                        if let Ok(txt) = fs::read_to_string(&meta_path) {
                            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                                characters.push(val);
                                continue;
                            }
                        }
                    }
                    characters.push(json!({
                        "id": entry.file_name().to_string_lossy(),
                        "name": entry.file_name().to_string_lossy()
                    }));
                }
            }
        }
    }
    Json(json!({ "characters": characters }))
}

async fn save_character(Query(q): Query<ProjectQuery>, Json(body): Json<Value>) -> Json<Value> {
    let id = body.get("id").and_then(|v| v.as_str()).unwrap_or("char_custom");
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    let dir = workspace::project_path(&proj, &["characters", id], true).unwrap_or_default();
    let _ = fs::write(dir.join("profile.json"), serde_json::to_string_pretty(&body).unwrap_or_default());
    Json(json!({ "ok": true, "id": id }))
}

async fn get_products(Query(q): Query<ProjectQuery>) -> Json<Value> {
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    let prod_dir = workspace::project_path(&proj, &["products"], false).unwrap_or_default();
    let mut products = Vec::new();
    if prod_dir.exists() {
        if let Ok(entries) = fs::read_dir(prod_dir) {
            for entry in entries.flatten() {
                let pfile = entry.path().join("product.json");
                if pfile.exists() {
                    if let Ok(txt) = fs::read_to_string(pfile) {
                        if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                            products.push(val);
                        }
                    }
                }
            }
        }
    }
    Json(json!({ "products": products }))
}

async fn save_product(Query(q): Query<ProjectQuery>, Json(body): Json<Value>) -> Json<Value> {
    let id = body.get("id").and_then(|v| v.as_str()).unwrap_or("prod_custom");
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    let dir = workspace::project_path(&proj, &["products", id], true).unwrap_or_default();
    let _ = fs::write(dir.join("product.json"), serde_json::to_string_pretty(&body).unwrap_or_default());
    Json(json!({ "ok": true, "id": id }))
}

async fn get_assets(Query(q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "assets": [] }))
}

async fn get_library(Query(q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "items": [] }))
}

// Social Handlers
async fn get_social_accounts(Query(q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "accounts": [] }))
}

async fn save_social_account(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({ "ok": true, "account": body }))
}

async fn get_social_posts(Query(q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "posts": [] }))
}

async fn create_social_post(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({ "ok": true, "postId": format!("post_{}", chrono::Utc::now().timestamp()) }))
}

async fn get_schedule(Query(q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "schedule": [] }))
}

async fn save_schedule(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({ "ok": true }))
}

async fn get_analytics(Query(q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "views": 0, "likes": 0, "shares": 0, "platform": "all" }))
}

// Video & HyperFrames Handlers
async fn get_video_templates() -> Json<Value> {
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

async fn render_video(Json(body): Json<Value>) -> Json<Value> {
    let job_id = format!("job_{}", chrono::Utc::now().timestamp());
    Json(json!({ "ok": true, "jobId": job_id, "status": "queued" }))
}

async fn get_render_status(AxumPath(job_id): AxumPath<String>) -> Json<Value> {
    Json(json!({ "jobId": job_id, "status": "completed", "progress": 100 }))
}

// Intelligence Handlers
async fn generate_ideas(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({
        "ideas": [
            { "title": "Bí quyết chọn áo cardigan mùa thu cực xinh", "hook": "Đừng mua cardigan nếu chưa biết điều này!", "score": 9.2 },
            { "title": "Phối đồ phong cách tối giản thanh lịch", "hook": "3 cách phối đồ chuẩn Hàn Quốc chỉ với 1 chiếc áo", "score": 8.8 }
        ]
    }))
}

async fn generate_script(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({
        "script": {
            "title": "Kịch bản review sản phẩm",
            "scenes": [
                { "scene": 1, "duration": 3.0, "narration": "Chiếc áo này đang làm mưa làm gió tuần qua..." },
                { "scene": 2, "duration": 5.0, "narration": "Chất vải dày dặn, form dáng cực kỳ tôn eo." }
            ]
        }
    }))
}

async fn check_compliance(Json(body): Json<Value>) -> Json<Value> {
    let text = body.get("text").and_then(|v| v.as_str()).unwrap_or("");
    let violations = crate::intelligence::IntelligenceEngine::check_policy_compliance(text);
    Json(json!({ "compliant": violations.is_empty(), "violations": violations }))
}

// Voice Handlers
async fn get_speakers() -> Json<Value> {
    Json(json!({
        "speakers": [
            { "id": "soc-nho", "name": "Sóc Nhỏ (Bi) - Giọng chuẩn", "gender": "child" },
            { "id": "koala", "name": "Bạn Koala - Giọng ấm", "gender": "male" },
            { "id": "koc-lyly", "name": "KOC LyLy - Reviewer", "gender": "female" }
        ]
    }))
}

async fn clone_voice(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({ "ok": true, "voiceId": "custom_cloned_voice", "status": "ready" }))
}

async fn synthesize_voice(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({ "ok": true, "audioUrl": "/assets/sample_speech.mp3", "duration": 3.5 }))
}
