//! yt-dlp / ffmpeg subprocess ingestion (port of sources/downloader.py + local.py).
use super::models::*;
use super::service::save_source;
use crate::intelligence::util::*;
use once_cell::sync::Lazy;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::process::Command;

static LAST: Lazy<Mutex<HashMap<String, Instant>>> = Lazy::new(Default::default);

pub fn detect_platform(url: &str) -> &'static str {
    let u = url.to_lowercase();
    if u.contains("douyin.com") || u.contains("iesdouyin.com") { "douyin" }
    else if u.contains("tiktok.com") { "tiktok" }
    else if u.contains("youtube.com") || u.contains("youtu.be") { "youtube" }
    else if u.contains("bilibili.com") { "bilibili" }
    else if u.contains("facebook.com") || u.contains("fb.watch") { "facebook" }
    else if u.contains("instagram.com") { "instagram" }
    else if u.starts_with("file://") || u.starts_with('/') || [".mp4", ".mov", ".mkv", ".avi", ".webm"].iter().any(|e| u.ends_with(e)) || Path::new(url).exists() { "local" }
    else { "youtube" }
}

pub fn generate_source_id(url: &str) -> String {
    let h = format!("{:x}", Sha256::digest(url.as_bytes()));
    format!("{}_{}", detect_platform(url), &h[..12])
}

async fn run(cmd: &str, args: &[&str], secs: u64) -> Option<std::process::Output> {
    tokio::time::timeout(Duration::from_secs(secs), Command::new(cmd).args(args).output()).await.ok()?.ok()
}

pub async fn extract_audio_wav(video: &Path, audio: &Path) -> bool {
    run("ffmpeg", &["-y", "-i", &video.to_string_lossy(), "-vn", "-acodec", "pcm_s16le", "-ar", "16000", "-ac", "1", &audio.to_string_lossy()], 120).await
        .is_some_and(|o| o.status.success()) && audio.exists()
}

pub async fn extract_thumbnail_jpg(video: &Path, thumb: &Path) -> bool {
    for ss in ["00:00:01", "00:00:00"] {
        if run("ffmpeg", &["-y", "-ss", ss, "-i", &video.to_string_lossy(), "-vframes", "1", "-q:v", "2", &thumb.to_string_lossy()], 30).await.is_some_and(|o| o.status.success()) && thumb.exists() { return true; }
    }
    false
}

pub async fn probe_video_duration(video: &Path) -> f64 {
    run("ffprobe", &["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", &video.to_string_lossy()], 15).await
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok()).unwrap_or(0.0)
}

fn failed(pid: &str, id: &str, platform: &str, url: &str, rights: &str, note: &str, err: String) -> SourceItem {
    let item = SourceItem { id: id.into(), project_id: pid.into(), platform: platform.into(), url: url.into(), rights: rights.into(), rights_note: note.into(), status: "failed".into(), error: Some(err), ..Default::default() };
    let _ = save_source(&item);
    item
}

pub async fn download_source(pid: &str, url: &str, rights: &str, rights_note: &str) -> anyhow::Result<SourceItem> {
    let platform = detect_platform(url);
    if platform == "local" && Path::new(url.trim_start_matches("file://")).exists() {
        return ingest_local_file(pid, Path::new(url.trim_start_matches("file://")), None, rights, rights_note).await;
    }
    let id = generate_source_id(url);
    let dir = pp(pid, &["raw", safe_id(&id)?, "x"], true).parent().unwrap().to_path_buf();
    let (video, audio, thumb) = (dir.join("video.mp4"), dir.join("audio.wav"), dir.join("thumb.jpg"));

    // 2s min interval per platform
    let wait = { let mut m = LAST.lock().unwrap(); let now = Instant::now();
        let w = m.get(platform).map(|t| Duration::from_secs(2).saturating_sub(now - *t)).unwrap_or_default();
        m.insert(platform.into(), now + w); w };
    tokio::time::sleep(wait).await;

    let out_tpl = dir.join("video.%(ext)s").to_string_lossy().to_string();
    let cookie = std::env::var("YTDLP_COOKIES").ok().filter(|p| Path::new(p).exists());
    let mut last_err = String::new();
    for attempt in 1..=3u64 {
        let mut args: Vec<&str> = vec!["-f", "bestvideo[ext=mp4][height<=1080]+bestaudio[ext=m4a]/best[ext=mp4][height<=1080]/best", "-o", &out_tpl,
            "--merge-output-format", "mp4", "--write-thumbnail", "--write-subs", "--write-auto-subs", "--sub-langs", "zh-Hans,zh,vi,en", "--no-playlist", "--no-warnings", "-q", "--print-json"];
        if let Some(c) = &cookie { args.extend(["--cookies", c]); }
        args.push(url);
        let out = Command::new("yt-dlp").args(&args).output().await;
        match out {
            Ok(o) if o.status.success() => {
                let info: Value = String::from_utf8_lossy(&o.stdout).lines().rev().find_map(|l| serde_json::from_str(l).ok()).unwrap_or(Value::Null);
                if !video.exists() { // yt-dlp may leave .mkv/.webm
                    if let Ok(rd) = std::fs::read_dir(&dir) {
                        for e in rd.flatten() { let p = e.path(); if p.file_stem().is_some_and(|s| s == "video") && matches!(p.extension().and_then(|x| x.to_str()), Some("mkv" | "webm")) { let _ = std::fs::rename(&p, &video); break; } }
                    }
                }
                if let Ok(rd) = std::fs::read_dir(&dir) {
                    for e in rd.flatten() { let p = e.path(); if p.file_stem().is_some_and(|s| s == "video") && matches!(p.extension().and_then(|x| x.to_str()), Some("jpg" | "jpeg" | "webp" | "png")) { let _ = std::fs::rename(&p, &thumb); break; } }
                }
                if !thumb.exists() && video.exists() { extract_thumbnail_jpg(&video, &thumb).await; }
                if video.exists() && !audio.exists() { extract_audio_wav(&video, &audio).await; }
                let sub = std::fs::read_dir(&dir).ok().and_then(|rd| rd.flatten().find(|e| e.path().extension().is_some_and(|x| x == "vtt")).map(|e| format!("raw/{id}/{}", e.file_name().to_string_lossy())));
                let g = |k: &str| info.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let n = |k: &str| info.get(k).and_then(|v| v.as_i64()).unwrap_or(0);
                let item = SourceItem {
                    id: id.clone(), project_id: pid.into(), platform: platform.into(), url: url.into(),
                    author: if g("uploader").is_empty() { g("channel") } else { g("uploader") },
                    author_url: if g("uploader_url").is_empty() { g("channel_url") } else { g("uploader_url") },
                    title: g("title"), description: g("description"), duration_s: info.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    published_at: g("upload_date"), stats: SourceStats { views: n("view_count"), likes: n("like_count"), comments: n("comment_count"), shares: n("repost_count") },
                    hashtags: info.get("tags").and_then(|t| t.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
                    music: g("track"), rights: rights.into(), rights_note: rights_note.into(),
                    files: SourceFiles { video: video.exists().then(|| format!("raw/{id}/video.mp4")), audio: audio.exists().then(|| format!("raw/{id}/audio.wav")), thumb: thumb.exists().then(|| format!("raw/{id}/thumb.jpg")), subtitle_orig: sub },
                    status: "ready".into(), ..Default::default()
                };
                save_source(&item)?;
                return Ok(item);
            }
            other => {
                last_err = match other { Ok(o) => String::from_utf8_lossy(&o.stderr).trim().to_string(), Err(e) => e.to_string() };
                let low = last_err.to_lowercase();
                if (platform == "douyin" || platform == "tiktok") && ["captcha", "verify", "login", "cookie"].iter().any(|k| low.contains(k)) {
                    return Ok(failed(pid, &id, platform, url, rights, rights_note, format!("Nền tảng {platform} yêu cầu cookie hoặc xác thực danh tính. Hệ thống tuân thủ chính sách: không cố vượt captcha. Vui lòng cấu hình cookiefile hợp lệ (biến YTDLP_COOKIES) từ trình duyệt cá nhân để tiếp tục.")));
                }
                if attempt < 3 { tokio::time::sleep(Duration::from_secs(attempt)).await; }
            }
        }
    }
    Ok(failed(pid, &id, platform, url, rights, rights_note, format!("Tải thất bại sau 3 lần thử: {last_err}")))
}

pub async fn ingest_local_file(pid: &str, src: &Path, title: Option<&str>, rights: &str, note: &str) -> anyhow::Result<SourceItem> {
    if !src.exists() { anyhow::bail!("Source file not found: {}", src.display()); }
    let id = format!("local_{}", &uuid::Uuid::new_v4().simple().to_string()[..12]);
    let dir = pp(pid, &["raw", &id, "x"], true).parent().unwrap().to_path_buf();
    let (video, audio, thumb) = (dir.join("video.mp4"), dir.join("audio.wav"), dir.join("thumb.jpg"));
    std::fs::copy(src, &video)?;
    let dur = probe_video_duration(&video).await;
    extract_thumbnail_jpg(&video, &thumb).await;
    extract_audio_wav(&video, &audio).await;
    let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("video");
    let item = SourceItem {
        id: id.clone(), project_id: pid.into(), platform: "local".into(), url: format!("file://{}", src.file_name().and_then(|s| s.to_str()).unwrap_or("")),
        author: "User".into(), title: title.unwrap_or(stem).into(), duration_s: dur, rights: rights.into(), rights_note: note.into(),
        files: SourceFiles { video: video.exists().then(|| format!("raw/{id}/video.mp4")), audio: audio.exists().then(|| format!("raw/{id}/audio.wav")), thumb: thumb.exists().then(|| format!("raw/{id}/thumb.jpg")), subtitle_orig: None },
        status: "ready".into(), ..Default::default()
    };
    save_source(&item)?;
    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn platform_and_id() {
        assert_eq!(detect_platform("https://v.douyin.com/abc"), "douyin");
        assert_eq!(detect_platform("https://youtu.be/x"), "youtube");
        assert_eq!(detect_platform("/tmp/a.mp4"), "local");
        let id = generate_source_id("https://www.tiktok.com/@a/video/1");
        assert!(id.starts_with("tiktok_") && id.len() == 7 + 12);
    }
}
