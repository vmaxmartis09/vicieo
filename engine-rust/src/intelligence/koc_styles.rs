#![allow(non_snake_case)]
//! KOC review-style analysis: video discovery (manual URLs / ego-browser), analysis (Gemini for YouTube, else yt-dlp+ffmpeg+LLM).
use super::products::get_product;
use super::util::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

fn d_hook_type() -> String { "curiosity".into() }
fn d_role() -> String { "hook".into() }
fn d_shot() -> String { "medium-shot".into() }
fn d_cam() -> String { "handheld".into() }
fn d_arch() -> String { "honest-review".into() }
fn d_tone() -> String { "casual".into() }
fn d_ar() -> String { "9:16".into() }
fn d_avg() -> f64 { 2.5 }
fn d_three() -> f64 { 3.0 }
fn d_conf() -> f64 { 0.8 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Source { pub videoUrl: String, pub creator: String, pub views: i64, pub likes: i64, pub durationSec: f64, pub collectedAt: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Hook { #[serde(default = "d_hook_type")] pub r#type: String, pub text: String, #[serde(default = "d_three")] pub firstSeconds: f64 }
impl Default for Hook { fn default() -> Self { Self { r#type: d_hook_type(), text: String::new(), firstSeconds: 3.0 } } }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Beat { #[serde(default = "d_role")] pub role: String, pub startSec: f64, pub endSec: f64, #[serde(default = "d_shot")] pub shot: String, #[serde(default = "d_cam")] pub camera: String, pub onScreenText: String, pub voiceover: String }
impl Default for Beat { fn default() -> Self { Self { role: d_role(), startSec: 0.0, endSec: 0.0, shot: d_shot(), camera: d_cam(), onScreenText: String::new(), voiceover: String::new() } } }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Pacing { #[serde(default = "d_avg")] pub avgShotSec: f64, pub cuts: i64 }
impl Default for Pacing { fn default() -> Self { Self { avgShotSec: 2.5, cuts: 0 } } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KocStyle {
    pub styleId: String,
    pub productId: String,
    #[serde(default)] pub source: Source,
    #[serde(default = "d_arch")] pub archetype: String,
    #[serde(default)] pub hook: Hook,
    #[serde(default)] pub structure: Vec<Beat>,
    #[serde(default)] pub pacing: Pacing,
    #[serde(default = "d_tone")] pub tone: String,
    #[serde(default)] pub cta: String,
    #[serde(default = "d_ar")] pub aspectRatio: String,
    #[serde(default)] pub music: String,
    #[serde(default)] pub promptTemplate: String,
    #[serde(default = "d_conf")] pub confidence: f64,
}

/// Prompt template must carry {product} {character} {price}; append the missing ones (as the Python validator does).
fn fix_prompt(v: &str) -> String {
    let v = v.trim();
    let missing: Vec<&str> = ["{product}", "{character}", "{price}"].into_iter().filter(|p| !v.contains(p)).collect();
    if missing.is_empty() { return v.into(); }
    let filler = missing.join(" ");
    if v.is_empty() { format!("Video review KOC cho {filler}") } else { format!("{v} - Thông tin: {filler}").trim_matches(|c| c == ' ' || c == '-').to_string() }
}

fn dir(pid: &str, prod: &str, ensure: bool) -> PathBuf { pp(pid, &["products", prod, "koc_styles", "index.json"], ensure).parent().map(|p| p.to_path_buf()).unwrap_or_default() }

pub fn list_koc_styles(pid: &str, prod: &str) -> Vec<KocStyle> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir(pid, prod, false)).map(|rd| rd.flatten().map(|e| e.path()).collect()).unwrap_or_default();
    files.sort();
    files.into_iter().filter(|p| p.extension().is_some_and(|e| e == "json") && p.file_name().is_some_and(|n| n != "index.json"))
        .filter_map(|p| serde_json::from_value::<KocStyle>(read_json(&p)?).ok()).collect()
}

pub fn load_koc_styles_index(pid: &str, prod: &str) -> Value {
    read_json(&dir(pid, prod, false).join("index.json")).unwrap_or_else(|| json!({"productId": prod, "updatedAt": "", "totalStyles": 0, "styles": []}))
}

pub fn update_index(pid: &str, prod: &str) -> anyhow::Result<Value> {
    let styles = list_koc_styles(pid, prod);
    let idx = json!({"productId": prod, "updatedAt": now_iso(), "totalStyles": styles.len(), "styles": styles.iter().map(|s| json!({
        "styleId": s.styleId, "archetype": s.archetype, "hookText": s.hook.text, "creator": s.source.creator, "videoUrl": s.source.videoUrl,
        "durationSec": s.source.durationSec, "views": s.source.views, "likes": s.source.likes, "confidence": s.confidence, "collectedAt": s.source.collectedAt})).collect::<Vec<_>>()});
    write_json(&dir(pid, prod, true).join("index.json"), &idx)?;
    Ok(idx)
}

pub fn save_koc_style(pid: &str, st: &KocStyle) -> anyhow::Result<()> {
    safe_id(&st.styleId)?;
    write_json(&dir(pid, safe_id(&st.productId)?, true).join(format!("{}.json", st.styleId)), st)?;
    update_index(pid, &st.productId)?;
    Ok(())
}

pub fn get_koc_style(pid: &str, prod: &str, id: &str) -> Option<KocStyle> {
    safe_id(prod).ok()?; safe_id(id).ok()?;
    serde_json::from_value(read_json(&dir(pid, prod, false).join(format!("{id}.json")))?).ok()
}

const CAPTCHA_JS: &str = r#"
const maxVideos = __MAX__; const productId = __PID__; const productName = __NAME__; const shopName = __SHOP__;
const isCaptcha = (h, t) => { const l = ((h||"")+" "+(t||"")).toLowerCase(); return ["security check","secsdk-captcha","verify to continue","xác minh để tiếp tục","trượt để hoàn thành","drag the slider","captcha_container","verify-bar"].some(k => l.includes(k)); };
const links = () => Array.from(new Set(Array.from(document.querySelectorAll('a[href*="/video/"]')).map(a => a.href)));
try {
  const task = await taskSpace("koc_search_" + productId); const page = task.page("p1");
  await page.goto(`https://www.tiktok.com/view/product/${productId}`, { timeout: 15000 }); await page.waitForTimeout(2500);
  let text = await page.evaluate(() => document.body ? document.body.innerText : ""), html = await page.evaluate(() => document.body ? document.body.innerHTML : "");
  if (isCaptcha(html, text)) { console.log(JSON.stringify({status:"captcha_needed", message:"TikTok yêu cầu Security Check / Captcha khi mở trang sản phẩm. Trình duyệt được giữ mở trên màn hình để bạn tự giải.", urls:[], browser_kept_open:true})); process.exit(0); }
  let urls = await page.evaluate(links);
  if (!urls || !urls.length) {
    const q = [shopName, productName].filter(Boolean).join(" ").trim() || productName;
    await page.goto(`https://www.tiktok.com/search?q=${encodeURIComponent(q)}`, { timeout: 15000 }); await page.waitForTimeout(3000);
    text = await page.evaluate(() => document.body ? document.body.innerText : ""); html = await page.evaluate(() => document.body ? document.body.innerHTML : "");
    if (isCaptcha(html, text)) { console.log(JSON.stringify({status:"captcha_needed", message:"TikTok yêu cầu Security Check / Captcha khi tìm kiếm. Trình duyệt được giữ mở trên màn hình để bạn tự giải.", urls:[], browser_kept_open:true})); process.exit(0); }
    urls = await page.evaluate(links);
  }
  const sel = (urls || []).slice(0, maxVideos); await task.finish({ keep: [] });
  console.log(JSON.stringify({status: sel.length ? "success" : "no_videos_found", urls: sel, message: sel.length ? `Tìm thấy ${sel.length} video KOC trên TikTok.` : "Không tìm thấy video review nào trên TikTok.", browser_kept_open:false}));
} catch (err) { console.log(JSON.stringify({status:"error", urls:[], message:"Lỗi thực thi ego-browser: " + (err && err.message ? err.message : String(err)), browser_kept_open:false})); }
"#;

pub async fn find_koc_video_urls(pid: &str, prod_id: &str, manual: &[String], max: usize) -> Value {
    let clean: Vec<&String> = manual.iter().filter(|u| u.trim().starts_with("http://") || u.trim().starts_with("https://")).collect();
    if !clean.is_empty() {
        let urls: Vec<String> = clean.into_iter().take(max).map(|u| u.trim().to_string()).collect();
        return json!({"status": "success", "message": format!("Sử dụng {} video từ danh sách nhập tay.", urls.len()), "urls": urls, "browser_kept_open": false});
    }
    let prod = get_product(pid, prod_id);
    let name = prod.as_ref().map(|p| p.name.clone()).unwrap_or_else(|| format!("Sản phẩm {prod_id}"));
    let shop = prod.map(|p| p.shop_name).unwrap_or_default();
    let missing = json!({"status": "ego_browser_missing", "urls": [], "message": "Không tìm thấy ego-browser trên máy. Vui lòng nhập tay danh sách URL video KOC.", "browser_kept_open": false});
    let script = CAPTCHA_JS.replace("__MAX__", &max.to_string()).replace("__PID__", &json!(prod_id).to_string()).replace("__NAME__", &json!(name).to_string()).replace("__SHOP__", &json!(shop).to_string());
    let Ok(mut child) = Command::new("ego-browser").arg("nodejs").stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).kill_on_drop(true).spawn() else { return missing };
    if let Some(mut si) = child.stdin.take() { let _ = si.write_all(script.as_bytes()).await; }
    match tokio::time::timeout(Duration::from_secs(40), child.wait_with_output()).await {
        Err(_) => json!({"status": "timeout", "urls": [], "message": "Quá thời gian chờ khi mở TikTok bằng ego-browser. Hãy kiểm tra trình duyệt hoặc nhập URL bằng tay.", "browser_kept_open": true}),
        Ok(Err(e)) => json!({"status": "error", "urls": [], "message": format!("Lỗi gọi ego-browser: {e}"), "browser_kept_open": false}),
        Ok(Ok(o)) => {
            let text = format!("{}\n{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
            text.lines().rev().map(str::trim).filter(|l| l.starts_with('{') && l.ends_with('}')).find_map(|l| serde_json::from_str::<Value>(l).ok())
                .unwrap_or_else(|| json!({"status": "error", "urls": [], "message": format!("ego-browser không trả kết quả JSON hợp lệ: {}", text.trim().chars().take(200).collect::<String>()), "browser_kept_open": false}))
        }
    }
}

pub async fn fetch_video_metadata(url: &str) -> Value {
    let fallback = |c: &str| json!({"videoUrl": url, "creator": c, "views": 0, "likes": 0, "durationSec": 30.0});
    let out = tokio::time::timeout(Duration::from_secs(25), Command::new("yt-dlp").args(["--dump-json", "--no-playlist", url]).output()).await;
    let Ok(Ok(o)) = out else { return fallback("unknown") };
    let Some(raw) = String::from_utf8_lossy(&o.stdout).lines().next().and_then(|l| serde_json::from_str::<Value>(l).ok()).filter(|_| o.status.success()) else { return fallback("koc_creator") };
    let g = |k: &str| raw.get(k).and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(String::from);
    json!({"videoUrl": url, "creator": g("uploader").or(g("channel")).or(g("creator")).unwrap_or("creator".into()),
           "views": raw["view_count"].as_i64().unwrap_or(0), "likes": raw["like_count"].as_i64().unwrap_or(0),
           "durationSec": raw["duration"].as_f64().unwrap_or(30.0), "title": g("title").unwrap_or_default(), "description": g("description").unwrap_or_default()})
}

fn gemini_key() -> Option<String> {
    if let Ok(k) = std::env::var("GEMINI_API_KEY") { if !k.trim().is_empty() { return Some(k.trim().into()); } }
    let f = dirs_home()?.join(".config/watch/.env");
    std::fs::read_to_string(f).ok()?.lines().find_map(|l| l.trim().strip_prefix("GEMINI_API_KEY=").map(|v| v.trim().trim_matches(|c| c == '"' || c == '\'').to_string())).filter(|v| !v.is_empty())
}
fn dirs_home() -> Option<PathBuf> { std::env::var("HOME").ok().map(PathBuf::from) }

const SCHEMA_PROMPT: &str = r#"BẮT BUỘC chỉ trả về DUY NHẤT một JSON hợp lệ:
{"archetype":"try-on|unboxing|before-after|honest-review|storytime|other","hook":{"type":"câu hỏi|vấn đề|giật gân|tò mò|hình ảnh","text":"câu hook mở đầu","firstSeconds":3.0},
"structure":[{"role":"hook|problem|demo|proof|cta","startSec":0.0,"endSec":3.0,"shot":"close-up|medium-shot|b-roll","camera":"static|handheld|panning|zoom-in","onScreenText":"","voiceover":""}],
"pacing":{"avgShotSec":2.5,"cuts":6},"tone":"","cta":"","aspectRatio":"9:16","music":"","promptTemplate":"prompt có {product} {character} {price}","confidence":0.9}"#;

/// Gemini reads YouTube URLs natively (fileData); other hosts fall through to the local engine.
async fn analyze_with_gemini(url: &str, product: &str) -> Option<Value> {
    if !(url.contains("youtube.com") || url.contains("youtu.be")) { return None; }
    let key = gemini_key()?;
    let model = llm_model();
    let prompt = format!("Bạn là chuyên gia phân tích kịch bản KOC review TikTok. Xem video review sản phẩm \"{product}\" và phân tích phong cách review.\n{SCHEMA_PROMPT}");
    let body = json!({"contents": [{"parts": [{"fileData": {"fileUri": url}}, {"text": prompt}]}], "generationConfig": {"responseMimeType": "application/json"}});
    let r: Value = reqwest::Client::builder().timeout(Duration::from_secs(120)).build().ok()?
        .post(format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent")).header("x-goog-api-key", key).json(&body).send().await.ok()?.json().await.ok()?;
    let t = r["candidates"][0]["content"]["parts"][0]["text"].as_str()?;
    let t = t.trim().trim_start_matches("```json").trim_end_matches("```").trim();
    serde_json::from_str(t).ok()
}

pub fn parse_vtt_lines(vtt: &str) -> Vec<Value> {
    crate::intelligence::sources::asr::parse_vtt(vtt).into_iter().map(|v| json!({"start": v["start"], "end": v["end"], "text": v["text"]})).collect()
}

async fn fetch_subs(url: &str) -> Vec<Value> {
    let tmp = std::env::temp_dir().join(format!("koc-subs-{}", uuid::Uuid::new_v4().simple()));
    let _ = std::fs::create_dir_all(&tmp);
    let _ = tokio::time::timeout(Duration::from_secs(20), Command::new("yt-dlp").args(["--write-auto-sub", "--write-sub", "--sub-lang", "vi.*,en.*", "--skip-download", "-o", &tmp.join("video.%(ext)s").to_string_lossy(), url]).output()).await;
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir(&tmp) {
        for e in rd.flatten() {
            if e.path().extension().is_some_and(|x| x == "vtt") {
                out = parse_vtt_lines(&std::fs::read_to_string(e.path()).unwrap_or_default());
                if !out.is_empty() { break; }
            }
        }
    }
    let _ = std::fs::remove_dir_all(&tmp);
    out
}

fn r1(x: f64) -> f64 { (x * 10.0).round() / 10.0 }

/// Deterministic analysis from metadata + subtitles (port of analyze_with_local_engine).
pub fn analyze_local(product: &str, meta: &Value, transcript: &[Value]) -> Value {
    let duration = meta["durationSec"].as_f64().unwrap_or(30.0);
    let title = s(meta, "title");
    let cuts = ((duration / 2.8) as i64).max(1);
    let avg = ((duration / (cuts + 1).max(1) as f64) * 100.0).round() / 100.0;
    let tr_text: Vec<String> = transcript.iter().map(|t| s(t, "text")).collect();
    let low = format!("{} {} {}", title, s(meta, "description"), tr_text.join(" ")).to_lowercase();
    let has = |ks: &[&str]| ks.iter().any(|k| low.contains(k));
    let archetype = if has(&["thử đồ", "mặc", "outfit", "set", "phối đồ", "lên dáng", "size", "kaki", "quần", "áo", "váy"]) { "try-on" }
        else if has(&["đập hộp", "mở hộp", "unboxing", "bóc hàng", "gói hàng", "khui"]) { "unboxing" }
        else if has(&["trước sau", "before after", "sau khi", "hồi trước", "bây giờ"]) { "before-after" }
        else if has(&["kể", "chuyện là", "hôm nay", "đi chơi", "hẹn hò", "tâm sự"]) { "storytime" } else { "honest-review" };
    let hook_sec = 3.5f64.min(2.0f64.max(duration * 0.2));
    let hook_text = { let t = transcript.iter().filter(|t| t["start"].as_f64().unwrap_or(0.0) <= hook_sec).map(|t| s(t, "text")).collect::<Vec<_>>().join(" ").trim().to_string();
        if !t.is_empty() { t } else if !title.is_empty() { title.clone() } else { format!("Trải nghiệm thực tế {product}") } };
    let hl = hook_text.to_lowercase();
    let hook_type = if hook_text.contains('?') { "câu hỏi" } else if ["đừng", "sai lầm", "cảnh báo", "mụn", "xấu", "lỗi"].iter().any(|k| hl.contains(k)) { "vấn đề" }
        else if ["sốc", "kinh", "bất ngờ", "không ngờ", "ảo"].iter().any(|k| hl.contains(k)) { "giật gân" } else if archetype == "try-on" { "thử đồ trực tiếp" } else { "tò mò" };
    let roles: &[&str] = if duration >= 25.0 { &["hook", "problem", "demo", "proof", "cta"] } else { &["hook", "problem", "demo", "cta"] };
    let step = duration / roles.len() as f64;
    let mut structure = vec![];
    for (i, role) in roles.iter().enumerate() {
        let st = r1(i as f64 * step);
        let en = r1(if i < roles.len() - 1 { (i + 1) as f64 * step } else { duration });
        let voice = transcript.iter().filter(|t| { let (a, b) = (t["start"].as_f64().unwrap_or(0.0), t["end"].as_f64().unwrap_or(0.0)); (st <= a && a < en) || (st <= b && b <= en) })
            .map(|t| s(t, "text")).collect::<Vec<_>>().join(" ").trim().to_string();
        let (shot, cam, on, dflt): (&str, &str, String, String) = match *role {
            "hook" => ("close-up", "zoom-in", hook_text.chars().take(40).collect(), hook_text.clone()),
            "problem" => ("medium-shot", "handheld", "Vấn đề thường gặp".into(), format!("Nhiều bạn băn khoăn khi chọn {product}")),
            "demo" => ("medium-shot", "handheld", "Cận cảnh trải nghiệm thực tế".into(), "Chất liệu và thiết kế lên dáng rất chuẩn".into()),
            "proof" => ("close-up", "static", "Đánh giá chi tiết".into(), "Form dáng và đường may chỉn chu ngoài mong đợi".into()),
            _ => ("medium-shot", "static", "Săn deal ngay tại giỏ hàng!".into(), "Mọi người bấm ngay vào giỏ hàng góc trái săn ưu đãi nhé!".into()),
        };
        structure.push(json!({"role": role, "startSec": st, "endSec": en, "shot": shot, "camera": cam, "onScreenText": on, "voiceover": if voice.is_empty() { dflt } else { voice }}));
    }
    let cta = { let v = s(structure.last().unwrap(), "voiceover"); if v.is_empty() { "Bấm vào giỏ hàng góc trái săn deal ngay hôm nay!".into() } else { v } };
    let tone = "năng động, chân thực, tạo cảm giác gần gũi và đáng tin cậy";
    json!({"archetype": archetype, "hook": {"type": hook_type, "text": hook_text, "firstSeconds": hook_sec}, "structure": structure,
        "pacing": {"avgShotSec": avg, "cuts": cuts}, "tone": tone, "cta": cta, "aspectRatio": "9:16", "music": "Trending upbeat pop, nhịp điệu nhanh và bắt tai",
        "promptTemplate": format!("Video KOC TikTok phong cách {archetype} cho sản phẩm {{product}}. KOC {{character}} xuất hiện tự nhiên, giọng điệu {tone}. Mở màn hook ấn tượng ({hook_sec}s): '{hook_text}'. Chuyển cảnh nhanh (nhịp cắt ~{avg}s/shot) khoe chi tiết form dáng và chất liệu. Nhấn mạnh giá sale sốc {{price}} và kết thúc bằng CTA kêu gọi: '{cta}'."),
        "confidence": if transcript.is_empty() { 0.75 } else { 0.85 }})
}

pub async fn analyze_koc_video(url: &str, product_id: &str, product_name: &str, use_gemini: bool) -> anyhow::Result<KocStyle> {
    let meta = fetch_video_metadata(url).await;
    let mut data = if use_gemini { analyze_with_gemini(url, product_name).await } else { None };
    if data.is_none() {
        let tr = fetch_subs(url).await;
        let mut local = analyze_local(product_name, &meta, &tr);
        // With a real LLM, refine the deterministic result from the transcript; keep local on any failure.
        if !tr.is_empty() && use_gemini {
            let p = format!("Sản phẩm: {product_name}\nTiêu đề: {}\nTranscript: {}\nPhân tích phong cách KOC review.\n{SCHEMA_PROMPT}", s(&meta, "title"), tr.iter().map(|t| s(t, "text")).collect::<Vec<_>>().join(" ").chars().take(6000).collect::<String>());
            if let Some(v) = ask_json("Bạn là chuyên gia phân tích kịch bản KOC review TikTok.", &p).await.filter(|v| v.get("structure").is_some()) { local = v; }
        }
        data = Some(local);
    }
    let a = data.unwrap();
    let creator = { let c = s(&meta, "creator"); if c.is_empty() { "koc_creator".into() } else { c } };
    let mut st: KocStyle = serde_json::from_value(json!({
        "styleId": format!("style_{}_{}", slug(&creator), &uuid::Uuid::new_v4().simple().to_string()[..6]), "productId": product_id,
        "source": {"videoUrl": url, "creator": creator, "views": meta["views"], "likes": meta["likes"], "durationSec": meta["durationSec"], "collectedAt": now_iso()},
    }))?;
    let merge: KocStyle = serde_json::from_value({ let mut m = a.clone(); m["styleId"] = json!(st.styleId); m["productId"] = json!(product_id); m })?;
    st.archetype = merge.archetype; st.hook = merge.hook; st.pacing = Pacing { avgShotSec: merge.pacing.avgShotSec, cuts: if merge.pacing.cuts == 0 { merge.structure.len() as i64 } else { merge.pacing.cuts } };
    st.structure = merge.structure; st.tone = merge.tone; st.cta = merge.cta; st.aspectRatio = merge.aspectRatio; st.music = merge.music;
    st.promptTemplate = fix_prompt(&merge.promptTemplate); st.confidence = merge.confidence.clamp(0.0, 1.0);
    Ok(st)
}

pub async fn collect_and_analyze(pid: &str, product_id: &str, manual: &[String], max: usize, use_gemini: bool) -> Value {
    let name = get_product(pid, product_id).map(|p| p.name).unwrap_or_else(|| format!("Sản phẩm {product_id}"));
    let disc = find_koc_video_urls(pid, product_id, manual, max).await;
    let status = disc.get("status").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
    let urls: Vec<String> = disc.get("urls").and_then(|u| u.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let kept = disc.get("browser_kept_open").and_then(|v| v.as_bool()).unwrap_or(false);
    if status == "captcha_needed" {
        return json!({"status": "captcha_needed", "message": disc.get("message").cloned().unwrap_or(json!("Gặp Captcha / Security Check trên TikTok.")), "browser_kept_open": true, "discovered_urls": [], "styles": [], "total": 0});
    }
    if urls.is_empty() {
        return json!({"status": status, "message": disc.get("message").cloned().unwrap_or(json!("Không tìm thấy URL video nào.")), "browser_kept_open": kept, "discovered_urls": [], "styles": [], "total": 0});
    }
    let (mut saved, mut errs) = (vec![], vec![]);
    for u in &urls {
        match analyze_koc_video(u, product_id, &name, use_gemini).await.and_then(|st| save_koc_style(pid, &st).map(|_| st)) {
            Ok(st) => saved.push(st),
            Err(e) => errs.push(format!("{u}: {e}")),
        }
    }
    let mut msg = format!("Đã phân tích và lưu {}/{} phong cách KOC.", saved.len(), urls.len());
    if !errs.is_empty() { msg += &format!(" (Lỗi: {})", errs.join("; ")); }
    json!({"status": if saved.is_empty() { "error" } else { "success" }, "message": msg, "browser_kept_open": false, "discovered_urls": urls, "styles": saved, "total": saved.len()})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_analysis_and_prompt() {
        let meta = json!({"durationSec": 30.0, "title": "Thử đồ áo khoác"});
        let a = analyze_local("Áo", &meta, &[json!({"start": 0.5, "end": 2.0, "text": "Bạn đã thấy chưa?"})]);
        assert_eq!(a["archetype"], "try-on");
        assert_eq!(a["structure"].as_array().unwrap().len(), 5);
        assert_eq!(a["hook"]["type"], "câu hỏi");
        assert!(fix_prompt("abc").contains("{price}"));
        assert_eq!(fix_prompt("{product} {character} {price}"), "{product} {character} {price}");
    }
}
