//! Saving generated media under data/projects/<id>/output and building the public file URL.
use crate::workspace;
use std::path::PathBuf;

pub fn files_url(project_id: &str, rel: &str) -> String {
    format!("/data/projects/{}/files/{}", project_id, rel.trim_start_matches('/'))
}

/// Returns (absolute path, project-relative path).
pub fn save_bytes(project_id: &str, run_id: &str, name: &str, ext: &str, bytes: &[u8]) -> anyhow::Result<(PathBuf, String)> {
    let dir = workspace::project_path(project_id, &["output", run_id], false)?;
    std::fs::create_dir_all(&dir)?;
    let file = format!("{}.{}", name, ext);
    let p = dir.join(&file);
    std::fs::write(&p, bytes)?;
    Ok((p, format!("output/{}/{}", run_id, file)))
}

pub async fn download(client: &reqwest::Client, url: &str) -> anyhow::Result<Vec<u8>> {
    let r = client.get(url).send().await?;
    if !r.status().is_success() {
        anyhow::bail!("tải kết quả thất bại: HTTP {}", r.status());
    }
    Ok(r.bytes().await?.to_vec())
}

pub fn ext_from_url(url: &str, default: &str) -> String {
    let path = url.split('?').next().unwrap_or(url);
    match path.rsplit('.').next() {
        Some(e) if e.len() <= 4 && e.chars().all(|c| c.is_ascii_alphanumeric()) => e.to_lowercase(),
        _ => default.to_string(),
    }
}
