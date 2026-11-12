//! Platform presets (port of video/platforms.py).
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeZone {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformPreset {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub aspect_ratio: String,
    pub fps: u32,
    pub max_duration_sec: Option<f64>,
    pub safe_zone: SafeZone,
    pub loudness_lufs: f64,
    pub audio_bitrate: String,
    pub video_bitrate: String,
    pub codec: String,
    pub audio_codec: String,
    pub notes: String,
}

#[allow(clippy::too_many_arguments)]
fn p(id: &str, name: &str, w: u32, h: u32, ar: &str, maxd: Option<f64>, sz: (u32, u32, u32, u32), vb: &str, notes: &str) -> PlatformPreset {
    PlatformPreset {
        id: id.into(),
        name: name.into(),
        width: w,
        height: h,
        aspect_ratio: ar.into(),
        fps: 30,
        max_duration_sec: maxd,
        safe_zone: SafeZone { top: sz.0, bottom: sz.1, left: sz.2, right: sz.3 },
        loudness_lufs: -14.0,
        audio_bitrate: "192k".into(),
        video_bitrate: vb.into(),
        codec: "libx264".into(),
        audio_codec: "aac".into(),
        notes: notes.into(),
    }
}

pub fn list_platforms() -> Vec<PlatformPreset> {
    vec![
        p("tiktok", "TikTok", 1080, 1920, "9:16", Some(600.0), (180, 320, 40, 120), "8000k", "cột nút phải"),
        p("douyin", "Douyin", 1080, 1920, "9:16", Some(900.0), (200, 340, 40, 120), "8000k", ""),
        p("reels", "Instagram Reels", 1080, 1920, "9:16", Some(180.0), (220, 380, 40, 100), "8000k", ""),
        p("shorts", "YouTube Shorts", 1080, 1920, "9:16", Some(180.0), (160, 300, 40, 110), "8000k", ""),
        p("youtube", "YouTube Standard", 1920, 1080, "16:9", None, (60, 90, 60, 60), "12000k", "title-safe 90%"),
        p("facebook", "Facebook Feed", 1080, 1350, "4:5", Some(240.0), (80, 160, 40, 40), "8000k", "feed"),
    ]
}

pub fn get_platform(id: &str) -> anyhow::Result<PlatformPreset> {
    let key = id.to_lowercase();
    let key = key.trim();
    list_platforms()
        .into_iter()
        .find(|p| p.id == key)
        .ok_or_else(|| anyhow::anyhow!("Nền tảng '{}' không hợp lệ. Hỗ trợ: tiktok, douyin, reels, shorts, youtube, facebook", id))
}
