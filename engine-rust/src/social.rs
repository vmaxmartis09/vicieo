use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    TikTok,
    YouTube,
    Meta,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostStatus {
    Draft,
    Scheduled,
    Publishing,
    Published,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialPost {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub caption: String,
    pub video_path: String,
    pub platforms: Vec<Platform>,
    pub status: PostStatus,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub metadata: Value,
}

pub struct SocialPublisher {
    pub project_id: String,
}

impl SocialPublisher {
    pub fn new(project_id: &str) -> Self {
        Self {
            project_id: project_id.to_string(),
        }
    }

    pub async fn publish_tiktok(&self, post: &SocialPost) -> Result<String> {
        println!("Dispatching TikTok publish job for video: {}", post.video_path);
        // Direct TikTok Content Posting API via async reqwest or Browser assisted
        Ok(format!("tiktok_publish_ok_{}", post.id))
    }

    pub async fn publish_youtube_shorts(&self, post: &SocialPost) -> Result<String> {
        println!("Dispatching YouTube Shorts publish job: {}", post.title);
        // YouTube Data API v3 upload
        Ok(format!("youtube_publish_ok_{}", post.id))
    }
}

