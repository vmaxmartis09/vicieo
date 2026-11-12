//! Cloudflare Workers AI (FLUX schnell). Needs CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID.
use super::{key_vault, media};
use base64::Engine;
use std::time::Duration;

pub async fn generate_image(project: &str, run: &str, name: &str, prompt: &str, seed: Option<i64>) -> anyhow::Result<String> {
    let token = key_vault::require_key("cloudflare")?;
    let account = std::env::var("CLOUDFLARE_ACCOUNT_ID").ok().filter(|s| !s.trim().is_empty())
        .or_else(|| std::fs::read_to_string(crate::workspace::repo_root().join(".env")).ok().and_then(|t| t.lines().find_map(|l| l.trim().strip_prefix("CLOUDFLARE_ACCOUNT_ID=").map(|v| v.trim().trim_matches('"').to_string()))))
        .ok_or_else(|| anyhow::anyhow!("Chưa có CLOUDFLARE_ACCOUNT_ID."))?;
    let mut body = serde_json::json!({"prompt": prompt, "steps": 4});
    if let Some(s) = seed {
        body["seed"] = s.into();
    }
    let c = reqwest::Client::builder().timeout(Duration::from_secs(60)).build()?;
    let r = c.post(format!("https://api.cloudflare.com/client/v4/accounts/{}/ai/run/@cf/black-forest-labs/flux-1-schnell", account)).bearer_auth(&token).json(&body).send().await?;
    let st = r.status();
    if st.as_u16() == 429 {
        anyhow::bail!("Cloudflare Workers AI chạm giới hạn neurons/ngày của gói Free (HTTP 429).");
    }
    let ct = r.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let bytes = r.bytes().await?;
    if !st.is_success() {
        anyhow::bail!("Cloudflare HTTP {}: {}", st, key_vault::redact(&String::from_utf8_lossy(&bytes).chars().take(240).collect::<String>()));
    }
    let img = if ct.contains("json") {
        let v: serde_json::Value = serde_json::from_slice(&bytes)?;
        base64::engine::general_purpose::STANDARD.decode(v["result"]["image"].as_str().ok_or_else(|| anyhow::anyhow!("Cloudflare không trả ảnh"))?)?
    } else {
        bytes.to_vec()
    };
    Ok(media::save_bytes(project, run, name, "jpg", &img)?.1)
}
