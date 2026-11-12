//! HTTP routes, one module per feature track. Add new tracks by creating a module and one `.merge()` line below.
pub mod batch;
pub mod core;
pub mod editor;
pub mod graph;
pub mod intel;
pub mod products;
pub mod social;
pub mod sources;
pub mod video;
pub mod voice;
#[path = "../voice/mod.rs"] pub mod voice_impl;

pub(crate) mod prelude {
    pub(crate) use crate::state::AppState;
    pub(crate) use crate::workspace;
    pub(crate) use super::{guess_mime, is_valid_slug, ProjectQuery};
    pub(crate) use axum::{
        extract::{Path as AxumPath, Query},
        response::sse::{Event, KeepAlive, Sse},
        routing::{delete, get, post, put},
        Json, Router,
    };
    pub(crate) use futures::stream::Stream;
    pub(crate) use serde::{Deserialize, Serialize};
    pub(crate) use serde_json::{json, Value};
    pub(crate) use std::collections::HashMap;
    pub(crate) use std::convert::Infallible;
    pub(crate) use std::fs;
    pub(crate) use std::path::{Path, PathBuf};
    pub(crate) use std::sync::Arc;
    pub(crate) use std::time::Duration;
    pub(crate) use tokio::sync::broadcast;
    pub(crate) use axum::http::StatusCode;
    pub(crate) use axum::response::IntoResponse;
}

use prelude::*;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub(crate) fn is_valid_slug(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub(crate) fn guess_mime(path: &std::path::Path) -> &'static str {
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

#[derive(Deserialize)]
pub(crate) struct ProjectQuery {
    project: Option<String>,
}


#[derive(OpenApi)]
#[openapi(
    info(
        title = "ViciEO API Engine",
        version = "0.2.0",
        description = "Lightweight High-Performance Native AI Video Studio REST API"
    ),
    paths(
        core::health_check,
        core::get_workspace_info,
        core::list_projects,
        core::list_projects_data,
        core::get_project_detail,
        core::get_project_characters,
        core::save_project_character,
        core::get_project_channels,
        core::save_project_channels,
        products::get_project_products,
        products::get_project_koc_styles,
        products::create_product_review_flow,
        core::get_project_library,
        intel::get_project_policy,
        intel::save_project_policy,
        graph::get_project_graph_by_path,
        graph::save_project_graph_by_path,
        graph::get_graph,
        graph::save_graph,
        graph::run_graph,
        graph::cancel_graph,
        graph::validate_graph,
        graph::estimate_cost,
        video::get_video_templates,
        video::render_video,
        video::get_render_status,
        intel::generate_ideas,
        intel::generate_script,
        intel::check_compliance,
        voice::get_voices_list,
        voice::get_speakers,
        voice::clone_voice,
        voice::synthesize_voice,
    )
)]
struct ApiDoc;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(core::router())
        .merge(graph::router())
        .merge(video::router())
        .merge(editor::router())
        .merge(voice::router())
        .merge(intel::router())
        .merge(products::router())
        .merge(sources::router())
        .merge(social::router())
        .merge(batch::router())
        // NEW TRACK ROUTERS: add `.merge(xxx::router())` above this line
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
        )
}
