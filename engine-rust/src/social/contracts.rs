use super::models::*;
use super::store::{atomic_write_json, SocialStore};
use crate::workspace;
use serde_json::{json, Value};
use std::path::Path;

pub fn create_draft_post(project: &str, media_path: &str, title: &str, caption: &str, platforms: &[String], source_ref: Option<&str>, render_job_id: Option<&str>) -> Result<String, String> {
    let st = SocialStore::new(project).map_err(|e| e.to_string())?;
    let id = new_id("post", 12);
    let mut title = title.to_string();
    if title.is_empty() || ["Video mới", "Video moi", "Untitled Draft"].contains(&title.as_str()) {
        title = if media_path.is_empty() { "Video Thành Phẩm".into() } else {
            let stem = Path::new(media_path).file_stem().map(|s| s.to_string_lossy().replace('_', " ")).unwrap_or_default();
            stem.split(' ').map(|w| { let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase()).unwrap_or_default() }).collect::<Vec<_>>().join(" ")
        };
    }
    let accounts = st.list_accounts();
    let mut targets = vec![];
    for p in platforms {
        let pl = p.to_lowercase();
        let pl = if PLATFORMS.contains(&pl.as_str()) { pl } else { "youtube".into() };
        let acc = accounts.iter().find(|a| s(a, "platform") == pl && s(a, "status") == "connected").or_else(|| accounts.iter().find(|a| s(a, "platform") == pl));
        let cap = match acc { Some(a) if !s(a, "default_caption_footer").is_empty() => format!("{caption}\n\n{}", s(a, "default_caption_footer")).trim().to_string(), _ => caption.to_string() };
        targets.push(json!({"account_id":acc.map(|a| s(a,"id").to_string()).unwrap_or_else(|| format!("acc_{pl}")),"platform":pl,"caption":cap,
            "hashtags":acc.map(|a| a["default_hashtags"].clone()).unwrap_or(json!([])),"title":title,"status":"draft"}));
    }
    let post = json!({"id":id,"project_id":project,"title":title,"media":[{"path":media_path,"kind":"video"}],"source_ref":source_ref,"render_job_id":render_job_id,
        "status":"draft","approval":{"required":true},"targets":targets});
    st.save_post(post).map_err(|e| e)?;
    st.append_audit("contracts", "create_draft_post", &id, None, None, None, "Created via social contract");
    // legacy inbox copy for old pollers (best-effort)
    if let Ok(p) = workspace::project_path(project, &["publishing", "inbox", &format!("{id}.json")], true) {
        let _ = atomic_write_json(&p, &json!({"post_id":id,"project_id":project,"media_path":media_path,"title":title,"caption":caption,"platforms":platforms,
            "source_ref":source_ref,"render_job_id":render_job_id,"status":"draft","created_at":chrono::Utc::now().to_rfc3339()}));
    }
    Ok(id)
}

#[allow(dead_code)] // wired when the inbox poller is moved off Python
pub fn ingest_inbox(project: &str) -> Vec<String> {
    let Ok(st) = SocialStore::new(project) else { return vec![] };
    let Ok(dir) = workspace::project_path(project, &["publishing", "inbox"], true) else { return vec![] };
    let done = dir.join("_done");
    let _ = std::fs::create_dir_all(&done);
    let mut out = vec![];
    let Ok(rd) = std::fs::read_dir(&dir) else { return out };
    for e in rd.flatten() {
        let p = e.path();
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        if name.starts_with('_') || !p.is_file() || !name.ends_with(".json") { continue; }
        let Some(d) = std::fs::read_to_string(&p).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) else { continue };
        let pid = d["post_id"].as_str().or(d["id"].as_str()).map(String::from).unwrap_or_else(|| new_id("post", 12));
        if st.get_post(&pid).is_none() {
            let plats: Vec<String> = d["platforms"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_else(|| vec!["youtube".into()]);
            let _ = create_draft_post(project, d["media_path"].as_str().or(d["video_uri"].as_str()).unwrap_or(""), d["title"].as_str().unwrap_or("Draft Post"), d["caption"].as_str().unwrap_or(""), &plats, d["source_ref"].as_str(), d["render_job_id"].as_str());
        }
        out.push(pid);
        let _ = std::fs::rename(&p, done.join(&name));
    }
    out
}
