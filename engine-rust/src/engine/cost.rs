//! Cost estimation from the model price book (port of estimate_node_cost / plan_execution).
use super::graph_runner::{is_mock_for, RunReq};
use super::validator::{self, is_bypassed};
use crate::providers::{key_vault, models, router::resolve_provider_for_model, spend_guard::SpendGuard};
use crate::workspace;
use serde_json::{json, Value};
use std::collections::HashSet;

fn p<'a>(n: &'a Value, k: &str) -> &'a str {
    n["params"].get(k).and_then(|x| x.as_str()).unwrap_or("").trim()
}

/// (model key, provider) that will be used for a node.
pub fn effective_model(n: &Value, real: bool) -> (String, String) {
    if is_bypassed(n) {
        return ("bypass".into(), "none".into());
    }
    let raw = p(n, "model").to_string();
    let t = n["type"].as_str().unwrap_or("");
    let (key, prov) = match t {
        "image_gen" | "image_edit" => {
            let k = if raw.is_empty() || (real && raw.starts_with("mock")) { if real { "local-flux-schnell".to_string() } else if raw.is_empty() { "mock-image".into() } else { raw } } else { raw };
            let pr = resolve_provider_for_model(&k, if real { "local" } else { "mock" });
            (k, pr)
        }
        "video_gen" => {
            let k = if raw.is_empty() || (real && raw.starts_with("mock")) { if real { "google-veo-2".to_string() } else if raw.is_empty() { "mock-video".into() } else { raw } } else { raw };
            let pr = resolve_provider_for_model(&k, if real { "google" } else { "mock" });
            (k, pr)
        }
        "voice_tts" => (if raw.is_empty() { if real { "local-omnivoice".into() } else { "mock-voice".into() } } else { raw }, if real { "local_tool".into() } else { "mock".into() }),
        "vision_analyze" | "quality_check" | "script" => {
            if real && !raw.is_empty() && !raw.starts_with("mock") {
                let pr = resolve_provider_for_model(&raw, "google");
                (raw, pr)
            } else if real && t != "script" {
                ("gemini-2.5-flash".into(), "google".into())
            } else {
                (if raw.is_empty() { "local".into() } else { raw }, "local".into())
            }
        }
        _ => (if raw.is_empty() { "local".into() } else { raw }, "local_tool".into()),
    };
    (key, prov)
}

pub fn estimate_node_cost(n: &Value, real: bool) -> f64 {
    if is_bypassed(n) {
        return 0.0;
    }
    let (model, _) = effective_model(n, real);
    let pa = &n["params"];
    let f = |k: &str, d: f64| pa.get(k).and_then(|x| x.as_f64()).unwrap_or(d);
    match n["type"].as_str().unwrap_or("") {
        "image_gen" | "image_edit" => models::estimate_cost(&model, &models::Usage { n: f("n", 1.0), duration_sec: 0.0, text_len: 0.0 }),
        "video_gen" => models::estimate_cost(&model, &models::Usage { n: 1.0, duration_sec: f("durationSec", 8.0) * f("variants", 1.0).max(1.0), text_len: 0.0 }),
        "voice_tts" => models::estimate_cost(&model, &models::Usage { n: 1.0, duration_sec: 0.0, text_len: p(n, "text").chars().count() as f64 }),
        "vision_analyze" | "quality_check" if real => models::estimate_cost(&model, &models::Usage { n: 1.0, duration_sec: 0.0, text_len: 0.0 }),
        _ => 0.0,
    }
}

/// Number of images an image_gen produces when its `scenes` port is wired (1 otherwise).
pub fn batch_count(graph: &Value, node: &Value) -> usize {
    let id = node["id"].as_str().unwrap_or("");
    let (mut count, mut found) = (0usize, false);
    for e in graph["edges"].as_array().into_iter().flatten() {
        if e["targetNodeId"] != id || e["targetPort"] != "scenes" {
            continue;
        }
        found = true;
        let src = graph["nodes"].as_array().into_iter().flatten().find(|n| n["id"] == e["sourceNodeId"]);
        count += match src.map(|s| (s["type"].as_str().unwrap_or(""), s)) {
            None => 0,
            Some(("script", s)) => episode_shots(&s["params"]).unwrap_or_else(|| s["params"]["targetSceneCount"].as_u64().unwrap_or(6) as usize),
            Some(("scene", _)) => 1,
            Some(_) => 6,
        };
    }
    if found { count.max(1) } else { 1 }
}

fn episode_shots(params: &Value) -> Option<usize> {
    if params["mode"] != "episode" {
        return None;
    }
    let ep = params["episodeId"].as_str().filter(|e| !e.is_empty())?;
    let proj = workspace::active_project_id();
    let path = workspace::project_path(&proj, &["content", "episodes", ep, "script.json"], false).ok()?;
    let v: Value = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    let n: usize = v["scenes"].as_array()?.iter().map(|s| if params["granularity"] == "scene" { 1 } else { s["shots"].as_array().map(|a| a.len()).unwrap_or(0) }).sum();
    (n > 0).then_some(n)
}

pub fn estimate_graph_cost(graph: &Value, ids: &[String], real: bool) -> f64 {
    let mut total = 0.0;
    for id in ids {
        if let Some(n) = graph["nodes"].as_array().into_iter().flatten().find(|n| n["id"] == id.as_str()) {
            let mut c = estimate_node_cost(n, real);
            if c > 0.0 && n["type"] == "image_gen" {
                c *= batch_count(graph, n).max(1) as f64;
            }
            total += c;
        }
    }
    (total * 1e6).round() / 1e6
}

/// Pre-run plan: per-node estimate, missing keys and spend status (GraphCostEstimateResponse).
pub fn plan(project: &str, graph: &Value, req: &RunReq) -> Result<Value, String> {
    let v = validator::validate(graph);
    if !v.is_valid {
        return Err(format!("Đồ thị không hợp lệ: {}", v.errors.join("; ")));
    }
    let to_run = super::graph_runner::select_nodes(graph, &v.topological_order, req)?;
    let skipped: Vec<&String> = v.topological_order.iter().filter(|i| !to_run.contains(i)).collect();
    let (mut estimates, mut missing, mut total) = (vec![], HashSet::new(), 0.0f64);
    for id in &to_run {
        let n = graph["nodes"].as_array().unwrap().iter().find(|n| n["id"] == id.as_str()).unwrap();
        let t = n["type"].clone();
        if is_bypassed(n) {
            estimates.push(json!({"nodeId": id, "nodeType": t, "model": "bypass", "provider": "none", "estimatedCostUsd": 0.0}));
        } else if is_mock_for(n, req) {
            let (m, _) = effective_model(n, false);
            estimates.push(json!({"nodeId": id, "nodeType": t, "model": m, "provider": "mock", "estimatedCostUsd": 0.0}));
        } else {
            let (m, prov) = effective_model(n, true);
            let mut c = estimate_node_cost(n, true);
            if c > 0.0 && n["type"] == "image_gen" {
                c = (c * batch_count(graph, n).max(1) as f64 * 1e6).round() / 1e6;
            }
            total += c;
            if matches!(prov.as_str(), "google" | "gemini" | "fal" | "replicate" | "cloudflare") && !key_vault::has_valid_key(&prov) {
                missing.insert(key_vault::normalize(&prov).to_string());
            }
            estimates.push(json!({"nodeId": id, "nodeType": t, "model": m, "provider": prov, "estimatedCostUsd": c}));
        }
    }
    total = (total * 1e4).round() / 1e4;
    let runs = workspace::project_path(project, &["runs"], false).unwrap_or_default();
    let mut spend = SpendGuard::new(runs).preview(total);
    if let Some(limit) = req.spending_limit_usd {
        if total > limit + 1e-9 {
            spend["canProceed"] = json!(false);
            spend["reason"] = json!(format!("Chi phí ước tính ${:.4} vượt trần mỗi lần chạy ${:.4}.", total, limit));
        }
    }
    let mut missing: Vec<_> = missing.into_iter().collect();
    missing.sort();
    Ok(json!({"projectId": project, "mode": req.mode, "mockMode": req.mock_mode, "runPolicy": req.policy_json(),
        "nodesToExecute": estimates, "skippedNodes": skipped, "totalEstimatedCostUsd": total, "requiresConfirmation": total > 0.0,
        "missingApiKeys": missing, "spendStatus": spend, "totalCostUsd": total, "isFreeMode": total == 0.0, "currency": "USD"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mock_models_are_free() {
        let n = json!({"id": "a", "type": "image_gen", "params": {"model": "mock-image"}});
        assert_eq!(estimate_node_cost(&n, false), 0.0);
        let (m, prov) = effective_model(&n, false);
        assert_eq!((m.as_str(), prov.as_str()), ("mock-image", "mock"));
    }
    #[test]
    fn batch_count_defaults_to_one() {
        let g = json!({"nodes": [{"id": "a", "type": "image_gen", "params": {}}], "edges": []});
        assert_eq!(batch_count(&g, &g["nodes"][0]), 1);
    }
    #[test]
    fn batch_count_from_script_scene_target() {
        let g = json!({"nodes": [{"id": "s", "type": "script", "params": {"targetSceneCount": 4}}, {"id": "i", "type": "image_gen", "params": {}}],
            "edges": [{"sourceNodeId": "s", "targetNodeId": "i", "targetPort": "scenes", "sourcePort": "scenes"}]});
        assert_eq!(batch_count(&g, &g["nodes"][1]), 4);
    }
}
