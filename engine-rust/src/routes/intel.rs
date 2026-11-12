use super::prelude::*;
use crate::intelligence::{
    analyzer, channels, chat, compliance, content_policy as cp, contracts as ct, ideas, node_runner, patterns,
    script_gen::{generate_complete_script, ScriptOpts}, strategy,
    util::{safe_id, ApiError, ApiResult},
};

type J = ApiResult<Json<Value>>;
fn v<T: Serialize>(x: &T) -> Value { serde_json::to_value(x).unwrap_or(Value::Null) }
fn ok_id(id: &str) -> ApiResult<()> { safe_id(id).map(|_| ()).map_err(|e| ApiError::bad(e.to_string())) }
fn sget(b: &Value, k: &str) -> Option<String> { b.get(k).and_then(|x| x.as_str()).map(String::from) }
fn req_str(b: &Value, k: &str) -> ApiResult<String> { sget(b, k).ok_or_else(|| ApiError::unproc(format!("field '{k}' is required"))) }
fn str_list(b: &Value, k: &str) -> Option<Vec<String>> { b.get(k).and_then(|x| x.as_array()).map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect()) }

// ---------- policy (data_routes parity; served under /projects and the dashboard's /data/projects prefix)

#[utoipa::path(get, path = "/data/projects/{project_id}/policy", params(("project_id" = String, Path, description = "Project Identifier")),
    responses((status = 200, description = "Get project content compliance policy", body = inline(Value))))]
pub(crate) async fn get_project_policy(AxumPath(pid): AxumPath<String>) -> J { ok_id(&pid)?; Ok(Json(cp::load_policy(&pid))) }

#[utoipa::path(put, path = "/data/projects/{project_id}/policy", params(("project_id" = String, Path, description = "Project Identifier")), request_body = inline(Value),
    responses((status = 200, description = "Save project content compliance policy", body = inline(Value))))]
pub(crate) async fn save_project_policy(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J { ok_id(&pid)?; Ok(Json(cp::save_policy(&pid, &b)?)) }

async fn policy_packs() -> Json<Value> { Json(json!(cp::list_packs())) }

async fn import_pack(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let pack = cp::load_pack(&sget(&b, "packId").unwrap_or_default()).ok_or_else(|| ApiError::nf("Không tìm thấy gói chính sách"))?;
    Ok(Json(cp::save_policy(&pid, &cp::import_pack(&cp::load_policy(&pid), &pack))?))
}

async fn import_file(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let name = sget(&b, "name").filter(|n| !n.is_empty()).unwrap_or_else(|| "upload.txt".into());
    let text = sget(&b, "text").unwrap_or_default();
    if !name.to_lowercase().ends_with(".json") && cp::looks_like_prose(&text) {
        let doc = cp::add_document(&pid, &name, &text)?;
        let msg = format!("Đã lưu '{}' làm tài liệu chính sách của dự án ({} mục). Mọi video phải qua kiểm định theo tài liệu này.", doc["name"].as_str().unwrap_or(""), doc["sections"].as_array().map(|s| s.len()).unwrap_or(0));
        return Ok(Json(json!({"policy": cp::load_policy(&pid), "document": doc, "added": 0, "terms": 0, "message": msg})));
    }
    let rules = cp::parse_policy_text(&name, &text).map_err(|e| ApiError::bad(format!("Không đọc được tệp: {e}")))?;
    if rules.is_empty() { return Err(ApiError::bad("Tệp không có từ/quy tắc nào")); }
    let stem = std::path::Path::new(&name).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let source = format!("upload-{}", crate::intelligence::util::slug(&stem));
    let saved = cp::save_policy(&pid, &cp::merge_rules(&cp::load_policy(&pid), &rules, &source))?;
    let terms: usize = rules.iter().map(|r| r["terms"].as_array().map(|t| t.len()).unwrap_or(0)).sum();
    Ok(Json(json!({"policy": saved, "added": rules.len(), "terms": terms})))
}

async fn fetch_url(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let url = sget(&b, "url").unwrap_or_default().trim().to_string();
    let page = cp::fetch_policy_page(&url).await.map_err(|e| ApiError::bad(format!("Không lấy được trang: {e}")))?;
    let text = page["text"].as_str().unwrap_or("");
    let r = json!({"url": url, "title": page["title"], "kind": "url", "fetchedAt": chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string(), "excerpt": text.chars().take(600).collect::<String>()});
    let saved = cp::save_policy(&pid, &cp::add_reference(&cp::load_policy(&pid), r))?;
    Ok(Json(json!({"policy": saved, "page": {"title": page["title"], "chars": page["chars"], "text": page["text"]}})))
}

pub(crate) async fn check_project_policy(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let texts: Vec<(String, String)> = match b.get("texts").and_then(|t| t.as_object()).filter(|t| !t.is_empty()) {
        Some(o) => o.iter().map(|(k, x)| (k.clone(), x.as_str().map(String::from).unwrap_or_else(|| x.to_string()))).collect(),
        None => sget(&b, "text").filter(|t| !t.is_empty()).map(|t| vec![("text".to_string(), t)]).unwrap_or_default(),
    };
    Ok(Json(cp::check_texts(&texts, &cp::load_policy(&pid))))
}

// ---------- /projects/{p}/intel/*

async fn ping() -> Json<Value> { Json(json!({"status": "ok", "module": "intel"})) }

async fn analyze_video(AxumPath((pid, sid)): AxumPath<(String, String)>) -> J {
    ok_id(&pid)?;
    let bp = analyzer::analyze_video(&pid, &sid).await.map_err(|e| if e.0 == 404 { e } else { ApiError::ise(format!("Lỗi phân tích blueprint: {}", e.1)) })?;
    Ok(Json(v(&bp)))
}
async fn list_blueprints(AxumPath(pid): AxumPath<String>) -> J { ok_id(&pid)?; Ok(Json(v(&ct::list_video_blueprints(&pid)))) }
async fn get_blueprint(AxumPath((pid, id)): AxumPath<(String, String)>) -> J {
    ok_id(&pid)?;
    ct::load_video_blueprint(&pid, &id).map(|b| Json(v(&b))).ok_or_else(|| ApiError::nf("Không tìm thấy blueprint."))
}
async fn get_patterns(AxumPath(pid): AxumPath<String>, Query(q): Query<HashMap<String, String>>) -> J { ok_id(&pid)?; Ok(Json(patterns::aggregate_patterns(&pid, q.get("niche").map(String::as_str)))) }

async fn generate_script_route(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let product = crate::intelligence::products::get_product(&pid, &req_str(&b, "product_id")?).ok_or_else(|| ApiError::nf("Không tìm thấy sản phẩm."))?;
    let bp = sget(&b, "blueprint_id").and_then(|i| ct::load_video_blueprint(&pid, &i));
    let gt = sget(&b, "generator_type").unwrap_or_else(|| "koc_review".into());
    if !["koc_review", "tiktok_sales", "affiliate_compare"].contains(&gt.as_str()) { return Err(ApiError::unproc("invalid generator_type")); }
    let (ch, voice) = (sget(&b, "character_name").unwrap_or_else(|| "LyLy".into()), sget(&b, "voice_id").unwrap_or_else(|| "soc-nho".into()));
    let platforms = str_list(&b, "platforms").unwrap_or_else(|| vec!["tiktok".into(), "reels".into(), "shorts".into()]);
    Ok(Json(generate_complete_script(&pid, &product, &ScriptOpts { generator_type: &gt, blueprint: bp.as_ref(), character_name: &ch, voice_id: &voice, platforms })?))
}

async fn ingest_channel(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let url = req_str(&b, "url")?;
    let max = b.get("max_videos").and_then(|m| m.as_u64()).unwrap_or(50) as u32;
    let meta = channels::ingest_channel(&pid, &url, max, 5).await.map_err(|e| ApiError::ise(format!("Lỗi nạp và phân tích kênh: {e}")))?;
    Ok(Json(channels::analyze_channel(&pid, meta["id"].as_str().unwrap_or(""))?))
}
async fn list_channels(AxumPath(pid): AxumPath<String>) -> J {
    ok_id(&pid)?;
    let dir = crate::intelligence::util::pp(&pid, &["knowledge", "channels"], false);
    let mut out = vec![];
    if let Ok(rd) = fs::read_dir(dir) {
        let mut ds: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| p.is_dir() && !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'))).collect();
        ds.sort();
        for d in ds { if let Some(c) = crate::intelligence::util::read_json(&d.join("channel.json")) { out.push(c); } }
    }
    Ok(Json(json!(out)))
}
async fn get_channel(AxumPath((pid, cid)): AxumPath<(String, String)>) -> J {
    ok_id(&pid)?;
    Ok(Json(channels::analyze_channel(&pid, &cid).map_err(|e| if e.0 == 404 { e } else { ApiError::ise(format!("Lỗi phân tích kênh: {}", e.1)) })?))
}
async fn user_data(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let videos = b.get("videos").and_then(|x| x.as_array()).cloned().unwrap_or_default();
    let meta = channels::ingest_user_channel_data(&pid, &req_str(&b, "channel_url")?, &sget(&b, "channel_name").unwrap_or_default(), &videos)
        .map_err(|e| ApiError::ise(format!("Lỗi nạp dữ liệu người dùng: {e}")))?;
    Ok(Json(channels::analyze_channel(&pid, meta["id"].as_str().unwrap_or(""))?))
}

async fn gen_strategy(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    Ok(Json(strategy::generate_channel_strategy(&pid, &str_list(&b, "channel_ids").unwrap_or_default()).map_err(|e| ApiError::ise(format!("Lỗi tạo chiến lược: {e}")))?))
}
async fn get_strategy(AxumPath(pid): AxumPath<String>) -> J {
    ok_id(&pid)?;
    Ok(Json(match strategy::load_channel_strategy(&pid) { Some(s) => s, None => strategy::generate_channel_strategy(&pid, &[])? }))
}

async fn gen_ideas(AxumPath(pid): AxumPath<String>, Query(q): Query<HashMap<String, String>>) -> J {
    ok_id(&pid)?;
    Ok(Json(v(&ideas::generate_ideas_batch(&pid, q.get("count").and_then(|c| c.parse().ok()).unwrap_or(30)))))
}
async fn list_ideas(AxumPath(pid): AxumPath<String>, Query(q): Query<HashMap<String, String>>) -> J {
    ok_id(&pid)?;
    let all: Vec<_> = ct::list_content_ideas(&pid).into_iter().filter(|i| q.get("status").is_none_or(|s| &i.status == s)).collect();
    Ok(Json(v(&all)))
}
async fn update_idea(AxumPath((pid, id)): AxumPath<(String, String)>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    ideas::update_idea_status(&pid, &id, sget(&b, "status").as_deref(), b.get("priority").and_then(|p| p.as_i64())).map(|i| Json(v(&i))).ok_or_else(|| ApiError::nf(format!("Không tìm thấy ý tưởng {id}")))
}
async fn promote_idea(AxumPath((pid, id)): AxumPath<(String, String)>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    Ok(Json(ideas::promote_idea(&pid, &id, sget(&b, "product_id").as_deref(), &sget(&b, "character_name").unwrap_or_else(|| "LyLy".into()), &sget(&b, "voice_id").unwrap_or_else(|| "soc-nho".into()))
        .map_err(|e| if e.0 == 404 { e } else { ApiError::ise(format!("Lỗi thăng hạng ý tưởng: {}", e.1)) })?))
}
async fn schedule_idea(AxumPath((pid, id)): AxumPath<(String, String)>) -> J {
    ok_id(&pid)?;
    Ok(Json(ideas::schedule_idea_to_social(&pid, &id).await?))
}

async fn intel_chat(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    Ok(Json(chat::handle_intel_chat(&pid, &req_str(&b, "prompt")?, b.get("context")).await.map_err(|e| ApiError::ise(format!("Lỗi trợ lý Intelligence: {}", e.1)))?))
}
async fn execute_node(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let nt = req_str(&b, "node_type")?;
    Ok(Json(node_runner::execute_intel_node(&pid, &nt, b.get("params"), b.get("inputs")).await
        .map_err(|e| if e.0 == 400 || e.0 == 404 { e } else { ApiError::ise(format!("Lỗi thực thi node {nt}: {}", e.1)) })?))
}
async fn compliance_check(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    Ok(Json(v(&compliance::check_and_clean_script(&req_str(&b, "text")?, b.get("is_affiliate").and_then(|a| a.as_bool()).unwrap_or(true)))))
}

// ---------- /api/intel/* (formerly canned JSON): real logic against the active project

fn active(b: &Value) -> String { sget(b, "project").or_else(|| sget(b, "projectId")).unwrap_or_else(workspace::active_project_id) }

#[utoipa::path(post, path = "/api/intel/ideas", request_body = inline(Value), responses((status = 200, description = "Generate KOC ideas and video hooks", body = inline(Value))))]
pub(crate) async fn generate_ideas(Json(b): Json<Value>) -> J {
    let pid = active(&b);
    ok_id(&pid)?;
    let list = ideas::generate_ideas_batch(&pid, b.get("count").and_then(|c| c.as_u64()).unwrap_or(10) as usize);
    Ok(Json(json!({"ideas": list.iter().map(|i| json!({"id": i.id, "title": i.title, "hook": i.title, "score": i.priority as f64 * 2.0, "priority": i.priority, "status": i.status, "template_id": i.template_id, "platform_targets": i.platform_targets})).collect::<Vec<_>>()})))
}

#[utoipa::path(post, path = "/api/intel/script", request_body = inline(Value), responses((status = 200, description = "Generate scene-by-scene KOC script", body = inline(Value))))]
pub(crate) async fn generate_script(Json(b): Json<Value>) -> J {
    let pid = active(&b);
    ok_id(&pid)?;
    let prods = crate::intelligence::products::list_products(&pid);
    let product = sget(&b, "product_id").and_then(|i| prods.iter().find(|p| p.id == i).cloned()).or_else(|| prods.first().cloned())
        .ok_or_else(|| ApiError::nf("Dự án chưa có sản phẩm. Thêm sản phẩm trước khi sinh kịch bản."))?;
    let gt = sget(&b, "generator_type").unwrap_or_else(|| "koc_review".into());
    let r = generate_complete_script(&pid, &product, &ScriptOpts { generator_type: &gt, character_name: &sget(&b, "character_name").unwrap_or_else(|| "LyLy".into()), voice_id: &sget(&b, "voice_id").unwrap_or_else(|| "soc-nho".into()), ..Default::default() })?;
    let scenes: Vec<Value> = r["lines"].as_array().into_iter().flatten().enumerate().map(|(i, l)| json!({"scene": i + 1, "role": l["role"], "duration": l["duration_s"], "narration": l["text"]})).collect();
    Ok(Json(json!({"script": {"title": format!("Kịch bản review {}", product.name), "scenes": scenes}, "script_id": r["script_id"], "template_id": r["template_id"],
        "compliance_passed": r["compliance_passed"], "compliance_warnings": r["compliance_warnings"], "hook_lab": r["hook_lab"], "render_request": r["render_request"]})))
}

#[utoipa::path(post, path = "/api/intel/compliance", request_body = inline(Value), responses((status = 200, description = "Validate script policy compliance", body = inline(Value))))]
pub(crate) async fn check_compliance(Json(b): Json<Value>) -> J {
    let text = sget(&b, "text").unwrap_or_default();
    let r = compliance::check_and_clean_script(&text, b.get("is_affiliate").and_then(|a| a.as_bool()).unwrap_or(false));
    // project policy pack (strict/standard) runs alongside the commercial-ad rules
    let pid = active(&b);
    let pol = if safe_id(&pid).is_ok() { cp::check_texts(&[("text".to_string(), text.clone())], &cp::load_policy(&pid)) } else { json!({"passed": true, "violations": []}) };
    let compliant = r.passed && pol["passed"] == true;
    let mut violations: Vec<Value> = r.violations.iter().map(v).collect();
    violations.extend(pol["violations"].as_array().cloned().unwrap_or_default());
    Ok(Json(json!({"compliant": compliant, "violations": violations, "clean_text": r.modified_text, "warnings": r.warnings, "disclosure_added": r.disclosure_added, "policy": pol})))
}

// ---------- studio chat / script drafting used by the dashboard

/// Chat: routed through the intelligence assistant; also returns `action` (first suggested action) like the old stub.
pub(crate) async fn chat_with_assistant(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let msg = sget(&b, "message").or_else(|| sget(&b, "prompt")).unwrap_or_default();
    let mut r = chat::handle_intel_chat(&pid, &msg, b.get("context")).await?;
    let action = r["actions"].as_array().and_then(|a| a.first().cloned()).unwrap_or(Value::Null);
    r["action"] = action;
    Ok(Json(r))
}

pub(crate) async fn get_policy_packs_handler() -> Json<Value> {
    let path = workspace::shared_path(&["policy_packs.json"], false).unwrap_or_default();
    Json(crate::intelligence::util::read_json(&path).unwrap_or(json!([])))
}

pub(crate) async fn generate_script_ai(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let prompt = sget(&b, "prompt").unwrap_or_default();
    let sys = "Bạn là biên kịch phim hoạt hình thiếu nhi (3–7 tuổi). Viết kịch bản Markdown ngắn theo cảnh, mỗi cảnh gồm lời thoại nhân vật và mô tả hành động trong ngoặc. Nội dung an toàn, tích cực.";
    let real = std::env::var("GEMINI_API_KEY").map(|k| !k.is_empty()).unwrap_or(false);
    let text = crate::providers::llm().generate(&format!("Ý tưởng: {prompt}"), Some(sys), Some(&crate::intelligence::util::llm_model()), false).await.map_err(|e| ApiError::ise(e.to_string()))?;
    Ok(Json(json!({"mode": if real { "gemini" } else { "mock" }, "markdown": text})))
}

pub fn router() -> Router<AppState> {
    Router::new()
        // dashboard prefix (kept from Phase 0)
        .route("/data/projects/{project_id}/ai/script", post(generate_script_ai))
        .route("/data/projects/{project_id}/policy/check", post(check_project_policy))
        .route("/data/projects/{project_id}/policy", get(get_project_policy).put(save_project_policy))
        .route("/projects/{project_id}/ai/script", post(generate_script_ai))
        .route("/projects/{project_id}/ai/character", post(chat_with_assistant))
        .route("/projects/{project_id}/chat", post(chat_with_assistant))
        .route("/data/policy-packs", get(get_policy_packs_handler))
        // policy (Python data_routes parity)
        .route("/projects/{project_id}/policy", get(get_project_policy).put(save_project_policy))
        .route("/policy-packs", get(policy_packs))
        .route("/projects/{project_id}/policy/import-pack", post(import_pack))
        .route("/projects/{project_id}/policy/import", post(import_file))
        .route("/projects/{project_id}/policy/fetch-url", post(fetch_url))
        .route("/projects/{project_id}/policy/check", post(check_project_policy))
        // intel API
        .route("/intel/_ping", get(ping))
        .route("/projects/{project_id}/intel/analyze/{source_id}", post(analyze_video))
        .route("/projects/{project_id}/intel/blueprints", get(list_blueprints))
        .route("/projects/{project_id}/intel/blueprints/{blueprint_id}", get(get_blueprint))
        .route("/projects/{project_id}/intel/patterns", get(get_patterns))
        .route("/projects/{project_id}/intel/generate-script", post(generate_script_route))
        .route("/projects/{project_id}/intel/channels", get(list_channels).post(ingest_channel))
        .route("/projects/{project_id}/intel/channels/user-data", post(user_data))
        .route("/projects/{project_id}/intel/channels/{channel_id}", get(get_channel))
        .route("/projects/{project_id}/intel/strategy", get(get_strategy).post(gen_strategy))
        .route("/projects/{project_id}/intel/ideas/generate", post(gen_ideas))
        .route("/projects/{project_id}/intel/ideas", get(list_ideas))
        .route("/projects/{project_id}/intel/ideas/{idea_id}", axum::routing::patch(update_idea))
        .route("/projects/{project_id}/intel/ideas/{idea_id}/promote", post(promote_idea))
        .route("/projects/{project_id}/intel/ideas/{idea_id}/schedule", post(schedule_idea))
        .route("/projects/{project_id}/intel/chat", post(intel_chat))
        .route("/projects/{project_id}/intel/nodes/execute", post(execute_node))
        .route("/projects/{project_id}/intel/compliance/check", post(compliance_check))
        // formerly canned
        .route("/api/intel/ideas", post(generate_ideas))
        .route("/api/intel/script", post(generate_script))
        .route("/api/intel/compliance", post(check_compliance))
}
