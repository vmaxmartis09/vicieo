use super::prelude::*;

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
pub(crate) async fn get_project_graph_by_path(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
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
pub(crate) struct GraphSaveQuery {
    #[serde(alias = "baseUpdatedAt")]
    base_updated_at: Option<String>,
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
pub(crate) async fn save_project_graph_by_path(
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


#[utoipa::path(
    get,
    path = "/graph",
    responses(
        (status = 200, description = "Get current graph state", body = inline(Value))
    )
)]
pub(crate) async fn get_graph(Query(q): Query<ProjectQuery>) -> Json<Value> {
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
pub(crate) async fn save_graph(Query(q): Query<ProjectQuery>, Json(body): Json<Value>) -> Json<Value> {
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
pub(crate) async fn run_graph(Query(q): Query<ProjectQuery>, Json(body): Json<Value>) -> Json<Value> {
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
pub(crate) async fn cancel_graph(Query(q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "ok": true, "cancelled": true }))
}

pub(crate) async fn sse_events(
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
pub(crate) async fn validate_graph(Json(_body): Json<Value>) -> Json<Value> {
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
pub(crate) async fn estimate_cost(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({ "totalCostUsd": 0.0, "currency": "USD", "isFreeMode": true }))
}

// Data Catalog & Character Handlers

pub(crate) async fn run_project_graph(
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
            let ctx = crate::engine::executor::NodeCtx {
                project_id: project_id.clone(),
                run_id: run_id.clone(),
                node_id: node_id.clone(),
                node_type: node_type.clone(),
                custom_name: custom_name.clone(),
                params: params.clone(),
                inputs: Default::default(),
                tx: state.tx.clone(),
                cancel: tokio_util::sync::CancellationToken::new(),
                mock: true,
            };
            let (preview_type, preview_url, summary_text) = match state.executors.get(&node_type) {
                Some(ex) => match ex.execute(&ctx).await {
                    Ok(o) => (o.preview_type, o.preview_url, o.summary),
                    Err(e) => ("text".to_string(), None, format!("Lỗi: {}", e)),
                },
                None => unreachable!("fallback executor always resolves"),
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

pub(crate) async fn get_project_runs(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
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

pub(crate) async fn get_project_active_run(AxumPath(_project_id): AxumPath<String>) -> Json<Value> {
    Json(Value::Null)
}

pub(crate) async fn get_project_outputs(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
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

pub(crate) async fn get_node_types_list() -> Json<Value> {
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

pub(crate) async fn list_models() -> Json<Value> {
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

pub(crate) async fn list_providers() -> Json<Value> {
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
pub(crate) struct CustomNodeTypesQuery {
    #[serde(alias = "projectId")]
    project_id: Option<String>,
    scope: Option<String>,
}

pub(crate) async fn get_custom_node_types(Query(q): Query<CustomNodeTypesQuery>) -> Json<Value> {
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

pub(crate) async fn save_custom_node_types_handler(
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

pub(crate) async fn delete_custom_node_types_handler(
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

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/data/projects/{project_id}/graph", get(get_project_graph_by_path).post(save_project_graph_by_path))
        .route("/projects/{project_id}/graph", get(get_project_graph_by_path).post(save_project_graph_by_path))
        .route("/projects/{project_id}/run", post(run_project_graph))
        .route("/projects/{project_id}/run/cancel", post(cancel_graph))
        .route("/projects/{project_id}/events", get(sse_events))
        .route("/projects/{project_id}/estimate", post(estimate_cost))
        .route("/projects/{project_id}/runs", get(get_project_runs))
        .route("/projects/{project_id}/runs/active", get(get_project_active_run))
        .route("/projects/{project_id}/outputs", get(get_project_outputs))
        .route("/node-types", get(get_node_types_list))
        .route("/models", get(list_models))
        .route("/providers", get(list_providers))
        .route("/custom-node-types", get(get_custom_node_types).post(save_custom_node_types_handler))
        .route("/custom-node-types/{type_name}", delete(delete_custom_node_types_handler))
        .route("/graph", get(get_graph).post(save_graph))
        .route("/graph/run", post(run_graph))
        .route("/graph/cancel", post(cancel_graph))
        .route("/graph/events", get(sse_events))
        .route("/graph/validate", post(validate_graph))
        .route("/graph/cost", post(estimate_cost))
}
