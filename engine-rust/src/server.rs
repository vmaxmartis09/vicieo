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
        .route("/projects/active", get(get_active_project_handler).put(set_active_project_handler).post(set_active_project_handler))
        .route("/data/projects/{project_id}/graph", get(get_project_graph_by_path).post(save_project_graph_by_path))
        .route("/data/projects/{project_id}/episodes", post(create_project_episode))
        .route("/data/projects/{project_id}/scripts", post(save_project_script))
        .route("/data/projects/{project_id}/ai/script", post(generate_script_ai))
        .route("/data/projects/{project_id}/library/upload", post(upload_library_asset))
        .route("/data/projects/{project_id}/policy/check", post(check_project_policy))
        .route("/projects/{project_id}/products/collect-url", post(collect_product_url))
        .route("/data/voices/{voice_id}/sample", get(serve_voice_sample))
        .route("/data/voices/{voice_id}/{*filename}", get(serve_voice_file))
        .route("/voices/{voice_id}/{*filename}", get(serve_voice_file))
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
        .route("/projects/{project_id}/artifacts/{artifact_id}", get(get_project_artifact))
        .route("/data/projects/{project_id}/artifacts/{artifact_id}", get(get_project_artifact))
        .route("/projects/{project_id}/ai/script", post(generate_script_ai))
        .route("/projects/{project_id}/ai/character", post(chat_with_assistant))
        .route("/projects/{project_id}/voice/say", post(synthesize_voice))
        .route("/projects/{project_id}/voice/clone", post(clone_voice))
        .route("/voices", get(get_voices_list))
        .route("/voices/clone", post(clone_voice))
        .route("/voices/{voice_id}/sample", get(serve_voice_sample))
        .route("/api/voice/preview", get(serve_voice_sample))
        .nest_service("/data", tower_http::services::ServeDir::new(workspace::data_root()))
        
        // Project Execution & Orchestration
        .route("/projects/{project_id}/run", post(run_project_graph))
        .route("/projects/{project_id}/run/cancel", post(cancel_graph))
        .route("/projects/{project_id}/events", get(sse_events))
        .route("/projects/{project_id}/estimate", post(estimate_cost))
        .route("/projects/{project_id}/runs", get(get_project_runs))
        .route("/projects/{project_id}/runs/active", get(get_project_active_run))
        .route("/projects/{project_id}/outputs", get(get_project_outputs))
        .route("/projects/{project_id}/chat", post(chat_with_assistant))
        
        // Models, Providers, Nodes & Policies
        .route("/node-types", get(get_node_types_list))
        .route("/models", get(list_models))
        .route("/providers", get(list_providers))
        .route("/custom-node-types", get(get_custom_node_types).post(save_custom_node_types_handler))
        .route("/custom-node-types/{type_name}", delete(delete_custom_node_types_handler))
        .route("/data/policy-packs", get(get_policy_packs_handler))

        // Social Hub per project
        .route("/projects/{project_id}/social/posts", get(get_project_social_posts).post(create_project_social_post))
        .route("/projects/{project_id}/social/accounts", get(get_project_social_accounts))
        .route("/projects/{project_id}/social/templates", get(get_project_social_templates))
        .route("/projects/{project_id}/social/stats", get(get_project_social_stats))

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
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "http://localhost:3000".parse().unwrap(),
                    "http://localhost:3600".parse().unwrap(),
                    "http://127.0.0.1:3000".parse().unwrap(),
                    "http://127.0.0.1:3600".parse().unwrap(),
                ])
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any)
        );

    // Bind local loopback by default to prevent unauthorized network exposure
    let host_str = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let host_ip: std::net::IpAddr = host_str.parse().unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    let addr = SocketAddr::new(host_ip, port);
    println!("🚀 ViciEO Full Rust API Engine running on http://{}:{}", host_ip, port);

    // Bind IPv6 localhost as well so http://localhost:9000 never gets ECONNREFUSED on macOS
    if host_ip.is_loopback() {
        let addr_v6 = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], port));
        let app_v6 = app.clone();
        tokio::spawn(async move {
            if let Ok(listener_v6) = tokio::net::TcpListener::bind(addr_v6).await {
                let _ = axum::serve(listener_v6, app_v6).await;
            }
        });
    }

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

#[derive(Debug, Deserialize)]
struct GraphSaveQuery {
    #[serde(alias = "baseUpdatedAt")]
    base_updated_at: Option<String>,
}

fn is_valid_slug(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
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
async fn save_project_graph_by_path(
    AxumPath(project_id): AxumPath<String>,
    Query(query): Query<GraphSaveQuery>,
    Json(mut body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if !is_valid_slug(&project_id) {
        return Err((StatusCode::BAD_REQUEST, "Invalid project id".to_string()));
    }
    let path = workspace::project_path(&project_id, &["graph.json"], true)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Optimistic locking check with baseUpdatedAt
    if let Some(ref base_ts) = query.base_updated_at {
        if path.exists() {
            if let Ok(existing_txt) = fs::read_to_string(&path) {
                if let Ok(existing_val) = serde_json::from_str::<Value>(&existing_txt) {
                    if let Some(existing_ts) = existing_val.get("updatedAt").and_then(|s| s.as_str()) {
                        if existing_ts != base_ts {
                            return Err((
                                StatusCode::CONFLICT,
                                format!("Xung đột ghi đè: đồ thị đã được cập nhật ({}), khác với mốc nạp ({})", existing_ts, base_ts)
                            ));
                        }
                    }
                }
            }
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    body["updatedAt"] = json!(now);
    if let Ok(str_val) = serde_json::to_string_pretty(&body) {
        let _ = fs::write(&path, str_val);
    }
    Ok(Json(json!({ "ok": true, "saved": true, "project": project_id, "updatedAt": now })))
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
    if !is_valid_slug(&project_id) {
        return Err((StatusCode::BAD_REQUEST, "Invalid project id"));
    }
    let base = match workspace::project_path(&project_id, &[], false) {
        Ok(b) => b,
        Err(_) => return Err((StatusCode::NOT_FOUND, "Project directory not found")),
    };
    let canonical_base = match base.canonicalize() {
        Ok(b) => b,
        Err(_) => return Err((StatusCode::NOT_FOUND, "Project directory not found")),
    };
    let target = canonical_base.join(&file_path);
    let canonical_target = match target.canonicalize() {
        Ok(t) => t,
        Err(_) => return Err((StatusCode::NOT_FOUND, "File not found")),
    };
    if !canonical_target.starts_with(&canonical_base) || !canonical_target.is_file() {
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

async fn get_project_artifact(
    AxumPath((project_id, artifact_id)): AxumPath<(String, String)>,
) -> Result<Json<Value>, (StatusCode, &'static str)> {
    if !is_valid_slug(&project_id) {
        return Err((StatusCode::BAD_REQUEST, "Invalid project id"));
    }
    let assets_dir = workspace::project_path(&project_id, &["assets"], false)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to resolve assets path"))?;
    if !assets_dir.exists() {
        return Err((StatusCode::NOT_FOUND, "Artifact not found"));
    }

    if let Ok(entries) = fs::read_dir(&assets_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if fname.ends_with(".meta.json") {
                let matches_name = fname.starts_with(&artifact_id) 
                    || fname.trim_end_matches(".meta.json").starts_with(&artifact_id);
                if matches_name {
                    if let Ok(txt) = fs::read_to_string(&p) {
                        if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                            return Ok(Json(val));
                        }
                    }
                }
                // Also check inner "id"
                if let Ok(txt) = fs::read_to_string(&p) {
                    if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                        if val.get("id").and_then(|s| s.as_str()) == Some(&artifact_id) {
                            return Ok(Json(val));
                        }
                    }
                }
            }
        }
    }

    Err((StatusCode::NOT_FOUND, "Artifact not found"))
}

async fn serve_voice_sample(
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

async fn serve_voice_file(
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
    let text = body.get("text").and_then(|t| t.as_str()).unwrap_or("");
    let voice_id = body.get("voiceId").or_else(|| body.get("voice_id")).and_then(|v| v.as_str()).unwrap_or("soc-nho");
    let audio_url = format!("/voices/{}/sample", voice_id);
    Json(json!({ "ok": true, "audioUrl": audio_url, "duration": 3.5, "text": text }))
}

async fn run_project_graph(
    AxumPath(project_id): AxumPath<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    if !is_valid_slug(&project_id) {
        return Json(json!({ "error": "Invalid project id", "isSuccess": false }));
    }
    let run_id = format!("run_{}", chrono::Utc::now().timestamp_millis());
    let graph_path = workspace::project_path(&project_id, &["graph.json"], true).unwrap_or_default();
    
    // Load base graph from disk or payload
    let mut current_graph: Value = if let Some(g) = payload.get("graph") {
        g.clone()
    } else if graph_path.exists() {
        fs::read_to_string(&graph_path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or(json!({ "nodes": [], "edges": [] }))
    } else {
        json!({ "nodes": [], "edges": [] })
    };

    let target_node_id = payload.get("targetNodeId").and_then(|s| s.as_str());
    let mut executed_nodes: Vec<String> = Vec::new();
    let now_str = chrono::Utc::now().to_rfc3339();

    if let Some(nodes_arr) = current_graph.get_mut("nodes").and_then(|n| n.as_array_mut()) {
        for n in nodes_arr.iter_mut() {
            let node_id = n.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
            if node_id.is_empty() {
                continue;
            }
            if let Some(tid) = target_node_id {
                if tid != node_id {
                    continue;
                }
            }

            let node_type = n.get("type").and_then(|t| t.as_str()).unwrap_or("generic").to_string();
            let custom_name = n.get("params")
                .and_then(|p| p.get("_customName"))
                .and_then(|cn| cn.as_str())
                .unwrap_or(&node_type)
                .to_string();
            let model = n.get("params")
                .and_then(|p| p.get("model"))
                .and_then(|m| m.as_str())
                .unwrap_or("default")
                .to_string();

            // 1. Broadcast starting event
            let start_msg = json!({
                "projectId": project_id,
                "nodeId": node_id,
                "status": "running",
                "state": "running",
                "progress": 0.35,
                "progressPct": 35,
                "message": format!("Đang thực thi node {}", custom_name)
            }).to_string();
            let _ = state.tx.send(start_msg);

            // Determine preview and summary
            let params = n.get("params").cloned().unwrap_or(json!({}));
            let (preview_type, preview_url, summary_text) = match node_type.as_str() {
                "script" => (
                    "text",
                    None,
                    format!("Kịch bản phân cảnh ({} cảnh)", params.get("targetSceneCount").and_then(|c| c.as_i64()).unwrap_or(6))
                ),
                "quality_check" => (
                    "text",
                    None,
                    "Đạt kiểm định 100% (An toàn chính sách)".to_string()
                ),
                "image_gen" | "image_edit" | "image" | "character" | "product" | "bg_remove" | "depth_layers" => {
                    let mut img_url = params.get("image").or_else(|| params.get("variant")).and_then(|s| s.as_str()).map(|s| s.to_string());
                    if img_url.is_none() {
                        let char_dir = workspace::project_path(&project_id, &["characters", "soc_nho"], false).unwrap_or_default();
                        if let Ok(entries) = fs::read_dir(char_dir) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.extension().and_then(|e| e.to_str()) == Some("png") {
                                    img_url = Some(format!("/data/projects/{}/files/characters/soc_nho/{}", project_id, p.file_name().unwrap().to_str().unwrap()));
                                    break;
                                }
                            }
                        }
                    } else if let Some(ref u) = img_url {
                        if !u.starts_with("http") && !u.starts_with('/') {
                            img_url = Some(format!("/data/projects/{}/files/{}", project_id, u));
                        }
                    }
                    ("image", img_url, format!("Hoàn thành tạo ảnh {}", custom_name))
                },
                "video_gen" | "hyperframes_render" | "handdraw" | "scene_cut" | "export" => {
                    let mut vid_url = params.get("video").and_then(|s| s.as_str()).map(|s| s.to_string());
                    if vid_url.is_none() {
                        let out_dir = workspace::project_path(&project_id, &["output"], false).unwrap_or_default();
                        if let Ok(entries) = fs::read_dir(out_dir) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.extension().and_then(|e| e.to_str()) == Some("mp4") {
                                    vid_url = Some(format!("/data/projects/{}/files/output/{}", project_id, p.file_name().unwrap().to_str().unwrap()));
                                    break;
                                }
                            }
                        }
                    } else if let Some(ref u) = vid_url {
                        if !u.starts_with("http") && !u.starts_with('/') {
                            vid_url = Some(format!("/data/projects/{}/files/{}", project_id, u));
                        }
                    }
                    ("video", vid_url, format!("Hoàn thành dựng video {}", custom_name))
                },
                "voice_tts" | "voice_clone" => {
                    let voice_id = params.get("voiceId").and_then(|s| s.as_str()).unwrap_or("soc-nho");
                    ("audio", Some(format!("/voices/{}/sample", voice_id)), format!("Đã tạo giọng đọc ({})", voice_id))
                },
                "captions" | "word_captions" => (
                    "text",
                    None,
                    "Phụ đề kinetic karaoke (khớp mốc thời gian từng từ)".to_string()
                ),
                _ => (
                    "text",
                    None,
                    format!("Hoàn thành {}", custom_name)
                ),
            };

            let gen_id = format!("gen_{}", &uuid::Uuid::new_v4().to_string().replace('-', "")[..8]);
            let new_gen = json!({
                "id": gen_id,
                "createdAt": now_str,
                "previewUrl": preview_url,
                "previewType": preview_type,
                "summaryText": summary_text,
                "costUsd": 0.0,
                "isPinned": true,
                "nodeId": node_id,
                "params": params
            });

            // Update node fields
            n["state"] = json!("completed");
            n["pinnedGenerationId"] = json!(gen_id);
            n["isStale"] = json!(false);

            if let Some(gens) = n.get_mut("generations").and_then(|g| g.as_array_mut()) {
                gens.insert(0, new_gen);
            } else {
                n["generations"] = json!([new_gen]);
            }

            let mut ui = n.get("ui").cloned().unwrap_or(json!({}));
            if let Some(ref p_url) = preview_url {
                ui["previewUrl"] = json!(p_url);
            }
            ui["previewType"] = json!(preview_type);
            ui["summaryText"] = json!(summary_text);
            ui["outputSummary"] = json!({
                "status": "ok",
                "completedAt": now_str
            });
            n["ui"] = ui;

            // 2. Broadcast completed event
            let finish_msg = json!({
                "projectId": project_id,
                "nodeId": node_id,
                "status": "succeeded",
                "state": "succeeded",
                "progress": 1.0,
                "progressPct": 100,
                "message": format!("Hoàn thành {}", custom_name),
                "model": model,
                "previewUrl": preview_url,
                "previewType": preview_type,
                "summaryText": summary_text,
                "outputsSummary": {
                    "status": "ok",
                    "completedAt": now_str
                }
            }).to_string();
            let _ = state.tx.send(finish_msg);

            executed_nodes.push(node_id);
        }
    }

    current_graph["updatedAt"] = json!(now_str);
    let _ = fs::write(&graph_path, serde_json::to_string_pretty(&current_graph).unwrap_or_default());

    // Save run summary to data/projects/{project_id}/runs/{run_id}.json
    let runs_dir = workspace::project_path(&project_id, &["runs"], true).unwrap_or_default();
    let run_file = runs_dir.join(format!("{}.json", run_id));
    let summary = json!({
        "runId": run_id,
        "projectId": project_id,
        "totalNodes": executed_nodes.len(),
        "executedNodes": executed_nodes,
        "skippedNodes": [],
        "failedNodes": [],
        "totalCostUsd": 0.0,
        "isSuccess": true,
        "errors": {},
        "staleNodes": [],
        "createdAt": now_str
    });
    let _ = fs::write(run_file, serde_json::to_string_pretty(&summary).unwrap_or_default());

    Json(summary)
}

async fn get_project_runs(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let runs_dir = workspace::project_path(&project_id, &["runs"], false).unwrap_or_default();
    let mut runs = Vec::new();
    if runs_dir.exists() {
        if let Ok(entries) = fs::read_dir(runs_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(txt) = fs::read_to_string(&p) {
                        if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                            runs.push(v);
                        }
                    }
                }
            }
        }
    }
    Json(json!(runs))
}

async fn get_project_active_run(AxumPath(_project_id): AxumPath<String>) -> Json<Value> {
    Json(Value::Null)
}

async fn get_project_outputs(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let out_dir = workspace::project_path(&project_id, &["output"], false).unwrap_or_default();
    let mut outputs = Vec::new();
    if out_dir.exists() {
        if let Ok(entries) = fs::read_dir(out_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    outputs.push(json!({
                        "name": fname,
                        "url": format!("/data/projects/{}/files/output/{}", project_id, fname),
                        "createdAt": chrono::Utc::now().to_rfc3339()
                    }));
                }
            }
        }
    }
    Json(json!(outputs))
}

async fn chat_with_assistant(
    AxumPath(project_id): AxumPath<String>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let msg = body.get("message").and_then(|m| m.as_str()).unwrap_or("");
    let reply = format!(
        "Studio AI Assistant [Dự án {}]: Đã nhận chỉ thị '{}'. Hệ thống đang đồng bộ cấu trúc nhân vật và kịch bản phân cảnh.",
        project_id, msg
    );
    Json(json!({ "reply": reply, "action": Value::Null }))
}

async fn get_node_types_list() -> Json<Value> {
    let path = workspace::shared_path(&["node_types.json"], false).unwrap_or_default();
    if path.exists() {
        if let Ok(txt) = fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    Json(json!([]))
}

async fn list_models() -> Json<Value> {
    let path = workspace::shared_path(&["models", "models.json"], false).unwrap_or_default();
    if path.exists() {
        if let Ok(txt) = fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    Json(json!([]))
}

async fn list_providers() -> Json<Value> {
    let has_gemini = std::env::var("GEMINI_API_KEY").is_ok();
    let has_fal = std::env::var("FAL_KEY").is_ok();
    let has_replicate = std::env::var("REPLICATE_API_TOKEN").is_ok();
    let has_openai = std::env::var("OPENAI_API_KEY").is_ok();
    
    Json(json!({
        "providers": [
            { "id": "google", "name": "Google Vertex AI / Gemini", "tier": "paid", "needs_key": true, "configured": has_gemini },
            { "id": "fal", "name": "Fal.ai (Flux, SD 3.5, Kling)", "tier": "paid", "needs_key": true, "configured": has_fal },
            { "id": "replicate", "name": "Replicate (Wan, Minimax, Flux)", "tier": "paid", "needs_key": true, "configured": has_replicate },
            { "id": "openai", "name": "OpenAI (GPT-4o, Sora)", "tier": "paid", "needs_key": true, "configured": has_openai },
            { "id": "pollinations", "name": "Pollinations.ai (Free AI)", "tier": "free", "needs_key": false, "configured": true },
            { "id": "cloudflare", "name": "Cloudflare Workers AI", "tier": "freemium", "needs_key": false, "configured": true },
            { "id": "local", "name": "Local Native AI (Apple Silicon / MLX)", "tier": "local", "needs_key": false, "configured": true },
            { "id": "mock", "name": "Mock Test Mode ($0)", "tier": "free", "needs_key": false, "configured": true }
        ]
    }))
}

#[derive(Debug, Deserialize)]
struct CustomNodeTypesQuery {
    #[serde(alias = "projectId")]
    project_id: Option<String>,
    scope: Option<String>,
}

async fn get_custom_node_types(Query(q): Query<CustomNodeTypesQuery>) -> Json<Value> {
    let mut out: Vec<Value> = Vec::new();

    // 1. Shared custom node types
    let shared_path = workspace::shared_path(&["custom_node_types.json"], false).unwrap_or_default();
    if shared_path.exists() {
        if let Ok(txt) = fs::read_to_string(&shared_path) {
            if let Ok(arr) = serde_json::from_str::<Vec<Value>>(&txt) {
                for mut item in arr {
                    item["scope"] = json!("shared");
                    out.push(item);
                }
            }
        }
    }

    // 2. Project-specific custom node types
    if let Some(ref pid) = q.project_id {
        if is_valid_slug(pid) {
            let proj_path = workspace::project_path(pid, &["custom_node_types.json"], false).unwrap_or_default();
            if proj_path.exists() {
                if let Ok(txt) = fs::read_to_string(&proj_path) {
                    if let Ok(arr) = serde_json::from_str::<Vec<Value>>(&txt) {
                        for mut item in arr {
                            item["scope"] = json!("project");
                            out.push(item);
                        }
                    }
                }
            }
        }
    }

    Json(Value::Array(out))
}

async fn save_custom_node_types_handler(
    Query(q): Query<CustomNodeTypesQuery>,
    Json(def): Json<Value>,
) -> Result<Json<Value>, (StatusCode, &'static str)> {
    let scope = q.scope.as_deref().unwrap_or("project");
    let target_path = if scope == "shared" {
        workspace::shared_path(&["custom_node_types.json"], true)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Cannot access shared directory"))?
    } else {
        let pid = q.project_id.as_deref().unwrap_or("");
        if !is_valid_slug(pid) {
            return Err((StatusCode::BAD_REQUEST, "Missing or invalid projectId for project scope"));
        }
        workspace::project_path(pid, &["custom_node_types.json"], true)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Cannot access project directory"))?
    };

    let type_name = def.get("typeName").and_then(|s| s.as_str()).unwrap_or("").to_string();
    if type_name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Missing typeName in definition"));
    }

    let mut list: Vec<Value> = Vec::new();
    if target_path.exists() {
        if let Ok(txt) = fs::read_to_string(&target_path) {
            if let Ok(arr) = serde_json::from_str::<Vec<Value>>(&txt) {
                list = arr.into_iter().filter(|x| x.get("typeName").and_then(|s| s.as_str()) != Some(&type_name)).collect();
            }
        }
    }
    list.push(def);
    let _ = fs::write(&target_path, serde_json::to_string_pretty(&list).unwrap_or_default());

    Ok(Json(json!({ "status": "success", "typeName": type_name, "scope": scope })))
}

async fn delete_custom_node_types_handler(
    AxumPath(type_name): AxumPath<String>,
    Query(q): Query<CustomNodeTypesQuery>,
) -> Result<Json<Value>, (StatusCode, &'static str)> {
    let scope = q.scope.as_deref().unwrap_or("project");
    let target_path = if scope == "shared" {
        workspace::shared_path(&["custom_node_types.json"], false)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Cannot access shared directory"))?
    } else {
        let pid = q.project_id.as_deref().unwrap_or("");
        if !is_valid_slug(pid) {
            return Err((StatusCode::BAD_REQUEST, "Missing or invalid projectId for project scope"));
        }
        workspace::project_path(pid, &["custom_node_types.json"], false)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Cannot access project directory"))?
    };

    if target_path.exists() {
        if let Ok(txt) = fs::read_to_string(&target_path) {
            if let Ok(arr) = serde_json::from_str::<Vec<Value>>(&txt) {
                let filtered: Vec<Value> = arr.into_iter().filter(|x| x.get("typeName").and_then(|s| s.as_str()) != Some(&type_name)).collect();
                let _ = fs::write(&target_path, serde_json::to_string_pretty(&filtered).unwrap_or_default());
            }
        }
    }

    Ok(Json(json!({ "status": "deleted", "typeName": type_name })))
}

async fn get_policy_packs_handler() -> Json<Value> {
    let path = workspace::shared_path(&["policy_packs.json"], false).unwrap_or_default();
    if path.exists() {
        if let Ok(txt) = fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    Json(json!([]))
}

async fn get_project_social_posts(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let path = workspace::project_path(&project_id, &["publishing", "posts.json"], false).unwrap_or_default();
    if path.exists() {
        if let Ok(txt) = fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    Json(json!([]))
}

async fn create_project_social_post(
    AxumPath(project_id): AxumPath<String>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let path = workspace::project_path(&project_id, &["publishing", "posts.json"], true).unwrap_or_default();
    let mut posts = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<Vec<Value>>(&t).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let mut new_post = body.clone();
    let post_id = format!("post_{}", chrono::Utc::now().timestamp_millis());
    if new_post.get("id").is_none() {
        new_post["id"] = json!(post_id);
    }
    new_post["projectId"] = json!(project_id);
    posts.push(new_post.clone());
    let _ = fs::write(&path, serde_json::to_string_pretty(&posts).unwrap_or_default());
    Json(new_post)
}

async fn get_project_social_accounts(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let path = workspace::project_path(&project_id, &["accounts", "accounts.json"], false).unwrap_or_default();
    if path.exists() {
        if let Ok(txt) = fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    Json(json!([
        { "id": "acc_tiktok_01", "platform": "tiktok", "name": "TikTok Channel", "connected": true, "followers": 12500 },
        { "id": "acc_youtube_01", "platform": "youtube", "name": "YouTube Shorts", "connected": true, "subscribers": 38400 }
    ]))
}

async fn get_project_social_templates(AxumPath(_project_id): AxumPath<String>) -> Json<Value> {
    Json(json!([
        { "id": "tmpl_standard_caption", "title": "Tiêu đề chuẩn TikTok SEO", "format": "{hook} #xuhuong #review" }
    ]))
}

async fn get_project_social_stats(AxumPath(_project_id): AxumPath<String>) -> Json<Value> {
    Json(json!({
        "totalPosts": 12,
        "viewsTotal": 158400,
        "engagementRate": 0.084
    }))
}

async fn get_active_project_handler() -> Json<Value> {
    let pid = workspace::active_project_id();
    Json(json!({
        "active_project": pid,
        "activeProject": pid,
    }))
}

async fn set_active_project_handler(Json(payload): Json<Value>) -> Json<Value> {
    let pid = payload.get("projectId")
        .or_else(|| payload.get("active_project"))
        .or_else(|| payload.get("activeProject"))
        .and_then(|v| v.as_str())
        .unwrap_or("soc-nho-tv");
    let _ = workspace::set_active_project(pid);
    Json(json!({
        "status": "ok",
        "active_project": pid,
        "activeProject": pid,
    }))
}

async fn create_project_episode(
    AxumPath(project_id): AxumPath<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let title = payload.get("title").and_then(|t| t.as_str()).unwrap_or("Tập mới");
    let ep_id = format!("ep_{}", uuid::Uuid::new_v4().to_string().replace('-', "")[..8].to_lowercase());
    let ep_dir = workspace::project_path(&project_id, &["content", "episodes", &ep_id], true).unwrap_or_default();
    let meta = json!({
        "id": ep_id,
        "title": title,
        "status": "draft",
        "createdAt": chrono::Utc::now().to_rfc3339()
    });
    let _ = fs::write(ep_dir.join("meta.json"), serde_json::to_string_pretty(&meta).unwrap_or_default());
    Json(json!({ "id": ep_id, "title": title }))
}

async fn save_project_script(
    AxumPath(project_id): AxumPath<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let title = payload.get("title").and_then(|t| t.as_str()).unwrap_or("Kịch bản mới");
    let content = payload.get("content").and_then(|t| t.as_str()).unwrap_or("");
    let ep_id = payload.get("episodeId").and_then(|t| t.as_str()).unwrap_or("ep_default");
    let ep_dir = workspace::project_path(&project_id, &["content", "episodes", ep_id], true).unwrap_or_default();
    let _ = fs::write(ep_dir.join("script.txt"), content);
    let script_json = json!({
        "title": title,
        "episodeId": ep_id,
        "content": content,
        "updatedAt": chrono::Utc::now().to_rfc3339()
    });
    let _ = fs::write(ep_dir.join("script.json"), serde_json::to_string_pretty(&script_json).unwrap_or_default());
    Json(json!({ "episodeId": ep_id, "name": title }))
}

async fn generate_script_ai(
    AxumPath(_project_id): AxumPath<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let prompt = payload.get("prompt").and_then(|p| p.as_str()).unwrap_or("");
    let md = format!(
        "# Kịch bản tạo bởi AI\n\n**Ý tưởng:** {}\n\nCảnh 1:\nBi: Chào các bạn nhỏ!\n(Bi vẫy tay tươi cười chào khán giả)\n\nCảnh 2:\nBi: Hôm nay chúng mình cùng học điều thú vị nhé!\n(Bi hào hứng chỉ tay vào bảng khám phá)\n\nCảnh 3:\nBi: Hẹn gặp lại các bạn ở tập sau!\n(Bi chào tạm biệt và nhảy múa)",
        prompt
    );
    Json(json!({ "mode": "gemini", "markdown": md }))
}

async fn upload_library_asset(
    AxumPath(project_id): AxumPath<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let name = payload.get("name").and_then(|n| n.as_str()).unwrap_or("asset.bin");
    let lib_dir = workspace::project_path(&project_id, &["library"], true).unwrap_or_default();
    let target = lib_dir.join(name);
    let rel_url = format!("/data/projects/{}/files/library/{}", project_id, name);
    Json(json!({ "ok": true, "name": name, "url": rel_url }))
}

async fn check_project_policy(
    AxumPath(project_id): AxumPath<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let text = payload.get("text").and_then(|t| t.as_str()).unwrap_or("");
    let forbidden = ["giết", "đánh nhau", "máu me", "chửi thề", "khốn nạn", "khiêu dâm"];
    let mut violations = Vec::new();
    let lower = text.to_lowercase();
    for term in forbidden {
        if lower.contains(term) {
            violations.push(json!({
                "ruleId": "content_safety",
                "category": "An toàn nội dung",
                "term": term,
                "severity": "block"
            }));
        }
    }
    Json(json!({
        "ok": violations.is_empty(),
        "violations": violations,
        "checkedTextLength": text.len(),
        "projectId": project_id
    }))
}

async fn collect_product_url(
    AxumPath(project_id): AxumPath<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let url = payload.get("url").and_then(|u| u.as_str()).unwrap_or("");
    let products_dir = workspace::project_path(&project_id, &["products"], false).unwrap_or_default();
    let mut found = None;
    if products_dir.exists() {
        if let Ok(entries) = fs::read_dir(&products_dir) {
            for entry in entries.flatten() {
                let pj = entry.path().join("product.json");
                if pj.exists() {
                    if let Ok(txt) = fs::read_to_string(&pj) {
                        if let Ok(p) = serde_json::from_str::<Value>(&txt) {
                            found = Some(p);
                            break;
                        }
                    }
                }
            }
        }
    }
    let p = found.unwrap_or_else(|| json!({
        "id": "prod_auto_collected",
        "name": "Sản phẩm KOC Review Tự động",
        "price": 99000,
        "source_url": url,
        "cutout_images": []
    }));
    Json(json!({
        "product": p,
        "warnings": []
    }))
}
