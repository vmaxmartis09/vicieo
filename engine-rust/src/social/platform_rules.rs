use serde_json::{json, Value};

pub struct Rule { pub name: &'static str, pub ratios: &'static [&'static str], pub max_dur: f64, pub max_mb: f64, pub max_title: usize, pub max_caption: usize, pub max_tags: usize, pub quota: u32 }

pub fn rule(platform: &str) -> Rule {
    let r = |name, ratios, max_dur, max_mb, max_title, max_caption, max_tags, quota| Rule { name, ratios, max_dur, max_mb, max_title, max_caption, max_tags, quota };
    match platform.to_lowercase().as_str() {
        "youtube" => r("YouTube", &["16:9", "9:16"], 43200.0, 256000.0, 100, 5000, 15, 6),
        "tiktok" => r("TikTok", &["9:16"], 600.0, 4000.0, 150, 2200, 10, 10),
        "facebook" => r("Facebook", &["4:5", "9:16", "16:9"], 14400.0, 4000.0, 255, 5000, 30, 25),
        "instagram" => r("Instagram", &["9:16", "4:5", "1:1"], 900.0, 4000.0, 150, 2200, 30, 50),
        "threads" => r("Threads", &["9:16", "1:1", "16:9"], 300.0, 1000.0, 100, 500, 5, 20),
        "x" => r("X (Twitter)", &["16:9", "1:1", "9:16"], 140.0, 512.0, 100, 280, 4, 20),
        _ => r("Thủ công", &["9:16", "16:9", "4:5", "1:1"], 86400.0, 10000.0, 255, 5000, 30, 999),
    }
}

fn issue(level: &str, code: &str, msg: String) -> Value { json!({"level": level, "code": code, "message_vi": msg}) }

/// Port of platform_rules.validate. `video`: {duration_s,size_mb,width,height,aspect_ratio,variants}.
pub fn validate(video: Option<&Value>, target: &Value, account: Option<&Value>) -> Vec<Value> {
    let mut out = vec![];
    let platform = target["platform"].as_str().unwrap_or("youtube");
    let title = target["title"].as_str().unwrap_or("");
    let caption = target["caption"].as_str().unwrap_or("");
    let tags = target["hashtags"].as_array().map_or(0, |a| a.len());
    let r = rule(platform);
    if let Some(a) = account {
        let dn = a["display_name"].as_str().unwrap_or("");
        match a["status"].as_str() {
            Some("expired") => out.push(issue("error", "account_expired", format!("Tài khoản {dn} đã hết hạn xác thực OAuth, cần kết nối lại."))),
            Some("disabled") => out.push(issue("error", "account_disabled", format!("Tài khoản {dn} đang bị tạm khoá/vô hiệu hoá."))),
            _ => {}
        }
    }
    if let Some(v) = video {
        let dur = v["duration_s"].as_f64().unwrap_or(0.0);
        let mb = v["size_mb"].as_f64().unwrap_or(0.0);
        let (w, h) = (v["width"].as_i64().unwrap_or(0), v["height"].as_i64().unwrap_or(0));
        if dur > r.max_dur { out.push(issue("error", "duration_exceeded", format!("Thời lượng video ({dur:.0}s) vượt quá giới hạn {:.0}s của {}.", r.max_dur, r.name))); }
        if mb > r.max_mb { out.push(issue("error", "size_exceeded", format!("Dung lượng video ({mb:.1}MB) vượt quá giới hạn {:.0}MB của {}.", r.max_mb, r.name))); }
        let mut ar = v["aspect_ratio"].as_str().unwrap_or("").to_string();
        if ar.is_empty() && w > 0 && h > 0 { ar = if h > w { "9:16" } else { "16:9" }.into(); }
        if r.ratios == ["9:16"] {
            let has = v["variants"].as_object().map_or(false, |m| m.keys().any(|k| ["tiktok", "shorts", "reels", "9:16"].contains(&k.as_str())));
            if ar == "16:9" && !has { out.push(issue("warn", "ratio_mismatch", format!("Video tỉ lệ 16:9 ngang không tối ưu cho {} (khuyên dùng 9:16 dọc). Đề xuất: dùng bản blur-pad hoặc render lại biến thể 9:16.", r.name))); }
        } else if r.ratios.contains(&"16:9") && platform == "youtube" && dur > 60.0 && ar == "9:16" {
            out.push(issue("warn", "ratio_mismatch", "Video tỉ lệ dọc (9:16) dài hơn 60s sẽ hiển thị viền đen 2 bên trên YouTube màn hình rộng.".into()));
        }
    }
    let (tl, cl) = (title.chars().count(), caption.chars().count());
    if !title.is_empty() && tl > r.max_title { out.push(issue("error", "title_too_long", format!("Tiêu đề ({tl} ký tự) vượt quá giới hạn {} ký tự của {}.", r.max_title, r.name))); }
    if !caption.is_empty() && cl > r.max_caption { out.push(issue("error", "caption_too_long", format!("Nội dung caption ({cl} ký tự) vượt quá giới hạn {} ký tự của {}.", r.max_caption, r.name))); }
    if tags > r.max_tags { out.push(issue("warn", "too_many_hashtags", format!("Số lượng hashtag ({tags}) vượt khuyến nghị {} của {}.", r.max_tags, r.name))); }
    out
}
