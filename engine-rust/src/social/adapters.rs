//! Publish adapters: youtube (Data API v3 resumable), tiktok (Content Posting API), manual package. Meta/browser_assisted fall back to manual.
use super::models::*;
use super::{vault, store::SocialStore};
use crate::workspace;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub struct PubResult { pub status: &'static str, pub external_id: Option<String>, pub url: Option<String>, pub error: Option<String> }
impl PubResult {
    fn fail(e: impl Into<String>) -> Self { Self { status: "failed", external_id: None, url: None, error: Some(e.into()) } }
}
fn trunc(s: &str) -> String { s.chars().take(300).collect() }
fn issue(level: &str, code: &str, m: String) -> Value { json!({"level": level, "code": code, "message": m}) }

pub fn kind(platform: &str, mode: &str) -> &'static str {
    if mode == "manual" || mode == "browser_assisted" { return "manual"; }
    match platform.to_lowercase().as_str() { "youtube" => "youtube", "tiktok" => "tiktok", _ => "manual" }
}

pub fn validate(platform: &str, mode: &str, target: &Value, media: Option<&Value>) -> Vec<Value> {
    let mut out = vec![];
    let cap = s(target, "caption");
    let media_path = media.and_then(|m| m["path"].as_str()).unwrap_or("");
    match kind(platform, mode) {
        "youtube" => {
            let t = if !s(target, "title").is_empty() { s(target, "title") } else { cap };
            if t.trim().is_empty() { out.push(issue("error", "missing_title", "Tiêu đề YouTube không được để trống".into())); }
            else if t.chars().count() > 100 { out.push(issue("error", "title_too_long", format!("Tiêu đề quá 100 ký tự (hiện có {})", t.chars().count()))); }
            let d = if !s(target, "description").is_empty() { s(target, "description") } else { cap };
            if d.chars().count() > 5000 { out.push(issue("error", "desc_too_long", format!("Mô tả quá 5000 ký tự (hiện có {})", d.chars().count()))); }
            if media_path.is_empty() { out.push(issue("error", "missing_media", "Chưa chọn tệp video".into())); }
            else if !["mp4", "mov", "mkv", "avi", "webm"].contains(&Path::new(media_path).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str()) {
                out.push(issue("warning", "uncommon_codec", "Nên sử dụng định dạng MP4 H.264".into()));
            }
        }
        "tiktok" => {
            if cap.chars().count() > 2200 { out.push(issue("error", "caption_too_long", format!("Caption TikTok quá 2200 ký tự (hiện có {})", cap.chars().count()))); }
            if media_path.is_empty() { out.push(issue("error", "missing_media", "Chưa chọn tệp video cho TikTok".into())); }
            else if media.and_then(|m| m["duration_s"].as_f64()).unwrap_or(0.0) > 600.0 { out.push(issue("warning", "duration_long", "Video dài hơn 10 phút, kiểm tra giới hạn tài khoản TikTok".into())); }
        }
        _ => if cap.is_empty() { out.push(issue("warning", "empty_caption", "Bài viết chưa có caption".into())); },
    }
    out
}

pub fn resolve_media_file(project_id: &str, raw: &str) -> Option<PathBuf> {
    if raw.is_empty() { return None; }
    let p = Path::new(raw);
    if p.is_absolute() && p.exists() { return Some(p.to_path_buf()); }
    if let Ok(pp) = workspace::project_path(project_id, &[raw], false) { if pp.exists() { return Some(pp); } }
    let repo = workspace::repo_root().join(raw);
    repo.exists().then_some(repo)
}

fn token(account: &Value, platform: &str) -> Option<String> {
    let tr = account["auth"]["token_ref"].as_str().filter(|x| !x.is_empty()).map(String::from).unwrap_or_else(|| format!("oauth_{platform}_{}", s(account, "id")));
    vault::get_key(&format!("{tr}_access"))
}

pub struct Ctx<'a> { pub project_id: &'a str, pub post_id: &'a str, pub dry_run: bool }

pub async fn publish(cx: &Ctx<'_>, account: &Value, target: &Value, media: Option<&Value>) -> PubResult {
    let mode = account["auth"]["mode"].as_str().unwrap_or("manual");
    let platform = s(target, "platform");
    let k = kind(platform, mode);
    if k == "manual" { return publish_manual(cx, account, target, media); }
    let errs: Vec<String> = validate(platform, mode, target, media).iter().filter(|i| i["level"] == "error").map(|i| s(i, "message").to_string()).collect();
    if !errs.is_empty() { return PubResult::fail(errs.join("; ")); }
    if cx.dry_run {
        let id = format!("dryrun_{}", &uuid::Uuid::new_v4().simple().to_string()[..10]);
        return PubResult { status: "published", url: Some(format!("dry-run://{k}/{id}")), external_id: Some(id), error: None };
    }
    let Some(tok) = token(account, k) else { return PubResult::fail("Missing OAuth access token in vault (unauthorized)") };
    let media_path = media.and_then(|m| m["path"].as_str()).and_then(|p| resolve_media_file(cx.project_id, p));
    let r = if k == "youtube" { youtube(&tok, target, media_path.as_deref()).await } else { tiktok(&tok, account, target, media_path.as_deref()).await };
    r.unwrap_or_else(|e| PubResult::fail(trunc(&e.to_string())))
}

async fn youtube(tok: &str, t: &Value, media: Option<&Path>) -> anyhow::Result<PubResult> {
    let media = media.ok_or_else(|| anyhow::anyhow!("Video file not found"))?;
    let title: String = [s(t, "title"), s(t, "caption")].into_iter().find(|x| !x.is_empty()).unwrap_or("Untitled Video").chars().take(100).collect();
    let desc: String = [s(t, "description"), s(t, "caption")].into_iter().find(|x| !x.is_empty()).unwrap_or("").chars().take(5000).collect();
    let tags = if t["tags"].as_array().map_or(false, |a| !a.is_empty()) { &t["tags"] } else { &t["hashtags"] };
    let privacy = t["privacy"].as_str().unwrap_or("public");
    let mut body = json!({"snippet":{"title":title,"description":desc,"tags":tags,"categoryId":"22"},
        "status":{"privacyStatus":privacy,"selfDeclaredMadeForKids":t["made_for_kids"] == json!(true)}});
    if privacy == "private" { if let Some(at) = t["scheduled_at"].as_str().filter(|x| !x.is_empty()) { body["status"]["publishAt"] = json!(at); } }
    let c = reqwest::Client::builder().timeout(std::time::Duration::from_secs(300)).build()?;
    let init = c.post("https://www.googleapis.com/upload/youtube/v3/videos?uploadType=resumable&part=snippet,status")
        .bearer_auth(tok).header("X-Upload-Content-Type", "video/mp4").json(&body).send().await?;
    let st = init.status();
    if st.as_u16() == 403 {
        let txt = init.text().await.unwrap_or_default();
        return Ok(PubResult::fail(if txt.contains("quotaExceeded") { "YouTube API Quota exceeded for today (10,000 units limit)".to_string() } else { format!("YouTube 403 Forbidden: {}", trunc(&txt)) }));
    }
    if !st.is_success() { return Ok(PubResult::fail(format!("YouTube init failed ({}): {}", st.as_u16(), trunc(&init.text().await.unwrap_or_default())))); }
    let loc = init.headers().get("location").and_then(|v| v.to_str().ok()).map(String::from).ok_or_else(|| anyhow::anyhow!("YouTube init returned no upload URL"))?;
    let up = c.put(loc).header("Content-Type", "video/mp4").body(tokio::fs::read(media).await?).send().await?;
    let st = up.status();
    if st.as_u16() == 200 || st.as_u16() == 201 {
        let j: Value = up.json().await.unwrap_or(json!({}));
        let id = j["id"].as_str().unwrap_or("").to_string();
        if id.is_empty() { return Ok(PubResult::fail("YouTube upload response had no video id")); }
        Ok(PubResult { status: "published", url: Some(format!("https://www.youtube.com/watch?v={id}")), external_id: Some(id), error: None })
    } else { Ok(PubResult::fail(format!("Upload failed with HTTP {}: {}", st.as_u16(), trunc(&up.text().await.unwrap_or_default())))) }
}

async fn tiktok(tok: &str, account: &Value, t: &Value, media: Option<&Path>) -> anyhow::Result<PubResult> {
    let media = media.ok_or_else(|| anyhow::anyhow!("Video file not found"))?;
    let bytes = tokio::fs::read(media).await?;
    let n = bytes.len();
    let caption: String = if s(t, "caption").is_empty() { "TikTok Video".into() } else { s(t, "caption").chars().take(2200).collect() };
    let payload = json!({"post_info":{"title":caption,"privacy_level":if s(t,"privacy")=="public" {"PUBLIC_TO_EVERYONE"} else {"SELF_ONLY"},
        "disable_duet":t["duet_enabled"]==json!(false),"disable_stitch":t["stitch_enabled"]==json!(false),"disable_comment":t["comment_enabled"]==json!(false),
        "brand_content_toggle":t["disclosure_ad"]==json!(true)||!s(t,"product_link").is_empty()},
        "source_info":{"source":"FILE_UPLOAD","video_size":n,"chunk_size":n,"total_chunk_count":1}});
    let c = reqwest::Client::builder().timeout(std::time::Duration::from_secs(300)).build()?;
    let r = c.post("https://open.tiktokapis.com/v2/post/publish/video/init/").bearer_auth(tok).json(&payload).send().await?;
    if r.status().as_u16() != 200 { return Ok(PubResult::fail(format!("TikTok Init failed ({}): {}", r.status().as_u16(), trunc(&r.text().await.unwrap_or_default())))); }
    let d: Value = r.json().await.unwrap_or(json!({}));
    let data = &d["data"];
    let (Some(pid), Some(url)) = (data["publish_id"].as_str(), data["upload_url"].as_str()) else { return Ok(PubResult::fail("TikTok init response missing publish_id/upload_url")) };
    let up = c.put(url).header("Content-Type", "video/mp4").header("Content-Range", format!("bytes 0-{}/{n}", n.saturating_sub(1))).body(bytes).send().await?;
    if !matches!(up.status().as_u16(), 200 | 201) { return Ok(PubResult::fail(format!("TikTok chunk upload failed: {}", up.status().as_u16()))); }
    Ok(PubResult { status: "published", external_id: Some(pid.into()), url: Some(format!("https://www.tiktok.com/@{}", s(account, "handle"))), error: None })
}

fn publish_manual(cx: &Ctx<'_>, account: &Value, t: &Value, media: Option<&Value>) -> PubResult {
    let dir = match workspace::project_path(cx.project_id, &["publishing", "packages", cx.post_id, s(t, "platform")], false) {
        Ok(d) => d, Err(e) => return PubResult::fail(e.to_string()) };
    if [cx.post_id, s(t, "platform")].iter().any(|x| x.is_empty() || x.contains('/') || x.contains("..")) { return PubResult::fail("invalid post/platform id"); }
    if let Err(e) = fs::create_dir_all(&dir) { return PubResult::fail(e.to_string()); }
    let w = |n: &str, c: &str| fs::write(dir.join(n), c);
    let tags = t["hashtags"].as_array().into_iter().flatten().filter_map(|h| h.as_str()).map(|h| if h.starts_with('#') { h.to_string() } else { format!("#{h}") }).collect::<Vec<_>>().join(" ");
    let _ = w("caption.txt", s(t, "caption"));
    let _ = w("hashtags.txt", &tags);
    let vid = dir.join("video.mp4");
    let _ = fs::remove_file(&vid);
    match media.and_then(|m| m["path"].as_str()).and_then(|p| resolve_media_file(cx.project_id, p)) {
        Some(src) => { if fs::hard_link(&src, &vid).is_err() { let _ = fs::copy(&src, &vid); } }
        None => { let _ = fs::write(&vid, "DUMMY_VIDEO_FILE"); }
    }
    match media.and_then(|m| m["thumb"].as_str()).and_then(|p| resolve_media_file(cx.project_id, p)) {
        Some(src) if fs::copy(&src, dir.join("cover.jpg")).is_ok() => {}
        _ => { let _ = w("cover.jpg", "DUMMY_COVER"); }
    }
    let meta = json!({"post_id":cx.post_id,"target_id":t["id"],"account":account,"platform":t["platform"],"title":t["title"],"caption":t["caption"],
        "hashtags":t["hashtags"],"product_link":t["product_link"],"disclosure_ad":t["disclosure_ad"],"privacy":t["privacy"],"scheduled_at":t["scheduled_at"]});
    let _ = w("meta.json", &serde_json::to_string_pretty(&meta).unwrap_or_default());
    let pl = s(t, "platform").to_uppercase();
    let none = "(Không có)";
    let title = if s(t, "title").is_empty() { none } else { s(t, "title") };
    let when = t["scheduled_at"].as_str().filter(|x| !x.is_empty()).unwrap_or("Đăng ngay");
    let mut chk = format!("# Hướng dẫn đăng bài thủ công: {pl}\n\n- **Tài khoản:** {} ({})\n- **Tiêu đề:** {title}\n- **Giờ đăng đề xuất:** {when}\n\n## Các bước thực hiện:\n1. Mở ứng dụng hoặc trang quản trị {pl}.\n2. Đính kèm tệp `video.mp4` và chọn ảnh bìa `cover.jpg`.\n3. Dán nội dung từ `caption.txt` và `hashtags.txt`.\n", s(account, "display_name"), s(account, "handle"));
    if !s(t, "product_link").is_empty() { chk += &format!("4. Thêm liên kết sản phẩm: {}\n", s(t, "product_link")); }
    if t["disclosure_ad"] == json!(true) { chk += "5. Bật tuỳ chọn 'Nội dung trả phí / Quảng cáo' (Affiliate disclosure)\n"; }
    chk += "6. Bấm ĐĂNG.\n7. Sau khi đăng xong, quay lại Social Manager và bấm 'Tôi đã đăng' để dán link URL thực tế.\n";
    let _ = w("checklist.md", &chk);
    PubResult { status: "published", external_id: Some(format!("pkg_{}_{}", cx.post_id, s(t, "platform"))), url: Some(dir.to_string_lossy().into()), error: None }
}

// ---------- manual packages ----------
fn pkg_dir(project: &str, post: &str, platform: &str) -> Option<PathBuf> {
    let ok = |x: &str| !x.is_empty() && x.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    (ok(post) && ok(platform)).then(|| workspace::project_path(project, &["publishing", "packages", post, platform], false).ok()).flatten()
}
pub fn export_package_zip(project: &str, post: &str, platform: &str) -> Result<PathBuf, String> {
    use std::io::Write;
    let dir = pkg_dir(project, post, platform).filter(|d| d.exists()).ok_or("Package directory not found")?;
    let zp = dir.parent().unwrap().join(format!("{platform}_package_{post}.zip"));
    let mut z = zip::ZipWriter::new(fs::File::create(&zp).map_err(|e| e.to_string())?);
    let opt = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for e in walkdir::WalkDir::new(&dir).into_iter().flatten().filter(|e| e.file_type().is_file()) {
        let rel = e.path().strip_prefix(&dir).unwrap().to_string_lossy().replace('\\', "/");
        z.start_file(rel, opt).map_err(|e| e.to_string())?;
        z.write_all(&fs::read(e.path()).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    }
    z.finish().map_err(|e| e.to_string())?;
    Ok(zp)
}
fn pkg_info(project: &str, post: &str, platform: &str, target_id: Option<&str>) -> Option<Value> {
    let dir = pkg_dir(project, post, platform).filter(|d| d.exists())?;
    let rd = |n: &str| fs::read_to_string(dir.join(n)).unwrap_or_default();
    let zp = dir.parent().unwrap().join(format!("{platform}_package_{post}.zip"));
    let real = fs::read(dir.join("video.mp4")).map_or(false, |b| !b.is_empty() && b != b"DUMMY_VIDEO_FILE");
    let mut v = json!({"post_id":post,"platform":platform,"package_dir":dir.to_string_lossy(),
        "zip_path":zp.exists().then(|| zp.to_string_lossy().to_string()),"zip_filename":zp.exists().then(|| zp.file_name().unwrap().to_string_lossy().to_string()),
        "has_real_video":real,"caption":rd("caption.txt"),"hashtags":rd("hashtags.txt"),"checklist":rd("checklist.md")});
    if let Some(t) = target_id { v["target_id"] = json!(t); }
    Some(v)
}
pub fn get_manual_package_info(project: &str, post: &str, platform: &str) -> Option<Value> { pkg_info(project, post, platform, None) }
pub fn create_manual_package(project: &str, post_id: &str, platform: &str) -> Result<Value, String> {
    let st = SocialStore::new(project).map_err(|e| e.to_string())?;
    let post = st.get_post(post_id).ok_or_else(|| format!("Post '{post_id}' not found in project '{project}'"))?;
    let targets = post["targets"].as_array().cloned().unwrap_or_default();
    let target = targets.iter().find(|t| s(t, "platform") == platform).or(targets.first()).cloned().unwrap_or_else(|| json!({
        "id": format!("tgt_{}", post_id.chars().take(8).collect::<String>()), "account_id":"manual_default","platform":platform,"title":post["title"],"caption":post["title"]}));
    let tp = s(&target, "platform").to_string();
    let acc = st.get_account(s(&target, "account_id")).unwrap_or_else(|| json!({"id":target["account_id"],"platform":tp,"handle":format!("@{tp}"),"display_name":format!("{} Channel", tp.to_uppercase())}));
    let res = publish_manual(&Ctx { project_id: project, post_id, dry_run: false }, &acc, &target, post["media"].get(0));
    if let Some(e) = res.error { return Err(e); }
    export_package_zip(project, post_id, &tp)?;
    pkg_info(project, post_id, &tp, target["id"].as_str()).ok_or_else(|| "package missing".into())
}

// ---------- OAuth ----------
pub fn connect_url(platform: &str, account_id: &str, redirect: &str) -> String {
    match platform {
        "youtube" => format!("https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={redirect}&response_type=code&scope=https://www.googleapis.com/auth/youtube.upload%20https://www.googleapis.com/auth/youtube.readonly&access_type=offline&prompt=consent&state={account_id}",
            vault::get_key("youtube_client_id").unwrap_or_default()),
        "tiktok" => format!("https://www.tiktok.com/v2/auth/authorize/?client_key={}&scope=video.publish,video.upload,user.info.basic&response_type=code&redirect_uri={redirect}&state={account_id}",
            vault::get_key("tiktok_client_key").unwrap_or_default()),
        _ => String::new(),
    }
}
pub async fn exchange_code(platform: &str, code: &str, redirect: &str) -> Result<Value, String> {
    let c = reqwest::Client::new();
    let r = match platform {
        "youtube" => c.post("https://oauth2.googleapis.com/token").form(&[("code", code), ("client_id", &vault::get_key("youtube_client_id").unwrap_or_default()),
            ("client_secret", &vault::get_key("youtube_client_secret").unwrap_or_default()), ("redirect_uri", redirect), ("grant_type", "authorization_code")]),
        "tiktok" => c.post("https://open.tiktokapis.com/v2/oauth/token/").form(&[("client_key", &vault::get_key("tiktok_client_key").unwrap_or_default()),
            ("client_secret", &vault::get_key("tiktok_client_secret").unwrap_or_default()), ("code", &code.to_string()), ("grant_type", &"authorization_code".to_string()), ("redirect_uri", &redirect.to_string())]),
        _ => return Ok(json!({"mode":"manual","status":"active"})),
    };
    let d: Value = r.send().await.map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())?;
    if d["access_token"].is_null() { return Err(format!("OAuth exchange failed: {}", trunc(&d["error_description"].as_str().unwrap_or("unknown error")))); }
    Ok(d)
}
pub async fn refresh(account: &Value) -> Value {
    let mode = account["auth"]["mode"].as_str().unwrap_or("manual");
    let platform = s(account, "platform");
    let k = kind(platform, mode);
    if k == "manual" { return json!({"mode":"manual","status":"active"}); }
    let tr = account["auth"]["token_ref"].as_str().filter(|x| !x.is_empty()).map(String::from).unwrap_or_else(|| format!("oauth_{k}_{}", s(account, "id")));
    let Some(rt) = vault::get_key(&format!("{tr}_refresh")) else { return json!({"error":"Missing refresh token in vault"}) };
    let c = reqwest::Client::new();
    let req = if k == "youtube" { c.post("https://oauth2.googleapis.com/token").form(&[("client_id", vault::get_key("youtube_client_id").unwrap_or_default()),
        ("client_secret", vault::get_key("youtube_client_secret").unwrap_or_default()), ("refresh_token", rt), ("grant_type", "refresh_token".into())]) }
        else { c.post("https://open.tiktokapis.com/v2/oauth/token/").form(&[("client_key", vault::get_key("tiktok_client_key").unwrap_or_default()), ("grant_type", "refresh_token".into()), ("refresh_token", rt)]) };
    let mut d: Value = match req.send().await { Ok(r) => r.json().await.unwrap_or(json!({})), Err(e) => return json!({"error": trunc(&e.to_string())}) };
    // never echo tokens back to API callers
    if let Some(o) = d.as_object_mut() { for k in ["access_token", "refresh_token", "id_token"] { if o.remove(k).is_some() { o.insert(format!("{k}_present"), json!(true)); } } }
    d
}
