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
        get_workspace_info,
        list_projects,
        list_projects_data,
        get_project_detail,
        get_project_characters,
        save_project_character,
        get_project_channels,
        save_project_channels,
        get_project_products,
        get_project_koc_styles,
        create_product_review_flow,
        get_project_library,
        get_project_policy,
        save_project_policy,
        get_project_graph_by_path,
        save_project_graph_by_path,
        get_graph,
        save_graph,
        run_graph,
        cancel_graph,
        validate_graph,
        estimate_cost,
        get_video_templates,
        render_video,
        get_render_status,
        generate_ideas,
        generate_script,
        check_compliance,
        get_voices_list,
        get_speakers,
        clone_voice,
        synthesize_voice,
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
        .route("/data/projects/{project_id}/products/{product_id}/review-flow", post(create_product_review_flow))
        .route("/data/projects/{project_id}/koc-styles", get(get_project_koc_styles))
        .route("/data/projects/{project_id}/library", get(get_project_library))
        .route("/projects/{project_id}/library", get(get_project_library))
        .route("/data/projects/{project_id}/script-data/{episode_id}", get(get_script_data))
        .route("/projects/{project_id}/script-data/{episode_id}", get(get_script_data))
        .route("/data/projects/{project_id}/policy", get(get_project_policy).put(save_project_policy))
        .route("/projects/{project_id}/products", get(get_project_products))
        .route("/projects/{project_id}/graph", get(get_project_graph_by_path).post(save_project_graph_by_path))
        .route("/projects/{project_id}/publishing", get(get_project_publishing))
        .route("/projects/{project_id}/publishing/schedule", post(save_project_scheduled_post))
        .route("/projects/{project_id}/files/{*file_path}", get(serve_project_file))
        .route("/data/projects/{project_id}/files/{*file_path}", get(serve_project_file))
        .route("/voices", get(get_voices_list))
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
    let root = workspace::repo_root();
    Json(json!({
        "status": "healthy",
        "engine": "vicieo-rust-native-v2",
        "version": "0.2.0",
        "active_backend": "rust-axum",
        "repo_root": root.to_string_lossy(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/info",
    responses(
        (status = 200, description = "Workspace info", body = inline(Value))
    )
)]
async fn get_workspace_info() -> Json<Value> {
    let root = workspace::repo_root();
    let act_proj = workspace::active_project_id();
    Json(json!({
        "repo_root": root.to_string_lossy(),
        "active_project": act_proj,
        "runtime": "Rust Axum Native"
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects",
    responses(
        (status = 200, description = "List registered projects", body = inline(Value))
    )
)]
async fn list_projects() -> Json<Value> {
    let projects = workspace::list_projects();
    Json(json!({ "projects": projects }))
}

async fn get_active_project_legacy() -> Json<Value> {
    let act_proj = workspace::active_project_id();
    Json(json!({ "active_project": act_proj, "activeProject": act_proj }))
}

#[utoipa::path(
    get,
    path = "/data/projects",
    responses(
        (status = 200, description = "List detailed project profiles", body = inline(Value))
    )
)]
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

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "Project profile metadata", body = inline(Value))
    )
)]
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

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}/characters",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "List of project characters", body = inline(Value))
    )
)]
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

#[utoipa::path(
    post,
    path = "/data/projects/{project_id}/characters",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    request_body = inline(Value),
    responses(
        (status = 200, description = "Save or create project character", body = inline(Value))
    )
)]
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

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}/channels",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "List of social channels for publishing", body = inline(Value))
    )
)]
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

#[utoipa::path(
    put,
    path = "/data/projects/{project_id}/channels",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    request_body = inline(Value),
    responses(
        (status = 200, description = "Save social channels for publishing", body = inline(Value))
    )
)]
async fn save_project_channels(AxumPath(project_id): AxumPath<String>, Json(body): Json<Value>) -> Json<Value> {
    let acc_path = workspace::project_path(&project_id, &["publishing", "accounts.json"], true).unwrap_or_default();
    let _ = fs::write(acc_path, serde_json::to_string_pretty(&body).unwrap_or_default());
    Json(body)
}

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}/products",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "List of catalog products", body = inline(Value))
    )
)]
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

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}/koc-styles",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "List available KOC review styles", body = inline(Value))
    )
)]
async fn get_project_koc_styles(AxumPath(_project_id): AxumPath<String>) -> Json<Value> {
    Json(json!([
        { "styleId": "koc_unboxing_tryon", "name": "Đập hộp & Thử đồ (Unboxing & Try-on)", "archetype": "unboxing_tryon" },
        { "styleId": "koc_problem_solution", "name": "Giải pháp phối đồ (Problem & Solution)", "archetype": "problem_solution" },
        { "styleId": "koc_outfit_check", "name": "Biến hình & Đánh giá (Outfit Check)", "archetype": "outfit_check" }
    ]))
}

#[utoipa::path(
    post,
    path = "/data/projects/{project_id}/products/{product_id}/review-flow",
    params(
        ("project_id" = String, Path, description = "Project Identifier"),
        ("product_id" = String, Path, description = "Product Identifier")
    ),
    request_body = inline(Value),
    responses(
        (status = 200, description = "Generate review flow graph for product", body = inline(Value))
    )
)]
async fn create_product_review_flow(
    AxumPath((project_id, product_id)): AxumPath<(String, String)>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let style_id = body.get("style_id").and_then(|v| v.as_str()).unwrap_or("koc_unboxing_tryon");
    let file_name = format!("graph.review-{}.json", style_id);
    let path = workspace::project_path(&project_id, &[&file_name], true).unwrap_or_default();
    
    // Save lightweight template if not exists
    if !path.exists() {
        let sample_graph = json!({
            "nodes": [],
            "edges": [],
            "productId": product_id,
            "styleId": style_id,
            "createdAt": chrono::Utc::now().to_rfc3339()
        });
        let _ = fs::write(&path, serde_json::to_string_pretty(&sample_graph).unwrap_or_default());
    }

    Json(json!({
        "ok": true,
        "file": file_name,
        "project": project_id,
        "productId": product_id,
        "styleId": style_id
    }))
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

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}/library",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "List library assets (episodes, scripts, images, videos)", body = inline(Value))
    )
)]
async fn get_project_library(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let mut episodes: Vec<Value> = Vec::new();
    let mut scripts: Vec<Value> = Vec::new();
    let mut images: Vec<Value> = Vec::new();
    let mut videos: Vec<Value> = Vec::new();
    let mut audios: Vec<Value> = Vec::new();

    let proj_root = workspace::project_path(&project_id, &[], false).unwrap_or_default();

    // 1. Scan assets/<hash>.meta.json
    let assets_dir = proj_root.join("assets");
    if assets_dir.exists() {
        if let Ok(entries) = fs::read_dir(&assets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.to_string_lossy().ends_with(".meta.json") {
                    if let Ok(txt) = fs::read_to_string(&path) {
                        if let Ok(m) = serde_json::from_str::<Value>(&txt) {
                            if let Some(uri) = m.get("uri").and_then(|u| u.as_str()) {
                                let target_file = proj_root.join(uri);
                                if target_file.is_file() {
                                    let kind = m.get("kind").and_then(|k| k.as_str()).unwrap_or("");
                                    let id = m.get("id").and_then(|i| i.as_str()).unwrap_or(uri);
                                    let url = format!("/data/projects/{}/files/{}", project_id, uri);
                                    let prov = m.get("provenance").cloned().unwrap_or(json!({}));
                                    let meta = m.get("meta").cloned().unwrap_or(json!({}));
                                    let item = json!({
                                        "id": id,
                                        "name": target_file.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                                        "url": url,
                                        "source": "artifact",
                                        "aspect": meta.get("aspect"),
                                        "durationSec": meta.get("duration"),
                                        "provider": prov.get("provider"),
                                        "model": prov.get("model"),
                                        "createdAt": prov.get("createdAt")
                                    });
                                    if kind == "video" {
                                        videos.push(item);
                                    } else if kind == "image" {
                                        images.push(item);
                                    } else if kind == "audio" {
                                        audios.push(item);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Scan content/episodes
    let ep_dir = proj_root.join("content").join("episodes");
    if ep_dir.exists() {
        if let Ok(entries) = fs::read_dir(&ep_dir) {
            let mut dirs: Vec<_> = entries.flatten().filter(|e| e.path().is_dir()).collect();
            dirs.sort_by_key(|a| a.file_name());
            for entry in dirs {
                let ep_path = entry.path();
                let ep_id = entry.file_name().to_string_lossy().to_string();
                let mut ep_videos: Vec<Value> = Vec::new();
                let mut ep_images: Vec<Value> = Vec::new();
                let mut title = ep_id.clone();

                if let Ok(files) = fs::read_dir(&ep_path) {
                    for f_entry in files.flatten() {
                        let fp = f_entry.path();
                        let fname = f_entry.file_name().to_string_lossy().to_string();
                        let ext = fp.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                        if ext == "md" {
                            if let Ok(txt) = fs::read_to_string(&fp) {
                                if fname.to_uppercase().starts_with("KICH") {
                                    for line in txt.lines() {
                                        if line.starts_with("# ") {
                                            title = line.trim_start_matches("# ").trim().to_string();
                                            break;
                                        }
                                    }
                                }
                                scripts.push(json!({
                                    "id": format!("{}/{}", ep_id, fname),
                                    "name": fname,
                                    "episodeId": ep_id,
                                    "url": format!("/data/projects/{}/files/content/episodes/{}/{}", project_id, ep_id, fname),
                                    "preview": txt.chars().take(320).collect::<String>(),
                                }));
                            }
                        } else if ext == "mp4" || ext == "mov" || ext == "webm" {
                            let item = json!({
                                "id": format!("{}/{}", ep_id, fname),
                                "name": fname,
                                "url": format!("/data/projects/{}/files/content/episodes/{}/{}", project_id, ep_id, fname),
                                "source": "episode",
                                "episodeId": ep_id
                            });
                            ep_videos.push(item.clone());
                            videos.push(item);
                        } else if ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "webp" {
                            let item = json!({
                                "id": format!("{}/{}", ep_id, fname),
                                "name": fname,
                                "url": format!("/data/projects/{}/files/content/episodes/{}/{}", project_id, ep_id, fname),
                                "source": "episode",
                                "episodeId": ep_id
                            });
                            ep_images.push(item.clone());
                            images.push(item);
                        }
                    }
                }

                episodes.push(json!({
                    "id": ep_id,
                    "title": title,
                    "videoCount": ep_videos.len(),
                    "imageCount": ep_images.len(),
                    "scriptCount": 1
                }));
            }
        }
    }

    // 3. Scan products for cutout images
    let prod_dir = proj_root.join("products");
    if prod_dir.exists() {
        if let Ok(entries) = fs::read_dir(&prod_dir) {
            for entry in entries.flatten() {
                let cutout_dir = entry.path().join("images").join("cutout");
                if cutout_dir.exists() {
                    if let Ok(cfiles) = fs::read_dir(&cutout_dir) {
                        for cf in cfiles.flatten() {
                            let cfp = cf.path();
                            if cfp.is_file() {
                                let fname = cf.file_name().to_string_lossy().to_string();
                                let rel = cfp.strip_prefix(&proj_root).unwrap_or(&cfp).to_string_lossy().to_string();
                                images.push(json!({
                                    "id": format!("prod_{}", fname),
                                    "name": fname,
                                    "url": format!("/data/projects/{}/files/{}", project_id, rel),
                                    "source": "product"
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    Json(json!({
        "episodes": episodes,
        "scripts": scripts,
        "images": images,
        "videos": videos,
        "audios": audios
    }))
}

async fn get_script_data(AxumPath((project_id, episode_id)): AxumPath<(String, String)>) -> Json<Value> {
    let script_path = workspace::project_path(&project_id, &["content", "episodes", &episode_id, "script.json"], false).unwrap_or_default();
    if script_path.exists() {
        if let Ok(txt) = fs::read_to_string(&script_path) {
            if let Ok(script_val) = serde_json::from_str::<Value>(&txt) {
                let mut shots: Vec<Value> = Vec::new();
                if let Some(scenes) = script_val.get("scenes").and_then(|s| s.as_array()) {
                    for sc in scenes {
                        let sc_idx = sc.get("index").or_else(|| sc.get("sceneNumber")).and_then(|i| i.as_i64()).unwrap_or(1);
                        let sc_title = sc.get("title").and_then(|t| t.as_str()).unwrap_or("");
                        if let Some(sh_arr) = sc.get("shots").and_then(|s| s.as_array()) {
                            for sh in sh_arr {
                                let sh_id = sh.get("id").and_then(|i| i.as_str()).unwrap_or("");
                                let dia = sh.get("dialogue").and_then(|d| d.as_str()).unwrap_or("");
                                let spk = sh.get("speaker").and_then(|s| s.as_str()).unwrap_or("");
                                shots.push(json!({
                                    "id": sh_id,
                                    "scene": sc_idx,
                                    "sceneTitle": sc_title,
                                    "dialogue": dia,
                                    "speaker": spk,
                                    "characterId": if spk.is_empty() { Value::Null } else { Value::String(spk.to_string()) }
                                }));
                            }
                        }
                    }
                }
                return Json(json!({
                    "script": script_val,
                    "facets": [
                        {"id": "fulltext", "label": "Toàn văn kịch bản", "type": "text"},
                        {"id": "summary", "label": "Tóm tắt nội dung", "type": "text"},
                        {"id": "lesson", "label": "Bài học cốt lõi", "type": "text"},
                        {"id": "mantra", "label": "Câu thần chú", "type": "text"},
                        {"id": "dialogue_all", "label": "Lời thoại toàn tập", "type": "text"},
                        {"id": "environment", "label": "Prompt khóa bối cảnh", "type": "text"},
                        {"id": "scenes", "label": "Danh sách cảnh / shot (scenes)", "type": "scenes"},
                        {"id": "script", "label": "Toàn bộ kịch bản có cấu trúc", "type": "script"}
                    ],
                    "shotFacets": [
                        {"id": "dialogue", "label": "Lời thoại"},
                        {"id": "imagePrompt", "label": "Image prompt"},
                        {"id": "videoPrompt", "label": "Video prompt"},
                        {"id": "action", "label": "Hành động trên hình"},
                        {"id": "expression", "label": "Biểu cảm"}
                    ],
                    "shots": shots
                }));
            }
        }
    }
    Json(json!({
        "script": null,
        "facets": [],
        "shotFacets": [],
        "shots": []
    }))
}

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}/policy",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "Get project content compliance policy", body = inline(Value))
    )
)]
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

#[utoipa::path(
    put,
    path = "/data/projects/{project_id}/policy",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    request_body = inline(Value),
    responses(
        (status = 200, description = "Save project content compliance policy", body = inline(Value))
    )
)]
async fn save_project_policy(AxumPath(project_id): AxumPath<String>, Json(body): Json<Value>) -> Json<Value> {
    let policy_path = workspace::project_path(&project_id, &["policy.json"], true).unwrap_or_default();
    let _ = fs::write(policy_path, serde_json::to_string_pretty(&body).unwrap_or_default());
    Json(body)
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/graph",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "Get project flow graph", body = inline(Value))
    )
)]
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

#[utoipa::path(
    post,
    path = "/projects/{project_id}/graph",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    request_body = inline(Value),
    responses(
        (status = 200, description = "Save project flow graph", body = inline(Value))
    )
)]
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

#[utoipa::path(
    get,
    path = "/graph",
    responses(
        (status = 200, description = "Get current graph state", body = inline(Value))
    )
)]
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

#[utoipa::path(
    post,
    path = "/graph",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Save graph state", body = inline(Value))
    )
)]
async fn save_graph(Query(q): Query<ProjectQuery>, Json(body): Json<Value>) -> Json<Value> {
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    let path = workspace::project_path(&proj, &["graph.json"], true).unwrap_or_default();
    if let Ok(str_val) = serde_json::to_string_pretty(&body) {
        let _ = fs::write(&path, str_val);
    }
    Json(json!({ "ok": true, "saved": true, "project": proj }))
}

#[utoipa::path(
    post,
    path = "/graph/run",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Execute graph workflow", body = inline(Value))
    )
)]
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

#[utoipa::path(
    post,
    path = "/graph/cancel",
    responses(
        (status = 200, description = "Cancel running graph", body = inline(Value))
    )
)]
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

#[utoipa::path(
    post,
    path = "/graph/validate",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Validate graph nodes and connections", body = inline(Value))
    )
)]
async fn validate_graph(Json(_body): Json<Value>) -> Json<Value> {
    Json(json!({ "valid": true, "errors": [] }))
}

#[utoipa::path(
    post,
    path = "/graph/cost",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Estimate graph run cost", body = inline(Value))
    )
)]
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
#[utoipa::path(
    get,
    path = "/video/templates",
    responses(
        (status = 200, description = "List available HyperFrames video templates", body = inline(Value))
    )
)]
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

#[utoipa::path(
    post,
    path = "/video/render",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Enqueue video render task", body = inline(Value))
    )
)]
async fn render_video(Json(body): Json<Value>) -> Json<Value> {
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
async fn get_render_status(AxumPath(job_id): AxumPath<String>) -> Json<Value> {
    Json(json!({ "jobId": job_id, "status": "completed", "progress": 100 }))
}

// Intelligence Handlers
#[utoipa::path(
    post,
    path = "/api/intel/ideas",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Generate KOC ideas and video hooks", body = inline(Value))
    )
)]
async fn generate_ideas(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({
        "ideas": [
            { "title": "Bí quyết chọn áo cardigan mùa thu cực xinh", "hook": "Đừng mua cardigan nếu chưa biết điều này!", "score": 9.2 },
            { "title": "Phối đồ phong cách tối giản thanh lịch", "hook": "3 cách phối đồ chuẩn Hàn Quốc chỉ với 1 chiếc áo", "score": 8.8 }
        ]
    }))
}

#[utoipa::path(
    post,
    path = "/api/intel/script",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Generate scene-by-scene KOC script", body = inline(Value))
    )
)]
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

#[utoipa::path(
    post,
    path = "/api/intel/compliance",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Validate script policy compliance", body = inline(Value))
    )
)]
async fn check_compliance(Json(body): Json<Value>) -> Json<Value> {
    let text = body.get("text").and_then(|v| v.as_str()).unwrap_or("");
    let violations = crate::intelligence::IntelligenceEngine::check_policy_compliance(text);
    Json(json!({ "compliant": violations.is_empty(), "violations": violations }))
}

// Voice Handlers
#[utoipa::path(
    get,
    path = "/voices",
    responses(
        (status = 200, description = "List all voices available in voice vault", body = inline(Value))
    )
)]
async fn get_voices_list() -> Json<Value> {
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
async fn get_speakers() -> Json<Value> {
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
async fn clone_voice(Json(body): Json<Value>) -> Json<Value> {
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
async fn synthesize_voice(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({ "ok": true, "audioUrl": "/assets/sample_speech.mp3", "duration": 3.5 }))
}
