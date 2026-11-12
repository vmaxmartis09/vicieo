//! Track A node executors: text, script, image_gen/image_edit, video_gen, character, product, cast,
//! quality_check, policy. `ctx.params` is the RESOLVED input map (node params overlaid with wired port
//! values); `ctx.inputs` holds the raw upstream outputs keyed by source node id.
//! Artifacts in outputs are `{kind, uri, url, provider, model}` objects (uri is project-relative).
use super::{fal, key_vault, media, models, pollinations, router::resolve_provider_for_model, spend_guard::SpendGuard};
use crate::engine::executor::{ExecutorRegistry, FallbackExecutor, NodeCtx, NodeExecutor, NodeOutput};
use crate::workspace;
use async_trait::async_trait;
use serde_json::{json, Map, Value};
use std::sync::Arc;

pub fn register(r: &mut ExecutorRegistry) {
    for ex in [
        Arc::new(TextExec) as Arc<dyn NodeExecutor>,
        Arc::new(ScriptExec),
        Arc::new(ImageExec),
        Arc::new(VideoExec),
        Arc::new(CharacterExec),
        Arc::new(ProductExec),
        Arc::new(CastExec),
        Arc::new(QualityExec),
        Arc::new(PolicyExec),
    ] {
        r.register(ex);
    }
}

pub fn artifact(kind: &str, project: &str, rel: &str, provider: &str, model: &str) -> Value {
    json!({"kind": kind, "uri": rel, "url": media::files_url(project, rel), "provider": provider, "model": model})
}

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(|x| x.as_str()).unwrap_or("")
}

fn guard(project: &str) -> SpendGuard {
    SpendGuard::new(workspace::project_path(project, &["runs"], true).unwrap_or_default())
}

fn out(preview_type: &str, preview_url: Option<String>, summary: String, cost: f64, outputs: Value) -> NodeOutput {
    NodeOutput { preview_type: preview_type.into(), preview_url, summary, cost_usd: cost, outputs, artifacts: vec![] }
}

fn image_uri(v: &Value) -> Option<String> {
    v.get("url").and_then(|u| u.as_str()).or_else(|| v.as_str()).map(String::from)
}

/// Mock path: placeholder preview from FallbackExecutor plus a port map so downstream nodes still wire up.
async fn mock_output(ctx: &NodeCtx, ports: &[&str]) -> anyhow::Result<NodeOutput> {
    let mut o = FallbackExecutor.execute(ctx).await?;
    let art = o.preview_url.as_ref().map(|u| json!({"kind": o.preview_type, "uri": u, "url": u, "provider": "mock", "model": "mock"}));
    let mut m = Map::new();
    if let Some(a) = art {
        for p in ports {
            m.insert((*p).into(), if *p == "images" { json!([a]) } else { a.clone() });
        }
    }
    o.outputs = Value::Object(m);
    Ok(o)
}

// ---------------------------------------------------------------- text
pub struct TextExec;
#[async_trait]
impl NodeExecutor for TextExec {
    fn node_types(&self) -> &'static [&'static str] {
        &["text"]
    }
    async fn execute(&self, ctx: &NodeCtx) -> anyhow::Result<NodeOutput> {
        let p = &ctx.params;
        let text = ["content", "text"].iter().map(|k| s(p, k)).find(|x| !x.is_empty()).unwrap_or("").to_string();
        Ok(out("text", None, format!("Văn bản ({} ký tự)", text.chars().count()), 0.0, json!({"text": text})))
    }
}

// ---------------------------------------------------------------- script
pub struct ScriptExec;

fn paste_to_scenes(raw: &str, target: usize, total_dur: f64) -> Vec<Value> {
    let paras: Vec<&str> = raw.split("\n\n").map(str::trim).filter(|x| !x.is_empty()).collect();
    let paras: Vec<&str> = if paras.len() <= 1 { raw.lines().map(str::trim).filter(|x| !x.is_empty()).collect() } else { paras };
    let n = paras.len().min(target.max(1)).max(1);
    // merge tail paragraphs so we never exceed the target count
    let mut groups: Vec<String> = paras.iter().take(n).map(|x| x.to_string()).collect();
    if paras.len() > n {
        if let Some(l) = groups.last_mut() {
            for extra in &paras[n..] {
                l.push('\n');
                l.push_str(extra);
            }
        }
    }
    let dur = ((total_dur / groups.len().max(1) as f64) * 10.0).round() / 10.0;
    groups
        .into_iter()
        .enumerate()
        .map(|(i, g)| {
            let first = g.lines().next().unwrap_or("");
            json!({"sceneNumber": i + 1, "title": format!("Cảnh {}: {}", i + 1, first.chars().take(30).collect::<String>()),
                "imagePrompt": format!("Phân cảnh {} hoạt hình 3D Pixar: {}", i + 1, g), "videoPrompt": format!("Camera chuyển động theo phân cảnh {}", i + 1),
                "dialogue": g, "durationSec": dur})
        })
        .collect()
}

fn episode_scenes(project: &str, ep: &str, granularity: &str) -> anyhow::Result<(Vec<Value>, Value, String)> {
    let p = workspace::project_path(project, &["content", "episodes", ep, "script.json"], false)?;
    let data: Value = serde_json::from_str(&std::fs::read_to_string(&p).map_err(|_| {
        anyhow::anyhow!("Chưa có script.json biên dịch cho tập '{}' (biên dịch .md → script.json thuộc Track D).", ep)
    })?)?;
    let title = s(&data, "title").to_string();
    let (mut scenes, mut lines, mut n) = (vec![], vec![], 0);
    for (si, sc) in data["scenes"].as_array().cloned().unwrap_or_default().iter().enumerate() {
        let shots = sc["shots"].as_array().cloned().unwrap_or_default();
        for sh in &shots {
            if !s(sh, "dialogue").trim().is_empty() {
                lines.push(json!({"id": format!("{}-{}", s(sh, "id"), lines.len() + 1), "order": lines.len(), "speaker": s(sh, "speaker"),
                    "characterId": sh.get("speakerId").cloned().unwrap_or(Value::Null), "text": s(sh, "dialogue").trim(), "shotId": sh["id"],
                    "sceneIndex": if granularity == "scene" { si } else { n }, "startSec": sh["startSec"], "endSec": sh["endSec"], "delivery": sh["voiceDelivery"]}));
            }
            n += 1;
            if granularity != "scene" {
                let mut x = sh.clone();
                x["sceneNumber"] = json!(scenes.len() + 1);
                x["title"] = json!(format!("{} · {}", s(sc, "title"), s(sh, "id")));
                scenes.push(x);
            }
        }
        if granularity == "scene" {
            let first = shots.first().cloned().unwrap_or(json!({}));
            let dlg: Vec<String> = shots.iter().filter(|h| !s(h, "dialogue").is_empty()).map(|h| format!("{}: {}", s(h, "speaker"), s(h, "dialogue"))).collect();
            let dur: f64 = shots.iter().map(|h| h["durationSec"].as_f64().unwrap_or(0.0)).sum();
            scenes.push(json!({"sceneNumber": si + 1, "title": format!("Cảnh {}: {}", sc["index"], s(sc, "title")), "imagePrompt": first["imagePrompt"],
                "videoPrompt": first["videoPrompt"], "dialogue": dlg.join("\n"), "durationSec": dur}));
        }
    }
    Ok((scenes, json!(lines), title))
}

#[async_trait]
impl NodeExecutor for ScriptExec {
    fn node_types(&self) -> &'static [&'static str] {
        &["script"]
    }
    async fn execute(&self, ctx: &NodeCtx) -> anyhow::Result<NodeOutput> {
        let p = &ctx.params;
        let mode = if s(p, "mode").is_empty() { "ai" } else { s(p, "mode") };
        let target = p["targetSceneCount"].as_u64().unwrap_or(6) as usize;
        let dur = p["targetDurationSec"].as_f64().unwrap_or(45.0);
        let (mut scenes, mut lines, mut cost) = (vec![], Value::Null, 0.0);
        match mode {
            "episode" => {
                let ep = s(p, "episodeId");
                if ep.is_empty() {
                    anyhow::bail!("Node '{}' (script) ở chế độ 'Kịch bản tập' nhưng chưa chọn tập (episodeId rỗng).", ctx.node_id);
                }
                let g = if s(p, "granularity").is_empty() { "shot" } else { s(p, "granularity") };
                let (sc, l, _) = episode_scenes(&ctx.project_id, ep, g)?;
                scenes = sc;
                lines = l;
            }
            "paste" => {
                let raw = s(p, "rawScript").trim();
                if raw.is_empty() {
                    anyhow::bail!("Node '{}' (script) ở chế độ dán nhưng rawScript rỗng.", ctx.node_id);
                }
                scenes = paste_to_scenes(raw, target, dur);
            }
            _ => {
                let text = ["text", "content"].iter().map(|k| s(p, k)).find(|x| !x.is_empty()).unwrap_or("").trim().to_string();
                if text.is_empty() {
                    anyhow::bail!("Node '{}' (script) thiếu ý tưởng (cổng 'text' chưa nối và chưa nhập text).", ctx.node_id);
                }
                if !ctx.mock && std::env::var("GEMINI_API_KEY").map(|k| !k.is_empty()).unwrap_or(false) {
                    let prompt = format!("Tách thành {} cảnh, trả JSON {{\"scenes\":[{{sceneNumber,title,imagePrompt,videoPrompt,dialogue,speaker,durationSec}}]}} cho ý tưởng: {}", target, text);
                    let raw = super::llm().generate(&prompt, Some("Bạn là biên kịch hoạt hình 3D Sóc Nhỏ TV."), None, true).await?;
                    if let Some(a) = serde_json::from_str::<Value>(raw.trim()).ok().and_then(|v| v["scenes"].as_array().cloned()) {
                        scenes = a;
                        cost = models::estimate_cost("gemini-2.5-flash", &models::Usage { n: 1.0, duration_sec: 0.0, text_len: 0.0 });
                    }
                }
                if scenes.is_empty() {
                    scenes = paste_to_scenes(&text, target, dur);
                }
            }
        }
        if scenes.is_empty() {
            anyhow::bail!("Node '{}' (script) không tạo được cảnh nào.", ctx.node_id);
        }
        if lines.is_null() {
            lines = json!(scenes.iter().enumerate().filter(|(_, sc)| !s(sc, "dialogue").trim().is_empty()).enumerate()
                .map(|(k, (i, sc))| json!({"id": format!("c{}-{}", i + 1, k + 1), "order": k, "speaker": s(sc, "speaker"), "characterId": Value::Null, "text": s(sc, "dialogue").trim(), "sceneIndex": i}))
                .collect::<Vec<_>>());
        }
        let dialogue: Vec<&str> = scenes.iter().map(|x| s(x, "dialogue").trim()).filter(|x| !x.is_empty()).collect();
        let n = scenes.len();
        let prompt = s(&scenes[0], "imagePrompt").to_string();
        Ok(out("text", None, format!("Kịch bản phân cảnh ({} cảnh)", n), cost,
            json!({"scenes": scenes, "prompt": prompt, "lines": lines, "dialogue": dialogue.join("\n\n")})))
    }
}

// ---------------------------------------------------------------- image
pub struct ImageExec;

fn pick_image_model(p: &Value, mock: bool) -> String {
    let raw = s(p, "model").trim().to_string();
    if mock {
        return if raw.is_empty() { "mock-image".into() } else { raw };
    }
    if raw.is_empty() || raw.starts_with("mock") {
        // local MLX is not ported: prefer fal when keyed, else the free Pollinations tier.
        return if key_vault::has_valid_key("fal") { "fal-flux-schnell".into() } else { "pollinations-flux".into() };
    }
    raw
}

async fn gen_one(ctx: &NodeCtx, model: &str, name: &str, prompt: &str, aspect: &str, seed: Option<i64>) -> anyhow::Result<(Value, f64)> {
    tokio::select! {
        _ = ctx.cancel.cancelled() => anyhow::bail!("cancelled"),
        r = gen_one_inner(ctx, model, name, prompt, aspect, seed) => r,
    }
}

async fn gen_one_inner(ctx: &NodeCtx, model: &str, name: &str, prompt: &str, aspect: &str, seed: Option<i64>) -> anyhow::Result<(Value, f64)> {
    let prov = resolve_provider_for_model(model, "pollinations");
    match prov.as_str() {
        "fal" => {
            let g = fal::generate_image(&guard(&ctx.project_id), &ctx.project_id, &ctx.run_id, name, prompt, model, aspect, 1, seed, &ctx.cancel).await?;
            let g = g.into_iter().next().ok_or_else(|| anyhow::anyhow!("không có ảnh"))?;
            Ok((artifact("image", &ctx.project_id, &g.rel, "fal", &g.model), g.cost_usd))
        }
        "pollinations" => {
            let (rel, m) = pollinations::generate_image(&ctx.project_id, &ctx.run_id, name, prompt, model, aspect, seed).await?;
            Ok((artifact("image", &ctx.project_id, &rel, "pollinations", &m), 0.0))
        }
        other => anyhow::bail!("Provider '{}' (model '{}') chưa được port sang Rust (hiện có: fal, pollinations).", other, model),
    }
}

#[async_trait]
impl NodeExecutor for ImageExec {
    fn node_types(&self) -> &'static [&'static str] {
        &["image_gen", "image_edit"]
    }
    async fn execute(&self, ctx: &NodeCtx) -> anyhow::Result<NodeOutput> {
        if ctx.mock {
            return mock_output(ctx, &["image", "images"]).await;
        }
        let p = &ctx.params;
        let model = pick_image_model(p, false);
        let aspect = if s(p, "aspectRatio").is_empty() { "9:16" } else { s(p, "aspectRatio") };
        let seed = p["seed"].as_i64();
        let extra = s(p, "prompt").trim().to_string();
        let scenes: Vec<Value> = p["scenes"].as_array().map(|a| a.iter().filter(|x| x.is_object()).cloned().collect()).unwrap_or_default();
        let (mut imgs, mut out_scenes, mut cost) = (vec![], vec![], 0.0);
        if !scenes.is_empty() {
            let total = scenes.len();
            for (i, sc) in scenes.iter().enumerate() {
                if ctx.cancel.is_cancelled() {
                    anyhow::bail!("cancelled");
                }
                let mut sp = ["imagePrompt", "prompt"].iter().map(|k| s(sc, k)).find(|x| !x.is_empty()).unwrap_or("").to_string();
                if sp.is_empty() {
                    sp = format!("{} {}", s(sc, "title"), s(sc, "dialogue").chars().take(200).collect::<String>()).trim().to_string();
                }
                if !extra.is_empty() && !sp.contains(&extra) {
                    sp = format!("{}\n{}", sp, extra);
                }
                let _ = ctx.tx.send(json!({"eventType": "node_progress", "runId": ctx.run_id, "projectId": ctx.project_id, "nodeId": ctx.node_id, "state": "running", "status": "running",
                    "progress": 0.05 + 0.9 * i as f64 / total as f64, "progressPct": ((0.05 + 0.9 * i as f64 / total as f64) * 100.0).round(),
                    "message": format!("Tạo ảnh cảnh {}/{}: {}", i + 1, total, s(sc, "title").chars().take(60).collect::<String>())}).to_string());
                let (a, c) = gen_one(ctx, &model, &format!("{}_s{}", ctx.node_id, i + 1), &sp, aspect, seed.map(|x| x + i as i64)).await?;
                cost += c;
                let mut o = sc.clone();
                o["image"] = a.clone();
                o["imageUri"] = a["uri"].clone();
                out_scenes.push(o);
                imgs.push(a);
            }
        } else {
            let prompt = if s(p, "prompt").is_empty() { "Sóc Nhỏ Bi đang khám phá khu rừng diệu kỳ" } else { s(p, "prompt") };
            let n = p["n"].as_u64().unwrap_or(1).max(1);
            for i in 0..n {
                let (a, c) = gen_one(ctx, &model, &format!("{}_{}", ctx.node_id, i), prompt, aspect, seed.map(|x| x + i as i64)).await?;
                cost += c;
                imgs.push(a);
            }
        }
        let url = imgs.first().and_then(image_uri);
        let mut o = json!({"image": imgs[0], "images": imgs});
        if !out_scenes.is_empty() {
            o["scenes"] = json!(out_scenes);
        }
        Ok(out("image", url, format!("Hoàn thành tạo {} ảnh ({})", imgs.len(), model), cost, o))
    }
}

// ---------------------------------------------------------------- video
pub struct VideoExec;
#[async_trait]
impl NodeExecutor for VideoExec {
    fn node_types(&self) -> &'static [&'static str] {
        &["video_gen"]
    }
    async fn execute(&self, ctx: &NodeCtx) -> anyhow::Result<NodeOutput> {
        if ctx.mock {
            return mock_output(ctx, &["video"]).await;
        }
        let p = &ctx.params;
        let raw = s(p, "model").trim();
        let model = if raw.is_empty() || raw.starts_with("mock") { "fal-kling-v3-std" } else { raw };
        let prov = resolve_provider_for_model(model, "fal");
        if prov != "fal" {
            anyhow::bail!("Provider '{}' (model '{}') cho video chưa port sang Rust (hiện có: fal; google Veo/replicate còn thiếu).", prov, model);
        }
        let prompt = ["prompt", "videoPrompt"].iter().map(|k| s(p, k)).find(|x| !x.is_empty()).unwrap_or("").to_string();
        if prompt.is_empty() {
            anyhow::bail!("Node '{}' (video_gen) thiếu prompt.", ctx.node_id);
        }
        let aspect = if s(p, "aspectRatio").is_empty() { "9:16" } else { s(p, "aspectRatio") };
        let dur = p["durationSec"].as_f64().unwrap_or(8.0);
        let first = p.get("image").and_then(|a| a["uri"].as_str()).or_else(|| p.get("first_frame").and_then(|a| a["uri"].as_str()));
        let data_uri = match first {
            Some(rel) => {
                use base64::Engine;
                let path = workspace::project_path(&ctx.project_id, &[rel], false)?;
                let bytes = std::fs::read(&path)?;
                let mime = if rel.ends_with("jpg") || rel.ends_with("jpeg") { "image/jpeg" } else { "image/png" };
                Some(format!("data:{};base64,{}", mime, base64::engine::general_purpose::STANDARD.encode(bytes)))
            }
            None => None,
        };
        let g = fal::generate_video(&guard(&ctx.project_id), &ctx.project_id, &ctx.run_id, &ctx.node_id, &prompt, model, aspect, dur, data_uri, &ctx.cancel).await?;
        let a = artifact("video", &ctx.project_id, &g.rel, "fal", &g.model);
        let url = a["url"].as_str().map(String::from);
        Ok(out("video", url, format!("Hoàn thành tạo video ({})", model), g.cost_usd, json!({"video": a})))
    }
}

// ---------------------------------------------------------------- character / product / cast
fn load_characters(project: &str) -> Vec<Value> {
    workspace::project_path(project, &["characters", "studio_characters.json"], false).ok()
        .and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| v["characters"].as_array().cloned()).unwrap_or_default()
}

fn character_info(project: &str, id: &str) -> Value {
    let prof = load_characters(project).into_iter().find(|c| c["id"] == id || c["characterId"] == id);
    let g = |k: &str| prof.as_ref().map(|p| p[k].clone()).unwrap_or(Value::Null);
    let height = if g("heightCm").is_null() && id == "soc_nho" { json!(100) } else { g("heightCm") };
    json!({"characterId": id, "name": prof.as_ref().map(|p| p["name"].clone()).unwrap_or(json!(id)), "heightCm": height,
        "weightKg": g("weightKg"), "promptLock": g("promptLock"), "voiceId": g("voiceId"), "defaultVoice": g("voiceId"),
        "aliases": g("aliases"), "profileFound": prof.is_some()})
}

fn character_images(project: &str, id: &str) -> Vec<Value> {
    let Ok(dir) = workspace::project_path(project, &["characters", id], false) else { return vec![] };
    let mut v: Vec<String> = std::fs::read_dir(dir).into_iter().flatten().flatten().map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".png") || n.ends_with(".jpg") || n.ends_with(".webp")).collect();
    v.sort();
    v.into_iter().map(|n| artifact("image", project, &format!("characters/{}/{}", id, n), "local", "character_dna")).collect()
}

pub struct CharacterExec;
#[async_trait]
impl NodeExecutor for CharacterExec {
    fn node_types(&self) -> &'static [&'static str] {
        &["character"]
    }
    async fn execute(&self, ctx: &NodeCtx) -> anyhow::Result<NodeOutput> {
        let id = if s(&ctx.params, "characterId").is_empty() { "soc_nho" } else { s(&ctx.params, "characterId") };
        let info = character_info(&ctx.project_id, id);
        let refs = character_images(&ctx.project_id, id);
        let url = refs.first().and_then(image_uri);
        Ok(out("image", url, format!("Nhân vật {} ({} ảnh tham chiếu)", info["name"].as_str().unwrap_or(id), refs.len()), 0.0,
            json!({"character": info, "reference_images": refs})))
    }
}

pub struct ProductExec;
#[async_trait]
impl NodeExecutor for ProductExec {
    fn node_types(&self) -> &'static [&'static str] {
        &["product"]
    }
    async fn execute(&self, ctx: &NodeCtx) -> anyhow::Result<NodeOutput> {
        let pid = s(&ctx.params, "productId").trim();
        let fail = |m: String| anyhow::anyhow!("Node '{}' (product): {}", ctx.node_id, m);
        if pid.is_empty() || pid.contains('/') || pid.contains("..") {
            return Err(fail("chưa chọn sản phẩm.".into()));
        }
        let dir = workspace::project_path(&ctx.project_id, &["products", pid], false)?;
        let meta: Value = ["product.json", "meta.json"].iter().find_map(|f| std::fs::read_to_string(dir.join(f)).ok()).and_then(|t| serde_json::from_str(&t).ok())
            .ok_or_else(|| fail(format!("sản phẩm '{}' không có trong dự án {}.", pid, ctx.project_id)))?;
        let mut pool: Vec<String> = ["cutout_images", "images"].iter().find_map(|k| meta[k].as_array().filter(|a| !a.is_empty()).cloned()).unwrap_or_default()
            .iter().filter_map(|x| x.as_str().map(String::from))
            .filter(|rel| workspace::project_path(&ctx.project_id, &[rel], false).map(|p| p.is_file()).unwrap_or(false)).collect();
        if pool.is_empty() {
            return Err(fail("sản phẩm chưa có ảnh đã tách nền.".into()));
        }
        let want = std::path::Path::new(s(&ctx.params, "variant")).file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
        if !want.is_empty() {
            pool.sort_by_key(|x| if x.ends_with(&want) { 0 } else { 1 });
        }
        let arts: Vec<Value> = pool.iter().take(6).map(|r| artifact("image", &ctx.project_id, r, "local", "product_cutout")).collect();
        let mut info = meta.clone();
        info["productId"] = json!(pid);
        info["selectedImage"] = json!(pool[0]);
        let url = image_uri(&arts[0]);
        Ok(out("image", url, format!("Sản phẩm {}", pid), 0.0, json!({"product": info, "images": arts, "image": arts[0]})))
    }
}

pub struct CastExec;
#[async_trait]
impl NodeExecutor for CastExec {
    fn node_types(&self) -> &'static [&'static str] {
        &["cast"]
    }
    async fn execute(&self, ctx: &NodeCtx) -> anyhow::Result<NodeOutput> {
        let p = &ctx.params;
        let mut ids: Vec<String> = vec![];
        let mut add = |x: &str| {
            if !x.is_empty() && !ids.iter().any(|i| i == x) {
                ids.push(x.to_string());
            }
        };
        for l in p["lines"].as_array().into_iter().flatten() {
            add(s(l, "characterId"));
        }
        for sc in p["scenes"].as_array().into_iter().flatten() {
            for c in sc["characterIds"].as_array().into_iter().flatten().filter_map(|c| c.as_str()) {
                add(c);
            }
        }
        for c in p["include"].as_array().into_iter().flatten().filter_map(|c| c.as_str()) {
            add(c);
        }
        if ids.is_empty() {
            anyhow::bail!("Node '{}' (cast) không tìm thấy nhân vật nào trong kịch bản. Nối cổng 'lines'/'scenes' của Kịch bản, hoặc ghi nhân vật vào params.include.", ctx.node_id);
        }
        let chars: Vec<Value> = ids.iter().map(|i| character_info(&ctx.project_id, i)).collect();
        let refs: Vec<Value> = ids.iter().flat_map(|i| character_images(&ctx.project_id, i).into_iter().take(1)).collect();
        let url = refs.first().and_then(image_uri);
        Ok(out("image", url, format!("Dàn diễn viên: {} (đọc thoại voice thuộc Track C, chưa có clip)", ids.join(", ")), 0.0, json!({"characters": chars, "reference_images": refs, "voice": []})))
    }
}

// ---------------------------------------------------------------- policy / quality
fn load_policy(project: &str) -> Value {
    workspace::project_path(project, &["policy.json"], false).ok().and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(json!({"rules": []}))
}

pub struct PolicyExec;
#[async_trait]
impl NodeExecutor for PolicyExec {
    fn node_types(&self) -> &'static [&'static str] {
        &["policy"]
    }
    async fn execute(&self, ctx: &NodeCtx) -> anyhow::Result<NodeOutput> {
        let mut pol = load_policy(&ctx.project_id);
        let extra: Vec<Value> = ctx.params["extraTerms"].as_array().cloned().unwrap_or_default();
        if !extra.is_empty() {
            let sev = if s(&ctx.params, "extraSeverity").is_empty() { "block" } else { s(&ctx.params, "extraSeverity") };
            if let Some(r) = pol["rules"].as_array_mut() {
                r.push(json!({"id": "extra", "label": "Từ khoá bổ sung", "severity": sev, "enabled": true, "terms": extra}));
            }
        }
        let n = pol["rules"].as_array().map(|r| r.len()).unwrap_or(0);
        Ok(out("text", None, format!("Chính sách nội dung ({} nhóm luật)", n), 0.0, json!({"policy": pol})))
    }
}

fn collect_texts(v: &Value, acc: &mut Vec<String>, depth: usize) {
    if depth > 6 {
        return;
    }
    match v {
        Value::String(t) if t.chars().count() > 1 && !t.starts_with("/data/") => acc.push(t.clone()),
        Value::Array(a) => a.iter().for_each(|x| collect_texts(x, acc, depth + 1)),
        Value::Object(m) => m.iter().filter(|(k, _)| !matches!(k.as_str(), "uri" | "url" | "image" | "imageUri" | "kind" | "provider" | "model")).for_each(|(_, x)| collect_texts(x, acc, depth + 1)),
        _ => {}
    }
}

pub struct QualityExec;
#[async_trait]
impl NodeExecutor for QualityExec {
    fn node_types(&self) -> &'static [&'static str] {
        &["quality_check"]
    }
    async fn execute(&self, ctx: &NodeCtx) -> anyhow::Result<NodeOutput> {
        let p = &ctx.params;
        let pol = if p["policy"]["rules"].is_array() { p["policy"].clone() } else { load_policy(&ctx.project_id) };
        let mut texts = vec![];
        for k in ["text", "captions", "scenes", "lines", "dialogue", "prompt"] {
            if let Some(v) = p.get(k) {
                collect_texts(v, &mut texts, 0);
            }
        }
        let has_media = p.get("media").map(|m| !m.is_null()).unwrap_or(false);
        if !ctx.mock && texts.is_empty() && !has_media {
            anyhow::bail!("Node '{}' (quality_check) chưa có gì để kiểm — nối 'media' hoặc 'text'/'captions'/'scenes'.", ctx.node_id);
        }
        let hay = texts.join("\n").to_lowercase();
        let mut hits: Vec<Value> = vec![];
        for r in pol["rules"].as_array().into_iter().flatten().filter(|r| r["enabled"] != false) {
            for t in r["terms"].as_array().into_iter().flatten().filter_map(|t| t.as_str()) {
                if !t.is_empty() && hay.contains(&t.to_lowercase()) {
                    hits.push(json!({"rule": r["id"], "label": r["label"], "term": t, "severity": r["severity"]}));
                }
            }
        }
        let blocked = hits.iter().any(|h| h["severity"] == "block");
        let note = if has_media { " (chấm điểm hình ảnh bằng vision chưa port: chỉ kiểm chính sách văn bản)" } else { "" };
        let summary = if blocked { format!("KHÔNG ĐẠT · {} vi phạm chính sách{}", hits.len(), note) } else { format!("ĐẠT · {} cảnh báo chính sách{}", hits.len(), note) };
        let hits_text = hits.iter().map(|h| format!("{} ({})", h["term"].as_str().unwrap_or(""), h["label"].as_str().unwrap_or(""))).collect::<Vec<_>>().join("; ");
        let report = json!({"passed": !blocked, "hits": hits, "checkedTexts": texts.len()});
        if blocked && !ctx.mock {
            anyhow::bail!("{} — {}", summary, serde_json::to_string(&report["hits"])?);
        }
        Ok(out("text", None, summary.clone(), 0.0, json!({"pass_fail": summary, "violations": hits_text, "media": p.get("media").cloned().unwrap_or(Value::Null), "passed": !blocked, "policy_report": report})))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paste_scenes_capped() {
        let sc = paste_to_scenes("a\n\nb\n\nc\n\nd", 2, 40.0);
        assert_eq!(sc.len(), 2);
        assert_eq!(sc[0]["durationSec"], 20.0);
        assert!(sc[1]["dialogue"].as_str().unwrap().contains('d'));
    }
}
