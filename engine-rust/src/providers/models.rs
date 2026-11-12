//! Model price book: data/shared/models/models.json (same file the dashboard `/models` serves).
use crate::workspace;
use serde_json::Value;

pub fn load() -> Vec<Value> {
    workspace::shared_path(&["models", "models.json"], false)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<Vec<Value>>(&t).ok())
        .unwrap_or_default()
}

/// Exact key first, then model_id equality / key substring (same fallback as ModelRegistry.get).
pub fn get(key: &str) -> Option<Value> {
    let all = load();
    if let Some(m) = all.iter().find(|m| m["key"] == key) {
        return Some(m.clone());
    }
    let k = key.to_lowercase();
    all.into_iter().find(|m| {
        m["model_id"].as_str().map(|s| s.to_lowercase() == k).unwrap_or(false)
            || m["key"].as_str().map(|s| s.to_lowercase().contains(&k)).unwrap_or(false)
    })
}

pub struct Usage {
    pub n: f64,
    pub duration_sec: f64,
    pub text_len: f64,
}

/// Port of ModelRegistry.estimate_cost.
pub fn estimate_cost(key: &str, u: &Usage) -> f64 {
    let Some(spec) = get(key) else { return 0.0 };
    let p = &spec["pricing"];
    let f = |k: &str| p.get(k).and_then(|v| v.as_f64());
    let r = |x: f64| (x * 10000.0).round() / 10000.0;
    if let Some(c) = f("cost_per_generation") {
        return r(c * u.n);
    }
    if let Some(c) = f("cost_per_second") {
        return r(c * u.duration_sec);
    }
    if let Some(c) = f("cost_per_minute") {
        return r(c * u.duration_sec / 60.0);
    }
    if let Some(c) = f("cost_per_1k_chars") {
        return r(c * u.text_len / 1000.0);
    }
    if let Some(c) = f("cost_per_1k_tokens") {
        return r(c * (u.text_len / 4.0).max(1.0) / 1000.0);
    }
    0.0
}
