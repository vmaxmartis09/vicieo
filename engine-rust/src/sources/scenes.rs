use super::downloader::probe_video_duration;
use crate::intelligence::util::*;
use regex::Regex;
use serde_json::{json, Value};
use std::path::Path;
use tokio::process::Command;

async fn frame(video: &Path, t: f64, out: &Path) -> bool {
    let _ = std::fs::create_dir_all(out.parent().unwrap());
    Command::new("ffmpeg").args(["-y", "-ss", &format!("{t:.3}"), "-i", &video.to_string_lossy(), "-vframes", "1", "-q:v", "2", &out.to_string_lossy()]).output().await.is_ok_and(|o| o.status.success()) && out.exists()
}

/// ffmpeg `select=gt(scene,0.3)` cuts, >1s apart; always at least one scene.
pub async fn detect_cuts(video: &Path, duration: f64) -> Vec<(f64, f64)> {
    let mut cuts = vec![0.0f64];
    if let Ok(o) = Command::new("ffmpeg").args(["-i", &video.to_string_lossy(), "-filter_complex", "select='gt(scene,0.3)',metadata=print:file=-", "-f", "null", "-"]).output().await {
        let re = Regex::new(r"pts_time:([0-9.]+)").unwrap();
        let text = format!("{}\n{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
        for c in re.captures_iter(&text) {
            if let Ok(p) = c[1].parse::<f64>() { if p > cuts.last().unwrap() + 1.0 { cuts.push(p); } }
        }
    }
    if *cuts.last().unwrap() < duration { cuts.push(duration); }
    if cuts.len() <= 1 { return vec![(0.0, duration.max(1.0))]; }
    cuts.windows(2).map(|w| (w[0], w[1])).collect()
}

pub async fn detect_and_save_scenes(pid: &str, source_id: &str) -> ApiResult<Vec<Value>> {
    let dir = pp(pid, &["raw", safe_id(source_id).map_err(|e| ApiError::bad(e.to_string()))?, "x"], false).parent().unwrap().to_path_buf();
    let video = dir.join("video.mp4");
    if !video.exists() { return Err(ApiError::nf(format!("Video file not found at: {}", video.display()))); }
    let dur = probe_video_duration(&video).await;
    let mut out = vec![];
    for (i, (st, en)) in detect_cuts(&video, dur).await.into_iter().enumerate() {
        let name = format!("scene_{:03}.jpg", i + 1);
        let ok = frame(&video, (st + en) / 2.0, &dir.join("scenes").join(&name)).await;
        out.push(json!({"scene_idx": i + 1, "start_s": (st * 100.0).round() / 100.0, "end_s": (en * 100.0).round() / 100.0,
                        "thumb": if ok { format!("raw/{source_id}/scenes/{name}") } else { String::new() }}));
    }
    write_json(&dir.join("scenes.json"), &out)?;
    Ok(out)
}
