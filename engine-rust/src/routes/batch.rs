//! Batch episode runs (port of api/routes/batch.py + engine/batch_runner.py).
//! Shortcut: episodes run sequentially (one active graph run per project, as in the run registry); the
//! `concurrency` field is accepted but ignored. Upgrade by keying active runs by run id.
use super::graph::ApiErr;
use super::prelude::*;
use crate::engine::{cost, graph_runner as gr};
use once_cell::sync::Lazy;
use std::sync::Mutex;

static CANCELS: Lazy<Mutex<HashMap<String, (String, bool)>>> = Lazy::new(Default::default); // batch_id -> (project, cancelled)

fn err(code: StatusCode, m: impl Into<String>) -> ApiErr {
    (code, Json(json!({ "detail": crate::providers::key_vault::redact(&m.into()) })))
}

struct Cfg {
    episodes: Vec<String>,
    mock: bool,
    confirm: bool,
    body: Value,
}

fn parse(body: &Value) -> Result<Cfg, ApiErr> {
    let eps: Vec<String> = body.get("episodeIds").or_else(|| body.get("episode_ids")).and_then(|v| v.as_array()).into_iter().flatten()
        .filter_map(|e| e.as_str().map(|x| x.trim().to_string())).filter(|e| !e.is_empty()).collect();
    if eps.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Danh sách 'episodeIds' không được để trống."));
    }
    let mock = ["mockMode", "mock", "isMockMode"].iter().find_map(|k| body.get(*k).and_then(|v| v.as_bool())).unwrap_or(true);
    let confirm = ["confirmPaidRun", "confirm_paid_run", "confirmed"].iter().any(|k| body[*k] == true);
    Ok(Cfg { episodes: eps, mock, confirm, body: body.clone() })
}

fn base_graph(project: &str, body: &Value) -> Value {
    let mut g = body.get("graph").filter(|g| g.is_object()).cloned().unwrap_or_else(|| {
        workspace::project_path(project, &["graph.json"], false).ok().and_then(|p| fs::read_to_string(p).ok())
            .and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(json!({"nodes": [], "edges": []}))
    });
    if g["id"].as_str().unwrap_or("").is_empty() {
        g["id"] = json!(format!("graph_{}", project));
    }
    g["projectId"] = json!(project);
    g
}

fn clone_for_episode(base: &Value, ep: &str) -> Value {
    let mut g = base.clone();
    g["id"] = json!(format!("{}__{}", base["id"].as_str().unwrap_or("graph"), ep));
    for n in g["nodes"].as_array_mut().into_iter().flatten() {
        let t = n["type"].as_str().unwrap_or("").to_string();
        if t == "script" {
            n["params"]["mode"] = json!("episode");
            n["params"]["episodeId"] = json!(ep);
        } else if matches!(t.as_str(), "cast" | "export" | "hyperframes_render") || n["params"].get("episodeId").is_some() {
            n["params"]["episodeId"] = json!(ep);
        }
        if t == "export" {
            let raw = n["params"]["outputFileName"].as_str().unwrap_or("").trim().to_string();
            if raw.is_empty() || raw == "final_video.mp4" || raw.ends_with(".mp4") {
                n["params"]["outputFileName"] = json!(format!("{}.mp4", ep));
            }
            n["params"]["outputSlug"] = json!(ep);
        }
    }
    g
}

fn estimate(project: &str, base: &Value, cfg: &Cfg) -> Value {
    let (mut total, mut per, mut missing) = (0.0f64, serde_json::Map::new(), std::collections::BTreeSet::new());
    for ep in &cfg.episodes {
        let g = clone_for_episode(base, ep);
        let mut b = cfg.body.clone();
        b["mode"] = json!("all");
        b.as_object_mut().map(|m| m.remove("targetNodeId"));
        let req = gr::RunReq::from_value(project, &b);
        match cost::plan(project, &g, &req) {
            Ok(p) => {
                let c = p["totalEstimatedCostUsd"].as_f64().unwrap_or(0.0);
                per.insert(ep.clone(), json!(c));
                total += c;
                p["missingApiKeys"].as_array().into_iter().flatten().filter_map(|k| k.as_str()).for_each(|k| {
                    missing.insert(k.to_string());
                });
            }
            Err(_) => {
                per.insert(ep.clone(), json!(0.0));
            }
        }
    }
    let runs = workspace::project_path(project, &["runs"], false).unwrap_or_default();
    let spend = crate::providers::spend_guard::SpendGuard::new(runs).preview(total);
    let (mut can, mut reason) = (true, Value::Null);
    if !cfg.mock {
        if !missing.is_empty() {
            can = false;
            reason = json!(format!("Thiếu khóa API hợp lệ cho: {}", missing.iter().cloned().collect::<Vec<_>>().join(", ")));
        } else if spend["canProceed"] == false {
            can = false;
            reason = spend["reason"].clone();
        }
    }
    json!({"totalEpisodes": cfg.episodes.len(), "totalEstimatedCostUsd": (total * 1e4).round() / 1e4, "perEpisodeEstimates": per,
        "missingApiKeys": missing, "spendStatus": spend, "canProceed": can, "requiresConfirmation": !cfg.mock && total > 0.0 && !cfg.confirm, "reason": reason})
}

async fn estimate_episodes(AxumPath(project): AxumPath<String>, Json(body): Json<Value>) -> Result<Json<Value>, ApiErr> {
    let cfg = parse(&body)?;
    Ok(Json(estimate(&project, &base_graph(&project, &body), &cfg)))
}

fn report_dir(project: &str) -> PathBuf {
    workspace::project_path(project, &["runs", "_batch_runs"], false).map(|d| { let _ = fs::create_dir_all(&d); d }).unwrap_or_default()
}

async fn run_episodes(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxumPath(project): AxumPath<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if !is_valid_slug(&project) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid project id"));
    }
    let cfg = parse(&body)?;
    let base = base_graph(&project, &body);
    let est = estimate(&project, &base, &cfg);
    if !cfg.mock {
        if est["missingApiKeys"].as_array().map(|a| !a.is_empty()).unwrap_or(false) {
            return Err(err(StatusCode::BAD_REQUEST, format!("Thiếu khóa API hợp lệ cho provider: {}", est["missingApiKeys"])));
        }
        if est["canProceed"] == false {
            return Err(err(StatusCode::BAD_REQUEST, est["reason"].as_str().unwrap_or("Vượt trần chi tiêu cho phép.")));
        }
        if est["requiresConfirmation"] == true {
            return Err((StatusCode::CONFLICT, Json(json!({"detail": "Cần xác nhận chi phí trước khi gọi model trả phí (truyền confirmPaidRun=true).", "estimate": est}))));
        }
    }
    let batch_id = format!("batch_{}_{}", chrono::Utc::now().timestamp(), &uuid::Uuid::new_v4().to_string().replace('-', "")[..6]);
    CANCELS.lock().unwrap_or_else(|e| e.into_inner()).insert(batch_id.clone(), (project.clone(), false));
    let started = chrono::Utc::now();
    let ev = |kind: &'static str, msg: String, progress: Option<f64>| {
        gr::emit(&state.tx, gr::Ev { run: &batch_id, project: &project, kind, node: None, state: None, progress, message: Some(msg), extra: json!({}) });
    };
    ev("run_started", format!("Bắt đầu chạy batch {} tập (Chế độ: {})", cfg.episodes.len(), if cfg.mock { "Mock" } else { "Live" }), None);
    let reg = gr::registry_with_track_a(&state.executors);
    let mut results: Vec<Value> = vec![];
    for (i, ep) in cfg.episodes.iter().enumerate() {
        let cancelled = || CANCELS.lock().unwrap_or_else(|e| e.into_inner()).get(&batch_id).map(|x| x.1).unwrap_or(false);
        if cancelled() {
            results.push(json!({"episodeId": ep, "isSuccess": false, "status": "cancelled", "totalCostUsd": 0.0, "executedNodes": [], "skippedNodes": [], "failedNodes": [], "cachedNodes": [],
                "errors": {"batch": "Đã hủy theo yêu cầu người dùng."}, "outputVideoPath": null, "durationSec": 0.0}));
            continue;
        }
        ev("node_progress", format!("[{}/{}] Bắt đầu xử lý tập '{}'", i + 1, cfg.episodes.len(), ep), Some(i as f64 / cfg.episodes.len() as f64));
        let t0 = std::time::Instant::now();
        let mut g = clone_for_episode(&base, ep);
        let mut b = body.clone();
        b["mode"] = json!("all");
        b.as_object_mut().map(|m| m.remove("targetNodeId"));
        let req = gr::RunReq::from_value(&project, &b);
        let cached = gr::prepare_cached_nodes(&project, &mut g, &req);
        let r = gr::run_graph(state.tx.clone(), reg.clone(), project.clone(), g, req).await;
        let dur = (t0.elapsed().as_secs_f64() * 100.0).round() / 100.0;
        let out_rel = format!("output/{}/{}.mp4", ep, ep);
        let out_path = workspace::project_path(&project, &["output", ep, &format!("{}.mp4", ep)], false).ok().filter(|p| p.is_file()).map(|_| out_rel);
        results.push(match r {
            Ok(s) => {
                let status = if cancelled() { "cancelled" } else if s["isSuccess"] == true { "completed" } else { "failed" };
                json!({"episodeId": ep, "runId": s["runId"], "isSuccess": s["isSuccess"], "status": status, "totalCostUsd": s["totalCostUsd"], "executedNodes": s["executedNodes"],
                    "skippedNodes": s["skippedNodes"], "failedNodes": s["failedNodes"], "cachedNodes": cached, "errors": s["errors"], "outputVideoPath": out_path, "durationSec": dur})
            }
            Err(e) => json!({"episodeId": ep, "runId": null, "isSuccess": false, "status": "failed", "totalCostUsd": 0.0, "executedNodes": [], "skippedNodes": [], "failedNodes": [],
                "cachedNodes": [], "errors": {"execution": crate::providers::key_vault::redact(&e)}, "outputVideoPath": null, "durationSec": dur}),
        });
    }
    CANCELS.lock().unwrap_or_else(|e| e.into_inner()).remove(&batch_id);
    let count = |st: &str| results.iter().filter(|r| r["status"] == st).count();
    let (done, failed, canc) = (count("completed"), count("failed"), count("cancelled"));
    let total: f64 = results.iter().map(|r| r["totalCostUsd"].as_f64().unwrap_or(0.0)).sum();
    let fin = chrono::Utc::now();
    let ok = done == results.len() && !results.is_empty();
    let summary = json!({"batchId": batch_id, "projectId": project, "totalEpisodes": cfg.episodes.len(), "completedCount": done, "failedCount": failed, "cancelledCount": canc,
        "totalCostUsd": (total * 1e4).round() / 1e4, "isSuccess": ok, "episodeResults": results, "startedAt": started.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "finishedAt": fin.format("%Y-%m-%dT%H:%M:%SZ").to_string(), "durationSec": ((fin - started).num_milliseconds() as f64 / 10.0).round() / 100.0});
    let _ = fs::write(report_dir(&project).join(format!("{}.json", batch_id)), serde_json::to_string_pretty(&summary).unwrap_or_default());
    ev(if ok { "run_completed" } else { "run_failed" }, format!("Hoàn thành batch {} tập: {} thành công, {} lỗi, {} đã hủy.", cfg.episodes.len(), done, failed, canc), Some(1.0));
    Ok(Json(summary))
}

async fn cancel_episodes(AxumPath(project): AxumPath<String>, body: Option<Json<Value>>) -> Json<Value> {
    let want = body.as_ref().and_then(|b| b["batchId"].as_str().map(String::from));
    let mut hit = false;
    {
        let mut g = CANCELS.lock().unwrap_or_else(|e| e.into_inner());
        for (id, (p, c)) in g.iter_mut() {
            if *p == project && want.as_ref().map(|w| w == id).unwrap_or(true) {
                *c = true;
                hit = true;
            }
        }
    }
    if hit {
        gr::cancel_project(&project); // stops the episode currently running
    }
    Json(json!({"status": if hit { "cancelled" } else { "not_found" }, "projectId": project, "batchId": want}))
}

async fn batch_report(AxumPath((project, batch_id)): AxumPath<(String, String)>) -> Result<Json<Value>, ApiErr> {
    if !is_valid_slug(&batch_id) {
        return Err(err(StatusCode::NOT_FOUND, format!("Không tìm thấy đợt chạy batch '{}'.", batch_id)));
    }
    fs::read_to_string(report_dir(&project).join(format!("{}.json", batch_id))).ok().and_then(|t| serde_json::from_str(&t).ok())
        .map(Json).ok_or_else(|| err(StatusCode::NOT_FOUND, format!("Không tìm thấy đợt chạy batch '{}'.", batch_id)))
}

#[derive(Deserialize)]
struct LimitQ {
    limit: Option<usize>,
}

async fn list_batches(AxumPath(project): AxumPath<String>, Query(q): Query<LimitQ>) -> Json<Value> {
    let mut files: Vec<_> = fs::read_dir(report_dir(&project)).into_iter().flatten().flatten().filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path()))).collect();
    files.sort_by(|a, b| b.0.cmp(&a.0));
    Json(json!(files.into_iter().take(q.limit.unwrap_or(20)).filter_map(|(_, p)| fs::read_to_string(p).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok())).collect::<Vec<_>>()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/projects/{project_id}/estimate-episodes", post(estimate_episodes))
        .route("/projects/{project_id}/run-episodes", post(run_episodes))
        .route("/projects/{project_id}/run-episodes/cancel", post(cancel_episodes))
        .route("/projects/{project_id}/batch-runs/{batch_id}", get(batch_report))
        .route("/projects/{project_id}/batch-runs", get(list_batches))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clone_sets_episode_and_export_name() {
        let g = json!({"id": "graph_p", "nodes": [{"id": "s", "type": "script", "params": {}}, {"id": "e", "type": "export", "params": {"outputFileName": "final_video.mp4"}}], "edges": []});
        let c = clone_for_episode(&g, "ep01");
        assert_eq!(c["id"], "graph_p__ep01");
        assert_eq!(c["nodes"][0]["params"]["mode"], "episode");
        assert_eq!(c["nodes"][1]["params"]["outputFileName"], "ep01.mp4");
        assert_eq!(g["nodes"][0]["params"], json!({}));
    }
    #[test]
    fn rejects_empty_episode_list() {
        assert!(parse(&json!({"episodeIds": []})).is_err());
    }
}
