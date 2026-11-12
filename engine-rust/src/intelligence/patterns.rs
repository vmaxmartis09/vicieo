use super::contracts::*;
use serde_json::{json, Value};

pub const HOOK_PATTERNS: &[&str] = &[
    "{pain_point}? Đây là cách {product} giải quyết chỉ trong 1 phút!",
    "Đừng mua {category} nếu bạn chưa biết bí quyết này!",
    "Top 1 món đồ không thể thiếu nếu bạn đang gặp vấn đề {pain_point}",
    "Tại sao mọi người lại săn lùng {product} nhiều đến vậy?",
];
pub const CTA_PATTERNS: &[&str] = &[
    "Bấm ngay vào giỏ hàng góc trái bên dưới để nhận ưu đãi hôm nay!",
    "Link chính hãng mình để sẵn ở bio, các bạn tham khảo nhé!",
    "Thả tim và theo dõi để xem thêm nhiều mẹo hữu ích nữa nha!",
];

fn round2(x: f64) -> f64 { (x * 100.0).round() / 100.0 }

pub fn aggregate_patterns(pid: &str, _niche: Option<&str>) -> Value {
    let bps = list_video_blueprints(pid);
    if bps.is_empty() {
        return json!({
            "total_analyzed": 0, "data_status": "MẪU",
            "note": "[MẪU] Chưa có blueprint nào trong dự án, số liệu dưới đây là giá trị mẫu tham khảo",
            "avg_pacing_cuts_per_10s": 2.5, "median_hook_duration_s": 3.0,
            "common_role_sequences": [["hook", "problem", "demo", "offer", "cta"]],
            "hook_templates": HOOK_PATTERNS, "cta_templates": CTA_PATTERNS,
        });
    }
    let mut hooks: Vec<f64> = vec![];
    let mut seqs: Vec<Vec<String>> = vec![];
    for b in &bps {
        if let Some(h) = b.beats.first() {
            hooks.push(h.t_end - h.t_start);
            seqs.push(b.beats.iter().map(|x| x.role.clone()).collect());
        }
    }
    hooks.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = if hooks.is_empty() { 3.0 } else if hooks.len() % 2 == 1 { hooks[hooks.len() / 2] } else { (hooks[hooks.len() / 2 - 1] + hooks[hooks.len() / 2]) / 2.0 };
    let avg = bps.iter().map(|b| b.pacing_cuts_per_10s).sum::<f64>() / bps.len() as f64;
    json!({
        "total_analyzed": bps.len(), "data_status": "DỮ LIỆU THẬT",
        "avg_pacing_cuts_per_10s": round2(avg), "median_hook_duration_s": round2(median),
        "common_role_sequences": seqs.into_iter().take(5).collect::<Vec<_>>(),
        "hook_templates": HOOK_PATTERNS, "cta_templates": CTA_PATTERNS,
    })
}
