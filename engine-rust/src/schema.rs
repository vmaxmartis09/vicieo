use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Core video script schema (Deterministic fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoScript {
    pub title: String,
    pub duration_seconds: f64,
    pub aspect_ratio: String,
    pub scenes: Vec<SceneNode>,
    /// Two-Tier dynamic extension payload: any arbitrary AI/custom field
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: String,
    pub order: u32,
    pub duration: f64,
    pub narration: Option<String>,
    pub visual_prompt: Option<String>,
    pub character: Option<String>,
    /// Extension attributes without breaking contract
    #[serde(flatten)]
    pub extra: Value,
}

/// Project Manifest Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub id: String,
    pub name: String,
    pub default_character: Option<String>,
    pub safe_zone: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

