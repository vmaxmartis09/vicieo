use super::contracts::*;
use super::util::*;
use serde_json::{json, Value};

/// Candidate title must not copy a competitor title (similarity < max).
pub fn validate_title_guardrail(candidate: &str, sources: &[String], max_similarity: f64) -> bool {
    let c = candidate.trim().to_lowercase();
    !sources.iter().any(|s| fuzz_ratio(&c, &s.trim().to_lowercase()) >= max_similarity)
}

pub fn generate_channel_strategy(pid: &str, channel_ids: &[String]) -> anyhow::Result<Value> {
    let mut titles = 0usize;
    for cid in channel_ids {
        if let Some(bp) = load_channel_blueprint(pid, cid) {
            titles += bp.top_videos.iter().filter(|v| v.get("title").is_some()).count();
        }
        if let Ok(txt) = std::fs::read_to_string(pp(pid, &["knowledge", "channels", safe_id(cid)?, "videos.jsonl"], false)) {
            titles += txt.lines().filter_map(|l| serde_json::from_str::<Value>(l).ok()).filter(|d| d.get("title").is_some()).count();
        }
    }
    let strategy = json!({
        "project_id": pid,
        "reference_channels": channel_ids,
        "differentiation": "Khác biệt hoá độc quyền: Kết hợp nhân vật hoạt hình 3D nguyên bản (Sóc Nhỏ Bi 100cm, Koala 150cm) với KOC AI LyLy/Neau review sản phẩm chân thực. Giọng đọc chuẩn OmniVoice SOTA ấm áp, tự nhiên, tạo sự gắn kết cảm xúc mạnh mẽ mà các kênh đối thủ reup/cắt ghép không thể sao chép.",
        "our_pillars": [
            {"id": "pillar_animation_life", "name": "Chuyện Hằng Ngày của Sóc Nhỏ (Hoạt hình 3D)", "description": "Các tình huống hài hước, giáo dục kỹ năng sống giữa Sóc Nhỏ (100cm) và Koala (150cm).", "target_ratio": 30},
            {"id": "pillar_science_fun", "name": "Thám Tử Tò Mò — Giải Mã Đời Sống", "description": "Giải thích các hiện tượng khoa học thú vị, mẹo vặt gia đình dễ hiểu cho phụ huynh & trẻ nhỏ.", "target_ratio": 25},
            {"id": "pillar_koc_review", "name": "KOC Review & Đập Hộp Công Nghệ / Gia Dụng", "description": "KOC LyLy/Neau review đồ dùng học tập, thiết bị gia đình, đồ chơi giáo dục với ưu + nhược điểm thật.", "target_ratio": 30},
            {"id": "pillar_affiliate_hacks", "name": "Bí Kíp Tiết Kiệm & Săn Deal Thông Minh", "description": "So sánh 2-3 sản phẩm, hướng dẫn chọn mua hàng chuẩn giá tốt kèm link tiếp thị liên kết an toàn.", "target_ratio": 15}
        ],
        "title_rules": [
            "Độ dài tối ưu: 45 - 65 ký tự, đảm bảo hiển thị trọn vẹn trên mobile",
            "Mỗi tiêu đề có 1 từ khoá gây tò mò + 1 kết quả cụ thể (VD: 'Bí mật...', 'Cái kết...', 'Sự thật...')",
            "Chỉ dùng tối đa 1 emoji phù hợp ngữ cảnh, không spam icon",
            "BẮT BUỘC kiểm tra độ tương đồng: Tỉ lệ trùng lặp với đối thủ phải dưới 70%"
        ],
        "thumbnail_rules": [
            "Luôn có mặt nhân vật Sóc Nhỏ hoặc KOC với biểu cảm ngạc nhiên / thích thú rõ nét",
            "Văn bản trên thumbnail ngắn gọn <= 4 từ, font không chân đậm nét, độ tương phản cao",
            "Màu chủ đạo ấm áp: Vàng cam (#FFA500) kết hợp Xanh ngọc hoặc Tím pastel",
            "Quy tắc 1/3: Chủ thể đặt ở 1/3 bên phải, text và icon trực quan đặt bên trái"
        ],
        "publishing_schedule": {
            "tiktok": "19:30 - 20:30 hàng ngày (giờ vàng lướt feed)",
            "shorts": "11:45 trưa (nghỉ trưa) và 18:00 chiều",
            "reels": "12:00 trưa và 20:00 tối",
            "youtube": "Tối Thứ 3, Thứ 5 và 09:00 sáng Chủ Nhật (video hoạt hình dài)"
        },
        "format_mix": {"long_animation_3d": 20, "shorts_vertical_story": 40, "koc_review_affiliate": 40},
        "kpi_30d": {"status": "MẪU", "note": "[MẪU - Mục tiêu kế hoạch tham khảo, không phải số liệu đo thực tế]", "target_total_views": 500000, "target_subscribers": 15000, "target_affiliate_clicks": 3500, "target_orders": 250},
        "guardrails": [
            "Tuyệt đối KHÔNG sử dụng tên, logo, watermark hoặc avatar của kênh đối thủ.",
            "Tuyệt đối KHÔNG sao chép nguyên văn tiêu đề (bắt buộc rapidfuzz < 70%).",
            "Không nhái lại visual hay tạo nhân vật gây nhầm lẫn thương hiệu.",
            "Mọi video affiliate phải có nhãn tiếp thị công khai (#affiliate #quangcao)."
        ],
        "source_title_count": titles
    });
    write_json(&pp(pid, &["content", "strategy.json"], true), &strategy)?;
    Ok(strategy)
}

pub fn load_channel_strategy(pid: &str) -> Option<Value> { read_json(&pp(pid, &["content", "strategy.json"], false)) }
