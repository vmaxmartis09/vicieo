use super::models::*;
use crate::intelligence::util::*;
use std::path::PathBuf;

pub fn list_sources(pid: &str, platform: Option<&str>, rights: Option<&str>, q: Option<&str>) -> Vec<SourceItem> {
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir(pp(pid, &["raw"], false)) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if !e.path().is_dir() || n.starts_with('_') || n.starts_with('.') { continue; }
            let Some(v) = read_json(&e.path().join("meta.json")) else { continue };
            let Ok(item) = serde_json::from_value::<SourceItem>(v) else { continue };
            if platform.is_some_and(|p| item.platform != p) || rights.is_some_and(|r| item.rights != r) { continue; }
            if let Some(q) = q {
                let text = format!("{} {} {} {}", item.title, item.description, item.author, item.hashtags.join(" ")).to_lowercase();
                if !text.contains(&q.to_lowercase()) { continue; }
            }
            out.push(item);
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    out
}

pub fn get_source(pid: &str, id: &str) -> Option<SourceItem> {
    safe_id(id).ok()?;
    serde_json::from_value(read_json(&pp(pid, &["raw", id, "meta.json"], false))?).ok()
}

pub fn save_source(item: &SourceItem) -> anyhow::Result<()> {
    write_json(&pp(&item.project_id, &["raw", safe_id(&item.id)?, "meta.json"], true), item)
}

pub fn update_source_rights(pid: &str, id: &str, new_rights: &str, note: Option<&str>, confirm: bool) -> ApiResult<SourceItem> {
    let mut item = get_source(pid, id).ok_or_else(|| ApiError::nf("Không tìm thấy nguồn nội dung."))?;
    if !RIGHTS.contains(&new_rights) { return Err(ApiError::unproc(format!("rights must be one of {RIGHTS:?}"))); }
    if item.rights == "reference_only" && new_rights != "reference_only" && !confirm {
        return Err(ApiError::bad("Đổi bản quyền khỏi 'reference_only' bắt buộc phải có confirm_rights=true để xác nhận bạn sở hữu hoặc được phép sử dụng nội dung này cho video thương mại."));
    }
    item.rights = new_rights.into();
    if let Some(n) = note { item.rights_note = n.into(); }
    save_source(&item)?;
    Ok(item)
}

pub fn delete_source_to_trash(pid: &str, id: &str) -> bool {
    if safe_id(id).is_err() { return false; }
    let dir = pp(pid, &["raw", id], false);
    if !dir.exists() { return false; }
    let target = pp(pid, &["raw", "_trash", id], true);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::rename(dir, target).is_ok()
}

/// Resolves `file_name` (may be nested like scenes/scene_001.jpg) inside raw/<id>/, refusing traversal.
pub fn get_source_file_path(pid: &str, id: &str, file_name: &str) -> Option<PathBuf> {
    safe_id(id).ok()?;
    let rel = file_name.trim_matches('/');
    if rel.is_empty() || rel.split('/').any(|p| p.is_empty() || p == ".." || p == ".") { return None; }
    let parts: Vec<&str> = std::iter::once("raw").chain(std::iter::once(id)).chain(rel.split('/')).collect();
    let f = pp(pid, &parts, false);
    f.is_file().then_some(f)
}
