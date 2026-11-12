use crate::intelligence::util::*;
use crate::providers;
use serde_json::{json, Value};
use std::collections::BTreeMap;

const DEFAULTS: &[(&str, &str)] = &[("你好", "Xin chào"), ("家人们", "Cả nhà ơi"), ("老铁们", "Các bạn ơi"), ("今天给大家分享", "Hôm nay mình chia sẻ"), ("点赞", "thả tim"), ("关注", "bấm theo dõi"), ("真的太好用了", "thật sự quá tiện lợi"), ("赶紧试试吧", "thử ngay nhé"), ("看这个", "Xem cái này nè"), ("非常棒", "rất tuyệt vời")];

pub fn load_glossary(pid: &str) -> BTreeMap<String, String> {
    let mut g: BTreeMap<String, String> = DEFAULTS.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect();
    let files = [crate::workspace::shared_path(&["knowledge", "glossary.json"], false).unwrap_or_default(), pp(pid, &["knowledge", "glossary.json"], false)];
    for f in files {
        if let Some(Value::Object(m)) = read_json(&f) { for (k, v) in m { if let Some(s) = v.as_str() { g.insert(k, s.into()); } } }
    }
    g
}

pub fn condense_text(text: &str, max: usize) -> String {
    if text.chars().count() <= max || max < 15 { return text.into(); }
    let mut out = String::new();
    for w in text.split_whitespace() {
        if out.chars().count() + w.chars().count() < max { if !out.is_empty() { out.push(' '); } out.push_str(w); } else { break; }
    }
    if out.is_empty() { text.chars().take(max).collect() } else { out }
}

/// One LLM call translating all segments to Vietnamese; None on any failure (mock provider, bad JSON, wrong count).
async fn llm_translate(texts: &[String]) -> Option<Vec<String>> {
    let prompt = format!("Dịch từng câu sau sang tiếng Việt tự nhiên, ngắn gọn, đúng giọng video bán hàng TikTok. Trả về JSON mảng chuỗi, đúng {} phần tử, cùng thứ tự.\n{}", texts.len(), serde_json::to_string(texts).ok()?);
    let raw = providers::llm().generate(&prompt, Some("Bạn là biên dịch viên Trung/Anh-Việt."), Some(&llm_model()), true).await.ok()?;
    let v: Vec<String> = serde_json::from_str(raw.trim().trim_start_matches("```json").trim_end_matches("```").trim()).ok()?;
    (v.len() == texts.len()).then_some(v)
}

pub async fn translate_segments(pid: &str, segs: &[Value]) -> Vec<Value> {
    let gl = load_glossary(pid);
    let origs: Vec<String> = segs.iter().map(|s| s.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string()).collect();
    let llm = llm_translate(&origs).await;
    segs.iter().enumerate().map(|(i, seg)| {
        let (st, en) = (seg["start"].as_f64().unwrap_or(0.0), seg["end"].as_f64().unwrap_or(0.0));
        let max = ((en - st).max(1.0) * 15.0 * 1.25) as usize;
        let mut vi = llm.as_ref().map(|l| l[i].clone()).unwrap_or_else(|| origs[i].clone());
        for (zh, v) in &gl { if vi.contains(zh.as_str()) { vi = vi.replace(zh.as_str(), v); } }
        if vi.chars().count() > max { vi = condense_text(&vi, max); }
        json!({"start": (st * 100.0).round() / 100.0, "end": (en * 100.0).round() / 100.0, "text": vi, "orig_text": origs[i], "char_count": vi.chars().count(), "max_allowed_chars": max})
    }).collect()
}

pub async fn run_translation(pid: &str, source_id: &str) -> ApiResult<Vec<Value>> {
    let dir = pp(pid, &["raw", safe_id(source_id).map_err(|e| ApiError::bad(e.to_string()))?, "x"], false).parent().unwrap().to_path_buf();
    let orig = read_json(&dir.join("transcript.orig.json")).and_then(|v| v.as_array().cloned())
        .ok_or_else(|| ApiError::nf(format!("Original transcript not found at: {}", dir.join("transcript.orig.json").display())))?;
    let vi = translate_segments(pid, &orig).await;
    write_json(&dir.join("transcript.vi.json"), &vi)?;
    Ok(vi)
}

#[cfg(test)]
mod tests {
    #[test]
    fn condense() {
        assert_eq!(super::condense_text("một hai ba bốn năm sáu bảy tám chín mười", 20), "một hai ba bốn năm");
        assert_eq!(super::condense_text("ngắn", 20), "ngắn");
    }
}
