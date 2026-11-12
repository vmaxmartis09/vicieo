//! Post-render verification with ffprobe/ffmpeg (port of verify.py; Whisper WER is not ported — it needs a model).
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use super::util::{ffmpeg_bin, probe, run};

#[derive(Debug, Clone, Default, Serialize)]
pub struct VerifyReport {
    pub video_path: String,
    pub pass_all: bool,
    pub duration_s: f64,
    pub width: u32,
    pub height: u32,
    pub video_codec: String,
    pub audio_codec: String,
    pub has_audio: bool,
    pub loudness_lufs: f64,
    pub mean_volume_db: f64,
    pub max_silence_s: f64,
    pub black_detected_s: f64,
    pub freeze_detected_s: f64,
    pub scene_count: u32,
    pub contact_sheet_path: Option<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl VerifyReport {
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(json!({}))
    }
}

#[derive(Default)]
pub struct VerifyOpts {
    pub expect_duration: Option<(f64, f64)>, // (seconds, tolerance)
    pub expect_size: Option<(u32, u32)>,
    pub expect_audio: bool,
    pub target_lufs: f64,
    pub contact_sheet: Option<PathBuf>,
}

fn nums(text: &str, key: &str) -> Vec<f64> {
    let mut out = vec![];
    for l in text.lines() {
        if let Some(i) = l.find(key) {
            if let Some(v) = l[i + key.len()..].trim_start().split(|c: char| c.is_whitespace() || c == '|').next().and_then(|s| s.parse::<f64>().ok()) {
                out.push(v);
            }
        }
    }
    out
}

pub async fn verify_video(path: &Path, o: &VerifyOpts) -> VerifyReport {
    let mut r = VerifyReport { video_path: path.display().to_string(), ..Default::default() };
    if std::fs::metadata(path).map(|m| m.len() == 0).unwrap_or(true) {
        r.errors.push(format!("Tệp video không tồn tại hoặc rỗng: {}", path.display()));
        return r;
    }
    match probe(path).await {
        Ok(p) => {
            r.duration_s = p.duration;
            r.width = p.width;
            r.height = p.height;
            r.video_codec = p.video_codec;
            r.audio_codec = p.audio_codec;
            r.has_audio = p.has_audio;
        }
        Err(e) => {
            r.errors.push(format!("Lỗi ffprobe: {e}"));
            return r;
        }
    }
    if r.video_codec.is_empty() {
        r.errors.push("Không có luồng video".into());
    } else if !["h264", "hevc", "av1", "vp9"].contains(&r.video_codec.as_str()) {
        r.warnings.push(format!("Codec video là {}, khuyến nghị h264", r.video_codec));
    }
    if r.duration_s <= 0.0 {
        r.errors.push("Thời lượng bằng 0".into());
    }
    if let Some((d, tol)) = o.expect_duration {
        if (r.duration_s - d).abs() > tol {
            r.errors.push(format!("Thời lượng {:.1}s không đạt yêu cầu {d}±{tol}s", r.duration_s));
        }
    }
    if let Some((w, h)) = o.expect_size {
        if (r.width, r.height) != (w, h) {
            r.errors.push(format!("Kích thước {}x{} không khớp {w}x{h}", r.width, r.height));
        }
    }
    let cancel = CancellationToken::new();

    if r.has_audio {
        if !["aac", "mp3"].contains(&r.audio_codec.as_str()) {
            r.warnings.push(format!("Codec audio là {}, khuyến nghị aac/mp3", r.audio_codec));
        }
        let mut c = Command::new(ffmpeg_bin());
        c.args(["-hide_banner", "-nostats", "-i"]).arg(path).args(["-vn", "-af", "ebur128=framelog=quiet,volumedetect,silencedetect=n=-40dB:d=0.5", "-f", "null", "-"]);
        match run(&mut c, &cancel, Duration::from_secs(300), None).await {
            Ok(out) => {
                if let Some(v) = nums(&out.stderr, "mean_volume:").first() {
                    r.mean_volume_db = *v;
                }
                // ebur128 summary: "    I:         -14.0 LUFS"
                if let Some(v) = out.stderr.lines().rev().find(|l| l.trim_start().starts_with("I:")).and_then(|l| nums(l, "I:").first().copied()) {
                    r.loudness_lufs = v;
                }
                let sil = nums(&out.stderr, "silence_duration:");
                r.max_silence_s = sil.iter().cloned().fold(0.0, f64::max);
                if r.mean_volume_db < -35.0 {
                    let m = format!("Âm lượng quá nhỏ (mean volume {} dB < -35 dB)", r.mean_volume_db);
                    if o.expect_audio { r.errors.push(m) } else { r.warnings.push(m) }
                }
                let target = if o.target_lufs != 0.0 { o.target_lufs } else { -14.0 };
                if r.loudness_lufs != 0.0 && (r.loudness_lufs - target).abs() > 3.0 {
                    r.warnings.push(format!("Loudness {} LUFS lệch chuẩn {target}±3 LUFS", r.loudness_lufs));
                }
            }
            Err(e) => r.warnings.push(format!("Không đo được âm lượng: {e}")),
        }
    } else if o.expect_audio {
        r.errors.push("Video không có luồng audio".into());
    } else {
        r.warnings.push("Video không có luồng audio (im lặng)".into());
    }

    let mut c = Command::new(ffmpeg_bin());
    c.args(["-hide_banner", "-nostats", "-i"]).arg(path).args(["-an", "-vf", "blackdetect=d=0.5:pix_th=0.1,freezedetect=n=0.003:d=0.8", "-f", "null", "-"]);
    match run(&mut c, &cancel, Duration::from_secs(300), None).await {
        Ok(out) => {
            r.black_detected_s = nums(&out.stderr, "black_duration:").into_iter().fold(0.0, f64::max);
            r.freeze_detected_s = nums(&out.stderr, "freeze_duration:").into_iter().fold(0.0, f64::max);
            if r.black_detected_s > 0.8 {
                r.errors.push(format!("Phát hiện khung đen kéo dài {:.2}s > 0.8s", r.black_detected_s));
            }
            if r.freeze_detected_s > 2.0 {
                r.warnings.push(format!("Phát hiện đứng hình {:.2}s", r.freeze_detected_s));
            }
        }
        Err(e) => r.warnings.push(format!("Lỗi kiểm tra black/freeze: {e}")),
    }

    let mut c = Command::new(ffmpeg_bin());
    c.args(["-hide_banner", "-nostats", "-i"]).arg(path).args(["-an", "-vf", "select='gt(scene,0.3)',showinfo", "-f", "null", "-"]);
    if let Ok(out) = run(&mut c, &cancel, Duration::from_secs(300), None).await {
        r.scene_count = out.stderr.matches("pts_time:").count() as u32 + 1;
    }

    if let Some(sheet) = &o.contact_sheet {
        let fps = 12.0 / r.duration_s.max(1.0);
        let mut c = Command::new(ffmpeg_bin());
        c.args(["-hide_banner", "-y", "-i"]).arg(path).args(["-vf", &format!("fps={fps:.4},scale=240:-1,tile=6x2"), "-frames:v", "1", "-q:v", "3"]).arg(sheet);
        if run(&mut c, &cancel, Duration::from_secs(120), None).await.map(|o| o.code == 0).unwrap_or(false) && sheet.exists() {
            r.contact_sheet_path = Some(sheet.display().to_string());
        }
    }
    r.pass_all = r.errors.is_empty();
    r
}
