//! Editor routes (port of core/api/routes/editor.py): timeline, manual decisions, selective re-render.
use super::prelude::*;
use crate::video::editor::{Editor, EditorError};

type ApiRes = Result<Json<Value>, (StatusCode, Json<Value>)>;

fn map(e: EditorError) -> (StatusCode, Json<Value>) {
    let (c, m) = match e {
        EditorError::NotFound(m) => (StatusCode::NOT_FOUND, m),
        EditorError::Invalid(m) => (StatusCode::BAD_REQUEST, m),
        EditorError::Other(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Lỗi xử lý: {e:#}")),
    };
    (c, Json(json!({ "detail": m })))
}

async fn get_timeline(AxumPath((pid, eid)): AxumPath<(String, String)>) -> ApiRes {
    Editor::new(&pid).and_then(|e| e.timeline(&eid)).map(Json).map_err(map)
}

#[derive(Deserialize)]
struct DecisionReq {
    action: String,
    #[serde(rename = "shotId")]
    shot_id: Option<String>,
    #[serde(default)]
    params: Value,
}

async fn apply_decision(AxumPath((pid, eid)): AxumPath<(String, String)>, Json(r): Json<DecisionReq>) -> ApiRes {
    Editor::new(&pid).and_then(|e| e.apply_decision(&eid, &r.action, r.shot_id.as_deref(), &r.params)).map(Json).map_err(map)
}

#[derive(Deserialize)]
struct RenderReq {
    #[serde(rename = "forceAll", default)]
    force_all: bool,
    #[serde(default = "w")]
    width: u32,
    #[serde(default = "h")]
    height: u32,
    #[serde(default = "fps")]
    fps: u32,
}
fn w() -> u32 { 1920 }
fn h() -> u32 { 1080 }
fn fps() -> u32 { 30 }
impl Default for RenderReq {
    fn default() -> Self {
        Self { force_all: false, width: 1920, height: 1080, fps: 30 }
    }
}

async fn render(AxumPath((pid, eid)): AxumPath<(String, String)>, body: Option<Json<RenderReq>>) -> ApiRes {
    let r = body.map(|b| b.0).unwrap_or_default();
    let ed = Editor::new(&pid).map_err(map)?;
    let (width, height) = (r.width.clamp(64, 7680) & !1, r.height.clamp(64, 7680) & !1);
    let mut res = ed.selective_rerender(&eid, r.force_all, width, height, r.fps.clamp(1, 120)).await.map_err(map)?;
    res["videoUrl"] = json!(format!("/projects/{pid}/files/{}", res["outputVideo"].as_str().unwrap_or("")));
    res["thumbUrl"] = json!(format!("/projects/{pid}/files/output/{eid}/thumb.jpg"));
    Ok(Json(res))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/projects/{project_id}/editor/{eid}", get(get_timeline))
        .route("/projects/{project_id}/editor/{eid}/decisions", post(apply_decision))
        .route("/projects/{project_id}/editor/{eid}/render", post(render))
}
