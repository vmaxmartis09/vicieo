//! Shared helpers: external binaries, cancellable child processes, ffprobe, media path resolution.
use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::workspace;

pub fn ffmpeg_bin() -> String {
    std::env::var("FFMPEG_BIN").unwrap_or_else(|_| "ffmpeg".into())
}
pub fn ffprobe_bin() -> String {
    std::env::var("FFPROBE_BIN").unwrap_or_else(|_| "ffprobe".into())
}

pub fn is_valid_slug(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub struct CmdOut {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub type LineHook = Arc<dyn Fn(&str) + Send + Sync>;

/// Run a child process. Cancel kills the whole process group (hyperframes spawns Chrome).
pub async fn run(cmd: &mut Command, cancel: &CancellationToken, timeout: Duration, on_line: Option<LineHook>) -> Result<CmdOut> {
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
    #[cfg(unix)]
    cmd.process_group(0);
    let mut child = cmd.spawn().context("không khởi chạy được tiến trình con")?;
    let pid = child.id();
    let mut out_lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut err_lines = BufReader::new(child.stderr.take().unwrap()).lines();
    let hook1 = on_line.clone();
    let t_out = tokio::spawn(async move {
        let mut s = String::new();
        while let Ok(Some(l)) = out_lines.next_line().await {
            if let Some(h) = &hook1 {
                h(&l);
            }
            s.push_str(&l);
            s.push('\n');
        }
        s
    });
    let t_err = tokio::spawn(async move {
        let mut s = String::new();
        while let Ok(Some(l)) = err_lines.next_line().await {
            if let Some(h) = &on_line {
                h(&l);
            }
            s.push_str(&l);
            s.push('\n');
        }
        s
    });
    let status = tokio::select! {
        r = child.wait() => r?,
        _ = cancel.cancelled() => {
            kill_group(pid);
            let _ = child.kill().await;
            bail!("cancelled");
        }
        _ = tokio::time::sleep(timeout) => {
            kill_group(pid);
            let _ = child.kill().await;
            bail!("timeout sau {}s", timeout.as_secs());
        }
    };
    let stdout = t_out.await.unwrap_or_default();
    let stderr = t_err.await.unwrap_or_default();
    Ok(CmdOut { code: status.code().unwrap_or(-1), stdout, stderr })
}

fn kill_group(pid: Option<u32>) {
    #[cfg(unix)]
    if let Some(pid) = pid {
        let _ = std::process::Command::new("kill").args(["-KILL", &format!("-{pid}")]).output();
    }
    #[cfg(not(unix))]
    let _ = pid;
}

/// ffmpeg with `-hide_banner -y` prepended; errors carry the stderr tail.
pub async fn ffmpeg(args: &[String], cancel: &CancellationToken, timeout_s: u64) -> Result<CmdOut> {
    let mut c = Command::new(ffmpeg_bin());
    c.args(["-hide_banner", "-y"]).args(args);
    let o = run(&mut c, cancel, Duration::from_secs(timeout_s), None).await?;
    if o.code != 0 {
        bail!("ffmpeg lỗi ({}): {}", o.code, tail(&o.stderr, 600));
    }
    Ok(o)
}

pub fn tail(s: &str, n: usize) -> String {
    let c: Vec<char> = s.chars().collect();
    c[c.len().saturating_sub(n)..].iter().collect()
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Probe {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub video_codec: String,
    pub audio_codec: String,
    pub has_audio: bool,
    pub size: u64,
}

pub fn probe_sync(path: &Path) -> Result<Probe> {
    let out = std::process::Command::new(ffprobe_bin())
        .args(["-v", "error", "-show_entries", "format=duration,size:stream=codec_type,codec_name,width,height", "-of", "json"])
        .arg(path)
        .output()
        .context("không chạy được ffprobe")?;
    if !out.status.success() {
        bail!("ffprobe lỗi: {}", String::from_utf8_lossy(&out.stderr));
    }
    let v: Value = serde_json::from_slice(&out.stdout)?;
    let mut p = Probe {
        duration: v["format"]["duration"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
        size: v["format"]["size"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0),
        ..Default::default()
    };
    for s in v["streams"].as_array().cloned().unwrap_or_default() {
        match s["codec_type"].as_str() {
            Some("video") if p.video_codec.is_empty() => {
                p.video_codec = s["codec_name"].as_str().unwrap_or("").into();
                p.width = s["width"].as_u64().unwrap_or(0) as u32;
                p.height = s["height"].as_u64().unwrap_or(0) as u32;
            }
            Some("audio") if !p.has_audio => {
                p.has_audio = true;
                p.audio_codec = s["codec_name"].as_str().unwrap_or("").into();
            }
            _ => {}
        }
    }
    Ok(p)
}

pub async fn probe(path: &Path) -> Result<Probe> {
    let p = path.to_path_buf();
    tokio::task::spawn_blocking(move || probe_sync(&p)).await?
}

/// Audio/video duration in seconds; `default` when the file is missing/unreadable.
pub fn media_duration(path: &Path, default: f64) -> f64 {
    probe_sync(path).map(|p| p.duration).ok().filter(|d| *d > 0.0).unwrap_or(default)
}

/// Resolve a media reference (URL served by the engine, project-relative, or absolute) to a file on disk.
pub fn resolve_media(project_id: &str, r: &str) -> Option<PathBuf> {
    let r = r.trim();
    if r.is_empty() || r.starts_with("http") {
        return None;
    }
    let proj = workspace::project_path(project_id, &[], false).ok()?;
    let candidates: Vec<PathBuf> = {
        let mut c = vec![];
        for pre in ["/data/projects/", "/projects/"] {
            if let Some(rest) = r.strip_prefix(pre) {
                if let Some((pid, tail)) = rest.split_once("/files/") {
                    if is_valid_slug(pid) {
                        c.push(workspace::data_root().join("projects").join(pid).join(tail));
                    }
                }
            }
        }
        let rp = Path::new(r);
        if rp.is_absolute() {
            c.push(rp.to_path_buf());
        }
        c.push(proj.join(r.trim_start_matches('/')));
        c.push(workspace::data_root().join("shared").join(r.trim_start_matches('/')));
        c
    };
    let data_root = workspace::data_root();
    candidates.into_iter().find(|p| {
        p.is_file() && !p.components().any(|c| c.as_os_str() == "..") && (p.starts_with(&data_root) || p.starts_with(std::env::temp_dir()) || p.is_absolute())
    })
}

pub fn project_file_url(project_id: &str, rel: &str) -> String {
    format!("/data/projects/{}/files/{}", project_id, rel)
}

/// Whether ffmpeg was built with the drawtext filter (not true for the stock Homebrew build).
pub fn has_drawtext() -> bool {
    static V: once_cell::sync::Lazy<bool> = once_cell::sync::Lazy::new(|| {
        std::process::Command::new(ffmpeg_bin())
            .args(["-hide_banner", "-filters"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(" drawtext "))
            .unwrap_or(false)
    });
    *V
}

pub fn font_file() -> Option<String> {
    if let Ok(f) = std::env::var("VICIEO_FONT") {
        return Some(f);
    }
    ["/System/Library/Fonts/Supplemental/Arial Unicode.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", "C:/Windows/Fonts/arial.ttf"]
        .iter()
        .find(|p| Path::new(p).exists())
        .map(|s| s.to_string())
}

pub fn atomic_write(path: &Path, content: &str) -> Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    let tmp = path.with_extension(format!("tmp{}", uuid::Uuid::new_v4().simple()));
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path).map_err(|e| anyhow!("ghi {} thất bại: {e}", path.display()))
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}
