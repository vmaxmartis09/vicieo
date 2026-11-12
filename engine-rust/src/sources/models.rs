use crate::intelligence::util::now_iso;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SourceFiles { pub video: Option<String>, pub audio: Option<String>, pub thumb: Option<String>, pub subtitle_orig: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SourceStats { pub views: i64, pub likes: i64, pub comments: i64, pub shares: i64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SourceItem {
    pub id: String, pub project_id: String, pub platform: String, pub url: String, pub author: String, pub author_url: String,
    pub title: String, pub description: String, pub duration_s: f64, pub published_at: String, pub stats: SourceStats,
    pub hashtags: Vec<String>, pub music: String, pub rights: String, pub rights_note: String, pub files: SourceFiles,
    pub status: String, pub error: Option<String>, pub created_at: String,
}
impl Default for SourceItem {
    fn default() -> Self {
        Self {
            id: String::new(), project_id: String::new(), platform: "local".into(), url: String::new(), author: String::new(), author_url: String::new(),
            title: String::new(), description: String::new(), duration_s: 0.0, published_at: String::new(), stats: SourceStats::default(),
            hashtags: vec![], music: String::new(), rights: "reference_only".into(), rights_note: String::new(), files: SourceFiles::default(),
            status: "queued".into(), error: None, created_at: now_iso(),
        }
    }
}

pub const RIGHTS: &[&str] = &["owned", "licensed", "permission", "reference_only"];
pub const PLATFORMS: &[&str] = &["douyin", "tiktok", "youtube", "bilibili", "facebook", "instagram", "local"];
