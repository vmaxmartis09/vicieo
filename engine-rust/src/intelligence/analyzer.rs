//! Video blueprint analysis: scenes + transcript -> beats (heuristics, then optional LLM refinement).
use super::contracts::*;
use super::sources::service::get_source;
use super::util::*;
use crate::providers;
use serde_json::{json, Value};

const TEMPLATES: &[(&str, &[&str])] = &[
    ("koc-review-v1", &["hook", "problem", "reveal", "demo", "proof", "cta"]),
    ("tiktok-sales-v1", &["hook", "agitate", "reveal", "demo", "offer", "urgency", "cta"]),
    ("affiliate-compare-v1", &["hook", "compare", "demo", "proof", "offer", "cta"]),
];

pub fn infer_beat_role(idx: usize, total: usize, start_s: f64, duration_s: f64, text: &str) -> &'static str {
    let t = text.to_lowercase();
    let any = |ks: &[&str]| ks.iter().any(|k| t.contains(k));
    if start_s <= 3.5 || idx == 0 { return "hook"; }
    if idx + 1 == total || any(&["theo dõi", "mua ngay", "giỏ hàng", "link"]) { return "cta"; }
    if any(&["so sánh", "khác biệt", "hơn hẳn", "thay vì"]) { return "compare"; }
    if any(&["vấn đề", "mệt mỏi", "khó chịu", "phiền phức", "đau đầu"]) { return "problem"; }
    if any(&["thì ra", "hóa ra", "đây là", "giải pháp"]) { return "reveal"; }
    if any(&["thử", "cách dùng", "dùng thử", "hướng dẫn", "xem này"]) { return "demo"; }
    if any(&["hiệu quả", "đã kiểm chứng", "chứng nhận", "đánh giá", "sao"]) { return "proof"; }
    if any(&["giá", "ưu đãi", "giảm", "sale", "khuyến mãi", "chỉ"]) { return "offer"; }
    if any(&["nhanh tay", "hôm nay", "kẻo lỡ", "gấp"]) { return "urgency"; }
    let rel = start_s / duration_s.max(1.0);
    if rel < 0.25 { "problem" } else if rel < 0.45 { "reveal" } else if rel < 0.70 { "demo" } else if rel < 0.85 { "offer" } else { "cta" }
}

fn f(v: &Value, ks: &[&str], d: f64) -> f64 { ks.iter().find_map(|k| v.get(*k).and_then(|x| x.as_f64())).unwrap_or(d) }

/// Ask the LLM to label each beat role / hook / cta. Any failure (mock provider, bad JSON) keeps heuristics.
async fn llm_refine(bp: &mut VideoBlueprint) -> bool {
    let beats: Vec<Value> = bp.beats.iter().enumerate().map(|(i, b)| json!({"i": i, "t": [b.t_start, b.t_end], "text": b.summary})).collect();
    let prompt = format!(
        "Phân tích cấu trúc video ngắn ({}). Gán vai trò cho từng beat trong: hook, problem, agitate, reveal, demo, proof, compare, offer, urgency, cta. \
         Trả JSON duy nhất dạng {{\"roles\":[...đúng {} phần tử...],\"hook\":\"...\",\"cta\":\"...\",\"visuals\":[...mô tả hình ảnh ngắn mỗi beat...]}}.\nBeats: {}",
        bp.platform, bp.beats.len(), Value::Array(beats)
    );
    let Ok(raw) = providers::llm().generate(&prompt, Some("Bạn là chuyên gia phân tích video thương mại điện tử TikTok."), Some(&llm_model()), true).await else { return false };
    let Some(j) = serde_json::from_str::<Value>(raw.trim().trim_start_matches("```json").trim_end_matches("```").trim()).ok() else { return false };
    let Some(roles) = j.get("roles").and_then(|r| r.as_array()) else { return false };
    if roles.len() != bp.beats.len() { return false; }
    for (b, r) in bp.beats.iter_mut().zip(roles) { if let Some(s) = r.as_str() { b.role = s.to_string(); } }
    if let Some(vs) = j.get("visuals").and_then(|v| v.as_array()) {
        for (b, v) in bp.beats.iter_mut().zip(vs) { if let Some(s) = v.as_str() { b.visual = s.to_string(); } }
    }
    if let Some(h) = j.get("hook").and_then(|x| x.as_str()).filter(|s| !s.is_empty()) { bp.hook = h.into(); }
    if let Some(c) = j.get("cta").and_then(|x| x.as_str()).filter(|s| !s.is_empty()) { bp.cta = c.into(); }
    true
}

pub async fn analyze_video(pid: &str, source_id: &str) -> ApiResult<VideoBlueprint> {
    let source = get_source(pid, source_id).ok_or_else(|| ApiError::nf(format!("Source not found: {source_id}")))?;
    let dir = pp(pid, &["raw", safe_id(source_id).map_err(|e| ApiError::bad(e.to_string()))?], false);
    let arr = |p: std::path::PathBuf| read_json(&p).and_then(|v| v.as_array().cloned()).unwrap_or_default();
    let scenes = arr(dir.join("scenes.json"));
    let mut tr = arr(dir.join("transcript.vi.json"));
    if tr.is_empty() { tr = arr(dir.join("transcript.orig.json")); }
    let duration = source.duration_s.max(5.0);

    let mut beats = vec![];
    if !scenes.is_empty() {
        for (idx, sc) in scenes.iter().enumerate() {
            let st = f(sc, &["start_s", "start_sec"], 0.0);
            let en = f(sc, &["end_s", "end_sec"], st + 3.0);
            let mut text = String::new();
            for t in &tr {
                if f(t, &["start"], 0.0) <= en && f(t, &["end"], 0.0) >= st { text.push(' '); text.push_str(&s(t, "text")); }
            }
            let text = text.trim().to_string();
            beats.push(Beat {
                t_start: st, t_end: en, role: infer_beat_role(idx, scenes.len(), st, duration, &text).into(),
                summary: if text.is_empty() { format!("Cảnh {}", idx + 1) } else { text.clone() },
                visual: format!("Cảnh hành động {}", idx + 1),
                on_screen_text: text.chars().take(30).collect(),
            });
        }
    } else {
        let seg = duration / 4.0;
        for (i, r) in ["hook", "problem", "demo", "cta"].iter().enumerate() {
            beats.push(Beat { t_start: (i as f64 * seg * 100.0).round() / 100.0, t_end: ((i + 1) as f64 * seg * 100.0).round() / 100.0, role: (*r).into(), summary: format!("Phần {r} của video"), ..Default::default() });
        }
    }
    let cuts = (beats.len().saturating_sub(1)).max(1) as f64;
    let pacing = (cuts / duration * 10.0 * 100.0).round() / 100.0;
    let mut bp = VideoBlueprint {
        id: format!("bp_{source_id}"), source_ref: source_id.into(), duration_s: duration, platform: source.platform.clone(),
        hook: beats.first().map(|b| b.summary.clone()).unwrap_or_else(|| "Xem video này ngay!".into()),
        cta: if beats.len() > 1 { beats.last().unwrap().summary.clone() } else { "Bấm theo dõi nhé!".into() },
        beats, pacing_cuts_per_10s: pacing, suggested_template_id: None,
        data_label: if scenes.is_empty() { "MẪU (Ước tính theo thời lượng do chưa có dữ liệu cắt cảnh)".into() } else { "DỮ LIỆU THẬT".into() },
        is_mock: scenes.is_empty(),
        metadata: json!({"scenes_detected": scenes.len(), "transcript_detected": tr.len()}),
    };
    let llm_used = llm_refine(&mut bp).await;
    bp.metadata["llm_refined"] = json!(llm_used);

    let seq = bp.beats.iter().map(|b| b.role.as_str()).collect::<Vec<_>>().join(" ");
    let (mut best, mut hi) = ("koc-review-v1", 0.0);
    for (id, roles) in TEMPLATES {
        let sc = fuzz_ratio(&seq, &roles.join(" "));
        if sc > hi { hi = sc; best = id; }
    }
    bp.suggested_template_id = Some(best.into());
    save_video_blueprint(pid, &bp)?;
    Ok(bp)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roles() {
        assert_eq!(infer_beat_role(0, 5, 0.0, 30.0, ""), "hook");
        assert_eq!(infer_beat_role(2, 5, 10.0, 30.0, "bấm theo dõi nhé"), "cta");
        assert_eq!(infer_beat_role(2, 5, 10.0, 30.0, "so sánh hai loại"), "compare");
        assert_eq!(infer_beat_role(2, 5, 10.0, 30.0, "xxx"), "reveal");
    }
}
