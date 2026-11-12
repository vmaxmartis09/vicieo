#![allow(clippy::too_many_arguments)] // provider entry points mirror the Python keyword-arg signatures
//! fal.ai queue client: submit -> poll status_url -> fetch response_url -> download -> save.
use super::{key_vault, media, models, spend_guard::SpendGuard};
use serde_json::{json, Value};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

const QUEUE: &str = "https://queue.fal.run";

fn size_for(aspect: &str) -> &'static str {
    match aspect {
        "16:9" => "landscape_16_9",
        "1:1" => "square_hd",
        "4:5" => "portrait_4_3",
        _ => "portrait_16_9",
    }
}

fn endpoint_for(model: &str, default_key: &str) -> String {
    for k in [model, default_key] {
        if let Some(s) = models::get(k) {
            if s["provider"] == "fal" {
                return s["model_id"].as_str().unwrap_or_default().to_string();
            }
        }
        if k.contains('/') {
            return k.to_string();
        }
    }
    "fal-ai/flux/schnell".into()
}

async fn submit_and_poll(c: &reqwest::Client, key: &str, endpoint: &str, body: &Value, cancel: &CancellationToken, timeout_s: u64) -> anyhow::Result<Value> {
    let auth = format!("Key {}", key);
    let r = c.post(format!("{}/{}", QUEUE, endpoint.trim_start_matches('/'))).header("Authorization", &auth).json(body).send().await?;
    let st = r.status();
    let job: Value = r.json().await.unwrap_or(json!({}));
    if !st.is_success() {
        anyhow::bail!("fal.ai HTTP {}: {}", st, key_vault::redact(&job.to_string().chars().take(240).collect::<String>()));
    }
    if job.get("images").is_some() || job.get("video").is_some() || job.get("image").is_some() {
        return Ok(job);
    }
    let (Some(status_url), Some(response_url)) = (job["status_url"].as_str(), job["response_url"].as_str()) else {
        anyhow::bail!("fal.ai thiếu status_url/response_url");
    };
    let started = std::time::Instant::now();
    loop {
        if cancel.is_cancelled() {
            if let Some(u) = job["cancel_url"].as_str() {
                let _ = c.put(u).header("Authorization", &auth).send().await;
            }
            anyhow::bail!("cancelled: đã hủy tác vụ fal.ai");
        }
        if started.elapsed().as_secs() > timeout_s {
            anyhow::bail!("fal.ai chưa xong sau {}s", timeout_s);
        }
        let s: Value = c.get(status_url).header("Authorization", &auth).send().await?.json().await.unwrap_or(json!({}));
        match s["status"].as_str().unwrap_or("").to_uppercase().as_str() {
            "COMPLETED" => break,
            "FAILED" | "ERROR" | "CANCELLED" | "CANCELED" => anyhow::bail!("fal.ai lỗi: {}", key_vault::redact(&s.to_string())),
            _ => {}
        }
        tokio::select! { _ = cancel.cancelled() => {}, _ = tokio::time::sleep(Duration::from_secs(2)) => {} }
    }
    Ok(c.get(response_url).header("Authorization", &auth).send().await?.json().await?)
}

pub struct Generated {
    pub rel: String,
    pub cost_usd: f64,
    pub model: String,
}

pub async fn generate_image(guard: &SpendGuard, project: &str, run: &str, name: &str, prompt: &str, model: &str, aspect: &str, n: u32, seed: Option<i64>, cancel: &CancellationToken) -> anyhow::Result<Vec<Generated>> {
    let key = key_vault::require_key("fal")?;
    let default_key = "fal-flux-schnell";
    let endpoint = endpoint_for(model, default_key);
    let cost = models::estimate_cost(if model.is_empty() { default_key } else { model }, &models::Usage { n: n as f64, duration_sec: 0.0, text_len: 0.0 });
    let label = format!("fal ảnh {} x{}", name, n);
    guard.reserve(cost, &label)?;
    let mut body = json!({"prompt": prompt, "num_images": n, "image_size": size_for(aspect), "aspect_ratio": aspect});
    if let Some(s) = seed {
        body["seed"] = json!(s);
    }
    let c = reqwest::Client::builder().timeout(Duration::from_secs(120)).build()?;
    let res = match submit_and_poll(&c, &key, &endpoint, &body, cancel, 300).await {
        Ok(r) => r,
        Err(e) => {
            guard.refund(cost, &label);
            return Err(e);
        }
    };
    let imgs: Vec<Value> = res["images"].as_array().cloned().unwrap_or_else(|| res.get("image").cloned().into_iter().collect());
    if imgs.is_empty() {
        guard.refund(cost, &label);
        anyhow::bail!("fal.ai không trả ảnh");
    }
    let per = cost / imgs.len() as f64;
    let mut out = vec![];
    for (i, im) in imgs.iter().enumerate() {
        let url = im["url"].as_str().ok_or_else(|| anyhow::anyhow!("ảnh thiếu url"))?;
        let bytes = media::download(&c, url).await?;
        let (_, rel) = media::save_bytes(project, run, &format!("{}_{}", name, i), &media::ext_from_url(url, "png"), &bytes)?;
        out.push(Generated { rel, cost_usd: per, model: endpoint.clone() });
    }
    Ok(out)
}

pub async fn generate_video(guard: &SpendGuard, project: &str, run: &str, name: &str, prompt: &str, model: &str, aspect: &str, duration: f64, first_frame_data_uri: Option<String>, cancel: &CancellationToken) -> anyhow::Result<Generated> {
    let key = key_vault::require_key("fal")?;
    let default_key = "fal-kling-v3-std";
    let endpoint = endpoint_for(model, default_key);
    let cost = models::estimate_cost(if model.is_empty() { default_key } else { model }, &models::Usage { n: 1.0, duration_sec: duration, text_len: 0.0 });
    let label = format!("fal video {} {}s", name, duration);
    guard.reserve(cost, &label)?;
    let mut body = json!({"prompt": prompt, "aspect_ratio": aspect});
    let ep = endpoint.to_lowercase();
    if ep.contains("kling") || ep.contains("seedance") || ep.contains("wan") {
        body["duration"] = json!((duration as i64).to_string());
    }
    if let Some(u) = first_frame_data_uri {
        body["image_url"] = json!(u);
        if ep.contains("kling") {
            body["start_image_url"] = body["image_url"].clone();
        }
    }
    let c = reqwest::Client::builder().timeout(Duration::from_secs(300)).build()?;
    let res = match submit_and_poll(&c, &key, &endpoint, &body, cancel, 900).await {
        Ok(r) => r,
        Err(e) => {
            guard.refund(cost, &label);
            return Err(e);
        }
    };
    let Some(url) = res["video"]["url"].as_str() else {
        guard.refund(cost, &label);
        anyhow::bail!("fal.ai không trả video");
    };
    let bytes = media::download(&c, url).await?;
    let (_, rel) = media::save_bytes(project, run, name, &media::ext_from_url(url, "mp4"), &bytes)?;
    Ok(Generated { rel, cost_usd: cost, model: endpoint })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn size_map() {
        assert_eq!(size_for("16:9"), "landscape_16_9");
        assert_eq!(size_for("9:16"), "portrait_16_9");
    }
}
