//! Channel ingest (yt-dlp flat extract) and statistical analysis.
use super::contracts::*;
use super::util::*;
use chrono::{Datelike, NaiveDate, TimeZone, Timelike, Utc};
use regex::Regex;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

pub fn generate_channel_id(url: &str) -> String {
    let clean = url.trim().trim_end_matches('/');
    if let Some(c) = Regex::new(r"(@[a-zA-Z0-9_\-\.]+)").unwrap().captures(clean) {
        return format!("ch_{}", c[1].trim_start_matches('@').to_lowercase());
    }
    let h = format!("{:x}", Sha256::digest(clean.as_bytes()));
    format!("ch_{}", &h[..12])
}

fn round1(x: f64) -> f64 { (x * 10.0).round() / 10.0 }
fn round2(x: f64) -> f64 { (x * 100.0).round() / 100.0 }
fn num(x: f64) -> Value { if x.fract() == 0.0 && x.abs() < 1e15 { json!(x as i64) } else { json!(x) } }

pub fn parse_video_item(e: &Value) -> Value {
    let id = e.get("id").map(|v| v.as_str().map(String::from).unwrap_or_else(|| v.to_string())).filter(|s| s != "null").unwrap_or_default();
    let title = e.get("title").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).unwrap_or("Untitled").to_string();
    let duration = e.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let raw_views = e.get("view_count").and_then(|v| v.as_i64());
    let views = raw_views.unwrap_or(0);
    let thumb = e.get("thumbnail").and_then(|v| v.as_str()).map(String::from)
        .or_else(|| e.get("thumbnails").and_then(|t| t.as_array()).and_then(|a| a.last()).and_then(|t| t.get("url")).and_then(|u| u.as_str()).map(String::from)).unwrap_or_default();
    let url = e.get("url").and_then(|v| v.as_str()).or_else(|| e.get("webpage_url").and_then(|v| v.as_str())).map(String::from)
        .unwrap_or_else(|| if id.is_empty() { String::new() } else { format!("https://www.youtube.com/watch?v={id}") });

    let mut upload_date = e.get("upload_date").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let mut upload_hour: Option<i64> = None;
    let mut src = "UNAVAILABLE";
    if let Some(ts) = e.get("timestamp").and_then(|v| v.as_f64()) {
        if let Some(dt) = Utc.timestamp_opt(ts as i64, 0).single() {
            upload_date = dt.format("%Y%m%d").to_string();
            upload_hour = Some(dt.hour() as i64);
            src = "REAL_TIMESTAMP";
        }
    } else if let Some(h) = e.get("upload_hour").and_then(|v| v.as_i64()) { upload_hour = Some(h); }
    let mut days = 1.0;
    if upload_date.len() == 8 {
        if let Ok(d) = NaiveDate::parse_from_str(&upload_date, "%Y%m%d") {
            let secs = (Utc::now().naive_utc() - d.and_hms_opt(0, 0, 0).unwrap()).num_seconds() as f64;
            days = (secs / 86400.0).max(1.0);
            if src == "UNAVAILABLE" { src = "REAL_DATE"; }
        }
    }
    json!({
        "id": id, "title": title, "duration": duration, "upload_date": upload_date, "upload_hour": upload_hour,
        "upload_date_source": src, "view_count": views, "view_count_source": if raw_views.is_some() { "REAL" } else { "UNAVAILABLE" },
        "like_count": e.get("like_count").and_then(|v| v.as_i64()).unwrap_or(0),
        "comment_count": e.get("comment_count").and_then(|v| v.as_i64()).unwrap_or(0),
        "thumbnail": thumb, "url": url, "days_age": round1(days),
        "views_per_day": if views > 0 { round2(views as f64 / days) } else { 0.0 },
    })
}

fn sorted_videos(raw: &[Value]) -> Vec<Value> {
    let mut v: Vec<Value> = raw.iter().filter(|e| !e.is_null()).map(parse_video_item).filter(|v| !s(v, "id").is_empty()).collect();
    v.sort_by(|a, b| b["views_per_day"].as_f64().partial_cmp(&a["views_per_day"].as_f64()).unwrap());
    v
}

fn save(pid: &str, cid: &str, videos: &[Value], meta: &Value) -> anyhow::Result<()> {
    let dir = pp(pid, &["knowledge", "channels", safe_id(cid)?, "videos.jsonl"], true);
    std::fs::write(&dir, videos.iter().map(|v| v.to_string() + "\n").collect::<String>())?;
    write_json(&dir.with_file_name("channel.json"), meta)
}

pub fn ingest_user_channel_data(pid: &str, channel_url: &str, channel_name: &str, raw: &[Value]) -> anyhow::Result<Value> {
    let cid = generate_channel_id(channel_url);
    let videos = sorted_videos(raw);
    let meta = json!({
        "id": cid, "url": channel_url, "name": if channel_name.is_empty() { cid.clone() } else { channel_name.to_string() },
        "uploader_id": cid, "avatar": videos.first().map(|v| s(v, "thumbnail")).unwrap_or_default(), "video_count": videos.len(),
        "top_ingested_sources": [], "data_source": "USER_PROVIDED", "data_label": "DỮ LIỆU THẬT", "updated_at": now_iso(),
    });
    save(pid, &cid, &videos, &meta)?;
    Ok(meta)
}

pub async fn ingest_channel(pid: &str, url: &str, max_videos: u32, download_top_n: usize) -> anyhow::Result<Value> {
    let cid = generate_channel_id(url);
    let out = tokio::process::Command::new("yt-dlp")
        .args(["--flat-playlist", "-J", "--no-warnings", "--playlist-items", &format!("1-{max_videos}"), url])
        .output().await;
    let info: Value = match out {
        Ok(o) if o.status.success() => serde_json::from_slice(&o.stdout).unwrap_or(json!({})),
        _ => {
            // extraction failed: serve the cached channel.json if present
            if let Some(c) = read_json(&pp(pid, &["knowledge", "channels", safe_id(&cid)?, "channel.json"], false)) { return Ok(c); }
            json!({})
        }
    };
    let entries = info.get("entries").and_then(|e| e.as_array()).cloned().unwrap_or_default();
    let videos = sorted_videos(&entries);
    let mut top = vec![];
    for v in videos.iter().take(download_top_n) {
        let u = s(v, "url");
        if u.is_empty() { continue; }
        if let Ok(item) = super::sources::downloader::download_source(pid, &u, "reference_only", "").await { top.push(item.id); }
    }
    let name = ["uploader", "channel", "title"].iter().find_map(|k| info.get(*k).and_then(|v| v.as_str()).filter(|s| !s.is_empty())).unwrap_or(&cid).to_string();
    let meta = json!({
        "id": cid, "url": url, "name": name, "uploader_id": info.get("uploader_id").and_then(|v| v.as_str()).unwrap_or(""),
        "avatar": info.get("thumbnail").and_then(|v| v.as_str()).map(String::from).unwrap_or_else(|| videos.first().map(|v| s(v, "thumbnail")).unwrap_or_default()),
        "video_count": videos.len(), "top_ingested_sources": top, "data_source": "PUBLIC_SCRAPE", "data_label": "DỮ LIỆU THẬT", "updated_at": now_iso(),
    });
    save(pid, &cid, &videos, &meta)?;
    Ok(meta)
}

pub fn abstract_title_pattern(title: &str) -> String {
    let mut r = Regex::new(r"\b\d+([,.]\d+)?\b").unwrap().replace_all(title.trim(), "[Số]").to_string();
    r = Regex::new(r"(?i)\[Số\]\s*(ngày|tháng|năm|giờ|phút|giây)").unwrap().replace_all(&r, "[Thời gian]").to_string();
    r = Regex::new(r"(?i)\[Số\]\s*(k|triệu|tỷ|đ|vnd|usd|\$)").unwrap().replace_all(&r, "[Giá tiền]").to_string();
    let low = r.to_lowercase();
    if (r.contains('?') || ["tại sao", "vì sao", "làm sao", "như thế nào"].iter().any(|w| low.contains(w))) && !r.ends_with('?') { r.push('?'); }
    r
}

const STOP: &[&str] = &["và", "của", "là", "cho", "có", "trong", "được", "với", "không", "một", "các", "những", "khi", "để", "về", "người", "ra", "vào", "lại", "đã", "ở", "từ", "này", "đó", "thì", "sẽ", "làm", "the", "a", "an", "and", "in", "to"];

pub fn cluster_pillars(videos: &[Value]) -> Vec<Value> {
    if videos.is_empty() { return vec![]; }
    let wre = Regex::new(r"\b[a-zA-ZÀ-ỹ0-9_]{3,}\b").unwrap();
    let mut freq: Vec<(String, usize)> = vec![];
    let mut vwords: Vec<Vec<String>> = vec![];
    for v in videos {
        let words: Vec<String> = wre.find_iter(&s(v, "title").to_lowercase()).map(|m| m.as_str().to_string())
            .filter(|w| !STOP.contains(&w.as_str()) && !w.chars().all(|c| c.is_ascii_digit())).collect();
        for w in &words {
            match freq.iter_mut().find(|(k, _)| k == w) { Some(e) => e.1 += 1, None => freq.push((w.clone(), 1)) }
        }
        vwords.push(words);
    }
    freq.sort_by(|a, b| b.1.cmp(&a.1));
    let mut top: Vec<String> = freq.into_iter().take(12).map(|(w, _)| w).collect();
    if top.is_empty() { top = ["tổng hợp", "đánh giá", "chia sẻ", "nổi bật"].iter().map(|s| s.to_string()).collect(); }
    let cap = |w: &str| { let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase()).unwrap_or_default() };
    let mut map: Vec<(String, Vec<&Value>)> = vec![];
    let mut un = vec![];
    for (v, words) in videos.iter().zip(&vwords) {
        match top.iter().find(|k| words.contains(k)) {
            Some(k) => { let n = cap(k); match map.iter_mut().find(|(x, _)| *x == n) { Some(e) => e.1.push(v), None => map.push((n, vec![v])) } }
            None => un.push(v),
        }
    }
    if !un.is_empty() { match map.iter_mut().find(|(x, _)| x == "Khác") { Some(e) => e.1.extend(un), None => map.push(("Khác".into(), un)) } }
    let total = videos.len() as f64;
    let total_views = (videos.iter().map(|v| v["view_count"].as_i64().unwrap_or(0)).sum::<i64>()).max(1) as f64;
    map.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    map.into_iter().take(6).map(|(name, vs)| {
        let pv = vs.iter().map(|v| v["view_count"].as_i64().unwrap_or(0)).sum::<i64>() as f64;
        json!({"name": name, "video_count": vs.len(), "video_pct": round1(vs.len() as f64 / total * 100.0), "total_views": pv as i64,
               "view_pct": round1(pv / total_views * 100.0), "avg_views": (pv / vs.len().max(1) as f64).round()})
    }).collect()
}

pub fn analyze_channel(pid: &str, cid: &str) -> ApiResult<Value> {
    let dir = pp(pid, &["knowledge", "channels", safe_id(cid).map_err(|e| ApiError::bad(e.to_string()))?], false);
    let txt = std::fs::read_to_string(dir.join("videos.jsonl")).map_err(|_| ApiError::nf(format!("Không tìm thấy dữ liệu video cho kênh {cid}. Hãy ingest trước.")))?;
    let mut videos: Vec<Value> = txt.lines().filter(|l| !l.trim().is_empty()).filter_map(|l| serde_json::from_str(l).ok()).collect();
    let meta = read_json(&dir.join("channel.json")).unwrap_or(json!({}));
    if videos.is_empty() { return Err(ApiError::ise(format!("Kênh {cid} chưa có video nào."))); }

    let mut views: Vec<f64> = videos.iter().map(|v| v["view_count"].as_f64().unwrap_or(0.0)).collect();
    let n = views.len() as f64;
    let avg_views = views.iter().sum::<f64>() / n;
    views.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = if views.len() % 2 == 1 { views[views.len() / 2] } else { (views[views.len() / 2 - 1] + views[views.len() / 2]) / 2.0 };
    let durs: Vec<f64> = videos.iter().map(|v| v["duration"].as_f64().unwrap_or(0.0)).filter(|d| *d > 0.0).collect();
    let avg_dur = if durs.is_empty() { 60.0 } else { durs.iter().sum::<f64>() / durs.len() as f64 };
    let shorts = durs.iter().filter(|d| **d < 60.0).count();
    let longs = durs.len() - shorts;

    for v in videos.iter_mut() {
        let vv = v["view_count"].as_f64().unwrap_or(0.0);
        v["outlier_score"] = json!(round2(vv / median.max(1.0)));
    }
    let mut outliers = videos.clone();
    outliers.sort_by(|a, b| b["outlier_score"].as_f64().partial_cmp(&a["outlier_score"].as_f64()).unwrap());
    outliers.truncate(10);

    let mut heat = vec![vec![0i64; 24]; 7];
    let mut real_hours = false;
    for v in &videos {
        let ud = s(v, "upload_date");
        if ud.len() == 8 {
            if let Ok(d) = NaiveDate::parse_from_str(&ud, "%Y%m%d") {
                let wd = d.weekday().num_days_from_monday() as usize;
                match v["upload_hour"].as_i64().filter(|h| (0..=23).contains(h)) {
                    Some(h) => { real_hours = true; heat[wd][h as usize] += 1 }
                    None => heat[wd][12] += 1,
                }
            }
        }
    }
    let heat_status = if real_hours { "DỮ LIỆU THẬT" } else { "MẪU (Giờ ước tính 12:00 do dữ liệu nguồn chỉ có ngày)" };
    let cadence = if videos.len() > 100 { "Hàng ngày (1-2 video/ngày)" } else if videos.len() < 20 { "1-2 video / tuần" } else { "3-4 video / tuần" };
    let pillars = cluster_pillars(&videos);

    let emoji = Regex::new(r"[\x{10000}-\x{10ffff}]").unwrap();
    let (mut emo, mut numc, mut lens) = (0, 0, vec![]);
    let mut tmpl: Vec<(String, usize)> = vec![];
    for v in &videos {
        let t = s(v, "title");
        lens.push(t.chars().count() as f64);
        if emoji.is_match(&t) { emo += 1 }
        if t.chars().any(|c| c.is_ascii_digit()) { numc += 1 }
        let a = abstract_title_pattern(&t);
        if a != t && a.contains('[') { match tmpl.iter_mut().find(|(k, _)| *k == a) { Some(e) => e.1 += 1, None => tmpl.push((a, 1)) } }
    }
    tmpl.sort_by(|a, b| b.1.cmp(&a.1));
    let mut patterns: Vec<String> = tmpl.into_iter().take(8).map(|(k, _)| k).collect();
    if patterns.is_empty() {
        patterns = ["Làm thế nào để [Hành động] trong [Thời gian]?", "Top [Số] sai lầm tai hại khi [Hành động]", "Tôi đã thử [Sản phẩm] và cái kết bất ngờ...", "Đừng bao giờ mua [Sản phẩm] nếu chưa xem video này!"].iter().map(|s| s.to_string()).collect();
    }
    let hooks: Vec<String> = outliers.iter().take(5).map(|o| match load_video_blueprint(pid, &format!("bp_{}", s(o, "id"))) {
        Some(bp) if !bp.hook.is_empty() => bp.hook,
        _ => format!("Xem ngay: {}", s(o, "title")),
    }).collect();
    let niche = if pillars.is_empty() { "Hoạt hình & Đời sống".to_string() } else { format!("Chủ đề: {}", pillars.iter().take(2).map(|p| s(p, "name")).collect::<Vec<_>>().join(", ")) };

    let bp = ChannelBlueprint {
        id: cid.into(), channel_url: meta.get("url").and_then(|v| v.as_str()).map(String::from).unwrap_or_else(|| format!("https://youtube.com/{cid}")),
        niche, pillars: pillars.iter().map(|p| s(p, "name")).collect(), title_patterns: patterns.clone(), avg_duration_s: round1(avg_dur),
        upload_cadence: cadence.into(),
        top_videos: outliers.iter().map(|v| json!({"id": v["id"], "title": v["title"], "view_count": v["view_count"], "outlier_score": v["outlier_score"], "thumbnail": v["thumbnail"], "url": v["url"]})).collect(),
        best_hooks: hooks.into_iter().take(5).collect(), data_label: "DỮ LIỆU THẬT".into(), is_mock: false,
        metadata: json!({"schedule_heatmap_status": heat_status, "has_real_hours": real_hours, "retention_data": "KHÔNG CÓ (Không bịa số liệu giữ chân)"}),
    };
    save_channel_blueprint(pid, &bp)?;
    let mut stats = Map::new();
    let tv = videos.len() as f64;
    for (k, v) in [
        ("total_videos", json!(videos.len())), ("median_views", num(median)), ("avg_views", json!(avg_views.round())), ("avg_duration_s", json!(round1(avg_dur))),
        ("shorts_ratio", json!(round2(shorts as f64 / tv))), ("longs_ratio", json!(round2(longs as f64 / tv))), ("upload_cadence", json!(cadence)),
        ("avg_title_length", json!(round1(lens.iter().sum::<f64>() / lens.len().max(1) as f64))), ("emoji_ratio", json!(round2(emo as f64 / tv))),
        ("number_ratio", json!(round2(numc as f64 / tv))), ("retention_rate", Value::Null),
        ("retention_note", json!("KHÔNG CÓ (Không bịa số liệu giữ chân theo luật cứng)")),
    ] { stats.insert(k.into(), v); }
    Ok(json!({
        "blueprint": bp, "channel_meta": meta, "data_label": "DỮ LIỆU THẬT", "stats": stats, "pillars": pillars,
        "schedule_heatmap": heat, "schedule_heatmap_status": heat_status, "top_outliers": outliers, "title_patterns": patterns,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ids_and_patterns() {
        assert_eq!(generate_channel_id("https://youtube.com/@SocNho/"), "ch_socnho");
        assert!(generate_channel_id("https://x.com/a").starts_with("ch_"));
        assert_eq!(abstract_title_pattern("Top 5 mẹo trong 10 ngày"), "Top [Số] mẹo trong [Thời gian]");
        let v = vec![json!({"title":"Review áo khoác đẹp","view_count":10}), json!({"title":"Review giày thể thao","view_count":30})];
        assert_eq!(cluster_pillars(&v)[0]["name"], "Review");
    }
}
