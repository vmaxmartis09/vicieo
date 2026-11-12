//! Provider gateway traits. Track A implements real backends (google/fal/replicate/...);
//! until then everything resolves to `Mock*` so callers never break.
pub mod cloudflare;
pub mod executors;
pub mod fal;
pub mod google;
pub mod google_media;
pub mod key_vault;
pub mod media;
pub mod models;
pub mod pollinations;
pub mod replicate;
pub mod router;
pub mod spend_guard;

use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

#[async_trait]
pub trait TextLlm: Send + Sync {
    async fn generate(&self, prompt: &str, system: Option<&str>, model: Option<&str>, json_mode: bool) -> anyhow::Result<String>;
}

#[derive(Debug, Clone)]
pub struct MediaResult {
    pub path: PathBuf,
    pub cost_usd: f64,
    pub meta: Value,
}

#[async_trait]
pub trait ImageGen: Send + Sync {
    async fn generate(&self, prompt: &str, model: Option<&str>, params: &Value, out_dir: &std::path::Path) -> anyhow::Result<MediaResult>;
}

#[async_trait]
pub trait VideoGen: Send + Sync {
    async fn generate(&self, prompt: &str, model: Option<&str>, params: &Value, out_dir: &std::path::Path) -> anyhow::Result<MediaResult>;
}

pub struct MockText;
#[async_trait]
impl TextLlm for MockText {
    async fn generate(&self, prompt: &str, _s: Option<&str>, _m: Option<&str>, _j: bool) -> anyhow::Result<String> {
        Ok(format!("[MOCK] {}", prompt))
    }
}

pub struct MockImage;
#[async_trait]
impl ImageGen for MockImage {
    async fn generate(&self, _p: &str, _m: Option<&str>, _params: &Value, _o: &std::path::Path) -> anyhow::Result<MediaResult> {
        anyhow::bail!("mock image provider produces no file")
    }
}

pub struct MockVideo;
#[async_trait]
impl VideoGen for MockVideo {
    async fn generate(&self, _p: &str, _m: Option<&str>, _params: &Value, _o: &std::path::Path) -> anyhow::Result<MediaResult> {
        anyhow::bail!("mock video provider produces no file")
    }
}

/// Text LLM selected by env: GEMINI_API_KEY -> Gemini, otherwise Mock.
pub fn llm() -> Arc<dyn TextLlm> {
    if std::env::var("GEMINI_API_KEY").map(|k| !k.is_empty()).unwrap_or(false) {
        Arc::new(google::GeminiText)
    } else {
        Arc::new(MockText)
    }
}
