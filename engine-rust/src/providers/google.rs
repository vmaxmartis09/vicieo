use anyhow::{Context, Result};
use serde_json::Value;

pub async fn generate_text(prompt: &str, provider: &str, model: Option<&str>) -> Result<String> {
    println!("Querying AI Provider: [{}] Model: [{}]", provider, model.unwrap_or("default"));

    match provider.to_lowercase().as_str() {
        "gemini" => {
            let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
            let model_name = model.unwrap_or("gemini-2.5-flash");
            println!("Calling Gemini Gateway via async reqwest for prompt: {}", prompt);
            
            // Native HTTP call to Gemini Generative Language REST API
            let client = reqwest::Client::new();
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model_name, api_key
            );

            let body = serde_json::json!({
                "contents": [{
                    "parts": [{"text": prompt}]
                }]
            });

            if api_key.is_empty() {
                println!("Note: GEMINI_API_KEY is not set in environment or .env. Running in dry-run mode.");
                return Ok(format!("[DRY RUN RESPONSE] Text generation successful for: {}", prompt));
            }

            let resp: Value = client.post(&url).json(&body).send().await?.json().await?;
            let output = resp["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .unwrap_or("No content returned")
                .to_string();

            println!("Response:\n{}", output);
            Ok(output)
        }
        _ => {
            anyhow::bail!("Provider {} is not yet supported in native core", provider);
        }
    }
}


pub struct GeminiText;

#[async_trait::async_trait]
impl super::TextLlm for GeminiText {
    async fn generate(&self, prompt: &str, _system: Option<&str>, model: Option<&str>, _json_mode: bool) -> Result<String> {
        generate_text(prompt, "gemini", model).await
    }
}
