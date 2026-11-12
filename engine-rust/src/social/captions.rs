//! Captions: deterministic templates (parity with captions.py). Optional `use_llm: true` rewrites each hook via providers::llm(); falls back silently to the template.
use super::models::s;
use super::platform_rules::rule;
use super::store::SocialStore;
use super::{video_library, ApiErr};
use serde_json::{json, Value};

fn hooks(goal: &str) -> &'static [&'static str] {
    match goal {
        "education" => &["3 bài học bổ ích cho bé từ câu chuyện của Sóc Nhỏ!", "Ba mẹ đã biết cách giúp bé tự giác đánh răng mỗi ngày chưa?",
            "Khám phá kiến thức thú vị về thế giới tự nhiên cùng Sóc Nhỏ!", "Mẹo nhỏ giúp bé vui học đánh răng chuẩn y khoa cực dễ hiểu!"],
        "sales" => &["Món quà đặc biệt mà bé nào cũng mê mẩn từ cái nhìn đầu tiên!", "Review chân thực bàn chải điện hình Sóc Nhỏ cho các bé yêu!",
            "Bé sợ đánh răng? Xem ngay giải pháp thần kỳ của mẹ bỉm sữa!", "Ưu đãi độc quyền hôm nay: Tặng kèm quà xinh xắn cho bé!"],
        _ => &["Chuyện gì sẽ xảy ra khi Sóc Bi gặp bác sĩ Koala khám răng?", "Bí mật đằng sau nụ cười rạng rỡ của Sóc Nhỏ hôm nay!",
            "Đừng bao giờ ăn kẹo trước khi đi ngủ nếu chưa xem video này!", "Khoảnh khắc hài hước nhất trong ngày của bé Sóc Bi!"],
    }
}
fn cta(goal: &str) -> &'static str {
    match goal {
        "education" => "👉 Đừng quên chia sẻ video này đến các bậc phụ huynh để cùng nhau chăm sóc bé thật tốt!",
        "sales" => "👉 Link sản phẩm chính hãng có ngay trong bio/phần mô tả. Đặt ngay kẻo hết quà!",
        _ => "👉 Nhấn Follow kênh để đón xem những tập phim hoạt hình mới nhất mỗi ngày nhé!",
    }
}
fn pool(p: &str) -> Vec<String> {
    let v: &[&str] = match p {
        "youtube" => &["#SocNhoTV", "#HoatHinh3D", "#ThieuNhi", "#GiaoDucSom", "#SocBi", "#Koala"],
        "tiktok" => &["#socnhotv", "#hoathinh3d", "#learnontiktok", "#mebimsua", "#embe", "#phimhoathinh"],
        "facebook" => &["#SocNhoTV", "#HoatHinhThieuNhi", "#NuoiDayCon", "#KienThucChoBe", "#GiaDinh"],
        "instagram" => &["#socnhotv", "#cartoon3d", "#kidstv", "#parenting", "#animation", "#family"],
        "threads" => &["#socnhotv", "#hoathinh", "#giadinh"],
        "x" => &["#SocNhoTV", "#Animation", "#Kids"],
        _ => return vec!["#socnhotv".into(), format!("#{p}")],
    };
    v.iter().map(|x| x.to_string()).collect()
}
fn cut(s: &str, n: usize) -> String { s.chars().take(n).collect() }

pub async fn generate(project: &str, req: &Value) -> Result<Value, ApiErr> {
    let st = SocialStore::new(project).map_err(|e| ApiErr::bad(e.to_string()))?;
    let vid = req["video_id"].as_str().ok_or_else(|| ApiErr::invalid("field required: video_id"))?;
    let platforms: Vec<String> = req["platforms"].as_array().ok_or_else(|| ApiErr::invalid("field required: platforms"))?.iter().filter_map(|x| x.as_str().map(String::from)).collect();
    let video = video_library::list_videos(project).into_iter().find(|v| s(v, "id") == vid).or_else(|| {
        let r = crate::workspace::project_path(project, &["runs", "render_queue", &format!("{vid}.req.json")], false).ok()?;
        let d: Value = serde_json::from_str(&std::fs::read_to_string(r).ok()?).ok()?;
        Some(json!({"id":vid,"title":d["data"]["title"].as_str().filter(|x| !x.is_empty()).unwrap_or("Video hoạt hình Sóc Nhỏ"),"script_excerpt":d["data"]["book_page_text"].as_str().unwrap_or("")}))
    }).unwrap_or_else(|| json!({"id":vid,"title":format!("Video {}", cut(vid, 8)),"script_excerpt":""}));
    let title = video["title"].as_str().unwrap_or("Video hoạt hình Sóc Nhỏ TV").to_string();
    let script = s(&video, "script_excerpt").to_string();
    let goal = match req["goal"].as_str() { Some(g @ ("story" | "education" | "sales")) => g, _ => "story" };
    let link = req["affiliate_link"].as_str().filter(|x| !x.is_empty());
    let aff = req["include_affiliate"] == json!(true) || link.is_some();
    let disclosure = if aff { "(Video có gắn liên kết tiếp thị liên kết / quảng cáo sản phẩm tài trợ #quangcao)" } else { "" };
    let mut c = cta(goal).to_string();
    if let Some(l) = link { c += &format!("\n🔗 Link sản phẩm: {l}"); }
    let accounts = st.list_accounts();
    let sel: Vec<&Value> = req["account_ids"].as_array().into_iter().flatten().filter_map(|id| accounts.iter().find(|a| Some(s(a, "id")) == id.as_str())).collect();
    let by_account = req["vary_per_account"] == json!(true) && !sel.is_empty();
    // (platform, account, base hashtags)
    let jobs: Vec<(String, Option<&Value>, Vec<String>)> = if by_account {
        sel.iter().enumerate().map(|(i, a)| {
            let mut t: Vec<String> = a["default_hashtags"].as_array().map(|x| x.iter().filter_map(|y| y.as_str().map(String::from)).collect()).unwrap_or_default();
            if t.is_empty() { t = pool(s(a, "platform")); }
            if t.len() > 1 && i > 0 { let k = i % t.len(); t.rotate_left(k); }
            (s(a, "platform").to_string(), Some(*a), t)
        }).collect()
    } else { platforms.iter().map(|p| (p.clone(), None, pool(p))).collect() };
    let use_llm = req["use_llm"] == json!(true);
    let mut out = vec![];
    for (idx, (plat, acc, base_tags)) in jobs.into_iter().enumerate() {
        let r = rule(&plat);
        let hs = hooks(goal);
        let mut hook = hs[idx % hs.len()].to_string();
        if use_llm {
            let prompt = format!("Viết lại câu hook mở đầu cho video '{title}' trên {plat}, mục tiêu {goal}, giọng {}. Chỉ trả về đúng 1 câu.\nHook gốc: {hook}", req["tone"].as_str().unwrap_or("friendly"));
            if let Ok(t) = crate::providers::llm().generate(&prompt, Some("Bạn là biên tập viên mạng xã hội tiếng Việt."), None, false).await {
                let t = t.trim().trim_matches('"').to_string();
                if !t.is_empty() && t.chars().count() <= 200 { hook = t; }
            }
        }
        let mut lines = vec![hook.clone()];
        if !script.is_empty() { lines.push(format!("\n{}...", cut(&script, 150))); }
        lines.push(format!("\n{c}"));
        if !disclosure.is_empty() { lines.push(format!("\n{disclosure}")); }
        let mut full = lines.join("\n");
        let tags: Vec<String> = base_tags.into_iter().take(r.max_tags).collect();
        let tag_str = tags.join(" ");
        if ["tiktok", "instagram", "threads", "x", "facebook"].contains(&plat.as_str()) && format!("{full}\n\n{tag_str}").chars().count() <= r.max_caption { full = format!("{full}\n\n{tag_str}"); }
        let full = cut(&full, r.max_caption);
        let yt = plat == "youtube";
        let yt_title = cut(&format!("{hook} - Sóc Nhỏ TV"), r.max_title);
        let yt_desc = cut(format!("{hook}\n\n{}\n\n{c}\n\n{disclosure}\n\n{tag_str}", if script.is_empty() { &title } else { &script }).trim(), r.max_caption);
        let mut o = json!({"platform":plat,"account_id":acc.map(|a| a["id"].clone()),"account_name":acc.map(|a| if s(a,"display_name").is_empty() { a["handle"].clone() } else { a["display_name"].clone() }),
            "hook":hook,"caption":full,"hashtags":tags,"cta":c,"title":if yt { yt_title } else { cut(&title, r.max_title) },
            "description":if yt { json!(yt_desc) } else { Value::Null },"tags":if yt { json!(tags) } else { Value::Null },"disclosure_ad":aff,"disclosure_text":disclosure});
        if acc.is_none() { o["account_id"] = Value::Null; o["account_name"] = Value::Null; }
        out.push(o);
    }
    Ok(json!(out))
}
