//! DAG runner (port of graph_runner.py run loop + run_manager.py).
//! - topo-sort + cycle detection come from `validator`; independent branches run in parallel (bounded by
//!   `concurrencyLimit`), unlike the sequential Python loop.
//! - SSE payloads carry the Python `SSEEvent` fields (eventType, runId, nodeId, state, progress, message, costUsd,
//!   timestamp, model, provider, detail, estimatedCostUsd, durationSec, nodeType, inputsSummary, outputsSummary) plus the
//!   `projectId/status/progressPct` fields the dashboard `onmessage` handler reads.
//! Shortcuts: no dynamic-slot resolver beyond `{{ $node.<id>.<port> }}` text substitution, no capability sub-runs
//! (speak / per-scene image), no cost-ledger file; see ai/plans/INTERFACE_REQUESTS.md.
use super::cost;
use super::executor::{ExecutorRegistry, NodeCtx, NodeOutput};
use super::validator::{self, build_deps, is_bypassed};
use crate::providers::key_vault::redact;
use crate::workspace;
use once_cell::sync::Lazy;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, Semaphore};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

// ------------------------------------------------------------------ request / policy
#[derive(Clone, Debug)]
pub struct RunReq {
    pub project_id: String,
    pub mode: String,
    pub target: Option<String>,
    pub spending_limit_usd: Option<f64>,
    pub mock_mode: bool,
    pub policy: Option<[String; 4]>, // image, video, text, voice
    pub max_retries: u32,
    pub concurrency: usize,
}

impl RunReq {
    pub fn from_value(project: &str, p: &Value) -> Self {
        let policy = p.get("runPolicy").filter(|v| v.is_object()).map(|v| {
            let g = |k: &str, d: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or(d).to_string();
            [g("image", "mock"), g("video", "off"), g("text", "mock"), g("voice", "local")]
        });
        Self {
            project_id: project.to_string(),
            mode: p["mode"].as_str().unwrap_or("all").to_string(),
            target: p["targetNodeId"].as_str().filter(|s| !s.is_empty()).map(String::from),
            spending_limit_usd: p["spendingLimitUsd"].as_f64(),
            mock_mode: p["mockMode"].as_bool().unwrap_or(true),
            policy,
            max_retries: p["maxRetries"].as_u64().unwrap_or(3).clamp(1, 10) as u32,
            concurrency: p["concurrencyLimit"].as_u64().unwrap_or(2).clamp(1, 8) as usize,
        }
    }

    pub fn resolved_policy(&self) -> [String; 4] {
        self.policy.clone().unwrap_or_else(|| {
            let (i, v, t) = if self.mock_mode { ("mock", "mock", "mock") } else { ("real", "real", "real") };
            [i.into(), v.into(), t.into(), "local".into()]
        })
    }

    pub fn policy_json(&self) -> Value {
        let p = self.resolved_policy();
        json!({"image": p[0], "video": p[1], "text": p[2], "voice": p[3]})
    }
}

/// Mock policy per node kind (graph_runner.py `_is_mock_for`).
pub fn is_mock_for(node: &Value, req: &RunReq) -> bool {
    if req.policy.is_none() && req.mock_mode {
        return true;
    }
    let p = req.resolved_policy();
    match node["type"].as_str().unwrap_or("") {
        "image_gen" | "image_edit" => p[0] == "mock",
        "video_gen" => p[1] == "mock" || p[1] == "off",
        "voice_tts" | "cast" | "character" => p[3] == "mock",
        _ => p[2] == "mock",
    }
}

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(|x| x.as_str()).unwrap_or("")
}

fn needs_run(n: &Value) -> bool {
    let pin = s(n, "pinnedGenerationId");
    let has_pin = !pin.is_empty() && n["generations"].as_array().map(|g| g.iter().any(|x| x["id"] == pin)).unwrap_or(false);
    n["state"] != "completed" || n["isStale"] == true || !has_pin
}

fn ancestors(parents: &HashMap<String, Vec<String>>, id: &str) -> HashSet<String> {
    let (mut seen, mut stack) = (HashSet::new(), vec![id.to_string()]);
    while let Some(x) = stack.pop() {
        for p in parents.get(&x).into_iter().flatten() {
            if seen.insert(p.clone()) {
                stack.push(p.clone());
            }
        }
    }
    seen
}

/// Which nodes a run executes (RunMode + targetNodeId rules).
pub fn select_nodes(graph: &Value, topo: &[String], req: &RunReq) -> Result<Vec<String>, String> {
    let nodes: HashMap<&str, &Value> = graph["nodes"].as_array().into_iter().flatten().map(|n| (s(n, "id"), n)).collect();
    let (_, parents, _) = build_deps(graph);
    let target = || match req.target.as_deref() {
        Some(t) if nodes.contains_key(t) => Ok(t.to_string()),
        t => Err(format!("Target node '{}' không tồn tại trong đồ thị.", t.unwrap_or(""))),
    };
    match (req.mode.as_str(), &req.target) {
        ("force_all", _) => Ok(topo.to_vec()),
        ("downstream", _) => {
            let t = target()?;
            Ok(topo.iter().filter(|i| **i == t || ancestors(&parents, i).contains(&t)).cloned().collect())
        }
        ("single_node", _) | (_, Some(_)) => {
            // target + every ancestor that is stale / never completed
            let t = target()?;
            let anc = ancestors(&parents, &t);
            Ok(topo.iter().filter(|i| **i == t || (anc.contains(*i) && needs_run(nodes[i.as_str()]))).cloned().collect())
        }
        _ => Ok(topo.iter().filter(|i| needs_run(nodes[i.as_str()])).cloned().collect()),
    }
}

// ------------------------------------------------------------------ SSE
fn now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn ui_status(state: &str) -> &str {
    match state {
        "completed" => "succeeded",
        x => x,
    }
}

pub struct Ev<'a> {
    pub run: &'a str,
    pub project: &'a str,
    pub kind: &'a str,
    pub node: Option<&'a str>,
    pub state: Option<&'a str>,
    pub progress: Option<f64>,
    pub message: Option<String>,
    pub extra: Value,
}

pub fn emit(tx: &broadcast::Sender<String>, e: Ev) {
    let mut m = Map::new();
    m.insert("eventType".into(), json!(e.kind));
    m.insert("runId".into(), json!(e.run));
    m.insert("projectId".into(), json!(e.project));
    m.insert("timestamp".into(), json!(now()));
    if let Some(n) = e.node {
        m.insert("nodeId".into(), json!(n));
    }
    if let Some(st) = e.state {
        m.insert("state".into(), json!(st));
        m.insert("status".into(), json!(ui_status(st)));
    }
    if let Some(p) = e.progress {
        m.insert("progress".into(), json!(p));
        m.insert("progressPct".into(), json!((p * 100.0).round()));
    }
    if let Some(msg) = e.message {
        m.insert("message".into(), json!(redact(&msg)));
    }
    if let Value::Object(x) = e.extra {
        m.extend(x);
    }
    let _ = tx.send(Value::Object(m).to_string());
}

// ------------------------------------------------------------------ run state (run_manager.py)
static ACTIVE: Lazy<Mutex<HashMap<String, (Value, CancellationToken)>>> = Lazy::new(Default::default);

fn persist(project: &str, run_id: &str, st: &Value) {
    if let Ok(dir) = workspace::project_path(project, &["runs", run_id], false) {
        let _ = std::fs::create_dir_all(&dir);
        let tmp = dir.join("state.json.tmp");
        if std::fs::write(&tmp, serde_json::to_string_pretty(st).unwrap_or_default()).is_ok() {
            let _ = std::fs::rename(tmp, dir.join("state.json"));
        }
    }
}

fn with_state(project: &str, f: impl FnOnce(&mut Value)) {
    let mut g = ACTIVE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((st, _)) = g.get_mut(project) {
        f(st);
        let run_id = s(st, "runId").to_string();
        persist(project, &run_id, st);
    }
}

fn stage_name(t: &str) -> String {
    match t {
        "text" => "Khởi tạo ý tưởng",
        "script" => "Biên kịch phân cảnh",
        "character" => "Nạp hồ sơ nhân vật",
        "voice_tts" => "Đọc thoại Sóc Nhỏ (OmniVoice)",
        "captions" => "Tạo phụ đề Karaoke",
        "image" => "Chuẩn bị hình ảnh",
        "image_gen" => "Tạo ảnh AI",
        "scene" => "Đóng gói phân cảnh",
        "hyperframes_render" => "Dựng video HyperFrames",
        "export" => "Xuất & Đóng gói sản phẩm",
        o => return format!("Xử lý {}", o),
    }
    .to_string()
}

fn update_node(project: &str, node: &str, state: &str, progress: f64, message: Option<&str>, cost: f64) {
    with_state(project, |st| {
        let started = st["startedAtTs"].as_f64().unwrap_or(0.0);
        st["updatedAt"] = json!(now());
        st["elapsedSec"] = json!(((chrono::Utc::now().timestamp_millis() as f64 / 1000.0 - started) * 10.0).round() / 10.0);
        st["currentNodeId"] = json!(node);
        let mut ntype = "node".to_string();
        let mut done = 0;
        if let Some(arr) = st["nodes"].as_array_mut() {
            for n in arr.iter_mut() {
                if n["id"] == node {
                    ntype = s(n, "type").to_string();
                    n["state"] = json!(state);
                    n["progressPct"] = json!((progress * 100.0).round());
                    if state == "failed" {
                        n["error"] = json!(message);
                    }
                }
                if matches!(n["state"].as_str(), Some("completed") | Some("succeeded") | Some("skipped")) {
                    done += 1;
                }
            }
        }
        let total = st["nodes"].as_array().map(|a| a.len()).unwrap_or(1).max(1) as f64;
        st["completedNodes"] = json!(done);
        let partial = if state == "running" { progress / total } else { 0.0 };
        let overall = (done as f64 / total + partial).clamp(0.05, 0.99);
        st["progress"] = json!((overall * 1000.0).round() / 1000.0);
        st["progressPct"] = json!((overall * 100.0).round());
        let stage = stage_name(&ntype);
        st["stage"] = json!(match message {
            Some(m) if !m.is_empty() => format!("{} · {}", stage, m),
            _ => stage,
        });
        if cost > 0.0 {
            let c = ((st["actualCostUsd"].as_f64().unwrap_or(0.0) + cost) * 1e4).round() / 1e4;
            st["actualCostUsd"] = json!(c);
            st["costUsd"] = json!(c);
        }
    });
}

pub fn active_run(project: &str) -> Option<Value> {
    ACTIVE.lock().unwrap_or_else(|e| e.into_inner()).get(project).map(|x| x.0.clone()).filter(|st| st["status"] == "running")
}

/// Cancel the project's running graph (CancellationToken) and mark its state; false when nothing runs.
pub fn cancel_project(project: &str) -> bool {
    let tok = ACTIVE.lock().unwrap_or_else(|e| e.into_inner()).get(project).filter(|x| x.0["status"] == "running").map(|x| x.1.clone());
    match tok {
        Some(t) => {
            t.cancel();
            with_state(project, |st| {
                st["status"] = json!("cancelled");
                st["stage"] = json!("Đã dừng chạy theo yêu cầu người dùng");
                st["updatedAt"] = json!(now());
            });
            true
        }
        None => false,
    }
}

/// runs/<id>/state.json records, newest first (active run first).
pub fn list_runs(project: &str, limit: usize) -> Vec<Value> {
    let mut out: Vec<Value> = active_run(project).into_iter().collect();
    let Ok(dir) = workspace::project_path(project, &["runs"], false) else { return out };
    let mut files: Vec<(std::time::SystemTime, std::path::PathBuf)> = std::fs::read_dir(dir).into_iter().flatten().flatten()
        .map(|e| e.path().join("state.json")).filter(|p| p.is_file())
        .filter_map(|p| Some((p.metadata().ok()?.modified().ok()?, p))).collect();
    files.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, f) in files {
        if out.len() >= limit {
            break;
        }
        if let Some(v) = std::fs::read_to_string(&f).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
            if !out.iter().any(|o| o["runId"] == v["runId"]) {
                out.push(v);
            }
        }
    }
    out
}

// ------------------------------------------------------------------ input resolution
fn script_ref(project: &str, r: &Value) -> Option<Value> {
    let ep = s(r, "episode");
    let p = workspace::project_path(project, &["content", "episodes", ep, "script.json"], false).ok()?;
    let data: Value = serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()?;
    let shots: Vec<&Value> = data["scenes"].as_array()?.iter().flat_map(|sc| sc["shots"].as_array().into_iter().flatten()).collect();
    if let Some(id) = r["shot"].as_str().filter(|x| !x.is_empty()) {
        return shots.iter().find(|h| h["id"] == id).map(|h| (*h).clone());
    }
    match s(r, "facet") {
        "script" | "" => Some(data),
        _ => Some(json!(shots.iter().filter_map(|h| h["dialogue"].as_str()).filter(|d| !d.is_empty()).collect::<Vec<_>>().join("\n"))),
    }
}

fn dialogue_text(v: &Value) -> Option<String> {
    let a = v.as_array().filter(|a| !a.is_empty() && a.iter().all(|x| x.get("text").is_some() && x.get("speaker").is_some()))?;
    Some(a.iter().map(|l| format!("{}: {}", s(l, "speaker"), s(l, "text"))).collect::<Vec<_>>().join("\n"))
}

fn subst_refs(v: &mut Value, outs: &HashMap<String, Value>, re: &regex::Regex) {
    match v {
        Value::String(t) if t.contains("{{") => {
            *t = re.replace_all(t, |c: &regex::Captures| {
                outs.get(&c[1]).and_then(|o| o.get(&c[2])).map(|x| x.as_str().map(String::from).unwrap_or_else(|| x.to_string())).unwrap_or_default()
            }).into_owned();
        }
        Value::Array(a) => a.iter_mut().for_each(|x| subst_refs(x, outs, re)),
        Value::Object(m) => m.values_mut().for_each(|x| subst_refs(x, outs, re)),
        _ => {}
    }
}

/// params overlaid with wired port values, `$ref: script` and `{{ $node.id.port }}` resolved, port defaults applied.
pub fn gather_inputs(node: &Value, graph: &Value, outs: &HashMap<String, Value>, types: &HashMap<String, Value>) -> Value {
    let mut r: Map<String, Value> = node["params"].as_object().cloned().unwrap_or_default();
    let id = s(node, "id");
    let def = types.get(s(node, "type"));
    let port_type = |name: &str| def.and_then(|d| d["inputPorts"].as_array()).and_then(|a| a.iter().find(|p| p["name"] == name)).and_then(|p| p["portType"].as_str()).unwrap_or("").to_string();
    let mut list_filled: HashSet<String> = HashSet::new();
    for e in graph["edges"].as_array().into_iter().flatten().filter(|e| e["targetNodeId"] == id) {
        let Some(mut val) = outs.get(s(e, "sourceNodeId")).and_then(|o| o.get(s(e, "sourcePort"))).cloned() else { continue };
        let (tp, tgt) = (port_type(s(e, "targetPort")), s(e, "targetPort").to_string());
        if tp == "image[]" && !val.is_array() {
            val = json!([val]);
        }
        if tp == "text" {
            if let Some(t) = dialogue_text(&val) {
                val = json!(t);
            }
        }
        if matches!(tp.as_str(), "image[]" | "scenes" | "voice_clips") {
            let items = match val { Value::Array(a) => a, Value::Null => vec![], x => vec![x] };
            if list_filled.insert(tgt.clone()) {
                r.insert(tgt, json!(items));
            } else if let Some(Value::Array(prev)) = r.get_mut(&tgt) {
                prev.extend(items);
            }
        } else {
            r.insert(tgt, val);
        }
    }
    let project = s(graph, "projectId").to_string();
    for (k, v) in r.clone() {
        if v["$ref"] == "script" {
            match script_ref(&project, &v) {
                Some(x) => r.insert(k, x),
                None => r.remove(&k),
            };
        }
    }
    for p in def.and_then(|d| d["inputPorts"].as_array()).into_iter().flatten() {
        let name = s(p, "name");
        let dv = &p["defaultValue"];
        let empty = r.get(name).map(|x| x.is_null() || x == "" || x == &json!([])).unwrap_or(true);
        if !dv.is_null() && dv != "" && empty {
            r.insert(name.into(), dv.clone());
        }
    }
    for (pn, rf) in node["inputs"].as_object().into_iter().flatten() {
        if let Some(src) = rf["nodeId"].as_str() {
            if let Some(v) = outs.get(src).and_then(|o| o.get(s(rf, "outputPort"))) {
                r.insert(pn.clone(), v.clone());
            }
        } else if !rf.is_null() && !r.contains_key(pn) {
            r.insert(pn.clone(), rf.clone());
        }
    }
    let mut out = Value::Object(r);
    let re = regex::Regex::new(r"\{\{\s*\$node\.([a-zA-Z0-9_-]+)\.([a-zA-Z0-9_]+)\s*\}\}").unwrap();
    subst_refs(&mut out, outs, &re);
    out
}

fn forward_bypass(node: &Value, resolved: &Value, types: &HashMap<String, Value>) -> Value {
    let mut out = Map::new();
    let Some(def) = types.get(s(node, "type")) else { return json!({}) };
    for op in def["outputPorts"].as_array().into_iter().flatten() {
        let name = s(op, "name");
        let v = resolved.get(name).cloned().or_else(|| {
            def["inputPorts"].as_array().into_iter().flatten().find(|ip| ip["portType"] == op["portType"]).and_then(|ip| resolved.get(s(ip, "name")).cloned())
        });
        if let Some(v) = v {
            out.insert(name.into(), v);
        }
    }
    Value::Object(out)
}

// ------------------------------------------------------------------ output snapshots
fn safe(x: &str) -> String {
    x.chars().map(|c| if c.is_ascii_alphanumeric() || "_.-".contains(c) { c } else { '_' }).take(120).collect()
}

fn params_sig(n: &Value) -> String {
    let raw = format!("{}|{}", s(n, "type"), n["params"]);
    format!("{:x}", Sha256::digest(raw.as_bytes()))
}

fn snap_path(project: &str, graph_id: &str, node: &str, mock: bool) -> Option<std::path::PathBuf> {
    let f = format!("{}_{}.json", safe(node), if mock { "mock" } else { "real" });
    workspace::project_path(project, &["runs", "_node_outputs_rs", &safe(graph_id), &f], true).ok()
}

fn load_snapshot(project: &str, graph_id: &str, n: &Value, mock: bool) -> Option<Value> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(snap_path(project, graph_id, s(n, "id"), mock)?).ok()?).ok()?;
    (v["paramsSig"] == params_sig(n).as_str()).then(|| v["outputs"].clone())
}

fn out_sig(o: &Value) -> String {
    format!("{:x}", Sha256::digest(o.to_string().as_bytes()))
}

fn port_sigs(o: &Value) -> HashMap<String, String> {
    o.as_object().into_iter().flatten().map(|(k, v)| (k.clone(), out_sig(v))).collect()
}

fn save_snapshot(project: &str, graph_id: &str, n: &Value, mock: bool, outputs: &Value) {
    if let Some(p) = snap_path(project, graph_id, s(n, "id"), mock) {
        let _ = std::fs::write(p, json!({"paramsSig": params_sig(n), "outputs": outputs, "portSigs": port_sigs(outputs)}).to_string());
    }
}

fn prev_port_sigs(project: &str, graph_id: &str, n: &Value, mock: bool) -> Option<HashMap<String, String>> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(snap_path(project, graph_id, s(n, "id"), mock)?).ok()?).ok()?;
    serde_json::from_value(v["portSigs"].clone()).ok()
}

fn pinned_outputs(n: &Value) -> Option<Value> {
    let pin = s(n, "pinnedGenerationId");
    let g = n["generations"].as_array()?.iter().find(|g| g["id"] == pin)?;
    let url = g["previewUrl"].as_str()?;
    let art = json!({"kind": g["previewType"], "uri": url, "url": url, "provider": "pinned", "model": ""});
    Some(json!({"artifact": art, "image": art, "images": [art], "video": art, "audio": art}))
}

// ------------------------------------------------------------------ summaries for events
fn summarize(v: &Value) -> String {
    match v {
        Value::Null => "—".into(),
        Value::String(t) => t.chars().take(80).collect(),
        Value::Array(a) => format!("{} mục", a.len()),
        Value::Object(m) if m.contains_key("url") => format!("{} {}", s(v, "kind"), s(v, "uri")),
        Value::Object(_) => "đối tượng".into(),
        x => x.to_string(),
    }
}

fn summarize_ports(o: &Value, only: Option<&HashSet<String>>) -> Value {
    json!(o.as_object().into_iter().flatten().filter(|(k, _)| only.map(|s| s.contains(*k)).unwrap_or(true)).map(|(k, v)| (k.clone(), summarize(v))).collect::<HashMap<_, _>>())
}

fn describe_result(t: &str, o: &Value) -> String {
    let n = |k: &str| o[k].as_array().map(|a| a.len()).unwrap_or(0);
    match t {
        "script" => format!("{} cảnh · {} câu thoại", n("scenes"), n("lines")),
        "image_gen" => format!("{} ảnh", n("images")),
        "cast" => format!("{} nhân vật", n("characters")),
        "product" => format!("{} ảnh đã tách nền", n("images")),
        _ => format!("Node {} hoàn thành", t),
    }
}

fn transient(e: &str) -> bool {
    let l = e.to_lowercase();
    ["http 5", "http 429", "timeout", "timed out", "connection", "error sending request", "reset by peer"].iter().any(|m| l.contains(m))
}

// ------------------------------------------------------------------ the run
struct NodeRun {
    id: String,
    result: Result<NodeOutput, String>,
    duration: f64,
    cancelled: bool,
}

/// Registry = caller's registry + Track A executors.
pub fn registry_with_track_a(base: &ExecutorRegistry) -> ExecutorRegistry {
    let mut r = base.clone();
    crate::providers::executors::register(&mut r);
    r
}

fn write_graph(project: &str, graph: &mut Value) {
    graph["updatedAt"] = json!(now());
    if let Ok(p) = workspace::project_path(project, &["graph.json"], true) {
        let tmp = p.with_extension("json.tmp");
        if std::fs::write(&tmp, serde_json::to_string_pretty(graph).unwrap_or_default()).is_ok() {
            let _ = std::fs::rename(tmp, p);
        }
    }
}

pub async fn run_graph(tx: broadcast::Sender<String>, registry: ExecutorRegistry, project: String, mut graph: Value, req: RunReq) -> Result<Value, String> {
    let v = validator::validate(&graph);
    if !v.is_valid {
        // Per-node errors go back as a normal run summary so the UI marks each node; global-only errors stay a hard failure.
        if v.node_errors.is_empty() {
            return Err(format!("Đồ thị không hợp lệ: {}", v.errors.join("; ")));
        }
        let failed: Vec<&String> = v.node_errors.keys().collect();
        return Ok(json!({"runId": format!("run_{}", chrono::Utc::now().timestamp_millis()), "projectId": project,
            "isSuccess": false, "executedNodes": [], "failedNodes": failed, "skippedNodes": [], "staleNodes": [],
            "errors": v.node_errors, "detail": format!("Đồ thị không hợp lệ: {}", v.errors.join("; ")), "createdAt": now()}));
    }
    if active_run(&project).is_some() {
        return Err("Dự án đang có một phiên chạy khác. Hãy dừng hoặc chờ phiên đó xong.".into());
    }
    let to_run = select_nodes(&graph, &v.topological_order, &req)?;
    let plan = cost::plan(&project, &graph, &req)?;
    if plan["spendStatus"]["canProceed"] == false {
        return Err(plan["spendStatus"]["reason"].as_str().unwrap_or("vượt trần chi tiêu").to_string());
    }
    let types = validator::node_types();
    let run_id = format!("run_{}", chrono::Utc::now().timestamp_millis());
    let graph_id = if s(&graph, "id").is_empty() { format!("graph_{}", project) } else { s(&graph, "id").to_string() };
    graph["projectId"] = json!(project);
    let (_, parents, _) = build_deps(&graph);
    let cancel = CancellationToken::new();

    let idx: HashMap<String, usize> = graph["nodes"].as_array().unwrap().iter().enumerate().map(|(i, n)| (s(n, "id").to_string(), i)).collect();
    let node_at = |g: &Value, id: &str| g["nodes"][idx[id]].clone();

    // state.json
    let started = chrono::Utc::now().timestamp_millis() as f64 / 1000.0;
    let state = json!({"runId": run_id, "id": run_id, "projectId": project, "status": "running", "mode": req.mode, "runPolicy": req.policy_json(),
        "stage": "Bắt đầu", "currentNodeId": null, "progress": 0.05, "progressPct": 5, "startedAtTs": started, "createdAt": now(), "updatedAt": now(),
        "elapsedSec": 0.0, "actualCostUsd": 0.0, "costUsd": 0.0, "totalNodes": idx.len(), "completedNodes": 0, "output": null, "error": null,
        "nodes": graph["nodes"].as_array().unwrap().iter().map(|n| json!({"id": n["id"], "type": n["type"], "name": n["params"]["title"].as_str().map(String::from).unwrap_or_else(|| s(n, "id").to_string()), "state": "pending", "progressPct": 0, "error": null})).collect::<Vec<_>>()});
    persist(&project, &run_id, &state);
    ACTIVE.lock().unwrap_or_else(|e| e.into_inner()).insert(project.clone(), (state, cancel.clone()));
    let evx = |kind: &'static str, node: Option<&str>, st: Option<&str>, pr: Option<f64>, msg: Option<String>, extra: Value| {
        emit(&tx, Ev { run: &run_id, project: &project, kind, node, state: st, progress: pr, message: msg, extra });
    };
    evx("run_started", None, None, None, Some(format!("Bắt đầu chạy {} node theo chế độ '{}'", to_run.len(), req.mode)), json!({}));

    // restore upstream outputs
    let mut outs: HashMap<String, Value> = HashMap::new();
    for n in graph["nodes"].as_array().unwrap() {
        let id = s(n, "id");
        let o = load_snapshot(&project, &graph_id, n, is_mock_for(n, &req)).or_else(|| pinned_outputs(n));
        if let Some(o) = o {
            outs.insert(id.to_string(), o);
        }
    }

    let run_set: HashSet<&String> = to_run.iter().collect();
    let (mut executed, mut skipped, mut failed, mut stale): (Vec<String>, Vec<String>, Vec<String>, Vec<String>) = Default::default();
    let mut bypassed: HashSet<String> = HashSet::new();
    let mut errors: Map<String, Value> = Map::new();
    let mut total_cost = 0.0f64;
    let mut done: HashSet<String> = HashSet::new();
    let mut pending: Vec<String> = to_run.clone();
    let mut running: HashSet<String> = HashSet::new();
    let sem = Arc::new(Semaphore::new(req.concurrency));
    let mut set: JoinSet<NodeRun> = JoinSet::new();
    let ts_now = || chrono::Utc::now().timestamp_millis() as f64 / 1000.0;

    loop {
        // --- schedule everything that is ready
        let mut progressed = true;
        while progressed {
            progressed = false;
            for nid in pending.clone() {
                if cancel.is_cancelled() {
                    graph["nodes"][idx[&nid]]["state"] = json!("cancelled");
                    skipped.push(nid.clone());
                    evx("node_state", Some(&nid), Some("cancelled"), None, Some(format!("Đã hủy node '{}' theo yêu cầu người dùng.", nid)), json!({}));
                    update_node(&project, &nid, "cancelled", 0.0, None, 0.0);
                    pending.retain(|x| *x != nid);
                    progressed = true;
                    continue;
                }
                let ps = parents.get(&nid).cloned().unwrap_or_default();
                let bad: Vec<&String> = ps.iter().filter(|p| failed.contains(p) || (skipped.contains(p) && !bypassed.contains(*p))).collect();
                if !bad.is_empty() {
                    graph["nodes"][idx[&nid]]["state"] = json!("failed");
                    skipped.push(nid.clone());
                    let m = format!("Bỏ qua node '{}' do node cha {:?} bị lỗi.", nid, bad);
                    evx("node_state", Some(&nid), Some("failed"), None, Some(m.clone()), json!({}));
                    update_node(&project, &nid, "failed", 0.0, Some(&m), 0.0);
                    pending.retain(|x| *x != nid);
                    progressed = true;
                    continue;
                }
                let missing: Vec<&String> = ps.iter().filter(|p| !run_set.contains(p) && !outs.contains_key(*p)).collect();
                if !missing.is_empty() {
                    let m = format!("Chưa chạy được node '{}': đang chờ đầu vào từ node {:?}", nid, missing);
                    graph["nodes"][idx[&nid]]["state"] = json!("failed");
                    failed.push(nid.clone());
                    errors.insert(nid.clone(), json!(m));
                    evx("node_state", Some(&nid), Some("failed"), None, Some(m.clone()), json!({}));
                    update_node(&project, &nid, "failed", 0.0, Some(&m), 0.0);
                    pending.retain(|x| *x != nid);
                    progressed = true;
                    continue;
                }
                if ps.iter().any(|p| run_set.contains(p) && !done.contains(p)) || running.len() >= req.concurrency {
                    continue; // wait for parents / a free slot
                }
                let node = node_at(&graph, &nid);
                let resolved = gather_inputs(&node, &graph, &outs, &types);
                pending.retain(|x| *x != nid);
                progressed = true;
                if is_bypassed(&node) {
                    outs.insert(nid.clone(), forward_bypass(&node, &resolved, &types));
                    done.insert(nid.clone());
                    bypassed.insert(nid.clone());
                    skipped.push(nid.clone());
                    graph["nodes"][idx[&nid]]["state"] = json!("skipped");
                    evx("node_state", Some(&nid), Some("skipped"), Some(1.0), Some(format!("Bỏ qua node '{}' (_bypass=True), chuyển tiếp đầu vào sang đầu ra.", nid)), json!({}));
                    update_node(&project, &nid, "skipped", 1.0, None, 0.0);
                    continue;
                }
                let mock = is_mock_for(&node, &req);
                let (mkey, mprov) = cost::effective_model(&node, !mock);
                let mut est = if mock { 0.0 } else { cost::estimate_node_cost(&node, true) };
                if est > 0.0 && node["type"] == "image_gen" {
                    est *= cost::batch_count(&graph, &node).max(1) as f64;
                }
                let wired: HashSet<String> = graph["edges"].as_array().into_iter().flatten().filter(|e| e["targetNodeId"] == nid.as_str()).map(|e| s(e, "targetPort").to_string()).collect();
                graph["nodes"][idx[&nid]]["state"] = json!("running");
                evx("node_state", Some(&nid), Some("running"), Some(0.1), Some(format!("Đang thực thi node {} ({})...", s(&node, "type"), nid)), json!({}));
                evx("node_progress", Some(&nid), Some("running"), Some(0.12), Some(format!("Đang chạy {}", s(&node, "type"))),
                    json!({"nodeType": node["type"], "model": mkey, "provider": mprov, "estimatedCostUsd": (est * 1e6).round() / 1e6, "detail": format!("Đang chạy {}", s(&node, "type")),
                        "inputsSummary": summarize_ports(&resolved, Some(&wired))}));
                update_node(&project, &nid, "running", 0.1, None, 0.0);

                let ctx = NodeCtx {
                    project_id: project.clone(), run_id: run_id.clone(), node_id: nid.clone(), node_type: s(&node, "type").to_string(),
                    custom_name: node["params"]["_customName"].as_str().unwrap_or(s(&node, "type")).to_string(),
                    params: resolved,
                    inputs: ps.iter().filter_map(|p| outs.get(p).map(|o| (p.clone(), o.clone()))).collect(),
                    tx: tx.clone(), cancel: cancel.child_token(), mock,
                };
                let ex = registry.get(s(&node, "type")).expect("fallback executor always resolves");
                let (sem, txc, run_c, proj_c, retries, nid_c) = (sem.clone(), tx.clone(), run_id.clone(), project.clone(), req.max_retries, nid.clone());
                running.insert(nid.clone());
                set.spawn(async move {
                    let _permit = sem.acquire_owned().await;
                    let t0 = std::time::Instant::now();
                    let (mut last, mut cancelled) = (String::new(), false);
                    for attempt in 1..=retries {
                        if ctx.cancel.is_cancelled() {
                            return NodeRun { id: nid_c, result: Err("cancelled".into()), duration: 0.0, cancelled: true };
                        }
                        emit(&txc, Ev { run: &run_c, project: &proj_c, kind: "node_progress", node: Some(&nid_c), state: Some("running"),
                            progress: Some(((0.2 + 0.6 * attempt as f64 / retries as f64) * 100.0).round() / 100.0),
                            message: Some(format!("Đang xử lý (lần thử {}/{})...", attempt, retries)), extra: json!({}) });
                        match ex.execute(&ctx).await {
                            Ok(o) => return NodeRun { id: nid_c, result: Ok(o), duration: t0.elapsed().as_secs_f64(), cancelled: false },
                            Err(e) => {
                                last = redact(&e.to_string());
                                cancelled = last.to_lowercase().contains("cancelled");
                                emit(&txc, Ev { run: &run_c, project: &proj_c, kind: "node_log", node: Some(&nid_c), state: None, progress: None,
                                    message: Some(format!("Lỗi lần thử {}/{}: {}", attempt, retries, last)), extra: json!({}) });
                                if cancelled || !transient(&last) {
                                    break;
                                }
                                if attempt < retries {
                                    tokio::time::sleep(std::time::Duration::from_millis(200 * (1 << (attempt - 1)))).await;
                                }
                            }
                        }
                    }
                    NodeRun { id: nid_c, result: Err(last), duration: t0.elapsed().as_secs_f64(), cancelled }
                });
            }
        }
        if running.is_empty() {
            break;
        }
        let Some(Ok(r)) = set.join_next().await else { break };
        running.remove(&r.id);
        let nid = r.id.clone();
        let node = node_at(&graph, &nid);
        let mock = is_mock_for(&node, &req);
        match r.result {
            Ok(o) => {
                let mut outputs = o.outputs.clone();
                if let Value::Object(m) = &mut outputs {
                    m.entry("artifact").or_insert_with(|| o.preview_url.as_ref().map(|u| json!({"kind": o.preview_type, "uri": u, "url": u})).unwrap_or(Value::Null));
                }
                let prev = prev_port_sigs(&project, &graph_id, &node, mock);
                save_snapshot(&project, &graph_id, &node, mock, &outputs);
                outs.insert(nid.clone(), outputs.clone());
                total_cost += o.cost_usd;
                let gen_id = format!("gen_{}", &uuid::Uuid::new_v4().to_string().replace('-', "")[..8]);
                let created = now();
                let gen = json!({"id": gen_id, "nodeId": nid, "params": node["params"], "artifactId": format!("art_virtual_{}", &gen_id[4..]), "costUsd": o.cost_usd,
                    "createdAt": created, "isPinned": true, "previewUrl": o.preview_url, "previewType": o.preview_type, "summaryText": o.summary});
                if node["generations"].as_array().map(|g| !g.is_empty()).unwrap_or(false) {
                    let now_sigs = port_sigs(&outputs);
                    let children: HashSet<String> = graph["edges"].as_array().into_iter().flatten().filter(|e| e["sourceNodeId"] == nid.as_str()).filter(|e| {
                        let port = s(e, "sourcePort");
                        match &prev { None => true, Some(p) => p.get(port) != now_sigs.get(port) || !now_sigs.contains_key(port) }
                    }).map(|e| s(e, "targetNodeId").to_string()).collect();
                    for c in children {
                        if let Some(&i) = idx.get(&c) {
                            graph["nodes"][i]["isStale"] = json!(true);
                            if !stale.contains(&c) {
                                stale.push(c);
                            }
                        }
                    }
                }
                let n = &mut graph["nodes"][idx[&nid]];
                if let Some(gs) = n["generations"].as_array_mut() {
                    gs.iter_mut().for_each(|g| g["isPinned"] = json!(false));
                    gs.insert(0, gen);
                } else {
                    n["generations"] = json!([gen]);
                }
                n["pinnedGenerationId"] = json!(gen_id);
                n["state"] = json!("completed");
                n["isStale"] = json!(false);
                let mut ui = n["ui"].as_object().cloned().unwrap_or_default();
                if let Some(u) = &o.preview_url {
                    ui.insert("previewUrl".into(), json!(u));
                }
                ui.insert("previewType".into(), json!(o.preview_type));
                ui.insert("summaryText".into(), json!(o.summary));
                ui.insert("outputSummary".into(), json!({"status": "ok", "completedAt": created}));
                n["ui"] = Value::Object(ui);
                if o.cost_usd > 0.0 {
                    evx("cost_update", Some(&nid), None, None, None, json!({"costUsd": o.cost_usd}));
                }
                let msg = describe_result(s(&node, "type"), &outputs);
                evx("node_state", Some(&nid), Some("completed"), Some(1.0), Some(msg.clone()), json!({"detail": msg, "nodeType": node["type"], "durationSec": (r.duration * 100.0).round() / 100.0,
                    "costUsd": o.cost_usd, "outputsSummary": summarize_ports(&outputs, None), "previewUrl": o.preview_url, "previewType": o.preview_type, "summaryText": o.summary}));
                update_node(&project, &nid, "completed", 1.0, Some(&msg), o.cost_usd);
                done.insert(nid.clone());
                executed.push(nid);
            }
            Err(e) => {
                let st = if r.cancelled { "cancelled" } else { "failed" };
                graph["nodes"][idx[&nid]]["state"] = json!(st);
                failed.push(nid.clone());
                errors.insert(nid.clone(), json!(e));
                evx("node_state", Some(&nid), Some(st), Some(0.0), Some(format!("Node dừng lại: {}", e)), json!({}));
                update_node(&project, &nid, st, 0.0, Some(&e), 0.0);
            }
        }
        let _ = ts_now();
    }
    // anything still pending (cannot happen for a DAG) is reported rather than silently dropped
    for nid in pending {
        failed.push(nid.clone());
        errors.insert(nid.clone(), json!("Không thể lên lịch node (phụ thuộc không thoả)."));
    }

    if graph_id == format!("graph_{}", project) {
        write_graph(&project, &mut graph);
    }
    let ok = failed.is_empty();
    let err_text = errors.iter().map(|(k, v)| format!("{}: {}", k, v.as_str().unwrap_or(""))).collect::<Vec<_>>().join("; ");
    with_state(&project, |st| {
        if st["status"] != "cancelled" {
            st["status"] = json!(if ok { "completed" } else { "failed" });
        }
        st["progress"] = json!(if ok { 1.0 } else { st["progress"].as_f64().unwrap_or(0.0) });
        st["progressPct"] = json!(if ok { 100 } else { st["progressPct"].as_u64().unwrap_or(0) });
        st["completedAt"] = json!(now());
        st["updatedAt"] = json!(now());
        st["stage"] = json!(if ok { "Hoàn tất xuất sắc".to_string() } else { format!("Thất bại: {}", err_text) });
        if !ok {
            st["error"] = json!(err_text);
        }
    });
    let msg = format!("Phiên chạy hoàn tất. Đã thực thi {} node, thất bại {} node, bỏ qua {} node. Tổng chi phí: ${:.4} USD.", executed.len(), failed.len(), skipped.len(), total_cost);
    evx(if ok { "run_completed" } else { "run_failed" }, None, None, None, Some(msg), json!({"costUsd": total_cost}));
    let stale: Vec<&String> = stale.iter().filter(|c| !executed.contains(c)).collect();
    Ok(json!({"runId": run_id, "projectId": project, "totalNodes": idx.len(), "executedNodes": executed, "skippedNodes": skipped, "failedNodes": failed,
        "totalCostUsd": (total_cost * 1e6).round() / 1e6, "isSuccess": ok, "errors": errors, "staleNodes": stale, "createdAt": now()}))
}

/// Marks nodes that have a valid output snapshot (same params/mode) as completed so mode `all` skips them
/// (batch_runner._prepare_cached_nodes). Returns their ids.
pub fn prepare_cached_nodes(project: &str, graph: &mut Value, req: &RunReq) -> Vec<String> {
    let gid = s(graph, "id").to_string();
    let mut cached = vec![];
    for n in graph["nodes"].as_array_mut().into_iter().flatten() {
        if load_snapshot(project, &gid, n, is_mock_for(n, req)).is_some() {
            cached.push(s(n, "id").to_string());
            n["state"] = json!("completed");
            n["isStale"] = json!(false);
            if s(n, "pinnedGenerationId").is_empty() {
                let g = format!("gen_cached_{}", s(n, "id"));
                n["pinnedGenerationId"] = json!(g);
                n["generations"] = json!([{"id": g, "nodeId": n["id"], "params": n["params"], "artifactId": "", "costUsd": 0.0, "createdAt": now(), "isPinned": true}]);
            }
        }
    }
    cached
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(mode: &str, target: Option<&str>) -> RunReq {
        RunReq::from_value("p", &json!({"mode": mode, "targetNodeId": target}))
    }

    fn chain() -> Value {
        // a -> b -> c ; a done+pinned, b stale
        json!({"nodes": [
            {"id": "a", "type": "x", "params": {}, "state": "completed", "pinnedGenerationId": "g1", "generations": [{"id": "g1"}]},
            {"id": "b", "type": "x", "params": {}, "state": "completed", "isStale": true, "pinnedGenerationId": "g2", "generations": [{"id": "g2"}]},
            {"id": "c", "type": "x", "params": {}}],
            "edges": [{"sourceNodeId": "a", "targetNodeId": "b"}, {"sourceNodeId": "b", "targetNodeId": "c"}]})
    }

    fn topo(g: &Value) -> Vec<String> {
        let (ids, p, _) = build_deps(g);
        validator::topo_sort(&ids, &p).unwrap()
    }

    #[test]
    fn target_runs_node_plus_stale_ancestors_only() {
        let g = chain();
        assert_eq!(select_nodes(&g, &topo(&g), &req("single_node", Some("c"))).unwrap(), vec!["b", "c"]);
    }

    #[test]
    fn all_skips_valid_pins_and_downstream_includes_descendants() {
        let g = chain();
        assert_eq!(select_nodes(&g, &topo(&g), &req("all", None)).unwrap(), vec!["b", "c"]);
        assert_eq!(select_nodes(&g, &topo(&g), &req("downstream", Some("b"))).unwrap(), vec!["b", "c"]);
        assert_eq!(select_nodes(&g, &topo(&g), &req("force_all", None)).unwrap().len(), 3);
        assert!(select_nodes(&g, &topo(&g), &req("single_node", Some("zzz"))).is_err());
    }

    #[test]
    fn mock_policy_per_kind() {
        let r = RunReq::from_value("p", &json!({"mockMode": false, "runPolicy": {"image": "real", "video": "off", "text": "mock", "voice": "local"}}));
        assert!(!is_mock_for(&json!({"type": "image_gen"}), &r));
        assert!(is_mock_for(&json!({"type": "video_gen"}), &r));
        assert!(is_mock_for(&json!({"type": "script"}), &r));
        assert!(!is_mock_for(&json!({"type": "cast"}), &r));
        assert!(is_mock_for(&json!({"type": "image_gen"}), &RunReq::from_value("p", &json!({"mockMode": true}))));
    }

    #[test]
    fn gather_inputs_wires_ports_and_templates() {
        let g = json!({"projectId": "p", "nodes": [], "edges": [{"sourceNodeId": "a", "sourcePort": "text", "targetNodeId": "b", "targetPort": "prompt"}]});
        let node = json!({"id": "b", "type": "nope", "params": {"x": "{{ $node.a.text }}!"}});
        let outs: HashMap<String, Value> = [("a".to_string(), json!({"text": "hi"}))].into();
        let r = gather_inputs(&node, &g, &outs, &HashMap::new());
        assert_eq!(r["prompt"], "hi");
        assert_eq!(r["x"], "hi!");
    }
}

/// One run's saved state (runs/<id>/state.json). Ids are restricted to [A-Za-z0-9_-] so they cannot escape runs/.
pub fn get_run(project: &str, run_id: &str) -> Option<Value> {
    if !valid_run_id(run_id) {
        return None;
    }
    let p = workspace::project_path(project, &["runs", run_id, "state.json"], false).ok()?;
    serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()
}

/// Remove a run's record (not its outputs). A running run is cancelled first.
pub fn delete_run(project: &str, run_id: &str) -> bool {
    if !valid_run_id(run_id) {
        return false;
    }
    if active_run(project).map(|a| a["runId"] == run_id).unwrap_or(false) {
        cancel_project(project);
    }
    let Ok(dir) = workspace::project_path(project, &["runs", run_id], false) else { return false };
    dir.is_dir() && std::fs::remove_dir_all(dir).is_ok()
}

fn valid_run_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}
