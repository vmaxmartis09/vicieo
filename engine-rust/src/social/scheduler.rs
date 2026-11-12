//! Scheduler: every 30s picks due `scheduled` targets, locks them `publishing`, publishes via adapter,
//! retries with 2/10/30 min backoff, marks permanent failures. Mirrors scheduler.py.
use super::adapters::{self, Ctx};
use super::models::*;
use super::store::SocialStore;
use crate::workspace;
use chrono::{DateTime, Duration, FixedOffset};
use serde_json::{json, Value};
use std::fs;

pub const POLL_SECS: u64 = 30;

fn heartbeat_path(project: &str) -> Option<std::path::PathBuf> { workspace::project_path(project, &["publishing", "scheduler.heartbeat"], true).ok() }
pub fn get_heartbeat(project: &str) -> Option<String> { fs::read_to_string(heartbeat_path(project)?).ok().map(|s| s.trim().to_string()) }

/// One evaluation cycle; `now` injectable for tests.
pub async fn tick(project: &str, now: Option<DateTime<FixedOffset>>, dry_run: bool) -> Vec<Value> {
    let Ok(st) = SocialStore::new(project) else { return vec![] };
    if let Some(p) = heartbeat_path(project) { let _ = fs::write(p, now_iso()); }
    let now = now.unwrap_or_else(|| chrono::Utc::now().with_timezone(&tz()));
    let mut results = vec![];
    for mut post in st.list_posts(Some("scheduled"), None, None, None, None, None) {
        if post["approval"]["required"] != json!(false) && !matches!(s(&post, "status"), "approved" | "scheduled") { continue; }
        let mut modified = false;
        let n = post["targets"].as_array().map_or(0, |t| t.len());
        for i in 0..n {
            let t = post["targets"][i].clone();
            if s(&t, "status") != "scheduled" { continue; }
            let Some(when) = t["scheduled_at"].as_str().and_then(parse_dt) else { continue };
            if when > now { continue; }
            let (pid, tid) = (s(&post, "id").to_string(), s(&t, "id").to_string());
            post["targets"][i]["status"] = json!("publishing");
            if let Ok(p) = st.save_post(post.clone()) { post = p; }
            st.append_audit("scheduler", "lock_target", &pid, Some(&tid), Some("scheduled"), Some("publishing"), "");
            let mut account = st.get_account(s(&t, "account_id"));
            let acc_v = account.clone().unwrap_or_else(|| json!({"id":t["account_id"],"platform":t["platform"],"handle":t["account_id"],"display_name":t["account_id"],"auth":{"mode":"manual"}}));
            let res = adapters::publish(&Ctx { project_id: project, post_id: &pid, dry_run }, &acc_v, &post["targets"][i], post["media"].get(0)).await;
            let plat = s(&t, "platform").to_string();
            let cap = { let mut c = plat.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() };
            let ptitle = s(&post, "title").to_string();
            let tg = &mut post["targets"][i];
            if res.status == "published" {
                tg["status"] = json!("published"); tg["external_id"] = json!(res.external_id); tg["external_url"] = json!(res.url); tg["published_at"] = json!(now_iso());
                modified = true;
                let url = res.url.clone().unwrap_or_default();
                st.append_audit("scheduler", "publish_success", &pid, Some(&tid), None, Some("published"), &format!("Published to {plat}: {url}{}", if dry_run { " [dry_run]" } else { "" }));
                let _ = st.add_notification(json!({"type":"publish_success","title":format!("Đăng bài thành công lên {cap}"),"message":format!("Bài '{ptitle}' đã xuất bản thành công: {url}"),
                    "action_url":if url.is_empty() { "/social?tab=posts".to_string() } else { url.clone() },"data":{"post_id":pid,"target_id":tid,"platform":plat}}));
                results.push(json!({"post_id":pid,"target_id":tid,"status":"published","url":res.url}));
            } else {
                let attempts = tg["attempts"].as_i64().unwrap_or(0) + 1;
                tg["attempts"] = json!(attempts); tg["last_error"] = json!(res.error);
                let err = res.error.clone().unwrap_or_default();
                let el = err.to_lowercase();
                let auth_err = el.contains("quota") || el.contains("unauthorized") || el.contains("401");
                if auth_err || el.contains("forbidden") || attempts >= 3 {
                    tg["status"] = json!("failed");
                    if auth_err { if let Some(a) = account.as_mut() {
                        a["status"] = json!("expired");
                        let _ = st.save_account(a.clone(), false);
                        let _ = st.add_notification(json!({"type":"token_expired","title":format!("Token tài khoản {} đã hết hạn", s(a,"display_name")),
                            "message":format!("Lỗi xác thực hoặc vượt hạn ngạch: {err}. Vui lòng kết nối lại tài khoản."),"action_url":"/social?tab=accounts","data":{"account_id":a["id"]}}));
                    } }
                    let _ = st.add_notification(json!({"type":"publish_failed","title":format!("Đăng bài thất bại ({cap})"),"message":format!("Bài '{ptitle}' không thể đăng: {err}"),
                        "action_url":"/social?tab=posts","data":{"post_id":pid,"target_id":tid,"error":err}}));
                } else {
                    let mins = [2, 10, 30][(attempts as usize - 1).min(2)];
                    tg["scheduled_at"] = json!((now + Duration::minutes(mins)).to_rfc3339());
                    tg["status"] = json!("scheduled");
                }
                modified = true;
                let status = tg["status"].clone();
                st.append_audit("scheduler", "publish_failed", &pid, Some(&tid), None, None, &format!("Error: {err} (attempt {attempts})"));
                results.push(json!({"post_id":pid,"target_id":tid,"status":status,"error":err}));
            }
        }
        if modified {
            let ts: Vec<String> = post["targets"].as_array().unwrap().iter().map(|t| s(t, "status").to_string()).collect();
            if ts.iter().all(|x| x == "published") { post["status"] = json!("published"); }
            else if ts.iter().all(|x| x == "published" || x == "failed") { post["status"] = json!(if ts.iter().any(|x| x == "failed") { "failed" } else { "published" }); }
            let _ = st.save_post(post);
        }
    }
    results
}

/// Background loop over every project with a `publishing/` dir. Opt-in (SOCIAL_SCHEDULER=1): a tick mutates post state, even in dry_run.
pub fn spawn() {
    if std::env::var("SOCIAL_SCHEDULER").ok().as_deref() != Some("1") { return; }
    let Ok(h) = tokio::runtime::Handle::try_current() else { return };
    h.spawn(async {
        let mut iv = tokio::time::interval(std::time::Duration::from_secs(POLL_SECS));
        iv.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            iv.tick().await;
            let root = workspace::data_root().join("projects");
            let Ok(rd) = fs::read_dir(root) else { continue };
            for e in rd.flatten() {
                if !e.path().join("publishing").is_dir() { continue; }
                let name = e.file_name().to_string_lossy().to_string();
                let dry = super::vault::dry_run_default();
                let r = tick(&name, None, dry).await;
                if !r.is_empty() { tracing::info!(project = %name, published_or_failed = r.len(), dry_run = dry, "social scheduler tick"); }
            }
        }
    });
}
