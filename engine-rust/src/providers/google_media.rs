#![allow(clippy::too_many_arguments)] // provider entry points mirror the Python keyword-arg signatures
//! Gemini image (generateContent) and Veo video (predictLongRunning + poll) via the Generative Language API.
use super::{fal::Generated, key_vault, media, models, spend_guard::SpendGuard};
use base64::Engine;
use serde_json::{json, Value};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

const API: &str = "https://generativelanguage.googleapis.com/v1beta";

fn model_id(key: &str, default_key: &str) -> (String, String) {
    let k = if key.is_empty() { default_key } else { key };
    match models::get(k).filter(|m| m["provider"] == "google") {
        Some(m) => (k.to_string(), m["model_id"].as_str().unwrap_or(k).to_string()),
        None => (default_key.to_string(), models::get(default_key).and_then(|m| m["model_id"].as_str().map(String::from)).unwrap_or_else(|| k.to_string())),
    }
}

async fn post(c: &reqwest::Client, key: &str, url: &str, body: &Value) -> anyhow::Result<Value> {
    let r = c.post(url).header("x-goog-api-key", key).json(body).send().await?;
    let st = r.status();
    let v: Value = r.json().await.unwrap_or(json!({}));
    if !st.is_success() {
        anyhow::bail!("Google HTTP {}: {}", st, key_vault::redact(&v.to_string().chars().take(300).collect::<String>()));
    }
    Ok(v)
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub async fn generate_image(guard: &SpendGuard, project: &str, run: &str, name: &str, prompt: &str, model: &str, aspect: &str, seed: Option<i64>, refs: &[(Vec<u8>, String)]) -> anyhow::Result<Generated> {
    let key = key_vault::require_key("gemini")?;
    let (mkey, mid) = model_id(model, "gemini-2.5-flash-image");
    let cost = models::estimate_cost(&mkey, &models::Usage { n: 1.0, duration_sec: 0.0, text_len: 0.0 });
    let label = format!("google ảnh {}", name);
    guard.reserve(cost, &label)?;
    let mut parts: Vec<Value> = refs.iter().map(|(d, m)| json!({"inlineData": {"mimeType": m, "data": b64(d)}})).collect();
    parts.push(json!({"text": prompt}));
    let mut cfg = json!({"responseModalities": ["IMAGE"], "imageConfig": {"aspectRatio": aspect}});
    if let Some(s) = seed {
        cfg["seed"] = json!(s);
    }
    let c = reqwest::Client::builder().timeout(Duration::from_secs(120)).build()?;
    let resp = match post(&c, &key, &format!("{}/models/{}:generateContent", API, mid), &json!({"contents": [{"parts": parts}], "generationConfig": cfg})).await {
        Ok(r) => r,
        Err(e) => {
            guard.refund(cost, &label);
            return Err(e);
        }
    };
    let img = resp["candidates"].as_array().into_iter().flatten().flat_map(|c| c["content"]["parts"].as_array().into_iter().flatten()).find_map(|p| p.get("inlineData").cloned());
    let Some(img) = img else {
        guard.refund(cost, &label);
        anyhow::bail!("Gemini không trả ảnh: {}", key_vault::redact(&resp.to_string().chars().take(300).collect::<String>()));
    };
    let bytes = base64::engine::general_purpose::STANDARD.decode(img["data"].as_str().unwrap_or(""))?;
    let ext = if img["mimeType"].as_str().unwrap_or("").contains("jpeg") { "jpg" } else { "png" };
    let (_, rel) = media::save_bytes(project, run, name, ext, &bytes)?;
    Ok(Generated { rel, cost_usd: cost, model: mid })
}

pub async fn generate_video(guard: &SpendGuard, project: &str, run: &str, name: &str, prompt: &str, model: &str, aspect: &str, duration: f64, first: Option<(Vec<u8>, String)>, cancel: &CancellationToken) -> anyhow::Result<Generated> {
    let key = key_vault::require_key("gemini")?;
    let (mkey, mid) = model_id(model, "google-veo-3.1-fast");
    let cost = models::estimate_cost(&mkey, &models::Usage { n: 1.0, duration_sec: duration, text_len: 0.0 });
    let label = format!("google video {} {}s", name, duration);
    guard.reserve(cost, &label)?;
    let mut inst = json!({"prompt": prompt});
    if let Some((d, m)) = first {
        inst["image"] = json!({"bytesBase64Encoded": b64(&d), "mimeType": m});
    }
    let c = reqwest::Client::builder().timeout(Duration::from_secs(300)).build()?;
    let run_it = async {
        let mut op = post(&c, &key, &format!("{}/models/{}:predictLongRunning", API, mid), &json!({"instances": [inst], "parameters": {"aspectRatio": aspect, "durationSeconds": duration as i64}})).await?;
        let started = std::time::Instant::now();
        while op["done"] != true {
            if cancel.is_cancelled() {
                anyhow::bail!("cancelled: đã hủy chờ video Veo");
            }
            if started.elapsed().as_secs() > 900 {
                anyhow::bail!("Veo chưa xong sau 900s (việc có thể vẫn chạy trên Google)");
            }
            tokio::select! { _ = cancel.cancelled() => {}, _ = tokio::time::sleep(Duration::from_secs(5)) => {} }
            let name = op["name"].as_str().unwrap_or("").to_string();
            op = c.get(format!("{}/{}", API, name)).header("x-goog-api-key", &key).send().await?.json().await?;
        }
        if let Some(e) = op.get("error") {
            anyhow::bail!("Veo lỗi: {}", key_vault::redact(&e.to_string()));
        }
        let r = &op["response"]["generateVideoResponse"];
        if r["raiMediaFilteredCount"].as_u64().unwrap_or(0) > 0 {
            anyhow::bail!("Veo chặn video do bộ lọc an toàn: {}", r["raiMediaFilteredReasons"]);
        }
        let uri = r["generatedSamples"][0]["video"]["uri"].as_str().ok_or_else(|| anyhow::anyhow!("Veo không trả video"))?.to_string();
        let resp = c.get(&uri).header("x-goog-api-key", &key).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("tải video Veo thất bại: HTTP {}", resp.status());
        }
        anyhow::Ok(resp.bytes().await?.to_vec())
    };
    match run_it.await {
        Ok(bytes) => {
            let (_, rel) = media::save_bytes(project, run, name, "mp4", &bytes)?;
            Ok(Generated { rel, cost_usd: cost, model: mid })
        }
        Err(e) => {
            guard.refund(cost, &label);
            Err(e)
        }
    }
}
