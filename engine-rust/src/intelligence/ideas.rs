use super::contracts::*;
use super::products::*;
use super::script_gen::{generate_complete_script, ScriptOpts};
use super::strategy::validate_title_guardrail;
use super::util::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

// (pillar, title, hook, template, priority)
const SEEDS: &[(&str, &str, &str, u8)] = &[
    ("Sóc Nhỏ và Thử Thách Nhịn Ăn Vặt Trong 24 Giờ", "koc-review-v1", "", 5),
    ("Koala Lần Đầu Đi Siêu Thị Cùng Sóc Nhỏ", "koc-review-v1", "", 4),
    ("Chiếc Hộp Bí Ẩn Dưới Gốc Cây Dẻ Gai", "koc-review-v1", "", 3),
    ("Một Ngày Làm Trợ Lý Cho Bạn Koala", "koc-review-v1", "", 4),
    ("Sóc Nhỏ Học Cách Quản Lý Tiền Tiết Kiệm", "koc-review-v1", "", 5),
    ("Kỳ Nghỉ Hè Bất Ổn Của Đôi Bạn Thân", "koc-review-v1", "", 3),
    ("Khi Sóc Nhỏ Bị Ốm và Koala Làm Bác Sĩ", "koc-review-v1", "", 4),
    ("Tại Sao Mắt Chúng Ta Lại Chớp Hơn 20.000 Lần Mỗi Ngày?", "tiktok-sales-v1", "", 4),
    ("Sự Thật Về Chiếc Bụng Kêu Ục Ục Khi Đói", "tiktok-sales-v1", "", 5),
    ("Top 5 Động Vật Có Siêu Năng Lực Kỳ Lạ Nhất Trái Đất", "tiktok-sales-v1", "", 5),
    ("Vì Sao Nước Biển Lại Mặn Mà Nước Sông Lại Ngọt?", "tiktok-sales-v1", "", 3),
    ("Giải Mã Hiện Tượng Nấc Cụt và Mẹo Dứt Điểm Trong 10 Giây", "tiktok-sales-v1", "", 4),
    ("Cầu Vồng Thực Chất Có Hình Gì Từ Trên Máy Bay?", "tiktok-sales-v1", "", 4),
    ("Tại Sao Chúng Ta Lại Quên Giấc Mơ Ngay Sau Khi Thức Dậy?", "tiktok-sales-v1", "", 3),
    ("KOC LyLy Thử Bút Luyện Chữ Tự Xoá: Có Đáng Tiền Không?", "koc-review-v1", "", 5),
    ("Đập Hộp Hộp Cơm Giữ Nhiệt Cắm Điện Mini Cho Học Sinh", "koc-review-v1", "", 4),
    ("Trải Nghiệm Đèn Bàn Chống Cận Thị Thông Minh Sóc Nhỏ", "koc-review-v1", "", 5),
    ("Test Khả Năng Kháng Nước Của Balo Học Sinh Chống Gù", "koc-review-v1", "", 4),
    ("LyLy Review Bình Giữ Nhiệt Báo Nhiệt Độ Cảm Ứng", "koc-review-v1", "", 3),
    ("Thử Thách Dùng Máy In Nhiệt Cầm Tay In Ảnh Học Tập", "koc-review-v1", "", 4),
    ("Review Quạt Tích Điện Cầm Tay Mini Pin Trâu 24 Giờ", "koc-review-v1", "", 5),
    ("Đánh Giá Gối Tựa Lưng Công Thái Học Cho Người Ngồi Lâu", "koc-review-v1", "", 4),
    ("So Sánh 3 Dòng Bàn Chải Điện Giá Rẻ Dưới 300K", "affiliate-compare-v1", "", 5),
    ("Top 3 Món Đồ Gia Dụng Tiện Ích Đáng Mua Nhất Dịp Sale", "affiliate-compare-v1", "", 5),
    ("So Sánh Nồi Chiên Không Dầu Cơ vs Điện Tử Cùng Tầm Giá", "affiliate-compare-v1", "", 4),
    ("Bí Kíp Săn Mã Giảm Giá Shopee & TikTok Shop Giảm 50%", "affiliate-compare-v1", "", 4),
    ("So Sánh 2 Dòng Máy Xay Sinh Tố Cầm Tay Mini", "affiliate-compare-v1", "", 4),
    ("Có Nên Mua Tai Nghe Bluetooth Chống Ồn Giá Học Sinh?", "affiliate-compare-v1", "", 3),
    ("Top 4 Dụng Cụ Vệ Sinh Bàn Học Thông Minh Cực Gọn", "affiliate-compare-v1", "", 4),
    ("So Sánh Sạc Dự Phòng Sạc Nhanh 20W và 30W", "affiliate-compare-v1", "", 4),
];

pub fn generate_ideas_batch(pid: &str, count: usize) -> Vec<ContentIdea> {
    let mut source_titles = vec![];
    for bp in list_channel_blueprints(pid) {
        for v in &bp.top_videos { if let Some(t) = v.get("title").and_then(|t| t.as_str()) { source_titles.push(t.to_string()); } }
    }
    let mut existing: std::collections::HashSet<String> = list_content_ideas(pid).into_iter().map(|i| i.title.to_lowercase()).collect();
    let mut created: Vec<ContentIdea> = vec![];
    for (title, tmpl, _, pri) in SEEDS {
        if created.len() >= count { break; }
        let mut cand = title.to_string();
        if existing.contains(&cand.to_lowercase()) { continue; }
        if !source_titles.is_empty() && !validate_title_guardrail(&cand, &source_titles, 70.0) { cand = format!("{cand} — Bí Quyết Của Sóc Nhỏ"); }
        let h = format!("{:x}", Sha256::digest(format!("{pid}{cand}").as_bytes()));
        let idea = ContentIdea {
            id: format!("idea_{}", &h[..10]), project_id: pid.into(), title: cand.clone(),
            platform_targets: ["tiktok", "shorts", "reels", "youtube"].iter().map(|s| s.to_string()).collect(),
            template_id: Some(tmpl.to_string()), blueprint_ref: None, priority: *pri as i64, status: "new".into(),
        };
        let _ = save_content_idea(pid, &idea);
        existing.insert(cand.to_lowercase());
        created.push(idea);
    }
    if created.len() < count {
        let all = list_content_ideas(pid);
        if !all.is_empty() {
            let ids: Vec<String> = created.iter().map(|c| c.id.clone()).collect();
            created.extend(all.into_iter().filter(|i| !ids.contains(&i.id)));
            created.truncate(count);
        }
    }
    created
}

pub fn update_idea_status(pid: &str, id: &str, status: Option<&str>, priority: Option<i64>) -> Option<ContentIdea> {
    let mut i = load_content_idea(pid, id)?;
    if let Some(s) = status.filter(|s| ["new", "scripted", "rendered", "scheduled", "dropped"].contains(s)) { i.status = s.into(); }
    if let Some(p) = priority.filter(|p| (1..=5).contains(p)) { i.priority = p; }
    let _ = save_content_idea(pid, &i);
    Some(i)
}

pub fn promote_idea(pid: &str, id: &str, product_id: Option<&str>, character: &str, voice: &str) -> ApiResult<Value> {
    let mut idea = load_content_idea(pid, id).ok_or_else(|| ApiError::nf(format!("Không tìm thấy ý tưởng {id}")))?;
    let prods = list_products(pid);
    let mut target = product_id.and_then(|pi| prods.iter().find(|p| p.id == pi).cloned()).or_else(|| prods.first().cloned());
    if target.is_none() {
        let mut p = Product::new(&format!("prod_{}", idea.id), &format!("[MẪU] {}", idea.title));
        p.brand = "Sóc Nhỏ Lab".into(); p.category = "Đời sống".into(); p.price = 199000.0; p.sale_price = Some(159000.0);
        p.usp = vec!["Chất lượng chuẩn mực".into(), "Bảo hành 1 đổi 1".into()];
        p.affiliate_links.tiktok_shop = "https://shop.tiktok.com".into();
        p.compliance_notes = "[MẪU] Sản phẩm mẫu tự động sinh để phục vụ thử nghiệm kịch bản".into();
        save_product(pid, &p)?;
        target = Some(p);
    }
    let gt = match idea.template_id.as_deref() { Some("tiktok-sales-v1") => "tiktok_sales", Some("affiliate-compare-v1") => "affiliate_compare", _ => "koc_review" };
    let res = generate_complete_script(pid, &target.unwrap(), &ScriptOpts { generator_type: gt, character_name: character, voice_id: voice, platforms: vec!["tiktok".into(), "shorts".into(), "reels".into()], ..Default::default() })?;
    idea.status = "scripted".into();
    save_content_idea(pid, &idea)?;
    Ok(json!({"idea": idea, "script_result": res}))
}

/// Push to the social scheduler on this same engine (best effort), then mark the idea scheduled.
pub async fn schedule_idea_to_social(pid: &str, id: &str) -> ApiResult<Value> {
    let mut idea = load_content_idea(pid, id).ok_or_else(|| ApiError::nf(format!("Không tìm thấy ý tưởng {id}")))?;
    let mut resp = json!({"status": "ok", "message": "Ý tưởng đã chuyển trạng thái Lên lịch"});
    let port = std::env::var("VICIEO_PORT").ok().and_then(|p| p.parse::<u16>().ok()).unwrap_or(9000);
    if let Ok(c) = reqwest::Client::builder().timeout(std::time::Duration::from_secs(3)).build() {
        if let Ok(r) = c.post(format!("http://127.0.0.1:{port}/projects/{pid}/social/ideas/{id}/schedule")).send().await {
            if r.status().is_success() { if let Ok(j) = r.json::<Value>().await { resp = j; } }
        }
    }
    idea.status = "scheduled".into();
    save_content_idea(pid, &idea)?;
    Ok(json!({"idea": idea, "social_response": resp}))
}
