use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct AudioMastering {
    pub ffmpeg_bin: String,
}

impl AudioMastering {
    pub fn new(ffmpeg_bin: Option<String>) -> Self {
        Self {
            ffmpeg_bin: ffmpeg_bin.unwrap_or_else(|| "ffmpeg".to_string()),
        }
    }

    /// Mastering chuẩn TikTok KOC: de-esser -> EQ presence -> compressor -> -14 LUFS -> MP3/WAV
    pub fn master_tiktok(
        &self,
        input_audio: &Path,
        output_audio: &Path,
        target_lufs: f64,
        presence_boost_db: f64,
        tempo_factor: f64,
        bitrate: &str,
    ) -> Result<PathBuf> {
        if let Some(parent) = output_audio.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let tempo = tempo_factor.clamp(0.5, 2.0);
        let mut filters = vec![
            "deesser=i=0.4:m=0.4:f=6500:s=o".to_string(),
            format!("equalizer=f=3200:t=q:w=2.0:g={}", presence_boost_db),
            "acompressor=threshold=-18dB:ratio=4:attack=15:release=120:makeup=2dB".to_string(),
        ];

        if (tempo - 1.0).abs() > 1e-3 {
            filters.push(format!("atempo={}", tempo));
        }

        filters.push(format!("loudnorm=I={}:LRA=8:TP=-1.5", target_lufs));
        let filter_str = filters.join(",");

        let status = Command::new(&self.ffmpeg_bin)
            .args([
                "-y",
                "-i",
                input_audio.to_str().context("invalid input path")?,
                "-af",
                &filter_str,
                "-ar",
                "44100",
                "-ac",
                "1",
                "-b:a",
                bitrate,
                output_audio.to_str().context("invalid output path")?,
            ])
            .status()?;

        if !status.success() {
            bail!("FFmpeg mastering failed with status {}", status);
        }

        Ok(output_audio.to_path_buf())
    }

    /// Trộn giọng đọc và nhạc nền BGM với hiệu ứng Ducking (-14 LUFS)
    pub fn mix_voice_and_bgm_ducking(
        &self,
        voice_path: &Path,
        bgm_path: &Path,
        output_path: &Path,
        target_lufs: f64,
    ) -> Result<PathBuf> {
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let filtergraph = format!(
            "[1:a]volume=0.35[bgm];[bgm][0:a]sidechaincompress=threshold=0.08:ratio=4:attack=80:release=350[ducked_bgm];[0:a][ducked_bgm]amix=inputs=2:duration=first:dropout_transition=2,loudnorm=I={}:LRA=8:TP=-1.5[out]",
            target_lufs
        );

        let status = Command::new(&self.ffmpeg_bin)
            .args([
                "-y",
                "-i",
                voice_path.to_str().context("invalid voice path")?,
                "-i",
                bgm_path.to_str().context("invalid bgm path")?,
                "-filter_complex",
                &filtergraph,
                "-map",
                "[out]",
                "-ar",
                "44100",
                "-ac",
                "2",
                "-b:a",
                "192k",
                output_path.to_str().context("invalid out path")?,
            ])
            .status()?;

        if !status.success() {
            bail!("FFmpeg mix ducking failed with status {}", status);
        }

        Ok(output_path.to_path_buf())
    }
}

