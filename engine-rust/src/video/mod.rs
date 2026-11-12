//! Video track (Track B): HyperFrames rendering, job queue, templates, episode pipeline, editor, craft.
pub mod craft;
pub mod editor;
pub mod episode;
pub mod executors;
pub mod fallback;
pub mod hyperframes;
pub mod platforms;
pub mod queue;
pub mod render;
pub mod templates;
pub mod util;
pub mod verify;

use crate::engine::executor::ExecutorRegistry;
use anyhow::{Context, Result};
use std::sync::Arc;

/// Registers hyperframes_render, captions, word_captions, export, scene_cut, handdraw.
pub fn register(r: &mut ExecutorRegistry) {
    r.register(Arc::new(executors::VideoExecutor));
}

/// `vicieo render <project> <script.json>`: the JSON is a RenderRequest body (`data.scenes`, `voice`, `platforms`…).
pub async fn render_pipeline(project: &str, script_path: &str, output_path: Option<&str>) -> Result<()> {
    let body = std::fs::read_to_string(script_path).with_context(|| format!("không đọc được {script_path}"))?;
    let mut req: render::RenderRequest = serde_json::from_str(&body).context("script không phải RenderRequest JSON")?;
    req.project_id = project.to_string();
    if req.platforms.is_empty() {
        req.platforms = vec!["tiktok".into()];
    }
    if req.output_name.is_empty() {
        req.output_name = output_path.and_then(|o| std::path::Path::new(o).file_stem()).map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "final_video".into());
    }
    let work = crate::workspace::project_path(project, &["runs", "cli_render"], true)?;
    std::fs::create_dir_all(&work)?;
    let ctx = render::RenderCtx { cancel: tokio_util::sync::CancellationToken::new(), progress: Arc::new(|p, s| println!("[{p:5.1}%] {s}")), work_dir: work };
    let res = render::render_request(&req, &ctx).await?;
    for (p, o) in &res.outputs {
        println!("{p} -> data/projects/{project}/{o}");
    }
    res.warnings.iter().for_each(|w| eprintln!("warn: {w}"));
    Ok(())
}
#[cfg(test)]
mod tests;
