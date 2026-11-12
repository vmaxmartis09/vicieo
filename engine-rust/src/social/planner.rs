use super::models::*;
use super::platform_rules::{self, rule};
use super::store::SocialStore;
use super::video_library;
use super::ApiErr;
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, TimeZone};
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn pick_best_variant(variants: &Value, platform: &str) -> (Option<String>, Option<String>) {
    let Some(m) = variants.as_object().filter(|m| !m.is_empty()) else { return (None, None) };
    let prio: &[&str] = match platform.to_lowercase().as_str() {
        "tiktok" => &["tiktok", "shorts", "9:16", "reels", "vertical", "youtube"],
        "instagram" => &["reels", "tiktok", "shorts", "9:16", "vertical", "youtube"],
        "threads" => &["shorts", "tiktok", "reels", "9:16", "vertical", "youtube"],
        "facebook" => &["reels", "facebook", "tiktok", "shorts", "youtube", "16:9"],
        "youtube" => &["youtube", "16:9", "horizontal", "shorts", "tiktok"],
        "x" => &["x", "twitter", "youtube", "16:9", "shorts", "tiktok"],
        _ => &[],
    };
    for c in prio.iter().map(|x| x.to_string()).chain(std::iter::once(platform.to_lowercase())) {
        if let Some((k, v)) = m.iter().find(|(k, _)| k.to_lowercase() == c) { return (Some(k.clone()), v.as_str().map(String::from)); }
    }
    let (k, v) = m.iter().next().unwrap();
    (Some(k.clone()), v.as_str().map(String::from))
}

fn hm(t: &str) -> (u32, u32) {
    let mut it = t.trim().split(':').map(|x| x.parse::<u32>());
    match (it.next(), it.next()) { (Some(Ok(h)), Some(Ok(m))) => (h, m), _ => (19, 0) }
}
fn at(d: NaiveDate, h: u32, m: u32) -> DateTime<FixedOffset> { tz().with_ymd_and_hms(d.year(), d.month(), d.day(), h.min(23), m.min(59), 0).single().unwrap() }

pub fn preview_plan(project: &str, req: &Value) -> Result<Value, ApiErr> {
    let st = SocialStore::new(project).map_err(|e| ApiErr::bad(e.to_string()))?;
    let strs = |k: &str| -> Result<Vec<String>, ApiErr> { req[k].as_array().ok_or_else(|| ApiErr::invalid(format!("field required: {k}")))
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()) };
    let (vids, aids) = (strs("video_ids")?, strs("account_ids")?);
    let now = chrono::Utc::now().with_timezone(&tz());
    let mut base = req["start_at"].as_str().and_then(|x| parse_dt(&x.replace('Z', "+00:00"))).unwrap_or(now);
    if base < now { base = now; }
    let accounts = st.list_accounts();
    let sel_acc: Vec<&Value> = aids.iter().filter_map(|id| accounts.iter().find(|a| s(a, "id") == id)).collect();
    let all_v = video_library::list_videos(project);
    let sel_v: Vec<&Value> = vids.iter().filter_map(|id| all_v.iter().find(|v| s(v, "id") == id)).collect();
    let strategy = req["strategy"].as_str().unwrap_or("auto");
    let mut planned: Vec<Value> = vec![];
    let mk = |v: &Value, a: &Value, when: String, issues: Vec<Value>| {
        let (k, p) = pick_best_variant(&v["variants"], s(a, "platform"));
        json!({"video_id":v["id"],"video_title":v["title"],"account_id":a["id"],"account_name":if s(a,"display_name").is_empty() { a["handle"].clone() } else { a["display_name"].clone() },
            "platform":a["platform"],"scheduled_at":when,"issues":issues,"variant_used":k,"media_path":p.or_else(|| v["primary_path"].as_str().map(String::from))})
    };
    let issues_for = |v: &Value, a: &Value| platform_rules::validate(Some(v), &json!({"platform":a["platform"],"title":v["title"]}), Some(a))
        .into_iter().map(|mut i| { i["level"] = json!(if i["level"] == "error" { "error" } else { "warn" }); i }).collect::<Vec<_>>();
    match strategy {
        "now" => { let t = (now + Duration::minutes(5)).to_rfc3339(); for v in &sel_v { for a in &sel_acc { planned.push(mk(v, a, t.clone(), issues_for(v, a))); } } }
        "fixed" => { let mut step = base; for v in &sel_v { for a in &sel_acc { planned.push(mk(v, a, step.to_rfc3339(), issues_for(v, a))); step += Duration::minutes(10); } } }
        _ => {
            let cur = base.date_naive();
            let mut acc_cnt: HashMap<(NaiveDate, String), i64> = HashMap::new();
            let mut plat_cnt: HashMap<(NaiveDate, String), u32> = HashMap::new();
            let mut last: HashMap<(NaiveDate, String), DateTime<FixedOffset>> = HashMap::new();
            let mut ptimes: HashMap<(NaiveDate, String), Vec<DateTime<FixedOffset>>> = HashMap::new();
            for v in &sel_v { for a in &sel_acc {
                let plat = s(a, "platform").to_string();
                let r = rule(&plat);
                let limit = req["per_day_limit"].as_i64().filter(|n| *n > 0).or(a["daily_limit"].as_i64().filter(|n| *n > 0)).unwrap_or(5);
                let gap = Duration::minutes(a["min_gap_minutes"].as_i64().filter(|n| *n > 0).unwrap_or(120));
                let (mut d, mut pushed) = (cur, false);
                while acc_cnt.get(&(d, s(a, "id").into())).copied().unwrap_or(0) >= limit || plat_cnt.get(&(d, plat.clone())).copied().unwrap_or(0) >= r.quota { pushed = true; d += Duration::days(1); }
                let wd = d.weekday().num_days_from_monday() as i64;
                let wins = a["posting_windows"].as_array().cloned().unwrap_or_default();
                let (h, m) = wins.iter().find(|w| w["weekday"].as_i64() == Some(wd)).or(wins.first()).map(|w| hm(s(w, "start"))).unwrap_or((19, 30));
                let mut t = at(d, h, m);
                if t <= now { t = now + Duration::minutes(15); }
                if let Some(l) = last.get(&(d, s(a, "id").into())) { if t - *l < gap { t = *l + gap; } }
                let pt = ptimes.entry((d, plat.clone())).or_default();
                for e in pt.clone() { if (t - e).num_seconds().abs() < 900 { t = e + Duration::minutes(15); } }
                *acc_cnt.entry((d, s(a, "id").into())).or_default() += 1;
                *plat_cnt.entry((d, plat.clone())).or_default() += 1;
                last.insert((d, s(a, "id").into()), t);
                pt.push(t);
                let mut issues = issues_for(v, a);
                if pushed { issues.push(json!({"level":"warn","code":"quota_pushed","message_vi":format!("Hạn ngạch API {} ({} video/ngày) đã đầy ở ngày trước, đã tự động dời sang {}.", r.name, r.quota, d.format("%d/%m/%Y"))})); }
                planned.push(mk(v, a, t.to_rfc3339(), issues));
            } }
        }
    }
    Ok(json!(planned))
}

pub fn commit_plan(project: &str, req: &Value) -> Result<Value, ApiErr> {
    let st = SocialStore::new(project).map_err(|e| ApiErr::bad(e.to_string()))?;
    let specs = req["targets"].as_array().ok_or_else(|| ApiErr::invalid("field required: targets"))?;
    let required = req["approval_required"] != json!(false);
    let vids = video_library::list_videos(project);
    let accounts = st.list_accounts();
    let mut order: Vec<String> = vec![];
    let mut by: HashMap<String, Vec<&Value>> = HashMap::new();
    for sp in specs {
        let vid = sp["video_id"].as_str().ok_or_else(|| ApiErr::invalid("field required: video_id"))?.to_string();
        if sp["account_id"].is_null() || sp["scheduled_at"].is_null() { return Err(ApiErr::invalid("field required: account_id/scheduled_at")); }
        if !by.contains_key(&vid) { order.push(vid.clone()); }
        by.entry(vid).or_default().push(sp);
    }
    let mut created = vec![];
    let mut n_targets = 0;
    for vid in order {
        let sp = &by[&vid];
        let v = vids.iter().find(|v| s(v, "id") == vid).cloned().unwrap_or(json!({}));
        let title = [s(&v, "title"), sp[0]["title"].as_str().unwrap_or("")].into_iter().find(|x| !x.is_empty()).map(String::from).unwrap_or_else(|| format!("Video {}", vid.chars().take(8).collect::<String>()));
        let prim = s(&v, "primary_path");
        let media = if prim.is_empty() { json!([]) } else { json!([{"path":prim,"kind":"video","duration_s":v["duration_s"].as_f64().unwrap_or(0.0),"thumb":s(&v,"thumb")}]) };
        let mut targets = vec![];
        for t in sp {
            let acc = accounts.iter().find(|a| s(a, "id") == s(t, "account_id"));
            let g = |k: &str, d: Value| if t[k].is_null() { d } else { t[k].clone() };
            targets.push(json!({"account_id":t["account_id"],"platform":acc.map(|a| s(a,"platform")).unwrap_or("youtube"),"scheduled_at":t["scheduled_at"],
                "title":t["title"].as_str().filter(|x| !x.is_empty()).unwrap_or(&title),"caption":t["caption"].as_str().filter(|x| !x.is_empty()).unwrap_or(&title),
                "hashtags":if t["hashtags"].is_null() { acc.map(|a| a["default_hashtags"].clone()).unwrap_or(json!([])) } else { t["hashtags"].clone() },
                "privacy":t["privacy"].as_str().filter(|x| !x.is_empty()).unwrap_or("public"),"made_for_kids":g("made_for_kids",json!(false)),"ai_generated_label":g("ai_generated_label",json!(false)),
                "comment_enabled":g("comment_enabled",json!(true)),"duet_enabled":g("duet_enabled",json!(true)),"stitch_enabled":g("stitch_enabled",json!(true)),
                "first_comment":t["first_comment"].as_str().unwrap_or(""),"product_link":t["product_link"].as_str().unwrap_or(""),"disclosure_ad":g("disclosure_ad",json!(false)),"status":"scheduled"}));
            n_targets += 1;
        }
        let post = json!({"project_id":project,"title":title,"media":media,"render_job_id":if vid.starts_with("render_") { json!(vid) } else { Value::Null },
            "status":"scheduled","approval":{"required":required},"targets":targets});
        created.push(st.save_post(post).map_err(ApiErr::invalid)?);
    }
    Ok(json!({"created_posts_count":created.len(),"created_targets_count":n_targets,"posts":created}))
}
