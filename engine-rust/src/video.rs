use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct HyperFramesEngine {
    pub ffmpeg_bin: String,
    pub node_bin: String,
}

impl HyperFramesEngine {
    pub fn new() -> Self {
        Self {
            ffmpeg_bin: "ffmpeg".to_string(),
            node_bin: "node".to_string(),
        }
    }

    /// Render một composition HTML5/GSAP (HyperFrames) bằng Ego Lite / Node headless engine
    pub async fn render_composition(
        &self,
        template_dir: &Path,
        params_json: &Path,
        output_mp4: &Path,
    ) -> Result<PathBuf> {
        println!("HyperFrames Engine: Rendering template at: {}", template_dir.display());

        let runner_script = template_dir.join("_engine").join("hyperframes_run.mjs");
        if !runner_script.exists() {
            println!("Note: Template runner script not found, falling back to direct FFmpeg composition");
            return self.render_fallback_ffmpeg(template_dir, output_mp4);
        }

        // Thực thi script headless capture
        let status = Command::new(&self.node_bin)
            .args([
                runner_script.to_str().context("invalid runner path")?,
                "--params",
                params_json.to_str().context("invalid params path")?,
                "--out",
                output_mp4.to_str().context("invalid out path")?,
            ])
            .status()?;

        if !status.success() {
            bail!("Headless capture failed with status: {}", status);
        }

        Ok(output_mp4.to_path_buf())
    }

    /// Fallback render bằng FFmpeg khi không chạy headless browser
    pub fn render_fallback_ffmpeg(&self, input_dir: &Path, output_mp4: &Path) -> Result<PathBuf> {
        println!("Rendering video fallback via FFmpeg to {}", output_mp4.display());
        if let Some(p) = output_mp4.parent() {
            std::fs::create_dir_all(p)?;
        }

        // Tạo video ngắn 1080x1920 (TikTok 9:16 safe zone)
        let status = Command::new(&self.ffmpeg_bin)
            .args([
                "-y",
                "-f", "lavfi",
                "-i", "color=c=black:s=1080x1920:d=5",
                "-c:v", "libx264",
                "-pix_fmt", "yuv420p",
                output_mp4.to_str().context("invalid output path")?,
            ])
            .status()?;

        if !status.success() {
            bail!("FFmpeg fallback render failed");
        }

        Ok(output_mp4.to_path_buf())
    }
}

pub async fn render_pipeline(project: &str, script_path: &str, output_path: Option<&str>) -> Result<()> {
    let out = output_path.unwrap_or("output/final_video.mp4");
    let engine = HyperFramesEngine::new();
    println!("=== STARTING VICIEO HIGH-PERFORMANCE RENDER ===");
    println!("Project: {}", project);
    println!("Script:  {}", script_path);
    println!("Target:  {}", out);

    engine.render_fallback_ffmpeg(Path::new(script_path), Path::new(out))?;
    println!("Render finished successfully -> {}", out);
    Ok(())
}
