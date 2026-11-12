use super::util::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

fn lbl() -> String { "DỮ LIỆU THẬT".into() }
fn obj() -> Value { json!({}) }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Beat {
    #[serde(default)] pub t_start: f64,
    #[serde(default)] pub t_end: f64,
    #[serde(default)] pub role: String,
    #[serde(default)] pub summary: String,
    #[serde(default)] pub visual: String,
    #[serde(default)] pub on_screen_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoBlueprint {
    pub id: String,
    #[serde(default)] pub source_ref: String,
    #[serde(default)] pub duration_s: f64,
    #[serde(default)] pub platform: String,
    #[serde(default)] pub hook: String,
    #[serde(default)] pub beats: Vec<Beat>,
    #[serde(default)] pub cta: String,
    #[serde(default)] pub pacing_cuts_per_10s: f64,
    #[serde(default)] pub suggested_template_id: Option<String>,
    #[serde(default = "lbl")] pub data_label: String,
    #[serde(default)] pub is_mock: bool,
    #[serde(default = "obj")] pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBlueprint {
    pub id: String,
    #[serde(default)] pub channel_url: String,
    #[serde(default)] pub niche: String,
    #[serde(default)] pub pillars: Vec<String>,
    #[serde(default)] pub title_patterns: Vec<String>,
    #[serde(default)] pub avg_duration_s: f64,
    #[serde(default)] pub upload_cadence: String,
    #[serde(default)] pub top_videos: Vec<Value>,
    #[serde(default)] pub best_hooks: Vec<String>,
    #[serde(default = "lbl")] pub data_label: String,
    #[serde(default)] pub is_mock: bool,
    #[serde(default = "obj")] pub metadata: Value,
}

fn st_new() -> String { "new".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentIdea {
    pub id: String,
    #[serde(default)] pub project_id: String,
    #[serde(default)] pub title: String,
    #[serde(default)] pub platform_targets: Vec<String>,
    #[serde(default)] pub template_id: Option<String>,
    #[serde(default)] pub blueprint_ref: Option<String>,
    #[serde(default)] pub priority: i64,
    #[serde(default = "st_new")] pub status: String,
}

fn load<T: serde::de::DeserializeOwned>(p: &std::path::Path) -> Option<T> {
    serde_json::from_value(read_json(p)?).ok()
}

fn list<T: serde::de::DeserializeOwned>(dir: &std::path::Path, prefix: &str) -> Vec<T> {
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir(dir) {
        let mut paths: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            let n = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if n.starts_with(prefix) && n.ends_with(".json") {
                if let Some(v) = load(&p) { out.push(v); }
            }
        }
    }
    out
}

pub fn save_video_blueprint(pid: &str, bp: &VideoBlueprint) -> anyhow::Result<()> {
    write_json(&pp(pid, &["content", "blueprints", &format!("video_{}.json", safe_id(&bp.id)?)], true), bp)
}
pub fn load_video_blueprint(pid: &str, id: &str) -> Option<VideoBlueprint> {
    safe_id(id).ok()?;
    load(&pp(pid, &["content", "blueprints", &format!("video_{id}.json")], false))
}
pub fn list_video_blueprints(pid: &str) -> Vec<VideoBlueprint> { list(&pp(pid, &["content", "blueprints"], false), "video_") }

pub fn save_channel_blueprint(pid: &str, bp: &ChannelBlueprint) -> anyhow::Result<()> {
    write_json(&pp(pid, &["content", "blueprints", &format!("channel_{}.json", safe_id(&bp.id)?)], true), bp)
}
pub fn load_channel_blueprint(pid: &str, id: &str) -> Option<ChannelBlueprint> {
    safe_id(id).ok()?;
    load(&pp(pid, &["content", "blueprints", &format!("channel_{id}.json")], false))
}
pub fn list_channel_blueprints(pid: &str) -> Vec<ChannelBlueprint> { list(&pp(pid, &["content", "blueprints"], false), "channel_") }

pub fn save_content_idea(pid: &str, i: &ContentIdea) -> anyhow::Result<()> {
    write_json(&pp(pid, &["content", "ideas", &format!("{}.json", safe_id(&i.id)?)], true), i)
}
pub fn load_content_idea(pid: &str, id: &str) -> Option<ContentIdea> {
    safe_id(id).ok()?;
    load(&pp(pid, &["content", "ideas", &format!("{id}.json")], false))
}
pub fn list_content_ideas(pid: &str) -> Vec<ContentIdea> { list(&pp(pid, &["content", "ideas"], false), "") }
