#![allow(clippy::too_many_arguments)] // provider entry points mirror the Python keyword-arg signatures
//! Replicate image generation: POST /models/<slug>/predictions then poll `urls.get`.
use super::{fal::Generated, key_vault, media, models, spend_guard::SpendGuard};
use serde_json::{json, Value};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub async fn generate_image(guard: &SpendGuard, project: &str, run: &str, name: &str, prompt: &str, model: &str, aspect: &str, n: u32, cancel: &CancellationToken) -> anyhow::Result<Vec<Generated>> {
    let key = key_vault::require_key("replicate")?;
    let k = if model.is_empty() { "replicate-flux-schnell" } else { model };
    let slug = models::get(k).and_then(|m| m["model_id"].as_str().map(String::from)).unwrap_or_else(|| k.to_string());
    let cost = models::estimate_cost(k, &models::Usage { n: n as f64, duration_sec: 0.0, text_len: 0.0 });
    let label = format!("replicate ảnh {} x{}", name, n);
    guard.reserve(cost, &label)?;
    let c = reqwest::Client::builder().timeout(Duration::from_secs(120)).build()?;
    let fail = |e: anyhow::Error| {
        guard.refund(cost, &label);
        e
    };
    let r = c.post(format!("https://api.replicate.com/v1/models/{}/predictions", slug)).bearer_auth(&key)
        .json(&json!({"input": {"prompt": prompt, "aspect_ratio": aspect, "num_outputs": n.max(1)}})).send().await.map_err(|e| fail(e.into()))?;
    let st = r.status();
    let mut pred: Value = r.json().await.unwrap_or(json!({}));
    if !st.is_success() {
        return Err(fail(anyhow::anyhow!("Replicate HTTP {}: {}", st, key_vault::redact(&pred.to_string().chars().take(240).collect::<String>()))));
    }
    let poll = pred["urls"]["get"].as_str().map(String::from);
    let started = std::time::Instant::now();
    while matches!(pred["status"].as_str(), Some("starting") | Some("processing")) {
        if cancel.is_cancelled() {
            return Err(fail(anyhow::anyhow!("cancelled: đã hủy tác vụ Replicate")));
        }
        if started.elapsed().as_secs() > 300 {
            return Err(fail(anyhow::anyhow!("Replicate chưa xong sau 300s")));
        }
        let Some(u) = &poll else { break };
        tokio::select! { _ = cancel.cancelled() => {}, _ = tokio::time::sleep(Duration::from_secs(2)) => {} }
        pred = c.get(u).bearer_auth(&key).send().await.map_err(|e| fail(e.into()))?.json().await.map_err(|e| fail(e.into()))?;
    }
    if pred["status"] != "succeeded" {
        return Err(fail(anyhow::anyhow!("Replicate thất bại: {}", key_vault::redact(&pred["error"].as_str().unwrap_or(pred["status"].as_str().unwrap_or("?")).to_string()))));
    }
    let urls: Vec<String> = match &pred["output"] {
        Value::String(s) => vec![s.clone()],
        Value::Array(a) => a.iter().filter_map(|x| x.as_str().map(String::from)).collect(),
        _ => vec![],
    };
    if urls.is_empty() {
        return Err(fail(anyhow::anyhow!("Replicate không trả ảnh")));
    }
    let per = cost / urls.len() as f64;
    let mut out = vec![];
    for (i, u) in urls.iter().enumerate() {
        let bytes = media::download(&c, u).await.map_err(&fail)?;
        let (_, rel) = media::save_bytes(project, run, &format!("{}_{}", name, i), &media::ext_from_url(u, "webp"), &bytes)?;
        out.push(Generated { rel, cost_usd: per, model: slug.clone() });
    }
    Ok(out)
}
