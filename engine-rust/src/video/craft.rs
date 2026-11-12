//! Hook lab + script lint (port of video/craft/{hooks,lint}.py). Deterministic, no LLM.
use regex::Regex;
use serde_json::{json, Value};

const GREETINGS: &[&str] = &[
    r"^(xin\s+)?chào(\s+cả\s+nhà|\s+mọi\s+người|\s+các\s+bạn)?",
    r"^(hello|hi)(\s+everyone|\s+các\s+bạn)?",
    r"^hôm\s+nay\s+(mình|em|tôi|chúng\s+mình)(\s+sẽ|\s+chia\s+sẻ|\s+muốn|\s+làm)?",
    r"^ngày\s+mới\s+chúc",
];
const SURPRISE: &[&str] = &[
    "thật ra", "sự thật", "bí mật", "đừng tin", "không phải", "đố bạn", "nghịch lý", "ai ngờ", "chưa từng", "bất ngờ", "tại sao", "vì sao", "chỉ mất", "tiết lộ", "lần đầu", "cảnh báo", "sai lầm",
];
const VISUAL: &[&str] = &["nhìn", "xem", "thấy", "màu", "cầu vồng", "lung linh", "nét", "đẹp", "ảnh", "vải", "mặt", "trời mưa", "ánh sáng", "chiếc đầm", "áo", "da"];

fn word_count(s: &str) -> usize {
    Regex::new(r"\w+").unwrap().find_iter(&s.to_lowercase()).count()
}
fn r1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}
fn r2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn score_hook(text: &str) -> Value {
    let t = text.trim();
    let lower = t.to_lowercase();
    let wc = word_count(t);
    let dur = r2(wc as f64 / 3.8);
    if wc == 0 {
        return json!({"total_score":0.0,"word_count":0,"estimated_duration_sec":0.0,"has_greeting_penalty":false,"penalties":["Hook rỗng"],"bonuses":[]});
    }
    let (mut score, mut pen, mut bon) = (60.0f64, vec![], vec![]);
    let greeting = GREETINGS.iter().any(|p| Regex::new(p).unwrap().is_match(&lower));
    if greeting {
        score -= 35.0;
        pen.push("Trừ điểm: Mở đầu bằng lời chào xã giao làm giảm tỷ lệ giữ chân 3s".to_string());
    }
    if wc <= 12 {
        score += 15.0;
        bon.push(format!("Cộng điểm: Độ dài lý tưởng (≤12 từ: {wc} từ, ~{dur}s)"));
    } else if wc <= 16 {
        score += 5.0;
        bon.push(format!("Chấp nhận được: {wc} từ (~{dur}s)"));
    } else {
        score -= ((wc - 16) as f64 * 4.0).min(30.0);
        pen.push(format!("Trừ điểm: Hook quá dài ({wc} từ, ~{dur}s > 3.5s)"));
    }
    if let Some(k) = SURPRISE.iter().find(|k| lower.contains(*k)) {
        score += 15.0;
        bon.push(format!("Cộng điểm: Chứa yếu tố bất ngờ/kích thích trí tò mò ('{k}')"));
    }
    if let Some(k) = VISUAL.iter().find(|k| lower.contains(*k)) {
        score += 10.0;
        bon.push(format!("Cộng điểm: Có từ ngữ gợi hình/thị giác ('{k}')"));
    }
    if t.contains('?') || t.contains('!') {
        score += 5.0;
        bon.push("Cộng điểm: Có dấu câu biểu cảm (! hoặc ?)".into());
    }
    json!({"total_score": r1(score.clamp(0.0, 100.0)), "word_count": wc, "estimated_duration_sec": dur, "has_greeting_penalty": greeting, "penalties": pen, "bonuses": bon})
}

pub fn generate_hooks(topic: &str, character: &str) -> Value {
    let t = topic.trim();
    let (a, b, c) = if character == "soc-nho" {
        (
            format!("Đố các bạn biết, bí mật đằng sau {t} thật ra là gì?"),
            format!("Đừng tin những gì bạn thấy! Sự thật về {t} sẽ làm bạn bất ngờ!"),
            format!("Bi vừa phát hiện ra {t} kỳ diệu lắm nè, xem ngay nhé!"),
        )
    } else {
        (
            format!("Thú thật là lúc đầu mình không tin {t} lại xịn thế này!"),
            format!("Đừng vội mua nếu bạn chưa biết bí mật này về {t}!"),
            format!("90% mọi người hiểu sai về {t}, xem ngay để không hối hận!"),
        )
    };
    let defs = [
        ("A", "paradox_question", a, "Khai thác tính tò mò và câu đố tương tác trực tiếp"),
        ("B", "confession_warning", b, "Cảnh báo đảo ngược định kiến (Pattern Interrupt)"),
        ("C", "first_person_discovery", c, "Trải nghiệm khám phá chân thực tự nhiên"),
    ];
    let variants: Vec<Value> = defs.iter().map(|(id, arch, text, why)| json!({"variant_id": id, "archetype": arch, "text": text, "reasoning": why, "score": score_hook(text)})).collect();
    // first max wins, like Python's max()
    let best = variants.iter().fold(&variants[0], |m, v| if v["score"]["total_score"].as_f64() > m["score"]["total_score"].as_f64() { v } else { m });
    json!({"topic": t, "best_variant_id": best["variant_id"], "selected_hook": best["text"], "variants": variants})
}

fn issue(level: &str, rule: &str, msg: String, tech: &str, sug: Option<&str>) -> Value {
    json!({"level": level, "rule_id": rule, "message_vi": msg, "technique_ref": tech, "suggestion_vi": sug})
}

pub fn lint_script(script: &str, duration_sec: Option<f64>, platform: &str) -> Value {
    let lines: Vec<&str> = script.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        let i = issue("error", "RULE_EMPTY_SCRIPT", "Kịch bản trống không có nội dung.".into(), "evergreen-shortform-foundations", None);
        return json!({"passed": false, "total_checks": 1, "error_count": 1, "warning_count": 0, "info_count": 0, "issues": [i]});
    }
    let mut issues = vec![];
    let hook = lines[0];
    let hwc = word_count(hook);
    let est = r2(hwc as f64 / 3.8);
    if let Some(m) = Regex::new(r"^(chào|hello|hi\b|hôm nay mình|xin chào)").unwrap().find(&hook.to_lowercase()) {
        issues.push(issue(
            "error",
            "RULE_NO_GREETINGS",
            format!("Hook mở đầu bằng lời chào xã giao ('{}'). Người xem sẽ lướt qua ngay trong 3 giây đầu.", m.as_str()),
            "confession-hook",
            Some("Vào thẳng vấn đề hoặc bí mật bất ngờ, bỏ hoàn toàn lời chào ở đầu video."),
        ));
    }
    if hwc > 16 || est > 3.5 {
        issues.push(issue(
            "warning",
            "RULE_HOOK_DURATION",
            format!("Hook quá dài ({hwc} từ, ~{est}s > 3.0s chuẩn)."),
            "pre-publish-hook-retention-prediction",
            Some("Gọt giũa hook xuống dưới 12 từ để truyền tải thông điệp trong 3 giây đầu."),
        ));
    }
    let concl = lines[lines.len().saturating_sub(2)..].join(" ").to_lowercase();
    let cta = [r"(hãy|nhớ|đừng quên)\s+chia sẻ", "bình luận", "comment", "bấm theo dõi", "follow", "đón xem", "đăng ký", "link", "ghim", "xem ngay", "bạn nghĩ sao", "để lại ý kiến"];
    if !cta.iter().any(|p| Regex::new(p).unwrap().is_match(&concl)) {
        issues.push(issue(
            "warning",
            "RULE_HAS_CTA",
            "Kịch bản chưa có lời kêu gọi hành động (CTA) rõ ràng ở phần kết.".into(),
            "3-layer-knowledge-retention",
            Some("Thêm câu hỏi kích thích tương tác hoặc điều hướng vào bình luận đầu tiên."),
        ));
    }
    if let Some(d) = duration_sec {
        if ["tiktok", "shorts", "reels"].contains(&platform) {
            if d > 60.0 {
                issues.push(issue(
                    "warning",
                    "RULE_PLATFORM_DURATION",
                    format!("Thời lượng video ({d}s) vượt quá mức tối ưu của video ngắn (15s - 50s)."),
                    "evergreen-shortform-foundations",
                    Some("Cân nhắc cô đọng nội dung hoặc cắt thành series tập."),
                ));
            } else if d < 5.0 {
                issues.push(issue("error", "RULE_TOO_SHORT", format!("Thời lượng video quá ngắn ({d}s < 5s)."), "evergreen-shortform-foundations", None));
            }
        }
    }
    let count = |l: &str| issues.iter().filter(|i| i["level"] == l).count();
    let (e, w, i) = (count("error"), count("warning"), count("info"));
    json!({"passed": e == 0, "total_checks": 5, "error_count": e, "warning_count": w, "info_count": i, "issues": issues})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn greeting_hook_is_penalised_and_linted() {
        assert!(score_hook("Xin chào các bạn hôm nay mình sẽ nói")["has_greeting_penalty"].as_bool().unwrap());
        let r = lint_script("Chào các bạn\nNội dung\nbình luận nhé", Some(30.0), "tiktok");
        assert_eq!(r["passed"], false);
        assert_eq!(generate_hooks("răng", "soc-nho")["variants"].as_array().unwrap().len(), 3);
    }
}
