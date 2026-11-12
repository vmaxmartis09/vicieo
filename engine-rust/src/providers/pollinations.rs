//! Free image generation via image.pollinations.ai (optional Pollen key from the vault).
use super::{key_vault, media, models};
use std::time::Duration;

pub async fn generate_image(project: &str, run: &str, name: &str, prompt: &str, model: &str, aspect: &str, seed: Option<i64>) -> anyhow::Result<(String, String)> {
    let model_id = models::get(model).and_then(|m| m["model_id"].as_str().map(String::from)).unwrap_or_else(|| "flux".into());
    let (w, h) = match aspect { "16:9" => (1280, 720), "1:1" => (1024, 1024), "4:5" => (1024, 1280), _ => (720, 1280) };
    let enc: String = prompt.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{:02X}", b) }).collect();
    let mut url = format!("https://image.pollinations.ai/prompt/{}?model={}&width={}&height={}&nologo=true", enc, model_id, w, h);
    if let Some(s) = seed {
        url += &format!("&seed={}", s);
    }
    let c = reqwest::Client::builder().timeout(Duration::from_secs(120)).build()?;
    let mut req = c.get(&url);
    let key = key_vault::raw_key("pollinations");
    if !key.is_empty() {
        req = req.bearer_auth(key);
    }
    let r = req.send().await?;
    if !r.status().is_success() {
        anyhow::bail!("pollinations HTTP {}", r.status());
    }
    let bytes = r.bytes().await?;
    let (_, rel) = media::save_bytes(project, run, name, "jpg", &bytes)?;
    Ok((rel, model_id))
}
