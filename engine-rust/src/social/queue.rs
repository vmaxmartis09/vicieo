use super::models::*;
use super::{platform_rules, store::SocialStore};
use chrono::{DateTime, FixedOffset};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub struct QueueParams<'a> { pub bucket: Option<&'a str>, pub account_id: Option<&'a str>, pub platform: Option<&'a str>, pub from: Option<&'a str>,
    pub group_by: Option<&'a str>, pub sort: Option<&'a str>, pub page: usize, pub page_size: usize }

pub fn get_social_queue(st: &SocialStore, q: &QueueParams) -> Value {
    let accounts = st.list_accounts();
    let now = chrono::Utc::now().with_timezone(&tz());
    let mut items: Vec<Value> = vec![];
    let mut c: BTreeMap<&str, i64> = BTreeMap::new();
    let mut next: Option<DateTime<FixedOffset>> = None;
    for p in st.list_posts(None, None, None, None, None, None) {
        let m0 = p["media"].get(0);
        let thumb = m0.and_then(|m| m["thumb"].as_str()).unwrap_or("").to_string();
        let video = json!({"title": p["title"], "duration_s": m0.map(|m| m["duration_s"].clone()).unwrap_or(json!(30.0))});
        let req = p["approval"]["required"] != json!(false);
        for t in p["targets"].as_array().into_iter().flatten() {
            let acc = accounts.iter().find(|a| s(a, "id") == s(t, "account_id"));
            let has_pkg = crate::workspace::project_path(&st.project_id, &["publishing", "packages", s(&p, "id"), s(t, "platform")], false).map_or(false, |x| x.exists());
            let sched = t["scheduled_at"].as_str().filter(|x| !x.is_empty());
            let t_dt = sched.and_then(|x| parse_dt(&x.replace('Z', "+00:00")));
            let ts = s(t, "status");
            let bucket = if ts == "published" { "published" } else if ts == "failed" { "failed" }
                else if req && (s(&p, "status") == "pending_approval" || ts == "pending_approval") { "pending_approval" }
                else if sched.is_none() || ts == "draft" { "draft" }
                else if let Some(d) = t_dt {
                    if d >= now { if next.map_or(true, |n| d < n) { next = Some(d); } "upcoming" } else { "overdue" }
                } else { "draft" };
            *c.entry(bucket).or_default() += 1;
            let appr = if !req || s(&p, "status") == "approved" { "approved" } else { "pending" };
            items.push(json!({"post_id":p["id"],"target_id":t["id"],"title":p["title"],"platform":t["platform"],"account_id":t["account_id"],
                "account_name":acc.map(|a| s(a,"display_name")).unwrap_or(s(t,"account_id")),"account_avatar":acc.map(|a| s(a,"avatar")).unwrap_or(""),
                "scheduled_at":t["scheduled_at"],"status":t["status"],"caption":t["caption"],"thumb":thumb,"bucket":bucket,"approval_required":req,
                "approval_status":appr,"is_manual":acc.map_or(false, |a| a["auth"]["mode"] == "manual"),"has_package":has_pkg,
                "issues":platform_rules::validate(Some(&video), t, acc)}));
        }
    }
    let n = |k: &str| *c.get(k).unwrap_or(&0);
    let summary = json!({"accounts_count":accounts.len(),"max_accounts":MAX_ACCOUNTS,"draft_count":n("draft"),"pending_approval_count":n("pending_approval"),
        "overdue_count":n("overdue"),"upcoming_count":n("upcoming"),"published_count":n("published"),"failed_count":n("failed"),
        "next_post_time":next.map(|d| d.format("%H:%M").to_string())});
    let bucket = q.bucket;
    items.retain(|i| bucket.map_or(true, |b| b == "all" || s(i, "bucket") == b)
        && q.account_id.map_or(true, |a| s(i, "account_id") == a) && q.platform.map_or(true, |p| s(i, "platform") == p)
        && q.from.map_or(true, |f| i["scheduled_at"].as_str().map_or(false, |x| x >= f)));
    let sa = |i: &Value| i["scheduled_at"].as_str().unwrap_or("").to_string();
    let lt = |i: &Value| s(i, "title").to_lowercase();
    match q.sort {
        Some("title") => items.sort_by_key(lt),
        Some("-title") => items.sort_by_key(|i| std::cmp::Reverse(lt(i))),
        Some("-scheduled_at") => items.sort_by_key(|i| std::cmp::Reverse(sa(i))),
        Some("scheduled_at") => items.sort_by_key(sa),
        _ if matches!(bucket, Some("overdue") | Some("published")) => items.sort_by_key(|i| std::cmp::Reverse(sa(i))),
        _ => items.sort_by_key(sa),
    }
    let total = items.len();
    let groups = q.group_by.map(|g| {
        let mut m: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        for it in &items {
            let key = match g {
                "date" => it["scheduled_at"].as_str().filter(|x| !x.is_empty()).map(|x| x.chars().take(10).collect()).unwrap_or("Chưa hẹn giờ".into()),
                "account" => { let n = s(it, "account_name"); if n.is_empty() { s(it, "account_id").into() } else { n.into() } }
                "platform" => s(it, "platform").into(),
                "status" => s(it, "status").into(),
                _ => "Tất cả".into(),
            };
            m.entry(key).or_default().push(it.clone());
        }
        json!(m)
    });
    let start = q.page.saturating_sub(1) * q.page_size;
    let paged: Vec<&Value> = items.iter().skip(start).take(q.page_size).collect();
    json!({"items":paged,"summary":summary,"total":total,"page":q.page,"page_size":q.page_size,"groups":groups})
}
