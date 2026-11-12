//! Template-based KOC script generation + compliance cleaning + RenderRequest packaging.
use super::compliance::check_and_clean_script;
use super::contracts::*;
use super::products::Product;
use super::util::*;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Value};

const GREETING: &[&str] = &[r"^(xin\s+)?chào(\s+cả\s+nhà|\s+mọi\s+người|\s+các\s+bạn)?", r"^(hello|hi)(\s+everyone|\s+các\s+bạn)?", r"^hôm\s+nay\s+(mình|em|tôi|chúng\s+mình)(\s+sẽ|\s+chia\s+sẻ|\s+muốn|\s+làm)?", r"^ngày\s+mới\s+chúc"];
const SURPRISE: &[&str] = &["thật ra", "sự thật", "bí mật", "đừng tin", "không phải", "đố bạn", "nghịch lý", "ai ngờ", "chưa từng", "bất ngờ", "tại sao", "vì sao", "chỉ mất", "tiết lộ", "lần đầu", "cảnh báo", "sai lầm"];
const VISUAL: &[&str] = &["nhìn", "xem", "thấy", "màu", "cầu vồng", "lung linh", "nét", "đẹp", "ảnh", "vải", "mặt", "trời mưa", "ánh sáng", "chiếc đầm", "áo", "da"];
static GREET_RE: Lazy<Vec<Regex>> = Lazy::new(|| GREETING.iter().map(|p| Regex::new(p).unwrap()).collect());

/// Heuristic hook score (port of video/craft/hooks.py::score_hook): (score, words, est. seconds, penalties, bonuses).
pub fn score_hook(text: &str) -> (f64, usize, f64, Vec<String>, Vec<String>) {
    let t = text.trim();
    let low = t.to_lowercase();
    let wc = Regex::new(r"\w+").unwrap().find_iter(&low).count();
    let dur = (wc as f64 / 3.8 * 100.0).round() / 100.0;
    if wc == 0 { return (0.0, 0, 0.0, vec!["Hook rỗng".into()], vec![]); }
    let (mut base, mut pen, mut bon) = (60.0f64, vec![], vec![]);
    if GREET_RE.iter().any(|r| r.is_match(&low)) {
        base -= 35.0;
        pen.push("Trừ điểm: Mở đầu bằng lời chào xã giao làm giảm tỷ lệ giữ chân 3s".to_string());
    }
    if wc <= 12 { base += 15.0; bon.push(format!("Cộng điểm: Độ dài lý tưởng (≤12 từ: {wc} từ, ~{dur}s)")); }
    else if wc <= 16 { base += 5.0; bon.push(format!("Chấp nhận được: {wc} từ (~{dur}s)")); }
    else { base -= ((wc - 16) as f64 * 4.0).min(30.0); pen.push(format!("Trừ điểm: Hook quá dài ({wc} từ, ~{dur}s > 3.5s)")); }
    if let Some(k) = SURPRISE.iter().find(|k| low.contains(*k)) { base += 15.0; bon.push(format!("Cộng điểm: Chứa yếu tố bất ngờ/kích thích trí tò mò ('{k}')")); }
    if let Some(k) = VISUAL.iter().find(|k| low.contains(*k)) { base += 10.0; bon.push(format!("Cộng điểm: Có từ ngữ gợi hình/thị giác ('{k}')")); }
    if t.contains('?') || t.contains('!') { base += 5.0; bon.push("Cộng điểm: Có dấu câu biểu cảm (! hoặc ?)".into()); }
    (((base.clamp(0.0, 100.0)) * 10.0).round() / 10.0, wc, dur, pen, bon)
}

fn first_or(v: &[String], d: &str) -> String { v.first().cloned().unwrap_or_else(|| d.to_string()) }
fn line(role: &str, d: f64, text: String) -> Value { json!({"role": role, "duration_s": d, "text": text}) }

pub fn koc_hook_variants(p: &Product, ch: &str) -> Vec<Value> {
    let pain = first_or(&p.pain_points, "tìm kiếm sản phẩm ưng ý");
    let usp = first_or(&p.usp, "chất lượng vượt trội");
    let mut vs = vec![
        ("A", "confession", format!("Đừng mua {} nếu mọi người chưa nghe qua trải nghiệm thực tế này của {ch} nha!", p.name), "Đánh vào tâm lý sợ sai lầm (Confession Hook)"),
        ("B", "paradox_question", format!("Thật ra {ch} từng không tin sản phẩm giá {}{} lại có thể {usp}!", fmt_thousands(p.price), p.currency), "Tạo mâu thuẫn giữa giá cả và tính năng vượt trội (Paradox Hook)"),
        ("C", "problem_solution", format!("Ai đang đau đầu vì {pain} thì xem ngay bí quyết của {ch} với {}!", p.name), "Gọi tên trực tiếp nỗi đau và đưa ngay giải pháp (Direct Pain Hook)"),
    ].into_iter().map(|(id, arch, text, why)| {
        let (sc, _, dur, pen, bon) = score_hook(&text);
        json!({"variant_id": id, "archetype": arch, "text": text, "reasoning": why, "score": sc, "estimated_duration_sec": dur, "penalties": pen, "bonuses": bon})
    }).collect::<Vec<_>>();
    vs.sort_by(|a, b| b["score"].as_f64().partial_cmp(&a["score"].as_f64()).unwrap());
    vs
}

pub fn koc_review_lines(p: &Product, ch: &str) -> Vec<Value> {
    let usp = first_or(&p.usp, "chất lượng vượt mong đợi");
    let pain = first_or(&p.pain_points, "tìm kiếm sản phẩm ưng ý");
    let hook = koc_hook_variants(p, ch).first().map(|v| s(v, "text")).unwrap_or_default();
    vec![
        line("hook", 3.5, hook),
        line("problem", 4.0, format!("Trước đây {ch} cũng rất đau đầu vì {pain}, thử qua nhiều loại nhưng không ăn thua.")),
        line("demo", 6.0, format!("Nhưng từ lúc đổi sang {} của nhà {}, điểm ưng ý nhất là {usp}.", p.name, if p.brand.is_empty() { "này" } else { &p.brand })),
        line("proof", 5.0, format!("Tuy nhiên, có một điểm trừ nhỏ là sản phẩm quá hot nên thường xuyên hết màu đẹp. {ch} chấm em này 9/10 nha!")),
        line("cta", 4.0, format!("Giá đang được sale chỉ còn {}{}. Link chính hãng {ch} gắn ngay góc trái giỏ hàng nhé!", fmt_thousands(p.price), p.currency)),
    ]
}

pub fn tiktok_sales_lines(p: &Product) -> Vec<Value> {
    let usp = if p.usp.is_empty() { "công năng vượt trội".to_string() } else { p.usp.iter().take(2).cloned().collect::<Vec<_>>().join(", ") };
    let pain = first_or(&p.pain_points, "sự bất tiện hàng ngày");
    vec![
        line("hook", 3.0, format!("Ai đang gặp tình trạng {pain} thì xem ngay video này nhé!")),
        line("agitate", 4.0, format!("Chỉ với một thao tác đơn giản cùng {}, mọi rắc rối đều được giải quyết gọn gàng.", p.name)),
        line("demo", 6.0, format!("Cận cảnh chi tiết đây ạ: thiết kế thông minh, tích hợp {usp}.")),
        line("offer", 4.0, format!("Đặc biệt hôm nay trên livestream giá ưu đãi chỉ {} {}.", fmt_thousands(p.price), p.currency)),
        line("cta", 4.0, "Nhanh tay nhấn vào giỏ hàng màu vàng góc dưới màn hình để chốt deal ngay hôm nay!".into()),
    ]
}

pub fn affiliate_compare_lines(p: &Product) -> Vec<Value> {
    // Python passes a single-element list, so p1 == p2.
    let u = first_or(&p.usp, "sự nhỏ gọn");
    let u2 = first_or(&p.usp, "độ bền cao");
    vec![
        line("hook", 3.5, format!("So sánh thực tế: Nên chọn {0} hay {0} cho nhu cầu của bạn?", p.name)),
        line("compare", 7.0, format!("Về thiết kế và tính năng: {0} nổi bật với {u}, còn {0} lại tập trung vào {u2}.", p.name)),
        line("proof", 5.0, format!("Nếu ưu tiên ngân sách vừa phải thì {0} là lựa chọn số 1. Còn muốn cao cấp hơn thì chọn {0}.", p.name)),
        line("cta", 4.5, "Cả hai sản phẩm chính hãng mình đều đã gắn link trong giỏ hàng và phần bio cho mọi người dễ chọn nha!".into()),
    ]
}

pub struct ScriptOpts<'a> {
    pub generator_type: &'a str,
    pub blueprint: Option<&'a VideoBlueprint>,
    pub character_name: &'a str,
    pub voice_id: &'a str,
    pub platforms: Vec<String>,
}
impl Default for ScriptOpts<'_> {
    fn default() -> Self {
        Self { generator_type: "koc_review", blueprint: None, character_name: "LyLy", voice_id: "soc-nho", platforms: vec!["tiktok".into(), "reels".into(), "shorts".into()] }
    }
}

pub fn generate_complete_script(pid: &str, p: &Product, o: &ScriptOpts) -> anyhow::Result<Value> {
    let script_id = format!("script_{}", &uuid::Uuid::new_v4().simple().to_string()[..10]);
    let (raw, template_id) = match o.generator_type {
        "tiktok_sales" => (tiktok_sales_lines(p), "tiktok-sales-v1"),
        "affiliate_compare" => (affiliate_compare_lines(p), "affiliate-compare-v1"),
        _ => (koc_review_lines(p, o.character_name), "koc-review-v1"),
    };
    let (mut cleaned, mut warnings, mut chunks) = (vec![], vec![], vec![]);
    for l in &raw {
        let r = check_and_clean_script(&s(l, "text"), false);
        cleaned.push(json!({"role": l["role"], "duration_s": l["duration_s"], "text": r.modified_text}));
        warnings.extend(r.warnings);
        chunks.push(r.modified_text);
    }
    let full = check_and_clean_script(&chunks.join("\n"), true);
    warnings.extend(full.warnings.clone());
    write_json(&pp(pid, &["content", "scripts", &format!("{script_id}.json")], true), &cleaned)?;
    let idea = ContentIdea {
        id: format!("idea_{script_id}"), project_id: pid.into(), title: format!("Review {} ({})", p.name, o.generator_type),
        platform_targets: o.platforms.clone(), template_id: Some(template_id.into()), blueprint_ref: o.blueprint.map(|b| b.id.clone()), priority: 1, status: "scripted".into(),
    };
    save_content_idea(pid, &idea)?;
    let voice_script = cleaned.iter().map(|c| s(c, "text")).collect::<Vec<_>>().join("\n");
    let render_req = json!({
        "project_id": pid, "template_id": template_id, "platforms": o.platforms,
        "data": {"product_id": p.id, "product_name": p.name, "price": p.price, "currency": p.currency, "images": p.images,
                 "script_id": script_id, "generator_type": o.generator_type, "character_name": o.character_name},
        "voice": {"voice_id": o.voice_id, "script": voice_script, "audio_path": null},
        "source_ref": o.blueprint.map(|b| b.source_ref.clone()), "output_name": format!("prod_{}_{}", p.id, o.generator_type),
    });
    warnings.sort(); warnings.dedup();
    Ok(json!({
        "script_id": script_id, "content_idea_id": idea.id, "template_id": template_id, "lines": cleaned, "full_text": full.modified_text,
        "compliance_passed": full.passed, "compliance_warnings": warnings, "compliance_violations": full.violations,
        "hook_lab": koc_hook_variants(p, o.character_name), "render_request": render_req,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hook_score_and_lines() {
        let (greet, ..) = score_hook("Xin chào các bạn hôm nay");
        let (good, ..) = score_hook("Sự thật bất ngờ về chiếc áo này!");
        assert!(good > greet);
        let mut p = Product::new("p1", "Áo");
        p.price = 199000.0;
        let ls = koc_review_lines(&p, "LyLy");
        assert_eq!(ls.len(), 5);
        assert!(s(&ls[4], "text").contains("199,000VND"));
    }
}
