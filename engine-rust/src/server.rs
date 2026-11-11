use axum::{
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

use crate::workspace;

pub async fn run_server(port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/info", get(get_workspace_info))
        .route("/api/v1/projects", get(list_projects))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 ViciEO Rust Backend listening on http://0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "engine": "vicieo-rust-native",
        "port": 9000
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
    Json(json!({
        "projects": projects
    }))
}

