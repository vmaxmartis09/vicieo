use super::models::s;
use super::store::{atomic_write_json, read_json, SocialStore};
use crate::workspace;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

fn pdir(p: &str) -> PathBuf { workspace::project_path(p, &[], false).unwrap_or_default() }
fn rel(p: &str, abs: &Path) -> String { abs.strip_prefix(pdir(p)).unwrap_or(abs).to_string_lossy().replace('\\', "/") }
fn mtime(p: &Path) -> f64 { p.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0.0, |d| d.as_secs_f64()) }

// shortcut: ffprobe/ffmpeg run without a kill timeout (Python had 5s); results are cached per mtime, upgrade to tokio::process + timeout if it hangs.
fn probe(f: &Path) -> Value {
    let mut meta = json!({"duration_s":30.0,"width":1080,"height":1920,"size_mb":5.0,"aspect_ratio":"9:16"});
    let Ok(md) = f.metadata() else { return meta };
    let size_mb = ((md.len() as f64 / 1048576.0) * 100.0).round() / 100.0;
    meta["size_mb"] = json!(size_mb);
    let Ok(out) = Command::new("ffprobe").args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams"]).arg(f).output() else { return meta };
    if !out.status.success() { return meta; }
    let Ok(info) = serde_json::from_slice::<Value>(&out.stdout) else { return meta };
    let dur = info["format"]["duration"].as_str().and_then(|d| d.parse::<f64>().ok()).unwrap_or(0.0);
    let (mut w, mut h) = (0, 0);
    if let Some(v) = info["streams"].as_array().and_then(|a| a.iter().find(|s| s["codec_type"] == "video")) { w = v["width"].as_i64().unwrap_or(0); h = v["height"].as_i64().unwrap_or(0); }
    let ar = if w == h && w > 0 { "1:1" } else if h > w { "9:16" } else { "16:9" };
    json!({"duration_s":(dur*100.0).round()/100.0,"width":w,"height":h,"size_mb":size_mb,"aspect_ratio":ar})
}

fn thumb_for(project: &str, video: &Path, id: &str) -> String {
    if !video.exists() { return String::new(); }
    let safe: String = id.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    let r = format!("publishing/.thumbs/{safe}.jpg");
    let abs = pdir(project).join(&r);
    if abs.exists() { return r; }
    let _ = std::fs::create_dir_all(abs.parent().unwrap());
    let ok = Command::new("ffmpeg").args(["-y", "-ss", "00:00:01", "-i"]).arg(video).args(["-vframes", "1", "-q:v", "2"]).arg(&abs).output().map_or(false, |o| o.status.success());
    if ok && abs.exists() { r } else { String::new() }
}

pub fn list_videos(project: &str) -> Vec<Value> {
    let Ok(st) = SocialStore::new(project) else { return vec![] };
    let cache_f = st.pub_dir.join(".video_cache.json");
    let mut cache = read_json(&cache_f, json!({}));
    let mut cached_probe = |abs: &Path, rel_p: &str| -> Value {
        let key = format!("{rel_p}_{}", mtime(abs));
        if let Some(m) = cache.get(&key) { return m.clone(); }
        let m = probe(abs);
        cache[&key] = m.clone();
        m
    };
    let mut posted: HashMap<String, (i64, Option<String>)> = HashMap::new();
    for p in st.list_posts(None, None, None, None, None, None) {
        let mut pt = s(&p, "created_at").to_string();
        for t in p["targets"].as_array().into_iter().flatten() {
            if let Some(x) = t["published_at"].as_str().filter(|x| !x.is_empty()).or(t["scheduled_at"].as_str().filter(|x| !x.is_empty())) { pt = x.to_string(); }
        }
        let mut bump = |k: String| { let e = posted.entry(k).or_insert((0, None)); e.0 += 1; if e.1.as_deref().map_or(true, |l| pt.as_str() > l) { e.1 = Some(pt.clone()); } };
        for m in p["media"].as_array().into_iter().flatten() { if let Some(path) = m["path"].as_str().filter(|x| !x.is_empty()) { bump(path.replace('\\', "/")); } }
        if let Some(r) = p["render_job_id"].as_str().filter(|x| !x.is_empty()) { bump(r.into()); }
    }
    let stat = |k: &[&str]| -> (i64, Value) { k.iter().find_map(|k| posted.get(*k)).map_or((0, Value::Null), |(c, l)| (*c, json!(l))) };
    let mut videos = vec![];
    let mut seen: HashSet<String> = HashSet::new();
    // 1. render queue
    let qd = pdir(project).join("runs/render_queue");
    if let Ok(rd) = std::fs::read_dir(&qd) {
        let mut files: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| { let n = p.file_name().unwrap().to_string_lossy().to_string(); n.starts_with("render_") && n.ends_with(".json") && !n.ends_with(".req.json") }).collect();
        files.sort();
        for jf in files {
            let job = read_json(&jf, Value::Null);
            if job["status"] != "done" { continue; }
            let id = job["job_id"].as_str().map(String::from).unwrap_or_else(|| jf.file_stem().unwrap().to_string_lossy().into());
            let Some(outputs) = job["outputs"].as_object().filter(|o| !o.is_empty()) else { continue };
            let req = read_json(&qd.join(format!("{id}.req.json")), json!({}));
            let d = &req["data"];
            let pick = |ks: &[&Value]| ks.iter().find_map(|v| v.as_str().filter(|x| !x.is_empty()).map(String::from));
            let title = pick(&[&d["title"], &d["story_title"], &req["output_name"]]).unwrap_or_else(|| format!("Video render {}", id.chars().take(8).collect::<String>()));
            let excerpt = pick(&[&d["book_page_text"], &req["voice"]["script"]]).unwrap_or_default();
            let prim = outputs.values().next().and_then(|v| v.as_str()).unwrap_or("").to_string();
            let abs = pdir(project).join(&prim);
            let meta = cached_probe(&abs, &prim);
            let mut thumb = String::new();
            if abs.exists() { if let Some(par) = abs.parent() { if let Ok(r) = std::fs::read_dir(par) {
                let mut imgs: Vec<_> = r.flatten().map(|e| e.path()).filter(|p| matches!(p.extension().and_then(|e| e.to_str()), Some("jpg") | Some("png"))).collect();
                imgs.sort();
                if let Some(i) = imgs.first() { thumb = rel(project, i); }
            } } }
            let (c, l) = stat(&[&id, &prim]);
            seen.insert(prim.clone());
            videos.push(json!({"id":id,"title":title,"thumb":thumb,"duration_s":meta["duration_s"],"width":meta["width"],"height":meta["height"],"size_mb":meta["size_mb"],
                "aspect_ratio":meta["aspect_ratio"],"variants":outputs,"primary_path":prim,"source":"job","posted_count":c,"last_posted_at":l,"script_excerpt":excerpt}));
        }
    }
    // 2. episodes
    for e in walkdir::WalkDir::new(pdir(project).join("content/episodes")).into_iter().flatten().filter(|e| e.path().extension().map_or(false, |x| x == "mp4")) {
        let r = rel(project, e.path());
        if !seen.insert(r.clone()) { continue; }
        let meta = cached_probe(e.path(), &r);
        let stem = e.path().file_stem().unwrap().to_string_lossy().to_string();
        let (c, l) = stat(&[&r]);
        videos.push(json!({"id":format!("ep_{stem}"),"title":title_case(&stem),"thumb":"","duration_s":meta["duration_s"],"width":meta["width"],"height":meta["height"],"size_mb":meta["size_mb"],
            "aspect_ratio":meta["aspect_ratio"],"variants":{"main":r},"primary_path":r,"source":"episode","posted_count":c,"last_posted_at":l,"script_excerpt":""}));
    }
    // 3. studio output packages
    if let Ok(rd) = std::fs::read_dir(pdir(project).join("output")) {
        let mut dirs: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| p.join("video.json").exists()).collect();
        dirs.sort();
        for d in dirs {
            let data = read_json(&d.join("video.json"), json!({}));
            let slug = d.file_name().unwrap().to_string_lossy().to_string();
            let f = &data["files"];
            let rv = f["video"].as_str().map(String::from).unwrap_or_else(|| format!("output/{slug}/final_45s.mp4"));
            let av = pdir(project).join(&rv);
            if !av.exists() || !seen.insert(rv.clone()) { continue; }
            let rt = f["thumb"].as_str().map(String::from).unwrap_or_else(|| format!("output/{slug}/thumb.jpg"));
            let thumb = if pdir(project).join(&rt).exists() { rt } else { String::new() };
            let rs = f["script"].as_str().map(String::from).unwrap_or_else(|| format!("output/{slug}/script.md"));
            let excerpt: String = std::fs::read_to_string(pdir(project).join(rs)).unwrap_or_default().chars().take(200).collect();
            let meta = cached_probe(&av, &rv);
            let (c, l) = stat(&[&rv, data["id"].as_str().unwrap_or("\u{0}")]);
            let mut variants = json!({"main": rv});
            for pl in data["platforms"].as_array().cloned().unwrap_or_else(|| vec![json!("tiktok"), json!("youtube")]) { if let Some(p) = pl.as_str() { variants[p] = json!(rv); } }
            videos.push(json!({"id":data["id"].as_str().map(String::from).unwrap_or_else(|| format!("studio_{slug}")),
                "title":data["title"].as_str().map(String::from).unwrap_or_else(|| title_case(&slug.replace(['_', '-'], " "))),"thumb":thumb,
                "duration_s":if data["duration_s"].is_null() { meta["duration_s"].clone() } else { data["duration_s"].clone() },"width":meta["width"],"height":meta["height"],
                "size_mb":if data["size_mb"].is_null() { meta["size_mb"].clone() } else { data["size_mb"].clone() },"aspect_ratio":"9:16","variants":variants,"primary_path":rv,
                "source":"studio","posted_count":c,"last_posted_at":l,"script_excerpt":excerpt}));
        }
    }
    // 4. remaining mp4s
    for folder in ["output", "library"] {
        for e in walkdir::WalkDir::new(pdir(project).join(folder)).into_iter().flatten().filter(|e| e.path().extension().map_or(false, |x| x == "mp4")) {
            let r = rel(project, e.path());
            if !seen.insert(r.clone()) { continue; }
            let meta = cached_probe(e.path(), &r);
            let stem = e.path().file_stem().unwrap().to_string_lossy().to_string();
            let (c, l) = stat(&[&r]);
            videos.push(json!({"id":format!("{folder}_{stem}"),"title":title_case(&stem),"thumb":"","duration_s":meta["duration_s"],"width":meta["width"],"height":meta["height"],
                "size_mb":meta["size_mb"],"aspect_ratio":meta["aspect_ratio"],"variants":{"main":r},"primary_path":r,"source":folder,"posted_count":c,"last_posted_at":l,"script_excerpt":""}));
        }
    }
    let _ = atomic_write_json(&cache_f, &cache);
    videos
}

fn title_case(s: &str) -> String {
    s.replace('_', " ").split(' ').map(|w| { let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase()).unwrap_or_default() }).collect::<Vec<_>>().join(" ")
}

pub fn list_library(project: &str, filter: Option<&str>, account_id: Option<&str>, platform: Option<&str>, q: Option<&str>) -> Vec<Value> {
    let Ok(st) = SocialStore::new(project) else { return vec![] };
    let posts = st.list_posts(None, None, None, None, None, None);
    let norm = |x: &str| x.replace('\\', "/");
    let mut out = vec![];
    for mut v in list_videos(project) {
        let id = s(&v, "id").to_string();
        let prim = s(&v, "primary_path").to_string();
        if s(&v, "thumb").is_empty() {
            let g = thumb_for(project, &pdir(project).join(&prim), &id);
            if !g.is_empty() { v["thumb"] = json!(g); }
        }
        let vpaths: Vec<String> = v["variants"].as_object().map(|m| m.values().filter_map(|x| x.as_str()).map(norm).collect()).unwrap_or_default();
        let mut matching = vec![];
        for p in &posts {
            let hit = p["render_job_id"].as_str() == Some(id.as_str()) || s(p, "id") == id
                || p["media"].as_array().into_iter().flatten().any(|m| m["path"].as_str().map_or(false, |mp| { let mp = norm(mp); mp == norm(&prim) || vpaths.contains(&mp) }));
            if hit { for t in p["targets"].as_array().into_iter().flatten() {
                matching.push(json!({"account_id":t["account_id"],"platform":t["platform"],"status":t["status"],"scheduled_at":t["scheduled_at"],
                    "published_url":t["external_url"].as_str().unwrap_or(""),"post_id":p["id"],"target_id":t["id"]}));
            } }
        }
        let any = |f: &dyn Fn(&Value) -> bool| matching.iter().any(f);
        if q.map_or(false, |q| !s(&v, "title").to_lowercase().contains(&q.to_lowercase())) { continue; }
        match filter {
            Some("unposted") if any(&|t| matches!(s(t, "status"), "published" | "scheduled")) => continue,
            Some("posted") if !any(&|t| s(t, "status") == "published") => continue,
            Some("scheduled") if !any(&|t| s(t, "status") == "scheduled") => continue,
            Some("draft") if !any(&|t| s(t, "status") == "draft") => continue,
            _ => {}
        }
        if let Some(a) = account_id {
            if let Some(n) = a.strip_prefix("not:") { if any(&|t| s(t, "account_id") == n && matches!(s(t, "status"), "published" | "scheduled")) { continue; } }
            else if !any(&|t| s(t, "account_id") == a) { continue; }
        }
        if platform.map_or(false, |pl| !any(&|t| s(t, "platform") == pl)) { continue; }
        v["posts_by_account"] = json!(matching);
        out.push(v);
    }
    out
}
