use super::prelude::*;
use crate::intelligence::sources::{asr, downloader, jobs, localize, models::*, scenes, service};
use crate::intelligence::util::{pp, safe_id, ApiError, ApiResult};
use axum::{body::Body, extract::Request, http::header, response::Response};

type J = ApiResult<Json<Value>>;
fn ok_id(id: &str) -> ApiResult<()> { safe_id(id).map(|_| ()).map_err(|e| ApiError::bad(e.to_string())) }
fn v<T: Serialize>(x: &T) -> Value { serde_json::to_value(x).unwrap_or(Value::Null) }
fn rights_ok(r: &str) -> ApiResult<()> { if RIGHTS.contains(&r) { Ok(()) } else { Err(ApiError::unproc(format!("rights must be one of {RIGHTS:?}"))) } }
fn nf_source() -> ApiError { ApiError::nf("Không tìm thấy nguồn nội dung.") }

async fn ping() -> Json<Value> { Json(json!({"status": "ok", "module": "sources"})) }

/// JSON `{urls, rights, rights_note}` or multipart `file` (+ rights, rights_note).
async fn ingest(AxumPath(pid): AxumPath<String>, req: Request) -> J {
    ok_id(&pid)?;
    let ct = req.headers().get(header::CONTENT_TYPE).and_then(|c| c.to_str().ok()).unwrap_or("").to_string();
    if ct.starts_with("multipart/form-data") {
        let boundary = multer::parse_boundary(&ct).map_err(|e| ApiError::bad(e.to_string()))?;
        let mut mp = multer::Multipart::new(req.into_body().into_data_stream(), boundary);
        let (mut rights, mut note, mut file) = ("reference_only".to_string(), String::new(), None::<(String, std::path::PathBuf)>);
        let tmp = tempfile::tempdir().map_err(|e| ApiError::ise(e.to_string()))?;
        while let Some(mut f) = mp.next_field().await.map_err(|e| ApiError::bad(e.to_string()))? {
            match f.name().unwrap_or("") {
                "rights" => rights = f.text().await.map_err(|e| ApiError::bad(e.to_string()))?,
                "rights_note" => note = f.text().await.map_err(|e| ApiError::bad(e.to_string()))?,
                "file" => {
                    let name = f.file_name().unwrap_or("").to_string();
                    if name.is_empty() { continue; }
                    let ext = std::path::Path::new(&name).extension().and_then(|e| e.to_str()).filter(|e| e.chars().all(|c| c.is_ascii_alphanumeric())).unwrap_or("mp4").to_string();
                    let path = tmp.path().join(format!("upload.{ext}"));
                    let mut out = tokio::fs::File::create(&path).await.map_err(|e| ApiError::ise(e.to_string()))?;
                    while let Some(chunk) = f.chunk().await.map_err(|e| ApiError::bad(e.to_string()))? { tokio::io::AsyncWriteExt::write_all(&mut out, &chunk).await.map_err(|e| ApiError::ise(e.to_string()))?; }
                    file = Some((name, path));
                }
                _ => {}
            }
        }
        rights_ok(&rights)?;
        if let Some((name, path)) = file {
            let r = if rights == "reference_only" { "owned" } else { &rights };
            let n = if note.is_empty() { format!("Tải lên trực tiếp: {name}") } else { note };
            let item = downloader::ingest_local_file(&pid, &path, Some(&name), r, &n).await.map_err(|e| ApiError::ise(e.to_string()))?;
            let _ = scenes::detect_and_save_scenes(&pid, &item.id).await;
            return Ok(Json(json!({"type": "file", "source": item})));
        }
        return Err(ApiError::bad("Vui lòng cung cấp danh sách URLs hoặc tải lên tập tin video."));
    }
    let bytes = axum::body::to_bytes(req.into_body(), 1 << 20).await.map_err(|e| ApiError::bad(e.to_string()))?;
    let b: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    let urls: Vec<String> = b.get("urls").and_then(|u| u.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    if urls.is_empty() { return Err(ApiError::bad("Vui lòng cung cấp danh sách URLs hoặc tải lên tập tin video.")); }
    let rights = b.get("rights").and_then(|r| r.as_str()).unwrap_or("reference_only");
    rights_ok(rights)?;
    let ids = jobs::enqueue_ingest(&pid, &urls, rights, b.get("rights_note").and_then(|r| r.as_str()).unwrap_or(""));
    Ok(Json(json!({"type": "jobs", "job_ids": ids})))
}

async fn list(AxumPath(pid): AxumPath<String>, Query(q): Query<HashMap<String, String>>) -> J {
    ok_id(&pid)?;
    Ok(Json(v(&service::list_sources(&pid, q.get("platform").map(String::as_str), q.get("rights").map(String::as_str), q.get("q").map(String::as_str)))))
}
async fn job(AxumPath((_pid, id)): AxumPath<(String, String)>) -> J { jobs::get_job_status(&id).map(Json).ok_or_else(|| ApiError::nf("Không tìm thấy tiến trình tải.")) }
async fn get_one(AxumPath((pid, id)): AxumPath<(String, String)>) -> J { service::get_source(&pid, &id).map(|s| Json(v(&s))).ok_or_else(nf_source) }
async fn patch(AxumPath((pid, id)): AxumPath<(String, String)>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let rights = b.get("rights").and_then(|r| r.as_str()).ok_or_else(|| ApiError::unproc("field 'rights' is required"))?;
    let item = service::update_source_rights(&pid, &id, rights, b.get("rights_note").and_then(|n| n.as_str()), b.get("confirm_rights").and_then(|c| c.as_bool()).unwrap_or(false))?;
    Ok(Json(v(&item)))
}
async fn delete_one(AxumPath((pid, id)): AxumPath<(String, String)>) -> J {
    ok_id(&pid)?;
    if !service::delete_source_to_trash(&pid, &id) { return Err(ApiError::nf("Không tìm thấy nguồn nội dung để chuyển vào thùng rác.")); }
    Ok(Json(json!({"status": "ok", "message": format!("Nguồn {id} đã chuyển vào thùng rác raw/_trash/")})))
}

async fn file(AxumPath((pid, id, name)): AxumPath<(String, String, String)>) -> Result<Response, ApiError> {
    ok_id(&pid)?;
    let path = service::get_source_file_path(&pid, &id, &name).ok_or_else(|| ApiError::nf(format!("Không tìm thấy tập tin: {name}")))?;
    let mime = match path.extension().and_then(|e| e.to_str()).unwrap_or("") { "mp4" => "video/mp4", "jpg" | "jpeg" => "image/jpeg", "png" => "image/png", "wav" => "audio/wav", "vtt" => "text/vtt", "json" => "application/json", _ => "application/octet-stream" };
    let mut f = tokio::fs::File::open(&path).await.map_err(|e| ApiError::ise(e.to_string()))?;
    let stream = async_stream::stream! {
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            match tokio::io::AsyncReadExt::read(&mut f, &mut buf).await {
                Ok(0) => break,
                Ok(n) => yield Ok::<_, std::io::Error>(axum::body::Bytes::copy_from_slice(&buf[..n])),
                Err(e) => { yield Err(e); break; }
            }
        }
    };
    let mut r = Response::new(Body::from_stream(stream));
    r.headers_mut().insert(header::CONTENT_TYPE, mime.parse().unwrap());
    Ok(r)
}

async fn detect_scenes(AxumPath((pid, id)): AxumPath<(String, String)>) -> J {
    ok_id(&pid)?;
    Ok(Json(json!(scenes::detect_and_save_scenes(&pid, &id).await.map_err(|e| if e.0 == 404 { e } else { ApiError::ise(format!("Lỗi tách cảnh: {}", e.1)) })?)))
}

async fn localize_source(AxumPath((pid, id)): AxumPath<(String, String)>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let g = |k: &str, d: &str| b.get(k).and_then(|x| x.as_str()).unwrap_or(d).to_string();
    let platforms = b.get("platforms").and_then(|p| p.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_else(|| vec!["tiktok".into(), "reels".into(), "shorts".into()]);
    let r = localize::LocalizeReq { mode: g("mode", "R"), style: g("style", "faithful"), voice_id: g("voice_id", "soc-nho"), platforms, template_id: b.get("template_id").and_then(|t| t.as_str()).map(String::from) };
    Ok(Json(localize::run_localize_pipeline(&pid, &id, r).await?))
}
async fn localize_job(AxumPath((_pid, _id, job_id)): AxumPath<(String, String, String)>) -> J {
    ok_id(&job_id)?;
    localize::self_get(&format!("/video/status/{job_id}")).await.map(Json).ok_or_else(|| ApiError::nf(format!("Không tìm thấy render job {job_id}")))
}

fn lang(q: &HashMap<String, String>) -> ApiResult<String> {
    let l = q.get("lang").cloned().unwrap_or_else(|| "vi".into());
    if l == "vi" || l == "orig" { Ok(l) } else { Err(ApiError::unproc("lang must be 'orig' or 'vi'")) }
}
async fn get_transcript(AxumPath((pid, id)): AxumPath<(String, String)>, Query(q): Query<HashMap<String, String>>) -> J {
    ok_id(&pid)?; ok_id(&id)?;
    let l = lang(&q)?;
    crate::intelligence::util::read_json(&pp(&pid, &["raw", &id, &format!("transcript.{l}.json")], false)).map(Json).ok_or_else(|| ApiError::nf(format!("Chưa có transcript {l}")))
}
async fn put_transcript(AxumPath((pid, id)): AxumPath<(String, String)>, Query(q): Query<HashMap<String, String>>, Json(segs): Json<Vec<Value>>) -> J {
    ok_id(&pid)?; ok_id(&id)?;
    let l = lang(&q)?;
    crate::intelligence::util::write_json(&pp(&pid, &["raw", &id, &format!("transcript.{l}.json")], true), &segs)?;
    Ok(Json(json!({"status": "ok", "saved_segments": segs.len()})))
}
async fn get_script(AxumPath((pid, id)): AxumPath<(String, String)>, Query(q): Query<HashMap<String, String>>) -> J {
    ok_id(&pid)?; ok_id(&id)?;
    let style = q.get("style").cloned().unwrap_or_else(|| "faithful".into());
    ok_id(&style)?;
    let a = pp(&pid, &["content", "scripts", &format!("{id}.{style}.json")], false);
    let b = pp(&pid, &["raw", &id, &format!("script.{style}.json")], false);
    crate::intelligence::util::read_json(&a).or_else(|| crate::intelligence::util::read_json(&b)).map(Json).ok_or_else(|| ApiError::nf("Chưa có kịch bản cho style này."))
}
async fn put_script(AxumPath((pid, id)): AxumPath<(String, String)>, Query(q): Query<HashMap<String, String>>, Json(lines): Json<Vec<Value>>) -> J {
    ok_id(&pid)?; ok_id(&id)?;
    let style = q.get("style").cloned().unwrap_or_else(|| "faithful".into());
    ok_id(&style)?;
    crate::intelligence::util::write_json(&pp(&pid, &["content", "scripts", &format!("{id}.{style}.json")], true), &lines)?;
    Ok(Json(json!({"status": "ok", "saved_lines": lines.len()})))
}

/// Run ASR on demand (not in the Python API surface; handy for the dashboard and tests).
async fn run_transcribe(AxumPath((pid, id)): AxumPath<(String, String)>) -> J {
    ok_id(&pid)?;
    Ok(Json(json!(asr::run_asr(&pid, &id, "zh", None).await?)))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sources/_ping", get(ping))
        .route("/projects/{project_id}/sources", get(list).post(ingest))
        .route("/projects/{project_id}/sources/jobs/{job_id}", get(job))
        .route("/projects/{project_id}/sources/{source_id}", get(get_one).patch(patch).delete(delete_one))
        .route("/projects/{project_id}/sources/{source_id}/file/{*file_name}", get(file))
        .route("/projects/{project_id}/sources/{source_id}/scenes", post(detect_scenes))
        .route("/projects/{project_id}/sources/{source_id}/localize", post(localize_source))
        .route("/projects/{project_id}/sources/{source_id}/localize/{job_id}", get(localize_job))
        .route("/projects/{project_id}/sources/{source_id}/transcript", get(get_transcript).put(put_transcript))
        .route("/projects/{project_id}/sources/{source_id}/transcribe", post(run_transcribe))
        .route("/projects/{project_id}/sources/{source_id}/script", get(get_script).put(put_script))
}
