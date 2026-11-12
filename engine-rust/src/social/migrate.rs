use super::models::*;
use super::store::{read_json, SocialStore};
use crate::workspace;
use serde_json::{json, Value};

fn hour_min(t: Option<&str>) -> (u32, u32) {
    let re = regex::Regex::new(r"(\d{1,2}):(\d{2})").unwrap();
    t.and_then(|t| re.captures(t)).map_or((19, 0), |c| (c[1].parse().unwrap_or(19), c[2].parse().unwrap_or(0)))
}

pub fn migrate_legacy_publishing(project: &str, force: bool) -> Result<Value, String> {
    let st = SocialStore::new(project).map_err(|e| e.to_string())?;
    let dir = st.pub_dir.clone();
    let marker = dir.join(".migrated_v1");
    if marker.exists() && !force { return Ok(json!({"status":"already_migrated","migrated_accounts":0,"migrated_posts":0,"migrated_slots":0})); }
    let sched = read_json(&dir.join("schedule.json"), json!({}));
    let mut channels: Vec<Value> = read_json(&dir.join("channels.json"), json!([])).as_array().cloned().unwrap_or_default();
    for c in sched["channels"].as_array().into_iter().flatten() { if !channels.iter().any(|x| x["id"] == c["id"]) { channels.push(c.clone()); } }
    let posts: Vec<Value> = sched["posts"].as_array().cloned().unwrap_or_default();
    let (mut ma, mut mp, mut ms) = (0, 0, 0);
    let accs = st.list_accounts();
    let slots = st.list_slots(None);
    let existing_posts: std::collections::HashSet<String> = st.list_posts(None, None, None, None, None, None).iter().map(|p| s(p, "id").to_string()).collect();
    let norm_plat = |p: &Value| { let p = p.as_str().unwrap_or("youtube").to_lowercase(); if PLATFORMS.contains(&p.as_str()) { p } else { "youtube".into() } };
    for c in &channels {
        let cid = match &c["id"] { Value::String(x) => x.clone(), Value::Null => String::new(), o => o.to_string() };
        if cid.is_empty() { continue; }
        let (h, m) = hour_min(c["defaultPublishHours"].as_str());
        let ts = format!("{h:02}:{m:02}");
        let end = format!("{:02}:{m:02}", (h + 2).min(23));
        let name = c["name"].as_str().unwrap_or(&cid).to_string();
        if !accs.iter().any(|a| s(a, "id") == cid) {
            let acc = json!({"id":cid,"platform":norm_plat(&c["platform"]),"handle":name,"display_name":name,"auth":{"mode":"manual"},
                "status":if c["connected"] == json!(false) { "manual" } else { "connected" },"default_hashtags":c["defaultHashtags"].as_array().cloned().unwrap_or_default(),
                "posting_windows":(0..7).map(|w| json!({"weekday":w,"start":ts,"end":end})).collect::<Vec<_>>(),"daily_limit":5});
            st.save_account(acc, false)?;
            ma += 1;
        }
        let sid = format!("slot_{cid}");
        if !slots.iter().any(|x| s(x, "id") == sid) {
            let mut all = st.list_slots(None);
            all.retain(|x| s(x, "id") != sid);
            all.push(norm_slot(json!({"id":sid,"account_id":cid,"rrule":format!("FREQ=DAILY;BYHOUR={h};BYMINUTE={m}"),"label":format!("Hàng ngày · {ts}")}))?);
            st.save_slots(&all);
            ms += 1;
        }
    }
    for p in &posts {
        let pid = p["id"].as_str().unwrap_or("").to_string();
        if pid.is_empty() || existing_posts.contains(&pid) { continue; }
        let at = p["scheduledDate"].as_str().filter(|x| !x.is_empty()).map(|d| { let (h, m) = hour_min(p["scheduledTime"].as_str().or(Some("19:00"))); format!("{d}T{h:02}:{m:02}:00+07:00") });
        let status = p["status"].as_str().filter(|x| POST_STATUSES.contains(x)).unwrap_or("pending_approval");
        let title = p["title"].as_str().unwrap_or("");
        let post = json!({"id":pid,"project_id":p["projectId"].as_str().unwrap_or(project),"title":if title.is_empty() { &pid } else { title },
            "media":p["videoUri"].as_str().filter(|x| !x.is_empty()).map(|u| json!([{"path":u,"kind":"video"}])).unwrap_or(json!([])),"status":status,"approval":{"required":true},
            "targets":[{"id":format!("tgt_{pid}"),"account_id":p["channelId"].as_str().unwrap_or(""),"platform":norm_plat(&p["platform"]),"scheduled_at":at,"title":title,"status":status}]});
        st.save_post(post)?;
        st.append_audit("migration", "migrate_legacy", &pid, None, None, None, "Migrated from legacy schedule.json");
        mp += 1;
    }
    std::fs::write(marker, now_iso()).map_err(|e| e.to_string())?;
    let _ = workspace::data_root;
    Ok(json!({"status":"success","migrated_accounts":ma,"migrated_posts":mp,"migrated_slots":ms}))
}
