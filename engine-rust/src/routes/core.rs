use super::prelude::*;

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
    
    match fs::read(&canonical_target) {
        Ok(bytes) => {
            let mime = guess_mime(&canonical_target);
            Ok((
                [(axum::http::header::CONTENT_TYPE, mime)],
                bytes,
            ))
        }
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

pub(crate) async fn get_assets(Query(q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "assets": [] }))
}

pub(crate) async fn get_library(Query(q): Query<ProjectQuery>) -> Json<Value> {
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
    let target = lib_dir.join(name);
    let rel_url = format!("/data/projects/{}/files/library/{}", project_id, name);
    Json(json!({ "ok": true, "name": name, "url": rel_url }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/info", get(get_workspace_info))
        .route("/api/v1/projects", get(list_projects))
        .route("/projects", get(get_active_project_legacy))
        .route("/data/projects", get(list_projects_data))
        .route("/data/projects/{project_id}", get(get_project_detail))
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
