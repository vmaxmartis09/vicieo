use super::prelude::*;
use super::graph::{api_err, ApiErr};

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Engine health status", body = inline(Value))
    )
)]
pub(crate) async fn health_check() -> Json<Value> {
    let root = workspace::repo_root();
    Json(json!({
        "status": "healthy",
        "engine": "vicieo-rust-native-v2",
        "version": "0.2.0",
        "active_backend": "rust-axum",
        "repo_root": root.to_string_lossy(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/info",
    responses(
        (status = 200, description = "Workspace info", body = inline(Value))
    )
)]
pub(crate) async fn get_workspace_info() -> Json<Value> {
    let root = workspace::repo_root();
    let act_proj = workspace::active_project_id();
    Json(json!({
        "repo_root": root.to_string_lossy(),
        "active_project": act_proj,
        "runtime": "Rust Axum Native"
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects",
    responses(
        (status = 200, description = "List registered projects", body = inline(Value))
    )
)]
pub(crate) async fn list_projects() -> Json<Value> {
    let projects = workspace::list_projects();
    Json(json!({ "projects": projects }))
}

pub(crate) async fn get_active_project_legacy() -> Json<Value> {
    let act_proj = workspace::active_project_id();
    Json(json!({ "active_project": act_proj, "activeProject": act_proj }))
}

#[utoipa::path(
    get,
    path = "/data/projects",
    responses(
        (status = 200, description = "List detailed project profiles", body = inline(Value))
    )
)]
pub(crate) async fn list_projects_data() -> Json<Value> {
    let mut list = Vec::new();
    let projs = workspace::list_projects();
    for p in projs {
        let p_path = workspace::project_path(&p.id, &["project.json"], false).unwrap_or_default();
        if p_path.exists() {
            if let Ok(txt) = fs::read_to_string(&p_path) {
                if let Ok(mut val) = serde_json::from_str::<Value>(&txt) {
                    // Populate counts
                    let chars_count = workspace::project_path(&p.id, &["characters", "studio_characters.json"], false)
                        .ok()
                        .and_then(|cp| fs::read_to_string(cp).ok())
                        .and_then(|ctxt| serde_json::from_str::<Value>(&ctxt).ok())
                        .and_then(|cval| cval.get("characters").and_then(|c| c.as_array().map(|a| a.len())))
                        .unwrap_or(0);
                    let eps_count = workspace::project_path(&p.id, &["content", "episodes"], false)
                        .ok()
                        .and_then(|ep| fs::read_dir(ep).ok())
                        .map(|entries| entries.flatten().filter(|e| e.path().is_dir()).count())
                        .unwrap_or(0);

                    if let Some(obj) = val.as_object_mut() {
                        obj.insert("charactersCount".to_string(), json!(chars_count));
                        obj.insert("episodesCount".to_string(), json!(eps_count));
                        if !obj.contains_key("defaultAspectRatio") {
                            let ar = obj.get("default_aspect_ratio").cloned().unwrap_or(json!("9:16"));
                            obj.insert("defaultAspectRatio".to_string(), ar);
                        }
                    }
                    list.push(val);
                    continue;
                }
            }
        }
        list.push(json!({
            "id": p.id,
            "name": p.name.unwrap_or(p.id.clone()),
            "description": p.description.unwrap_or_default(),
            "charactersCount": 0,
            "episodesCount": 0,
            "defaultAspectRatio": "9:16"
        }));
    }
    Json(json!(list))
}

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "Project profile metadata", body = inline(Value))
    )
)]
pub(crate) async fn get_project_detail(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let p_path = workspace::project_path(&project_id, &["project.json"], false).unwrap_or_default();
    if p_path.exists() {
        if let Ok(txt) = fs::read_to_string(&p_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    Json(json!({ "id": project_id }))
}

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}/characters",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "List of project characters", body = inline(Value))
    )
)]
pub(crate) async fn get_project_characters(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let sc_path = workspace::project_path(&project_id, &["characters", "studio_characters.json"], false).unwrap_or_default();
    if sc_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sc_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                if let Some(chars) = val.get("characters") {
                    return Json(chars.clone());
                }
            }
        }
    }
    Json(json!([]))
}

#[utoipa::path(
    post,
    path = "/data/projects/{project_id}/characters",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    request_body = inline(Value),
    responses(
        (status = 200, description = "Save or create project character", body = inline(Value))
    )
)]
pub(crate) async fn save_project_character(AxumPath(project_id): AxumPath<String>, Json(body): Json<Value>) -> Json<Value> {
    let sc_path = workspace::project_path(&project_id, &["characters", "studio_characters.json"], true).unwrap_or_default();
    let mut current_chars: Vec<Value> = Vec::new();
    if sc_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sc_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                if let Some(arr) = val.get("characters").and_then(|c| c.as_array()) {
                    current_chars = arr.clone();
                }
            }
        }
    }
    current_chars.push(body.clone());
    let _ = fs::write(&sc_path, serde_json::to_string_pretty(&json!({ "characters": current_chars })).unwrap_or_default());
    Json(body)
}

pub(crate) async fn save_project_character_put(AxumPath((project_id, char_id)): AxumPath<(String, String)>, Json(body): Json<Value>) -> Json<Value> {
    let sc_path = workspace::project_path(&project_id, &["characters", "studio_characters.json"], true).unwrap_or_default();
    let mut current_chars: Vec<Value> = Vec::new();
    if sc_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sc_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                if let Some(arr) = val.get("characters").and_then(|c| c.as_array()) {
                    current_chars = arr.clone();
                }
            }
        }
    }
    let mut found = false;
    for c in &mut current_chars {
        if c.get("id").and_then(|v| v.as_str()) == Some(&char_id) {
            *c = body.clone();
            found = true;
            break;
        }
    }
    if !found {
        current_chars.push(body.clone());
    }
    let _ = fs::write(&sc_path, serde_json::to_string_pretty(&json!({ "characters": current_chars })).unwrap_or_default());
    Json(body)
}

pub(crate) async fn delete_project_character(AxumPath((project_id, char_id)): AxumPath<(String, String)>) -> Json<Value> {
    let sc_path = workspace::project_path(&project_id, &["characters", "studio_characters.json"], false).unwrap_or_default();
    if sc_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sc_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                if let Some(arr) = val.get("characters").and_then(|c| c.as_array()) {
                    let next: Vec<Value> = arr.iter().filter(|c| c.get("id").and_then(|v| v.as_str()) != Some(&char_id)).cloned().collect();
                    let _ = fs::write(&sc_path, serde_json::to_string_pretty(&json!({ "characters": next })).unwrap_or_default());
                }
            }
        }
    }
    Json(json!({ "ok": true }))
}

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}/channels",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "List of social channels for publishing", body = inline(Value))
    )
)]
pub(crate) async fn get_project_channels(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    // Try publishing/accounts.json first, then publishing/schedule.json
    let acc_path = workspace::project_path(&project_id, &["publishing", "accounts.json"], false).unwrap_or_default();
    if acc_path.exists() {
        if let Ok(txt) = fs::read_to_string(&acc_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    let sched_path = workspace::project_path(&project_id, &["publishing", "schedule.json"], false).unwrap_or_default();
    if sched_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sched_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                if let Some(chans) = val.get("channels") {
                    return Json(chans.clone());
                }
            }
        }
    }
    Json(json!([]))
}

#[utoipa::path(
    put,
    path = "/data/projects/{project_id}/channels",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    request_body = inline(Value),
    responses(
        (status = 200, description = "Save social channels for publishing", body = inline(Value))
    )
)]
pub(crate) async fn save_project_channels(AxumPath(project_id): AxumPath<String>, Json(body): Json<Value>) -> Json<Value> {
    let acc_path = workspace::project_path(&project_id, &["publishing", "accounts.json"], true).unwrap_or_default();
    let _ = fs::write(acc_path, serde_json::to_string_pretty(&body).unwrap_or_default());
    Json(body)
}

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}/library",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "List library assets (episodes, scripts, images, videos)", body = inline(Value))
    )
)]
pub(crate) async fn get_project_library(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let mut episodes: Vec<Value> = Vec::new();
    let mut scripts: Vec<Value> = Vec::new();
    let mut images: Vec<Value> = Vec::new();
    let mut videos: Vec<Value> = Vec::new();
    let mut audios: Vec<Value> = Vec::new();

    let proj_root = workspace::project_path(&project_id, &[], false).unwrap_or_default();

    // 1. Scan assets/<hash>.meta.json
    let assets_dir = proj_root.join("assets");
    if assets_dir.exists() {
        if let Ok(entries) = fs::read_dir(&assets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.to_string_lossy().ends_with(".meta.json") {
                    if let Ok(txt) = fs::read_to_string(&path) {
                        if let Ok(m) = serde_json::from_str::<Value>(&txt) {
                            if let Some(uri) = m.get("uri").and_then(|u| u.as_str()) {
                                let target_file = proj_root.join(uri);
                                if target_file.is_file() {
                                    let kind = m.get("kind").and_then(|k| k.as_str()).unwrap_or("");
                                    let id = m.get("id").and_then(|i| i.as_str()).unwrap_or(uri);
                                    let url = format!("/data/projects/{}/files/{}", project_id, uri);
                                    let prov = m.get("provenance").cloned().unwrap_or(json!({}));
                                    let meta = m.get("meta").cloned().unwrap_or(json!({}));
                                    let item = json!({
                                        "id": id,
                                        "name": target_file.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                                        "url": url,
                                        "source": "artifact",
                                        "aspect": meta.get("aspect"),
                                        "durationSec": meta.get("duration"),
                                        "provider": prov.get("provider"),
                                        "model": prov.get("model"),
                                        "createdAt": prov.get("createdAt")
                                    });
                                    if kind == "video" {
                                        videos.push(item);
                                    } else if kind == "image" {
                                        images.push(item);
                                    } else if kind == "audio" {
                                        audios.push(item);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Scan content/episodes
    let ep_dir = proj_root.join("content").join("episodes");
    if ep_dir.exists() {
        if let Ok(entries) = fs::read_dir(&ep_dir) {
            let mut dirs: Vec<_> = entries.flatten().filter(|e| e.path().is_dir()).collect();
            dirs.sort_by_key(|a| a.file_name());
            for entry in dirs {
                let ep_path = entry.path();
                let ep_id = entry.file_name().to_string_lossy().to_string();
                let mut ep_videos: Vec<Value> = Vec::new();
                let mut ep_images: Vec<Value> = Vec::new();
                let mut title = ep_id.clone();

                if let Ok(files) = fs::read_dir(&ep_path) {
                    for f_entry in files.flatten() {
                        let fp = f_entry.path();
                        let fname = f_entry.file_name().to_string_lossy().to_string();
                        let ext = fp.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                        if ext == "md" {
                            if let Ok(txt) = fs::read_to_string(&fp) {
                                if fname.to_uppercase().starts_with("KICH") {
                                    for line in txt.lines() {
                                        if line.starts_with("# ") {
                                            title = line.trim_start_matches("# ").trim().to_string();
                                            break;
                                        }
                                    }
                                }
                                scripts.push(json!({
                                    "id": format!("{}/{}", ep_id, fname),
                                    "name": fname,
                                    "episodeId": ep_id,
                                    "url": format!("/data/projects/{}/files/content/episodes/{}/{}", project_id, ep_id, fname),
                                    "preview": txt.chars().take(320).collect::<String>(),
                                }));
                            }
                        } else if ext == "mp4" || ext == "mov" || ext == "webm" {
                            let item = json!({
                                "id": format!("{}/{}", ep_id, fname),
                                "name": fname,
                                "url": format!("/data/projects/{}/files/content/episodes/{}/{}", project_id, ep_id, fname),
                                "source": "episode",
                                "episodeId": ep_id
                            });
                            ep_videos.push(item.clone());
                            videos.push(item);
                        } else if ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "webp" {
                            let item = json!({
                                "id": format!("{}/{}", ep_id, fname),
                                "name": fname,
                                "url": format!("/data/projects/{}/files/content/episodes/{}/{}", project_id, ep_id, fname),
                                "source": "episode",
                                "episodeId": ep_id
                            });
                            ep_images.push(item.clone());
                            images.push(item);
                        }
                    }
                }

                episodes.push(json!({
                    "id": ep_id,
                    "title": title,
                    "videoCount": ep_videos.len(),
                    "imageCount": ep_images.len(),
                    "scriptCount": 1
                }));
            }
        }
    }

    // 3. Scan products for cutout images
    let prod_dir = proj_root.join("products");
    if prod_dir.exists() {
        if let Ok(entries) = fs::read_dir(&prod_dir) {
            for entry in entries.flatten() {
                let cutout_dir = entry.path().join("images").join("cutout");
                if cutout_dir.exists() {
                    if let Ok(cfiles) = fs::read_dir(&cutout_dir) {
                        for cf in cfiles.flatten() {
                            let cfp = cf.path();
                            if cfp.is_file() {
                                let fname = cf.file_name().to_string_lossy().to_string();
                                let rel = cfp.strip_prefix(&proj_root).unwrap_or(&cfp).to_string_lossy().to_string();
                                images.push(json!({
                                    "id": format!("prod_{}", fname),
                                    "name": fname,
                                    "url": format!("/data/projects/{}/files/{}", project_id, rel),
                                    "source": "product"
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    Json(json!({
        "episodes": episodes,
        "scripts": scripts,
        "images": images,
        "videos": videos,
        "audios": audios
    }))
}

pub(crate) async fn get_script_data(AxumPath((project_id, episode_id)): AxumPath<(String, String)>) -> Json<Value> {
    let script_path = workspace::project_path(&project_id, &["content", "episodes", &episode_id, "script.json"], false).unwrap_or_default();
    if script_path.exists() {
        if let Ok(txt) = fs::read_to_string(&script_path) {
            if let Ok(script_val) = serde_json::from_str::<Value>(&txt) {
                let mut shots: Vec<Value> = Vec::new();
                if let Some(scenes) = script_val.get("scenes").and_then(|s| s.as_array()) {
                    for sc in scenes {
                        let sc_idx = sc.get("index").or_else(|| sc.get("sceneNumber")).and_then(|i| i.as_i64()).unwrap_or(1);
                        let sc_title = sc.get("title").and_then(|t| t.as_str()).unwrap_or("");
                        if let Some(sh_arr) = sc.get("shots").and_then(|s| s.as_array()) {
                            for sh in sh_arr {
                                let sh_id = sh.get("id").and_then(|i| i.as_str()).unwrap_or("");
                                let dia = sh.get("dialogue").and_then(|d| d.as_str()).unwrap_or("");
                                let spk = sh.get("speaker").and_then(|s| s.as_str()).unwrap_or("");
                                shots.push(json!({
                                    "id": sh_id,
                                    "scene": sc_idx,
                                    "sceneTitle": sc_title,
                                    "dialogue": dia,
                                    "speaker": spk,
                                    "characterId": if spk.is_empty() { Value::Null } else { Value::String(spk.to_string()) }
                                }));
                            }
                        }
                    }
                }
                return Json(json!({
                    "script": script_val,
                    "facets": [
                        {"id": "fulltext", "label": "Toàn văn kịch bản", "type": "text"},
                        {"id": "summary", "label": "Tóm tắt nội dung", "type": "text"},
                        {"id": "lesson", "label": "Bài học cốt lõi", "type": "text"},
                        {"id": "mantra", "label": "Câu thần chú", "type": "text"},
                        {"id": "dialogue_all", "label": "Lời thoại toàn tập", "type": "text"},
                        {"id": "environment", "label": "Prompt khóa bối cảnh", "type": "text"},
                        {"id": "scenes", "label": "Danh sách cảnh / shot (scenes)", "type": "scenes"},
                        {"id": "script", "label": "Toàn bộ kịch bản có cấu trúc", "type": "script"}
                    ],
                    "shotFacets": [
                        {"id": "dialogue", "label": "Lời thoại"},
                        {"id": "imagePrompt", "label": "Image prompt"},
                        {"id": "videoPrompt", "label": "Video prompt"},
                        {"id": "action", "label": "Hành động trên hình"},
                        {"id": "expression", "label": "Biểu cảm"}
                    ],
                    "shots": shots
                }));
            }
        }
    }
    Json(json!({
        "script": null,
        "facets": [],
        "shotFacets": [],
        "shots": []
    }))
}

pub(crate) async fn serve_project_file(
    AxumPath((project_id, file_path)): AxumPath<(String, String)>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    if !is_valid_slug(&project_id) {
        return Err((StatusCode::BAD_REQUEST, "Invalid project id"));
    }
    let base = match workspace::project_path(&project_id, &[], false) {
        Ok(b) => b,
        Err(_) => return Err((StatusCode::NOT_FOUND, "Project directory not found")),
    };
    let canonical_base = match base.canonicalize() {
        Ok(b) => b,
        Err(_) => return Err((StatusCode::NOT_FOUND, "Project directory not found")),
    };
    let target = canonical_base.join(&file_path);
    let canonical_target = match target.canonicalize() {
        Ok(t) => t,
        Err(_) => return Err((StatusCode::NOT_FOUND, "File not found")),
    };
    if !canonical_target.starts_with(&canonical_base) || !canonical_target.is_file() {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }
    
    // ServeFile streams from disk and honours Range (video seeking) + Content-Length.
    use tower::ServiceExt;
    let mime: mime_type::Mime = super::guess_mime(&canonical_target).parse().unwrap_or(mime_type::APPLICATION_OCTET_STREAM);
    match tower_http::services::ServeFile::new_with_mime(&canonical_target, &mime).oneshot(req).await {
        Ok(r) => Ok(r.into_response()),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file")),
    }
}

pub(crate) async fn get_project_artifact(
    AxumPath((project_id, artifact_id)): AxumPath<(String, String)>,
) -> Result<Json<Value>, (StatusCode, &'static str)> {
    if !is_valid_slug(&project_id) {
        return Err((StatusCode::BAD_REQUEST, "Invalid project id"));
    }
    let assets_dir = workspace::project_path(&project_id, &["assets"], false)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to resolve assets path"))?;
    if !assets_dir.exists() {
        return Err((StatusCode::NOT_FOUND, "Artifact not found"));
    }

    if let Ok(entries) = fs::read_dir(&assets_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if fname.ends_with(".meta.json") {
                let matches_name = fname.starts_with(&artifact_id) 
                    || fname.trim_end_matches(".meta.json").starts_with(&artifact_id);
                if matches_name {
                    if let Ok(txt) = fs::read_to_string(&p) {
                        if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                            return Ok(Json(val));
                        }
                    }
                }
                // Also check inner "id"
                if let Ok(txt) = fs::read_to_string(&p) {
                    if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                        if val.get("id").and_then(|s| s.as_str()) == Some(&artifact_id) {
                            return Ok(Json(val));
                        }
                    }
                }
            }
        }
    }

    Err((StatusCode::NOT_FOUND, "Artifact not found"))
}

pub(crate) async fn get_characters(Query(q): Query<ProjectQuery>) -> Json<Value> {
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    let char_dir = workspace::project_path(&proj, &["characters"], false).unwrap_or_default();
    let mut characters = Vec::new();
    if char_dir.exists() {
        if let Ok(entries) = fs::read_dir(char_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let meta_path = entry.path().join("profile.json");
                    if meta_path.exists() {
                        if let Ok(txt) = fs::read_to_string(&meta_path) {
                            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                                characters.push(val);
                                continue;
                            }
                        }
                    }
                    characters.push(json!({
                        "id": entry.file_name().to_string_lossy(),
                        "name": entry.file_name().to_string_lossy()
                    }));
                }
            }
        }
    }
    Json(json!({ "characters": characters }))
}

pub(crate) async fn save_character(Query(q): Query<ProjectQuery>, Json(body): Json<Value>) -> Json<Value> {
    let id = body.get("id").and_then(|v| v.as_str()).unwrap_or("char_custom");
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    let dir = workspace::project_path(&proj, &["characters", id], true).unwrap_or_default();
    let _ = fs::write(dir.join("profile.json"), serde_json::to_string_pretty(&body).unwrap_or_default());
    Json(json!({ "ok": true, "id": id }))
}

pub(crate) async fn get_assets(Query(_q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "assets": [] }))
}

pub(crate) async fn get_library(Query(_q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "items": [] }))
}

// Social Handlers

pub(crate) async fn get_active_project_handler() -> Json<Value> {
    let pid = workspace::active_project_id();
    Json(json!({
        "active_project": pid,
        "activeProject": pid,
    }))
}

pub(crate) async fn set_active_project_handler(Json(payload): Json<Value>) -> Json<Value> {
    let pid = payload.get("projectId")
        .or_else(|| payload.get("active_project"))
        .or_else(|| payload.get("activeProject"))
        .and_then(|v| v.as_str())
        .unwrap_or("soc-nho-tv");
    let _ = workspace::set_active_project(pid);
    Json(json!({
        "status": "ok",
        "active_project": pid,
        "activeProject": pid,
    }))
}

pub(crate) async fn create_project_episode(
    AxumPath(project_id): AxumPath<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let title = payload.get("title").and_then(|t| t.as_str()).unwrap_or("Tập mới");
    let ep_id = format!("ep_{}", uuid::Uuid::new_v4().to_string().replace('-', "")[..8].to_lowercase());
    let ep_dir = workspace::project_path(&project_id, &["content", "episodes", &ep_id], true).unwrap_or_default();
    let meta = json!({
        "id": ep_id,
        "title": title,
        "status": "draft",
        "createdAt": chrono::Utc::now().to_rfc3339()
    });
    let _ = fs::write(ep_dir.join("meta.json"), serde_json::to_string_pretty(&meta).unwrap_or_default());
    Json(json!({ "id": ep_id, "title": title }))
}

pub(crate) async fn save_project_script(
    AxumPath(project_id): AxumPath<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let title = payload.get("title").and_then(|t| t.as_str()).unwrap_or("Kịch bản mới");
    let content = payload.get("content").and_then(|t| t.as_str()).unwrap_or("");
    let ep_id = payload.get("episodeId").and_then(|t| t.as_str()).unwrap_or("ep_default");
    let ep_dir = workspace::project_path(&project_id, &["content", "episodes", ep_id], true).unwrap_or_default();
    let _ = fs::write(ep_dir.join("script.txt"), content);
    let script_json = json!({
        "title": title,
        "episodeId": ep_id,
        "content": content,
        "updatedAt": chrono::Utc::now().to_rfc3339()
    });
    let _ = fs::write(ep_dir.join("script.json"), serde_json::to_string_pretty(&script_json).unwrap_or_default());
    Json(json!({ "episodeId": ep_id, "name": title }))
}

pub(crate) async fn upload_library_asset(
    AxumPath(project_id): AxumPath<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let name = payload.get("name").and_then(|n| n.as_str()).unwrap_or("asset.bin");
    let lib_dir = workspace::project_path(&project_id, &["library"], true).unwrap_or_default();
    let _target = lib_dir.join(name);
    let rel_url = format!("/data/projects/{}/files/library/{}", project_id, name);
    Json(json!({ "ok": true, "name": name, "url": rel_url }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/info", get(get_workspace_info))
        .route("/api/v1/projects", get(list_projects))
        .route("/projects", get(get_active_project_legacy))
        .route("/data/projects", get(list_projects_data).post(create_data_project))
        .route("/data/projects/{project_id}/characters/{character_id}/images", post(upload_character_image))
        .route("/data/projects/{project_id}/characters/{character_id}/images/{file}", delete(delete_character_image))
        .route("/data/voices/{voice_id}/samples", post(upload_voice_sample))
        .route("/data/projects/{project_id}/episodes/{episode_id}/shot-pack", get(get_shot_pack))
        .route("/data/projects/{project_id}/ai/character", post(ai_character))
        .route("/data/projects/{project_id}", get(get_project_detail).put(put_data_project))
        .route("/data/projects/{project_id}/characters", get(get_project_characters).post(save_project_character))
        .route("/data/projects/{project_id}/characters/{character_id}", put(save_project_character_put).delete(delete_project_character))
        .route("/data/projects/{project_id}/channels", get(get_project_channels).put(save_project_channels))
        .route("/projects/active", get(get_active_project_handler).put(set_active_project_handler).post(set_active_project_handler))
        .route("/data/projects/{project_id}/episodes", post(create_project_episode))
        .route("/data/projects/{project_id}/scripts", post(save_project_script))
        .route("/data/projects/{project_id}/library/upload", post(upload_library_asset))
        .route("/data/projects/{project_id}/library", get(get_project_library))
        .route("/projects/{project_id}/library", get(get_project_library))
        .route("/data/projects/{project_id}/script-data/{episode_id}", get(get_script_data))
        .route("/projects/{project_id}/script-data/{episode_id}", get(get_script_data))
        .route("/projects/{project_id}/files/{*file_path}", get(serve_project_file))
        .route("/data/projects/{project_id}/files/{*file_path}", get(serve_project_file))
        .route("/projects/{project_id}/artifacts/{artifact_id}", get(get_project_artifact))
        .route("/data/projects/{project_id}/artifacts/{artifact_id}", get(get_project_artifact))
        .nest_service("/data", tower_http::services::ServeDir::new(workspace::data_root()))
        .route("/api/data/characters", get(get_characters).post(save_character))
        .route("/api/data/assets", get(get_assets))
        .route("/api/data/library", get(get_library))
}

/// Create a project folder + project.json and register it in data/workspace.json (port of data_routes.create_data_project).
pub(crate) async fn create_data_project(Json(b): Json<Value>) -> Result<Json<Value>, ApiErr> {
    let pid = b["id"].as_str().unwrap_or("").trim().to_string();
    let valid = !pid.is_empty() && pid.chars().next().is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && pid.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
    if !valid {
        return Err(api_err(StatusCode::BAD_REQUEST, "id dự án chỉ gồm a-z, 0-9, _ và -"));
    }
    let dir = workspace::data_root().join("projects").join(&pid);
    if dir.exists() {
        return Err(api_err(StatusCode::CONFLICT, "Dự án đã tồn tại"));
    }
    for sub in ["characters", "content/episodes", "assets", "library", "output", "publishing"] {
        fs::create_dir_all(dir.join(sub)).map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    let name = b["name"].as_str().map(str::to_string).unwrap_or_else(|| pid.clone());
    let description = b.get("description").cloned().unwrap_or(json!(""));
    let project = json!({
        "id": pid, "name": name, "description": description, "version": "1.0.0",
        "language": b.get("language").cloned().unwrap_or(json!("vi")),
        "target_platforms": b.get("targetPlatforms").cloned().unwrap_or(json!(["youtube", "tiktok"])),
        "default_aspect_ratio": b.get("defaultAspectRatio").cloned().unwrap_or(json!("9:16")),
        "default_character": "", "default_voice_id": "", "characters": {}
    });
    fs::write(dir.join("project.json"), serde_json::to_string_pretty(&project).unwrap_or_default())
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let ws_path = workspace::data_root().join("workspace.json");
    let mut ws: Value = fs::read_to_string(&ws_path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(json!({}));
    if let Some(list) = ws["projects"].as_array_mut() {
        list.push(json!({"id": pid, "name": project["name"], "description": project["description"],
            "path": format!("projects/{pid}"), "status": "active", "created_at": chrono::Utc::now().date_naive().to_string()}));
        let _ = fs::write(&ws_path, serde_json::to_string_pretty(&ws).unwrap_or_default());
    }
    Ok(Json(project))
}

/// Update editable project fields (port of data_routes.put_data_project).
pub(crate) async fn put_data_project(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> Result<Json<Value>, ApiErr> {
    if !is_valid_slug(&pid) {
        return Err(api_err(StatusCode::BAD_REQUEST, "Invalid project id"));
    }
    let path = workspace::data_root().join("projects").join(&pid).join("project.json");
    let mut p: Value = fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok())
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "Không tìm thấy dự án"))?;
    for (src, dst) in [("name", "name"), ("description", "description"), ("language", "language"), ("targetPlatforms", "target_platforms"),
        ("defaultAspectRatio", "default_aspect_ratio"), ("defaultVoiceId", "default_voice_id")] {
        if let Some(v) = b.get(src) {
            p[dst] = v.clone();
        }
    }
    fs::write(&path, serde_json::to_string_pretty(&p).unwrap_or_default()).map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(p))
}

const IMAGE_EXT: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

/// Keep only [A-Za-z0-9._-] from a client file name (port of data_routes._safe_name).
fn safe_name(name: &str) -> String {
    let base = Path::new(name).file_name().and_then(|f| f.to_str()).unwrap_or("");
    let cleaned: String = base.chars().map(|c| if c.is_ascii_alphanumeric() || "._-".contains(c) { c } else { '_' }).collect();
    let t = cleaned.trim_matches(|c| c == '.' || c == '_').to_string();
    if t.is_empty() { "file".into() } else { t }
}

fn ext_of(name: &str) -> String {
    Path::new(name).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase()
}

fn decode_b64(b: &Value) -> Result<Vec<u8>, ApiErr> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(b["data"].as_str().unwrap_or(""))
        .map_err(|_| api_err(StatusCode::BAD_REQUEST, "Dữ liệu tệp (base64) không hợp lệ"))
}

/// Write `name` into `dir` without overwriting: adds _1, _2… before the extension.
fn write_unique(dir: &Path, name: &str, bytes: &[u8]) -> Result<String, ApiErr> {
    let io = |e: std::io::Error| api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    fs::create_dir_all(dir).map_err(io)?;
    let stem = Path::new(name).file_stem().and_then(|s| s.to_str()).unwrap_or("file").to_string();
    let ext = ext_of(name);
    let mut candidate = name.to_string();
    let mut n = 1;
    while dir.join(&candidate).exists() {
        candidate = if ext.is_empty() { format!("{stem}_{n}") } else { format!("{stem}_{n}.{ext}") };
        n += 1;
    }
    fs::write(dir.join(&candidate), bytes).map_err(io)?;
    Ok(candidate)
}

/// DNA folder of a character. Rust-created characters live only in characters/studio_characters.json,
/// so fall back to that list when project.json has no entry.
fn character_dna(pid: &str, cid: &str) -> Result<PathBuf, ApiErr> {
    let dir = workspace::data_root().join("projects").join(pid);
    let read = |f: &str| fs::read_to_string(dir.join(f)).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok());
    let from_project = read("project.json").and_then(|p| p["characters"].get(cid).cloned());
    let from_studio = read("characters/studio_characters.json")
        .and_then(|v| v["characters"].as_array().and_then(|a| a.iter().find(|c| c["id"] == cid).cloned()));
    let c = from_project.or(from_studio).ok_or_else(|| api_err(StatusCode::NOT_FOUND, "Không có nhân vật này"))?;
    let rel = c["dna_dir"].as_str().map(str::to_string).unwrap_or_else(|| format!("characters/{cid}"));
    Ok(dir.join(rel))
}

pub(crate) async fn upload_character_image(AxumPath((pid, cid)): AxumPath<(String, String)>, Json(b): Json<Value>) -> Result<Json<Value>, ApiErr> {
    let dna = character_dna(&pid, &cid)?;
    let name = safe_name(b["name"].as_str().unwrap_or("image.png"));
    if !IMAGE_EXT.contains(&ext_of(&name).as_str()) {
        return Err(api_err(StatusCode::BAD_REQUEST, "Chỉ nhận ảnh png/jpg/webp"));
    }
    let saved = write_unique(&dna, &name, &decode_b64(&b)?)?;
    Ok(Json(json!({ "ok": true, "file": saved })))
}

/// Soft delete: moves the image into _removed/ so it can be restored (port of delete_character_image).
pub(crate) async fn delete_character_image(AxumPath((pid, cid, file)): AxumPath<(String, String, String)>) -> Result<Json<Value>, ApiErr> {
    let dna = character_dna(&pid, &cid)?;
    let name = safe_name(&file);
    let src = dna.join(&name);
    if !src.is_file() || !IMAGE_EXT.contains(&ext_of(&name).as_str()) {
        return Err(api_err(StatusCode::NOT_FOUND, "Không có ảnh này"));
    }
    let trash = dna.join("_removed");
    fs::create_dir_all(&trash).map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    fs::rename(&src, trash.join(&name)).map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let side = dna.join("studio.json");
    if let Some(mut sc) = fs::read_to_string(&side).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
        if sc["primaryImage"] == name {
            sc.as_object_mut().map(|o| o.remove("primaryImage"));
            let _ = fs::write(&side, serde_json::to_string_pretty(&sc).unwrap_or_default());
        }
    }
    Ok(Json(json!({ "ok": true })))
}

pub(crate) async fn upload_voice_sample(AxumPath(vid): AxumPath<String>, Json(b): Json<Value>) -> Result<Json<Value>, ApiErr> {
    if !is_valid_slug(&vid) {
        return Err(api_err(StatusCode::BAD_REQUEST, "Invalid voice id"));
    }
    let name = safe_name(b["name"].as_str().unwrap_or("sample.mp3"));
    if !["mp3", "wav"].contains(&ext_of(&name).as_str()) {
        return Err(api_err(StatusCode::BAD_REQUEST, "Chỉ nhận mp3/wav"));
    }
    let dir = workspace::data_root().join("shared").join("voice_vault").join(&vid);
    let saved = write_unique(&dir, &name, &decode_b64(&b)?)?;
    Ok(Json(json!({ "ok": true, "file": saved })))
}

const CHARACTER_SYSTEM: &str = "Bạn là biên kịch nhân vật cho kênh hoạt hình 3D trẻ em Sóc Nhỏ TV (phong cách Pixar). \
Từ mô tả của người dùng, trả về JSON hồ sơ nhân vật bằng tiếng Việt. promptLock là mô tả hình dáng \
bằng tiếng Anh, ngắn gọn, bất biến, dùng để giữ nhận diện khi sinh ảnh.";

/// Rule-based draft when no Gemini key or in mock mode (port of data_routes._mock_character).
fn mock_character(prompt: &str) -> Value {
    let text = prompt.trim();
    let name = regex::Regex::new(r"(?:tên(?: là)?|named?|call(?:ed)?)\s+([A-ZÀ-Ỹ][\wÀ-ỹ]*(?:\s+[A-ZÀ-Ỹ][\wÀ-ỹ]*){0,2})")
        .unwrap()
        .captures(text)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| {
            let head: String = text.split(',').next().unwrap_or("").split('.').next().unwrap_or("").chars().take(28).collect();
            let t = head.trim().split_whitespace().map(|w| {
                let mut cs = w.chars();
                cs.next().map(|f| f.to_uppercase().collect::<String>() + &cs.as_str().to_lowercase()).unwrap_or_default()
            }).collect::<Vec<_>>().join(" ");
            if t.is_empty() { "Nhân vật mới".into() } else { t }
        });
    let height = regex::Regex::new(r"(\d{2,3})\s*cm").unwrap().captures(text).and_then(|c| c[1].parse::<f64>().ok()).unwrap_or(120.0);
    let lower = text.to_lowercase();
    let traits: Vec<&str> = ["vui", "nhút nhát", "thông minh", "tốt bụng", "nghịch ngợm", "hiền", "dũng cảm", "tò mò"]
        .into_iter().filter(|w| lower.contains(w)).collect();
    let personality = if traits.is_empty() { "thân thiện, tò mò".to_string() } else { traits.join(", ") };
    let head: String = text.chars().take(200).collect();
    json!({
        "name": name, "role": "supporting", "heightCm": height,
        "description": text.chars().take(240).collect::<String>(), "personality": personality,
        "promptLock": format!("{head}, cute 3D Pixar-style woodland character, {} cm tall, soft studio lighting, plain light gray background", height as i64),
        "catchphrases": [], "habits": []
    })
}

/// Draft a character from a description: Gemini when a key exists and not in mock mode, else rules.
pub(crate) async fn ai_character(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> Result<Json<Value>, ApiErr> {
    if !is_valid_slug(&pid) {
        return Err(api_err(StatusCode::BAD_REQUEST, "Invalid project id"));
    }
    let prompt = b["prompt"].as_str().unwrap_or("").trim().to_string();
    if prompt.is_empty() {
        return Err(api_err(StatusCode::BAD_REQUEST, "Thiếu mô tả nhân vật"));
    }
    let mock = b["mockMode"].as_bool().unwrap_or(true);
    if !mock && crate::providers::key_vault::has_valid_key("gemini") {
        let text = crate::providers::llm().generate(&prompt, Some(CHARACTER_SYSTEM), None, true).await
            .map_err(|e| api_err(StatusCode::BAD_GATEWAY, e.to_string()))?;
        let draft = serde_json::from_str::<Value>(&text).map_err(|_| api_err(StatusCode::BAD_GATEWAY, "Mô hình trả về dữ liệu không hợp lệ"))?;
        return Ok(Json(json!({ "mode": "gemini", "draft": draft })));
    }
    Ok(Json(json!({ "mode": "mock", "draft": mock_character(&prompt) })))
}

/// ZIP for making clips by hand on a free web tool (port of data_routes.get_shot_pack).
pub(crate) async fn get_shot_pack(AxumPath((pid, eid)): AxumPath<(String, String)>, Query(q): Query<HashMap<String, String>>) -> Result<impl IntoResponse, ApiErr> {
    use std::io::Write;
    if !is_valid_slug(&pid) || !is_valid_slug(&eid) {
        return Err(api_err(StatusCode::BAD_REQUEST, "episode_id không hợp lệ"));
    }
    let script: Value = fs::read_to_string(workspace::project_path(&pid, &["content", "episodes", &eid, "script.json"], false).map_err(|e| api_err(StatusCode::BAD_REQUEST, e.to_string()))?)
        .ok().and_then(|t| serde_json::from_str(&t).ok())
        .ok_or_else(|| api_err(StatusCode::NOT_FOUND, "Không có kịch bản tập này"))?;
    let title = script["title"].as_str().unwrap_or(&eid).to_string();
    let all_shots: Vec<Value> = script["scenes"].as_array().into_iter().flatten().flat_map(|sc| sc["shots"].as_array().cloned().unwrap_or_default()).collect();
    let want: Vec<String> = q.get("shots").map(|s| s.split(',').map(|x| x.trim().to_uppercase()).filter(|x| !x.is_empty()).collect()).unwrap_or_default();
    if want.iter().any(|s| !(s.len() == 3 && s.starts_with('S') && s[1..].chars().all(|c| c.is_ascii_digit()))) {
        return Err(api_err(StatusCode::BAD_REQUEST, "shots chỉ nhận dạng S01,S07,..."));
    }
    let picked: Vec<Value> = if !want.is_empty() {
        all_shots.into_iter().filter(|sh| want.contains(&sh["id"].as_str().unwrap_or("").to_uppercase())).collect()
    } else if q.get("all").is_some_and(|a| a == "true") {
        all_shots
    } else {
        let p1: Vec<Value> = all_shots.iter().filter(|sh| sh["priority"] == 1).cloned().collect();
        if p1.is_empty() { all_shots.into_iter().take(5).collect() } else { p1 }
    };
    if picked.is_empty() {
        return Err(api_err(StatusCode::NOT_FOUND, "Không có shot nào khớp"));
    }
    let clips_dir = format!("data/projects/{pid}/content/episodes/{eid}/clips");
    let chars_dir = workspace::project_path(&pid, &["characters"], false).unwrap_or_default();
    let keyframes = latest_keyframes(&pid);
    let mut buf = std::io::Cursor::new(Vec::new());
    let mut manifest = vec![];
    {
        let mut z = zip::ZipWriter::new(&mut buf);
        let opt = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let io = |e: std::io::Error| api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        let zerr = |e: zip::result::ZipError| api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        for sh in &picked {
            let sid = sh["id"].as_str().unwrap_or("").to_string();
            let cid = sh["speakerId"].as_str().or_else(|| sh["characterIds"][0].as_str()).unwrap_or("").to_string();
            let (src, src_label) = match keyframes.get(&sid) {
                Some(p) => (Some(p.clone()), "keyframe".to_string()),
                None => (dna_reference_files(&chars_dir, &cid, 3).into_iter().next(), format!("dna:{cid}")),
            };
            let (img_name, img_src) = match src {
                Some(p) => {
                    let ext = ext_of(&p.to_string_lossy());
                    z.start_file(format!("{sid}.{ext}"), opt).map_err(zerr)?;
                    z.write_all(&fs::read(&p).map_err(io)?).map_err(io)?;
                    (format!("{sid}.{ext}"), src_label)
                }
                None => (String::new(), "none".to_string()),
            };
            let clip_sec = sh["clipDurationSec"].as_f64().unwrap_or(5.0);
            let txt = [
                format!("Shot {sid} · {title}"),
                format!("Khung giờ trong tập: {}s → {}s (shot {}s)", sh["startSec"], sh["endSec"], sh["durationSec"]),
                format!("Clip cần làm: {clip_sec} giây, 16:9, KHÔNG cần âm thanh (giọng OmniVoice ghép sau)"),
                format!("Ảnh khung đầu: {img_name} ({img_src})"),
                format!("Camera: {} · cỡ cảnh {}", sh["camera"].as_str().unwrap_or(""), sh["shotSize"].as_str().unwrap_or("")),
                String::new(), "PROMPT (dán vào web làm video):".into(), sh["videoPrompt"].as_str().unwrap_or("").into(),
                String::new(), "NEGATIVE:".into(), sh["negativePrompt"].as_str().unwrap_or("").into(), String::new(),
                format!("Lời thoại (để canh khẩu hình): {}: {}", sh["speaker"].as_str().unwrap_or(""), sh["dialogue"].as_str().unwrap_or("")),
            ].join("\n");
            z.start_file(format!("{sid}.txt"), opt).map_err(zerr)?;
            z.write_all(txt.as_bytes()).map_err(io)?;
            manifest.push(json!({"shotId": sid, "image": img_src, "clipDurationSec": clip_sec, "priority": sh["priority"]}));
        }
        let ids: Vec<&str> = manifest.iter().filter_map(|m| m["shotId"].as_str()).collect();
        let readme = format!(
            "GÓI SHOT — {title} ({eid})\n\n1. Mở web làm video có credit miễn phí (danh sách: ai/knowledge/free_ai_sources.md).\n\
2. Với mỗi shot: tải ảnh SXX.png làm khung đầu (image-to-video), dán prompt trong SXX.txt.\n\
3. Tắt âm thanh của clip nếu web cho chọn (giọng nhân vật do OmniVoice đọc, ghép sau).\n\
4. Tải clip về, đổi tên ĐÚNG theo shot: S07.mp4 (chữ S hoa, 2 chữ số).\n5. Thả vào: {clips_dir}/\n\
6. Chạy lại flow trong Studio: bộ dựng tự dùng clip có sẵn, shot không có clip thì giữ ảnh.\n\nShot trong gói: {}",
            ids.join(", "));
        z.start_file("README.txt", opt).map_err(zerr)?;
        z.write_all(readme.as_bytes()).map_err(io)?;
        z.start_file("manifest.json", opt).map_err(zerr)?;
        z.write_all(serde_json::to_string_pretty(&json!({"episodeId": eid, "shots": manifest})).unwrap_or_default().as_bytes()).map_err(io)?;
        z.finish().map_err(zerr)?;
    }
    let headers = [
        (axum::http::header::CONTENT_TYPE, "application/zip".to_string()),
        (axum::http::header::CONTENT_DISPOSITION, format!("attachment; filename=\"{eid}_shot-pack.zip\"")),
    ];
    Ok((headers, buf.into_inner()))
}

/// shotId -> newest keyframe image from each image_gen node's saved output snapshot (port of data_routes._latest_keyframes).
fn latest_keyframes(pid: &str) -> HashMap<String, PathBuf> {
    let root = workspace::project_path(pid, &[], false).unwrap_or_default();
    let mut out = HashMap::new();
    let graph: Value = fs::read_to_string(root.join("graph.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(json!({}));
    let gid = graph["id"].as_str().map(str::to_string).unwrap_or_else(|| format!("graph_{pid}"));
    for n in graph["nodes"].as_array().into_iter().flatten().filter(|n| n["type"] == "image_gen") {
        let nid = n["id"].as_str().unwrap_or("");
        let snap: Value = fs::read_to_string(root.join("runs").join("_node_outputs").join(&gid).join(format!("{nid}.json")))
            .ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(json!({}));
        for sc in snap["outputs"]["scenes"].as_array().into_iter().flatten() {
            let (Some(sid), Some(uri)) = (sc["shotId"].as_str(), sc["imageUri"].as_str()) else { continue };
            // canonicalize + prefix check: the snapshot path must stay inside the project folder
            let p = root.join(uri).canonicalize().ok();
            if let (Some(p), Some(r)) = (p, root.canonicalize().ok()) {
                if p.is_file() && p.starts_with(&r) {
                    out.insert(sid.to_string(), p);
                }
            }
        }
    }
    out
}

/// DNA reference images, best first (port of character_profile.dna_reference_files).
/// Profile `dnaReferences` win; otherwise *.png minus sheet/turnaround/back/_old/cutout, ranked _02, _01, then name.
pub(crate) fn dna_reference_files(chars_dir: &Path, cid: &str, limit: usize) -> Vec<PathBuf> {
    let d = chars_dir.join(cid);
    if !d.is_dir() {
        return vec![];
    }
    let profile: Option<Value> = fs::read_to_string(chars_dir.join("studio_characters.json")).ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| v["characters"].as_array().and_then(|a| a.iter().find(|c| c["id"] == cid).cloned()));
    let mut out: Vec<PathBuf> = vec![];
    for name in profile.as_ref().and_then(|p| p["dnaReferences"].as_array()).into_iter().flatten().filter_map(|n| n.as_str()) {
        let Some(base) = Path::new(name).file_name() else { continue };
        let p = d.join(base);
        if p.is_file() && !out.contains(&p) {
            out.push(p);
        }
    }
    if out.is_empty() {
        let not_ref = regex::Regex::new(r"(?i)(dna_sheet|sheet|turnaround|back|_old|cutout)").unwrap();
        let mut pngs: Vec<PathBuf> = fs::read_dir(&d).ok().into_iter().flatten().flatten().map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("png"))
            .filter(|p| !not_ref.is_match(p.file_stem().and_then(|s| s.to_str()).unwrap_or("")))
            .collect();
        let rank = |p: &PathBuf| {
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            let n = regex::Regex::new(r"_(\d+)$").unwrap().captures(&stem).and_then(|c| c[1].parse::<i64>().ok()).unwrap_or(99);
            (match n { 2 => 0, 1 => 1, _ => 2 }, p.file_name().map(|f| f.to_os_string()))
        };
        pngs.sort_by_key(|p| rank(p));
        out = pngs;
    }
    out.truncate(limit.max(1));
    out
}

#[cfg(test)]
mod dna_tests {
    use super::dna_reference_files;

    #[test]
    fn ranks_front_views_first_and_skips_sheets() {
        let root = tempfile::tempdir().unwrap();
        let d = root.path().join("hero");
        std::fs::create_dir_all(&d).unwrap();
        for f in ["hero_01.png", "hero_02.png", "hero_04.png", "hero_back.png", "dna_sheet.png"] {
            std::fs::write(d.join(f), b"x").unwrap();
        }
        let names: Vec<String> = dna_reference_files(root.path(), "hero", 3).iter().map(|p| p.file_name().unwrap().to_string_lossy().into_owned()).collect();
        assert_eq!(names, ["hero_02.png", "hero_01.png", "hero_04.png"]);
    }
}
