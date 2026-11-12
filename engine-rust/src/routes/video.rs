//! Video routes (port of core/api/routes/video.py). Errors use FastAPI's `{"detail": ...}` shape.
use super::prelude::*;
use crate::video::{
    craft, episode,
    platforms::list_platforms,
    queue::queue,
    render::RenderRequest,
    templates::{self, validate_template_data},
    util::is_valid_slug,
};
use tokio_util::sync::CancellationToken;

type ApiErr = (StatusCode, Json<Value>);
type ApiRes<T = Json<Value>> = Result<T, ApiErr>;

fn err(code: StatusCode, msg: impl ToString) -> ApiErr {
    (code, Json(json!({ "detail": msg.to_string() })))
}
fn not_found(msg: impl ToString) -> ApiErr {
    err(StatusCode::NOT_FOUND, msg)
}
fn bad(msg: impl ToString) -> ApiErr {
    err(StatusCode::BAD_REQUEST, msg)
}
fn internal(msg: impl ToString) -> ApiErr {
    err(StatusCode::INTERNAL_SERVER_ERROR, msg)
}
fn tmpl(tid: &str) -> ApiRes<templates::TemplateMeta> {
    templates::get_template(tid).map_err(|_| not_found(format!("Template '{tid}' không tồn tại")))
}
fn project(pid: &str) -> ApiRes<()> {
    if is_valid_slug(pid) { Ok(()) } else { Err(bad("project_id không hợp lệ")) }
}

#[derive(Deserialize)]
pub(crate) struct TemplateFilter {
    category: Option<String>,
    platform: Option<String>,
}

#[utoipa::path(get, path = "/video/templates", responses((status = 200, description = "List HyperFrames video templates (TemplateMeta[])", body = inline(Value))))]
pub(crate) async fn get_video_templates(Query(f): Query<TemplateFilter>) -> Json<Value> {
    Json(json!(templates::list_templates(f.category.as_deref(), f.platform.as_deref())))
}

async fn ping() -> Json<Value> {
    Json(json!({"status": "ok", "module": "video"}))
}

async fn platforms() -> Json<Value> {
    Json(json!(list_platforms()))
}

async fn template_detail(AxumPath(tid): AxumPath<String>) -> ApiRes {
    Ok(Json(json!(tmpl(&tid)?)))
}

async fn validate_template(AxumPath(tid): AxumPath<String>, Json(data): Json<serde_json::Map<String, Value>>) -> ApiRes {
    let errors = validate_template_data(&tmpl(&tid)?, &data);
    Ok(Json(json!({"valid": errors.is_empty(), "errors": errors})))
}

async fn render_preview(AxumPath(tid): AxumPath<String>) -> ApiRes {
    tmpl(&tid)?;
    crate::video::render::render_template_preview(&tid, &CancellationToken::new()).await.map_err(|e| internal(format!("{e:#}")))?;
    Ok(Json(json!({"status": "ok", "preview_mp4": format!("/video/templates/{tid}/preview.mp4"), "preview_jpg": format!("/video/templates/{tid}/preview.jpg")})))
}

async fn preview_file(tid: String, name: &str, mime: &'static str) -> ApiRes<impl IntoResponse> {
    let dir = templates::template_dir(&tid).map_err(|_| not_found(format!("Template '{tid}' không tồn tại")))?;
    let f = dir.join(name);
    if !f.is_file() {
        crate::video::render::render_template_preview(&tid, &CancellationToken::new()).await.map_err(|e| not_found(format!("Chưa có {name}: {e:#}")))?;
    }
    let bytes = tokio::fs::read(&f).await.map_err(not_found)?;
    Ok(([(axum::http::header::CONTENT_TYPE, mime)], bytes))
}
async fn preview_mp4(AxumPath(tid): AxumPath<String>) -> ApiRes<impl IntoResponse> {
    preview_file(tid, "preview.mp4", "video/mp4").await
}
async fn preview_jpg(AxumPath(tid): AxumPath<String>) -> ApiRes<impl IntoResponse> {
    preview_file(tid, "preview.jpg", "image/jpeg").await
}

// ---- render queue ----

fn submit(pid: &str, req: RenderRequest) -> ApiRes {
    project(pid)?;
    queue().submit(req, pid).map(|j| Json(json!(j))).map_err(|e| bad(format!("{e:#}")))
}

async fn submit_render(AxumPath(pid): AxumPath<String>, Json(req): Json<RenderRequest>) -> ApiRes {
    submit(&pid, req)
}

/// Legacy `/video/render`: body carries project_id (defaults to the active project).
#[utoipa::path(post, path = "/video/render", request_body = inline(Value), responses((status = 200, description = "Enqueue a render job (RenderJob)", body = inline(Value))))]
pub(crate) async fn render_video(Json(req): Json<RenderRequest>) -> ApiRes {
    let pid = if req.project_id.is_empty() { workspace::active_project_id() } else { req.project_id.clone() };
    submit(&pid, req)
}

async fn get_render(AxumPath((pid, job)): AxumPath<(String, String)>) -> ApiRes {
    project(&pid)?;
    queue().get(&pid, &job).map(|j| Json(json!(j))).map_err(|_| not_found(format!("Job {job} không tồn tại")))
}

#[derive(Deserialize)]
pub(crate) struct StatusQ {
    status: Option<String>,
    project: Option<String>,
}

async fn list_renders(AxumPath(pid): AxumPath<String>, Query(q): Query<StatusQ>) -> ApiRes {
    project(&pid)?;
    Ok(Json(json!(queue().list(&pid, q.status.as_deref()))))
}

async fn list_renders_active(Query(q): Query<StatusQ>) -> ApiRes {
    let pid = q.project.clone().unwrap_or_else(workspace::active_project_id);
    project(&pid)?;
    Ok(Json(json!(queue().list(&pid, q.status.as_deref()))))
}

async fn cancel_render(AxumPath((pid, job)): AxumPath<(String, String)>) -> ApiRes {
    project(&pid)?;
    queue().cancel(&pid, &job).map(|j| Json(json!(j))).map_err(|_| not_found(format!("Job {job} không tồn tại")))
}

async fn render_events(AxumPath((pid, job)): AxumPath<(String, String)>) -> ApiRes<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    project(&pid)?;
    let stream = async_stream::stream! {
        loop {
            match queue().get(&pid, &job) {
                Ok(j) => {
                    let data = serde_json::to_string(&j).unwrap_or_default();
                    yield Ok(Event::default().event("progress").data(data.clone()));
                    if j.status == "done" || j.status == "failed" {
                        yield Ok(Event::default().event("completed").data(data));
                        break;
                    }
                }
                Err(_) => {
                    yield Ok(Event::default().event("error").data(json!({"error": format!("Job {job} not found")}).to_string()));
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    };
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

// ---- episode pipeline ----

#[derive(Deserialize)]
struct EpisodePlanReq {
    script: String,
    title: Option<String>,
}

async fn plan_episode(AxumPath((pid, eid)): AxumPath<(String, String)>, Json(req): Json<EpisodePlanReq>) -> ApiRes {
    project(&pid)?;
    episode::ep_dir(&pid, &eid).map_err(bad)?;
    let mut plan = episode::plan_episode(&req.script, &eid, req.title.as_deref().unwrap_or("Hoạt hình Sóc Nhỏ Bi & Bạn Koala"));
    plan.estimated_cost_usd = episode::estimate_episode_cost(&plan.scenes)["total_estimated_usd"].as_f64().unwrap_or(0.0);
    episode::save_plan(&pid, &plan).map_err(internal)?;
    Ok(Json(json!(plan)))
}

#[derive(Deserialize, Default)]
struct BuildReq {
    #[serde(default)]
    use_mock: bool,
}

async fn build_episode(AxumPath((pid, eid)): AxumPath<(String, String)>, body: Option<Json<BuildReq>>) -> ApiRes {
    project(&pid)?;
    episode::ep_dir(&pid, &eid).map_err(bad)?;
    let r = body.map(|b| b.0).unwrap_or_default();
    episode::build_episode(&pid, &eid, r.use_mock).await.map(Json).map_err(|e| internal(format!("{e:#}")))
}

async fn episode_status(AxumPath((pid, eid)): AxumPath<(String, String)>) -> ApiRes {
    project(&pid)?;
    episode::ep_dir(&pid, &eid).map_err(bad)?;
    episode::episode_status(&pid, &eid).map(Json).map_err(not_found)
}

// ---- craft ----

#[derive(Deserialize)]
struct HooksReq {
    topic: String,
    #[serde(default = "soc_nho")]
    character: String,
}
fn soc_nho() -> String {
    "soc-nho".into()
}
async fn craft_hooks(Json(r): Json<HooksReq>) -> Json<Value> {
    Json(craft::generate_hooks(&r.topic, &r.character))
}

#[derive(Deserialize)]
struct LintReq {
    script: String,
    duration_sec: Option<f64>,
    #[serde(default = "tiktok")]
    platform: String,
}
fn tiktok() -> String {
    "tiktok".into()
}
async fn craft_lint(Json(r): Json<LintReq>) -> Json<Value> {
    Json(craft::lint_script(&r.script, r.duration_sec, &r.platform))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/video/_ping", get(ping))
        .route("/video/platforms", get(platforms))
        .route("/video/templates", get(get_video_templates))
        .route("/video/templates/{tid}", get(template_detail))
        .route("/video/templates/{tid}/validate", post(validate_template))
        .route("/video/templates/{tid}/preview", post(render_preview))
        .route("/video/templates/{tid}/preview.mp4", get(preview_mp4))
        .route("/video/templates/{tid}/preview.jpg", get(preview_jpg))
        .route("/video/render", post(render_video))
        .route("/video/renders", get(list_renders_active))
        .route("/video/craft/hooks", post(craft_hooks))
        .route("/video/craft/lint", post(craft_lint))
        .route("/projects/{project_id}/video/render", post(submit_render))
        .route("/projects/{project_id}/video/render/{job_id}", get(get_render))
        .route("/projects/{project_id}/video/renders", get(list_renders))
        .route("/projects/{project_id}/video/render/{job_id}/cancel", post(cancel_render))
        .route("/projects/{project_id}/video/render/{job_id}/events", get(render_events))
        .route("/projects/{project_id}/video/episodes/{eid}/plan", post(plan_episode))
        .route("/projects/{project_id}/video/episodes/{eid}/build", post(build_episode))
        .route("/projects/{project_id}/video/episodes/{eid}/status", get(episode_status))
}
