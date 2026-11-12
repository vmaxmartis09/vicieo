use super::models::*;
use super::{store::SocialStore, vault};
use serde_json::{json, Value};

fn clean_num(v: &str) -> f64 {
    let mut s: String = v.trim().chars().filter(|c| *c != ',' && *c != '%' && *c != ' ').collect();
    if s.is_empty() || s == "-" { return 0.0; }
    let mut f = 1.0;
    if s.ends_with(['k', 'K']) { f = 1e3; s.pop(); } else if s.ends_with(['m', 'M']) { f = 1e6; s.pop(); }
    s.parse::<f64>().unwrap_or(0.0) * f
}

/// Minimal RFC4180 reader (quotes, escaped quotes, CRLF); avoids adding a csv dependency.
fn parse_rows(t: &str) -> Vec<Vec<String>> {
    let (mut rows, mut row, mut cell, mut q) = (vec![], vec![], String::new(), false);
    let mut it = t.chars().peekable();
    while let Some(c) = it.next() {
        match (c, q) {
            ('"', true) => if it.peek() == Some(&'"') { cell.push('"'); it.next(); } else { q = false },
            ('"', false) => q = true,
            (',', false) => row.push(std::mem::take(&mut cell)),
            ('\r', false) => {}
            ('\n', false) => { row.push(std::mem::take(&mut cell)); rows.push(std::mem::take(&mut row)); }
            _ => cell.push(c),
        }
    }
    if !cell.is_empty() || !row.is_empty() { row.push(cell); rows.push(row); }
    rows
}

pub fn parse_csv(text: &str) -> Vec<Value> {
    let rows = parse_rows(text.trim());
    let mut it = rows.into_iter();
    let mut header: Option<Vec<String>> = None;
    for row in it.by_ref() {
        if row.is_empty() || row.iter().all(|c| c.is_empty()) { continue; }
        let low: Vec<String> = row.iter().map(|c| c.trim().to_lowercase()).collect();
        if low.iter().any(|c| c.contains("title") || c.contains("tiêu đề") || c.contains("video")) && low.iter().any(|c| c.contains("view") || c.contains("xem")) { header = Some(low); break; }
        else if row.len() >= 3 { header = Some(low); break; }
    }
    let Some(h) = header else { return vec![] };
    let find = |pats: &[&str]| h.iter().position(|c| pats.iter().any(|p| c.contains(p)));
    let (ti, vi, li, ci, shi, sai) = (find(&["video title", "tiêu đề", "title", "tên video"]), find(&["view", "lượt xem", "views", "total views"]), find(&["like", "thích", "likes"]),
        find(&["comment", "bình luận", "comments"]), find(&["share", "chia sẻ", "shares"]), find(&["save", "lưu", "saves", "bookmark"]));
    let (wi, ri, sui, cti, rei) = (find(&["watch time", "thời gian xem", "giờ xem", "watch_time"]), find(&["retention", "percentage", "giữ chân", "avg view duration %"]),
        find(&["subscriber", "người đăng ký", "follow", "subs"]), find(&["click-through", "ctr", "tỷ lệ nhấp", "tỷ lệ click"]), find(&["revenue", "doanh thu", "estimated revenue", "tiền"]));
    let mut out = vec![];
    for row in it {
        if row.is_empty() || row.len() <= vi.unwrap_or(0) { continue; }
        let g = |i: Option<usize>| i.and_then(|i| row.get(i)).map_or(0.0, |c| clean_num(c));
        let title = ti.and_then(|i| row.get(i)).map(|c| c.trim().to_string()).unwrap_or_default();
        if title.is_empty() && ti.is_some() { continue; }
        let raw_wt = g(wi);
        // Python quirk: `watch_time_idx and ...` is falsy for index 0, so column 0 never converts hours.
        let hours = wi.map_or(false, |i| i != 0 && (h[i].contains("hour") || h[i].contains("giờ")));
        out.push(json!({"title":title,"views":g(vi) as i64,"likes":g(li) as i64,"comments":g(ci) as i64,"shares":g(shi) as i64,"saves":g(sai) as i64,
            "watch_time_s":if hours { raw_wt * 3600.0 } else { raw_wt },"avg_retention_pct":g(ri),"subscribers_gained":g(sui) as i64,"ctr":g(cti),"revenue":g(rei)}));
    }
    out
}

/// Fetch a snapshot for a published target. Read-only API calls; dry-run ids (`dryrun_*`) are never sent to a platform.
pub async fn collect(account: Option<&Value>, target: &Value, checkpoint: &str) -> Value {
    let mode = account.and_then(|a| a["auth"]["mode"].as_str()).unwrap_or("manual");
    let plat = s(target, "platform");
    let mut snap = norm_snapshot(json!({"post_id":"","target_id":target["id"],"platform":plat,"checkpoint":checkpoint})).unwrap();
    let ext = s(target, "external_id");
    if mode == "manual" || ext.is_empty() || ext.starts_with("dryrun_") { return snap; }
    let tok = account.and_then(|a| a["auth"]["token_ref"].as_str()).filter(|x| !x.is_empty()).and_then(|r| vault::get_key(&format!("{r}_access")));
    let c = reqwest::Client::builder().timeout(std::time::Duration::from_secs(10)).build().unwrap();
    match plat {
        "youtube" => {
            let mut req = c.get("https://www.googleapis.com/youtube/v3/videos").query(&[("part", "statistics,contentDetails"), ("id", ext)]);
            if let Some(t) = &tok { req = req.bearer_auth(t); }
            match req.send().await {
                Ok(r) if r.status().is_success() => if let Ok(d) = r.json::<Value>().await {
                    if let Some(st) = d["items"].get(0).map(|i| &i["statistics"]) {
                        let n = |k: &str| st[k].as_str().and_then(|x| x.parse::<i64>().ok()).unwrap_or(0);
                        snap["views"] = json!(n("viewCount")); snap["likes"] = json!(n("likeCount")); snap["comments"] = json!(n("commentCount")); snap["raw"] = d;
                    }
                },
                Err(e) => snap["raw"] = json!({"error": e.to_string()}),
                _ => {}
            }
            if let Some(t) = &tok {
                let q = [("ids", "channel==MINE".to_string()), ("metrics", "views,estimatedMinutesWatched,averageViewDuration,averageViewPercentage,subscribersGained".into()),
                    ("filters", format!("video=={ext}")), ("startDate", "2024-01-01".into()), ("endDate", "2030-01-01".into())];
                if let Ok(r) = c.get("https://youtubeanalytics.googleapis.com/v2/reports").query(&q).bearer_auth(t).send().await {
                    if let Ok(d) = r.json::<Value>().await { if let Some(row) = d["rows"].get(0).and_then(|r| r.as_array()).filter(|r| r.len() >= 5) {
                        snap["watch_time_s"] = json!(row[1].as_f64().unwrap_or(0.0) * 60.0); snap["avg_retention_pct"] = json!(row[3].as_f64().unwrap_or(0.0)); snap["subscribers_gained"] = json!(row[4].as_i64().unwrap_or(0));
                    } }
                }
            }
        }
        "tiktok" => if let Some(t) = tok {
            match c.post("https://open.tiktokapis.com/v2/post/publish/status/fetch/").bearer_auth(t).json(&json!({"publish_id":ext})).send().await {
                Ok(r) if r.status().is_success() => if let Ok(d) = r.json::<Value>().await {
                    let data = d["data"].clone();
                    if !data["video_views"].is_null() { for (k, src) in [("views", "video_views"), ("likes", "likes"), ("comments", "comments"), ("shares", "shares")] { snap[k] = json!(data[src].as_i64().unwrap_or(0)); } }
                    snap["raw"] = data;
                },
                Err(e) => snap["raw"] = json!({"error": e.to_string()}),
                _ => {}
            }
        },
        _ => {}
    }
    snap
}

pub async fn collect_all(st: &SocialStore) -> i64 {
    let mut n = 0;
    for p in st.list_posts(Some("published"), None, None, None, None, None) { for t in p["targets"].as_array().into_iter().flatten() {
        if s(t, "status") != "published" { continue; }
        let acc = st.get_account(s(t, "account_id"));
        let mut snap = collect(acc.as_ref(), t, "custom").await;
        snap["post_id"] = p["id"].clone();
        st.append_metrics_snapshot(&snap);
        n += 1;
    } }
    n
}
