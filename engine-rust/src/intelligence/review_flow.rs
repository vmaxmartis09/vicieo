#![allow(non_snake_case)]
//! KOC review flow: style + product + character -> ready-to-run graph (graph.review-<styleId>.json).
use super::products::{get_product, Product};
use super::util::*;
use crate::{providers, workspace};
use serde::Deserialize;
use serde_json::{json, Value};

fn d_role() -> String { "scene".into() }
fn d_shot() -> String { "medium".into() }
fn d_cam() -> String { "static".into() }
fn d_arch() -> String { "unboxing_tryon".into() }
fn d_tone() -> String { "hào hứng, tự nhiên".into() }
fn d_cta() -> String { "Bấm giỏ hàng góc trái ngay".into() }
fn d_ar() -> String { "9:16".into() }
fn d_end() -> f64 { 3.0 }

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Item { #[serde(default = "d_role")] pub role: String, pub startSec: f64, #[serde(default = "d_end")] pub endSec: f64, #[serde(default = "d_shot")] pub shot: String, #[serde(default = "d_cam")] pub camera: String, pub onScreenText: String, pub voiceover: String }

#[derive(Debug, Clone, Deserialize)]
pub struct Style {
    pub styleId: String,
    #[serde(default = "d_arch")] pub archetype: String,
    #[serde(default)] pub structure: Vec<Item>,
    #[serde(default)] pub pacing: Value,
    #[serde(default = "d_tone")] pub tone: String,
    #[serde(default = "d_cta")] pub cta: String,
    #[serde(default = "d_ar")] pub aspectRatio: String,
    #[serde(default)] pub promptTemplate: Option<String>,
}

fn it(role: &str, a: f64, b: f64, shot: &str, cam: &str, on: &str, vo: &str) -> Value { json!({"role": role, "startSec": a, "endSec": b, "shot": shot, "camera": cam, "onScreenText": on, "voiceover": vo}) }

pub fn builtin_styles() -> Vec<Value> {
    vec![
        json!({"styleId": "koc_unboxing_tryon", "name": "Đập hộp & Thử đồ (Unboxing & Try-on)", "archetype": "unboxing_tryon",
            "hook": {"type": "visual_hook", "text": "Set đồ này đang hot rần rần trên TikTok, cùng mình kiểm chứng xem có đáng tiền không nhé!", "firstSeconds": 3.0},
            "structure": [
                it("hook", 0.0, 3.0, "close-up", "static, eye-level", "Có đáng tiền như lời đồn?", "Set đồ này đang hot rần rần trên TikTok, cùng mình kiểm chứng xem có đáng tiền không nhé!"),
                it("tryon", 3.0, 8.0, "full-body", "slow pan up, dynamic", "Lên dáng siêu tôn eo", "Thử lên người form cực chuẩn luôn, eo may chiết gọn gàng, tôn dáng dã man."),
                it("detail", 8.0, 13.0, "medium close-up", "slow zoom in, showcase fabric", "Chất vải đứng form · Đường may nét", "Chất vải đứng form, dày dặn vừa phải, sờ mịn tay và đường may rất chỉn chu."),
                it("cta", 13.0, 18.0, "medium shot", "eye-level, point down to cart", "Săn deal hời ở giỏ hàng 👇", "Giá đang được trợ giá cực tốt ngay góc trái màn hình, mọi người bấm giỏ hàng để săn nha!")],
            "pacing": "fast", "tone": "hào hứng, chân thật, tự nhiên", "cta": "Bấm vào giỏ hàng góc trái ngay", "aspectRatio": "9:16",
            "promptTemplate": "Full body front view of {character_name} wearing {product_name}, clean modern apartment background, natural soft lighting, photoreal, 8k"}),
        json!({"styleId": "koc_problem_solution", "name": "Giải pháp phối đồ (Problem & Solution)", "archetype": "problem_solution",
            "hook": {"type": "problem_pain", "text": "Ai bảo đi làm hay đi chơi là khó phối đồ? Xem ngay set này nhé!", "firstSeconds": 3.0},
            "structure": [
                it("hook", 0.0, 3.0, "medium close-up", "punch-in, dynamic", "Bí quyết mặc đẹp mỗi ngày", "Ai bảo đi làm hay đi chơi là khó phối đồ? Xem ngay set này nhé!"),
                it("tryon", 3.0, 8.0, "full-body", "rotating 360 showcase", "Mặc 1 set cân mọi dịp", "Chỉ cần diện set này lên là tự tin xuống phố, phối với sneaker hay cao gót đều đỉnh."),
                it("detail", 8.0, 13.0, "close-up", "macro pan", "Chi tiết thiết kế tinh tế", "Từng đường kim mũi chỉ đều rất tinh xảo, chất vải giữ form cả ngày không lo nhăn."),
                it("cta", 13.0, 18.0, "medium shot", "static, smile", "Ưu đãi có hạn ở góc trái", "Số lượng sale có hạn thôi, nhanh tay bấm vào giỏ hàng để rinh về nha!")],
            "pacing": "dynamic", "tone": "năng động, thuyết phục", "cta": "Mua ngay kẻo hết sale", "aspectRatio": "9:16",
            "promptTemplate": "Full body walking shot of {character_name} wearing {product_name}, bright natural daylight, photoreal"}),
        json!({"styleId": "koc_outfit_check", "name": "Biến hình & Đánh giá (Outfit Check)", "archetype": "outfit_check",
            "hook": {"type": "trend_transition", "text": "Biến hình 1 giây với set đồ thần thánh này cùng mình nha!", "firstSeconds": 2.5},
            "structure": [
                it("hook", 0.0, 3.0, "close-up", "quick snap zoom", "1, 2, 3... Biến hình!", "Biến hình 1 giây với set đồ thần thánh này cùng mình nha!"),
                it("tryon", 3.0, 9.0, "full-body", "dynamic tracking shot", "Outfit 10/10", "Ta da! Mặc lên vừa cá tính vừa thanh lịch, form dáng chuẩn không cần chỉnh."),
                it("detail", 9.0, 14.0, "medium shot", "pan waist to collar", "Chất liệu cao cấp", "Chất vải đứng phom, các điểm nhấn may tinh tế, mặc cực kỳ thoáng khí."),
                it("cta", 14.0, 18.0, "medium shot", "friendly smile, wave", "Link giỏ hàng bên dưới 👇", "Link sản phẩm chính hãng mình ghim ở góc trái, ghé xem liền nha!")],
            "pacing": "fast", "tone": "vui tươi, trendy", "cta": "Ghé giỏ hàng góc trái rinh ngay", "aspectRatio": "9:16",
            "promptTemplate": "Full body fashion editorial of {character_name} in {product_name}, high contrast lighting, photorealistic"}),
    ]
}

fn read_styles_in(dir: &std::path::Path, seen: &mut Vec<String>, out: &mut Vec<Value>) {
    let mut files: Vec<_> = std::fs::read_dir(dir).map(|rd| rd.flatten().map(|e| e.path()).collect()).unwrap_or_default();
    files.sort();
    for f in files {
        if f.extension().is_some_and(|e| e == "json") && f.file_name().is_some_and(|n| n != "index.json") {
            if let Some(d) = read_json(&f) {
                let sid = d.get("styleId").and_then(|v| v.as_str()).map(String::from).unwrap_or_else(|| f.file_stem().unwrap().to_string_lossy().to_string());
                if !seen.contains(&sid) { seen.push(sid); out.push(d); }
            }
        }
    }
}

pub fn list_koc_styles(pid: &str, product_id: Option<&str>) -> Vec<Value> {
    let (mut seen, mut out) = (vec![], vec![]);
    match product_id.filter(|p| safe_id(p).is_ok()) {
        Some(p) => read_styles_in(&pp(pid, &["products", p, "koc_styles", "x"], false).parent().unwrap().to_path_buf(), &mut seen, &mut out),
        None => {
            if let Ok(rd) = std::fs::read_dir(pp(pid, &["products"], false)) {
                let mut dirs: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
                dirs.sort();
                for d in dirs { read_styles_in(&d.join("koc_styles"), &mut seen, &mut out); }
            }
        }
    }
    read_styles_in(&pp(pid, &["koc_styles", "x"], false).parent().unwrap().to_path_buf(), &mut seen, &mut out);
    read_styles_in(&workspace::shared_path(&["koc_styles"], false).unwrap_or_default(), &mut seen, &mut out);
    for b in builtin_styles() { let sid = s(&b, "styleId"); if !seen.contains(&sid) { seen.push(sid); out.push(b); } }
    out
}

pub fn load_koc_style(pid: &str, style_id: &str, inline: Option<&Value>, product_id: Option<&str>) -> ApiResult<Style> {
    let parse = |v: Value| serde_json::from_value::<Style>(v).map_err(|e| ApiError::bad(format!("Style không hợp lệ: {e}")));
    if let Some(v) = inline.filter(|v| v.is_object()) { return parse(v.clone()); }
    safe_id(style_id).map_err(|e| ApiError::bad(e.to_string()))?;
    let fname = format!("{style_id}.json");
    let mut cands = vec![];
    if let Some(p) = product_id.filter(|p| safe_id(p).is_ok()) { cands.push(pp(pid, &["products", p, "koc_styles", &fname], false)); }
    if let Ok(rd) = std::fs::read_dir(pp(pid, &["products"], false)) {
        let mut dirs: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        dirs.sort();
        cands.extend(dirs.into_iter().map(|d| d.join("koc_styles").join(&fname)));
    }
    cands.push(pp(pid, &["koc_styles", &fname], false));
    cands.push(workspace::shared_path(&["koc_styles", &fname], false).unwrap_or_default());
    for c in cands { if let Some(v) = read_json(&c) { return parse(v); } }
    if let Some(b) = builtin_styles().into_iter().find(|b| s(b, "styleId") == style_id) { return parse(b); }
    if let Some(v) = read_json(&workspace::data_root().join("koc_styles").join(&fname)) { return parse(v); }
    Err(ApiError::nf(format!("Không tìm thấy phong cách KOC với styleId='{style_id}'")))
}

fn money(v: Option<f64>, cur: &str) -> String {
    match v.filter(|v| *v > 0.0) {
        None => "giá ưu đãi".into(),
        Some(v) => { let f = fmt_thousands(v).replace(',', "."); if cur == "VND" { format!("{f}₫") } else { format!("{f} {cur}") } }
    }
}
fn price_of(p: &Product) -> String { money(p.sale_price.or(Some(p.price)).filter(|v| *v != 0.0).or(Some(p.price)), &p.currency) }
fn usp_of(p: &Product) -> String { if p.usp.is_empty() { p.name.clone() } else { p.usp.iter().take(3).cloned().collect::<Vec<_>>().join(" · ") } }

pub async fn rewrite_voiceovers(style: &Style, p: &Product, character: &str, use_llm: bool) -> Vec<String> {
    let price = price_of(p);
    let usp = usp_of(p);
    let fallback: Vec<String> = style.structure.iter().map(|sc| {
        let sub = sc.voiceover.replace("{product_name}", &p.name).replace("{brand}", &p.brand).replace("{price}", &price).replace("{sale_price}", &price)
            .replace("{usp}", &usp).replace("{character_name}", character).replace("{cta}", if style.cta.is_empty() { "Bấm giỏ hàng góc trái" } else { &style.cta });
        if sub.trim().is_empty() { sc.voiceover.clone() } else { sub }
    }).collect();
    if !use_llm || style.structure.is_empty() { return fallback; }
    let mut prompt = format!("Sản phẩm cần review:\n- Tên: {}\n- Thương hiệu: {}\n- Giá bán ưu đãi: {price}\n- Điểm nổi bật (USP): {usp}\n- Điểm lưu ý: {}\n- Nhân vật KOC: {character}\n- Tông giọng: {}\n- Kêu gọi hành động (CTA): {}\n\nCấu trúc các cảnh mẫu ({} cảnh):\n",
        p.name, if p.brand.is_empty() { "Chính hãng" } else { &p.brand }, p.pain_points.first().map(String::as_str).unwrap_or("Không có"), style.tone, style.cta, style.structure.len());
    for (i, sc) in style.structure.iter().enumerate() { prompt += &format!("Cảnh {} ({}, {}s - {}s): Mẫu = \"{}\"\n", i + 1, sc.role, sc.startSec, sc.endSec, sc.voiceover); }
    prompt += &format!("\nHãy trả về JSON mảng đúng {} câu thoại tiếng Việt tự nhiên cho từng cảnh.", style.structure.len());
    let sys = "Bạn là chuyên gia biên kịch KOC review video ngắn TikTok / Reels. Viết lại lời thoại cho từng cảnh theo đúng nhịp điệu, nói đúng tên sản phẩm, ưu điểm và giá. BẮT BUỘC chỉ trả về JSON thuần là một mảng chuỗi đúng số lượng phân cảnh.";
    let _ = providers::llm(); // provider chosen inside ask_json
    match ask_json(sys, &prompt).await.and_then(|v| v.as_array().cloned()) {
        Some(a) if a.len() == style.structure.len() => a.iter().map(|x| x.as_str().unwrap_or("").trim().to_string()).collect(),
        _ => fallback,
    }
}

pub fn build_video_prompt(style: &Style, p: &Product, character: &str, vos: &[String]) -> String {
    let price = price_of(p);
    let pacing = match &style.pacing {
        Value::Object(o) => format!("{} cuts ({}s/shot)", o.get("cuts").and_then(|v| v.as_i64()).unwrap_or(0), o.get("avgShotSec").and_then(|v| v.as_f64()).unwrap_or(2.5)),
        Value::String(s) if !s.is_empty() => s.clone(),
        _ => "nhanh, dứt khoát".into(),
    };
    let mut l = vec![
        format!("[KỊCH BẢN KOC REVIEW - PHONG CÁCH: {}]", style.archetype.to_uppercase()), format!("Sản phẩm: {}", p.name),
        format!("Giá bán: {price} (Thương hiệu: {})", if p.brand.is_empty() { "Chính hãng" } else { &p.brand }), format!("Điểm nổi bật: {}", usp_of(p)),
        format!("Nhân vật thực hiện: {character}"), format!("Nhịp điệu: {pacing} | Tông giọng: {}", style.tone), String::new(), "CẤU TRÚC PHÂN CẢNH VÀ CHUYỂN ĐỘNG:".into(),
    ];
    for (i, sc) in style.structure.iter().enumerate() {
        let vo = vos.get(i).cloned().unwrap_or_else(|| sc.voiceover.clone());
        l.push(format!("--- Cảnh {} ({} | {:.1}s - {:.1}s) ---", i + 1, sc.role.to_uppercase(), sc.startSec, sc.endSec));
        l.push(format!("• Góc quay & Camera: {}, chuyển động {}", sc.shot, sc.camera));
        if !sc.onScreenText.is_empty() { l.push(format!("• Text hiển thị: {}", sc.onScreenText.replace("{price}", &price).replace("{product_name}", &p.name))); }
        l.push(format!("• Lời thoại KOC: \"{vo}\""));
        l.push(format!("• Chuyển động: {character} mặc trang phục {}, diễn xuất tự nhiên tương tác với ống kính.", p.name));
        l.push(String::new());
    }
    l.push(format!("Kêu gọi hành động (CTA): {}", style.cta));
    l.join("\n").trim().to_string()
}

fn node(id: &str, ty: &str, x: f64, y: f64, params: Value, hint: String) -> Value {
    json!({"id": id, "type": ty, "params": params, "inputs": {}, "position": {"x": x, "y": y}, "ui": {"hint": hint}, "state": "idle", "isStale": false, "pinnedGenerationId": null, "generations": []})
}

#[derive(Default)]
pub struct FlowOpts { pub character_id: Option<String>, pub style_data: Option<Value>, pub video_model: String, pub image_model: String, pub resolution: String, pub variants: i64, pub use_llm: bool }

pub async fn build_review_flow_graph(pid: &str, product_id: &str, style_id: &str, o: &FlowOpts) -> ApiResult<(Value, Value)> {
    let style = load_koc_style(pid, style_id, o.style_data.as_ref(), Some(product_id))?;
    let p = get_product(pid, product_id).ok_or_else(|| ApiError::bad(format!("Không tìm thấy sản phẩm '{product_id}' trong dự án '{pid}'")))?;
    let cfg = read_json(&pp(pid, &["project.json"], false)).unwrap_or(json!({}));
    let chars = cfg.get("characters").and_then(|c| c.as_object()).cloned().unwrap_or_default();
    let char_id = o.character_id.clone().filter(|c| !c.is_empty())
        .or_else(|| cfg.get("default_character").and_then(|v| v.as_str()).map(String::from)).unwrap_or_else(|| chars.keys().next().cloned().unwrap_or_else(|| "gobi".into()));
    let info = chars.get(&char_id).cloned().unwrap_or(json!({}));
    let cname = info.get("name").and_then(|v| v.as_str()).unwrap_or(&char_id).to_string();
    let voice = info.get("voice_id").and_then(|v| v.as_str()).filter(|v| !v.is_empty()).map(String::from)
        .or_else(|| cfg.get("default_voice_id").and_then(|v| v.as_str()).filter(|v| !v.is_empty()).map(String::from))
        .unwrap_or_else(|| if pid == "gobi" { "koc-lyly".into() } else { "soc-nho".into() });
    let lock = safe_id(&char_id).ok().and_then(|c| read_json(&pp(pid, &["characters", c, "studio.json"], false))).map(|d| s(&d, "promptLock")).unwrap_or_default();

    let vos = rewrite_voiceovers(&style, &p, &cname, o.use_llm).await;
    let full = vos.iter().filter(|v| !v.trim().is_empty()).cloned().collect::<Vec<_>>().join(" ");
    let dur = if style.structure.is_empty() { 15.0 } else { style.structure.iter().map(|s| s.endSec).fold(15.0f64.min(0.0), f64::max).max(5.0) };
    let video_prompt = build_video_prompt(&style, &p, &cname, &vos);
    let price = price_of(&p);
    let tryon = match style.promptTemplate.as_deref().filter(|t| !t.is_empty()) {
        Some(t) => t.replace("{character_name}", &cname).replace("{character}", &cname).replace("{product_name}", &p.name).replace("{product}", &p.name).replace("{price}", &price)
            .replace("{sale_price}", &price).replace("{brand}", &p.brand).replace("{camera}", style.structure.first().map(|s| s.camera.as_str()).unwrap_or("static")),
        None => format!("Full body front and back view of {cname} wearing {} in modern apartment living room. Keep face, hair, body identical to character reference{}, photoreal, 8k", p.name, if lock.is_empty() { String::new() } else { format!(", {lock}") }),
    };
    let cutout = p.cutout_images.first().cloned().or_else(|| p.images.iter().find(|i| i.to_lowercase().contains("cutout")).or(p.images.first()).cloned()).unwrap_or_default();
    let ar = if style.aspectRatio.is_empty() { "9:16".to_string() } else { style.aspectRatio.clone() };
    let name35: String = p.name.chars().take(35).collect();
    let name25: String = p.name.chars().take(25).collect();
    let rating = match p.rating.filter(|r| *r != 0.0) { Some(r) => format!("⭐ {r} · {} đánh giá", p.review_count), None => "⭐ 5.0 · Đánh giá cao".into() };
    let cons = p.pain_points.first().cloned().unwrap_or_else(|| p.compliance_notes.clone());
    let or_def = |v: &str, d: &str| if v.is_empty() { d.to_string() } else { v.to_string() };

    let nodes = vec![
        node("n_char", "character", 40.0, 40.0, json!({"characterId": char_id, "voiceId": voice, "_customName": format!("{cname} (giọng {voice})"), "lines": full,
            "speak": {"enabled": true, "text": full, "lines": vos, "voiceId": voice}}), format!("Nhân vật {cname} & giọng {voice}")),
        node("n_product", "product", 40.0, 420.0, json!({"productId": p.id, "variant": cutout, "_customName": format!("Sản phẩm: {name35}")}), format!("Ảnh cutout & thông tin {name25}")),
        node("n_tryon", "image_gen", 450.0, 200.0, json!({"model": or_def(&o.image_model, "mock-image"), "aspectRatio": ar, "resolution": "1080p", "prompt": tryon, "_customName": format!("{cname} mặc {name25}")}), "Ghép trang phục lên nhân vật".into()),
        node("n_video", "video_gen", 950.0, 200.0, json!({"model": or_def(&o.video_model, "mock-video"), "variants": o.variants.max(1), "aspectRatio": ar, "resolution": or_def(&o.resolution, "1080p"),
            "durationSec": dur, "hasAudio": false, "prompt": video_prompt, "_customName": "Tạo video chuyển động"}), format!("Video {dur:.0}s kèm kịch bản style {}", style.archetype)),
        node("n_render", "template_render", 1480.0, 200.0, json!({"template_id": "koc-review-v1", "platforms": ["tiktok"],
            "slot_mapping": {"reviewer_badge": cname, "pros_text": usp_of(&p), "cons_text": cons, "rating_score": rating}, "_customName": "Dựng KOC review"}), "Dựng bố cục HyperFrames chuẩn KOC TikTok".into()),
        node("n_export", "platform_export", 1960.0, 200.0, json!({"target_platforms": ["tiktok"], "fit_strategy": "cover", "_customName": "Xuất TikTok 9:16"}), "Xuất video MP4 9:16 sẵn đăng".into()),
    ];
    let wires = [("n_char", "reference_images", "n_tryon", "references"), ("n_product", "image", "n_tryon", "references"), ("n_tryon", "image", "n_video", "first_frame"),
        ("n_video", "video", "n_render", "video"), ("n_char", "voice", "n_render", "voice_clips"), ("n_char", "reference_images", "n_render", "reference_images"),
        ("n_product", "product", "n_render", "product"), ("n_product", "image", "n_render", "image"), ("n_render", "video", "n_export", "video")];
    let edges: Vec<Value> = wires.iter().map(|(a, b, c, d)| json!({"id": format!("e_{a}_{b}__{c}_{d}"), "sourceNodeId": a, "sourcePort": b, "targetNodeId": c, "targetPort": d})).collect();
    let graph = json!({"id": format!("graph_review_{}_{}", style.styleId, p.id), "projectId": pid, "name": format!("Review Flow: {name25} ({})", style.archetype),
        "nodes": nodes, "edges": edges, "updatedAt": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(), "schemaVersion": 1, "variables": {}});
    Ok((graph, json!({"styleId": style.styleId, "productId": p.id, "characterId": char_id, "characterName": cname, "voiceovers": vos, "fullVoiceover": full, "durationSec": dur})))
}

pub async fn generate_and_save_review_flow(pid: &str, product_id: &str, style_id: &str, o: &FlowOpts) -> ApiResult<(Value, String)> {
    let (g, _) = build_review_flow_graph(pid, product_id, style_id, o).await?;
    let name = format!("graph.review-{}.json", safe_id(style_id).map_err(|e| ApiError::bad(e.to_string()))?);
    std::fs::write(pp(pid, &[&name], true), serde_json::to_string_pretty(&g).map_err(|e| ApiError::ise(e.to_string()))? + "\n").map_err(|e| ApiError::ise(e.to_string()))?;
    Ok((g, name))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builtin_and_prompt() {
        let st = load_koc_style("soc-nho-tv", "koc_outfit_check", None, None).unwrap();
        assert_eq!(st.structure.len(), 4);
        let mut p = Product::new("p", "Áo khoác");
        p.price = 199000.0;
        let pr = build_video_prompt(&st, &p, "LyLy", &[]);
        assert!(pr.contains("OUTFIT_CHECK") && pr.contains("Cảnh 4"));
        assert_eq!(money(Some(199000.0), "VND"), "199.000₫");
        assert!(load_koc_style("soc-nho-tv", "nope_x", None, None).is_err());
    }
}
