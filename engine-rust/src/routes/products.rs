use super::prelude::*;
use crate::intelligence::{
    koc_styles as koc, products as prod, review_flow as rf, tiktok_product,
    util::{safe_id, ApiError, ApiResult},
};

type J = ApiResult<Json<Value>>;

fn v<T: Serialize>(x: &T) -> Value { serde_json::to_value(x).unwrap_or(Value::Null) }
fn ok_id(id: &str) -> ApiResult<()> { safe_id(id).map(|_| ()).map_err(|e| ApiError::bad(e.to_string())) }
fn need_product(pid: &str, id: &str) -> ApiResult<prod::Product> { ok_id(pid)?; prod::get_product(pid, id).ok_or_else(|| ApiError::nf("Không tìm thấy sản phẩm.")) }

#[utoipa::path(get, path = "/data/projects/{project_id}/products", params(("project_id" = String, Path, description = "Project Identifier")),
    responses((status = 200, description = "List of catalog products", body = inline(Value))))]
pub(crate) async fn get_project_products(AxumPath(pid): AxumPath<String>) -> J { ok_id(&pid)?; Ok(Json(v(&prod::list_products(&pid)))) }

#[utoipa::path(get, path = "/data/projects/{project_id}/koc-styles", params(("project_id" = String, Path, description = "Project Identifier")),
    responses((status = 200, description = "List available KOC review styles", body = inline(Value))))]
pub(crate) async fn get_project_koc_styles(AxumPath(pid): AxumPath<String>, Query(q): Query<HashMap<String, String>>) -> J {
    ok_id(&pid)?;
    Ok(Json(json!(rf::list_koc_styles(&pid, q.get("product_id").map(String::as_str)))))
}

#[utoipa::path(post, path = "/data/projects/{project_id}/products/{product_id}/review-flow",
    params(("project_id" = String, Path, description = "Project Identifier"), ("product_id" = String, Path, description = "Product Identifier")),
    request_body = inline(Value), responses((status = 200, description = "Generate review flow graph for product", body = inline(Value))))]
pub(crate) async fn create_product_review_flow(AxumPath((pid, prod_id)): AxumPath<(String, String)>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let g = |ks: &[&str]| ks.iter().find_map(|k| b.get(*k).and_then(|x| x.as_str()).filter(|s| !s.is_empty()).map(String::from));
    let style_id = g(&["style_id", "styleId"]).unwrap_or_else(|| "koc_unboxing_tryon".into());
    let o = rf::FlowOpts {
        character_id: g(&["character_id", "characterId"]), style_data: b.get("style").cloned().filter(|s| !s.is_null()),
        video_model: g(&["video_model", "videoModel"]).unwrap_or_else(|| "mock-video".into()), image_model: g(&["image_model", "imageModel"]).unwrap_or_else(|| "mock-image".into()),
        resolution: g(&["resolution"]).unwrap_or_else(|| "1080p".into()), variants: b.get("variants").and_then(|x| x.as_i64()).filter(|x| *x != 0).unwrap_or(1),
        use_llm: b.get("use_llm").and_then(|x| x.as_bool()).unwrap_or(true),
    };
    let (graph, file) = rf::generate_and_save_review_flow(&pid, &prod_id, &style_id, &o).await?;
    Ok(Json(json!({"status": "success", "file": file, "path": format!("data/projects/{pid}/{file}"), "graph": graph})))
}

async fn list_products(AxumPath(pid): AxumPath<String>) -> J { get_project_products(AxumPath(pid)).await }

fn check_new(p: &prod::Product) -> ApiResult<()> {
    if p.id.is_empty() || p.name.is_empty() { return Err(ApiError::unproc("Product requires 'id' and 'name'")); }
    safe_id(&p.id).map(|_| ()).map_err(|e| ApiError::bad(e.to_string()))
}

async fn create_product(AxumPath(pid): AxumPath<String>, Json(p): Json<prod::Product>) -> J {
    ok_id(&pid)?; check_new(&p)?;
    prod::save_product(&pid, &p)?;
    Ok(Json(v(&p)))
}
async fn get_product(AxumPath((pid, id)): AxumPath<(String, String)>) -> J { Ok(Json(v(&need_product(&pid, &id)?))) }
async fn update_product(AxumPath((pid, id)): AxumPath<(String, String)>, Json(mut p): Json<prod::Product>) -> J {
    ok_id(&pid)?; p.id = id; if p.name.is_empty() { return Err(ApiError::unproc("Product requires 'name'")); }
    check_new(&p)?; prod::save_product(&pid, &p)?; Ok(Json(v(&p)))
}
async fn delete_product(AxumPath((pid, id)): AxumPath<(String, String)>) -> J {
    ok_id(&pid)?;
    if !prod::delete_product(&pid, &id) { return Err(ApiError::nf("Không tìm thấy sản phẩm để xoá.")); }
    Ok(Json(json!({"status": "ok", "message": format!("Sản phẩm {id} đã chuyển vào thùng rác.")})))
}

fn req_url(b: &Value) -> ApiResult<String> { b.get("url").and_then(|u| u.as_str()).map(String::from).ok_or_else(|| ApiError::unproc("field 'url' is required")) }

pub(crate) async fn collect_product_url(AxumPath(pid): AxumPath<String>, Json(b): Json<Value>) -> J {
    ok_id(&pid)?;
    let url = req_url(&b)?;
    let cutout = b.get("cutout").and_then(|c| c.as_bool()).unwrap_or(true);
    let (p, w) = tiktok_product::collect_product(&pid, &url, cutout).await.map_err(|e| ApiError::unproc(e.0))?;
    Ok(Json(json!({"product": p, "warnings": w})))
}
async fn import_product_url(AxumPath(_pid): AxumPath<String>, Json(b): Json<Value>) -> J { Ok(Json(prod::import_product_from_url(&req_url(&b)?).await)) }

async fn list_product_koc(AxumPath((pid, id)): AxumPath<(String, String)>) -> J {
    need_product(&pid, &id)?;
    Ok(Json(json!({"productId": id, "index": koc::load_koc_styles_index(&pid, &id), "styles": koc::list_koc_styles(&pid, &id)})))
}
async fn get_product_koc(AxumPath((pid, id, sid)): AxumPath<(String, String, String)>) -> J {
    ok_id(&pid)?;
    koc::get_koc_style(&pid, &id, &sid).map(|s| Json(v(&s))).ok_or_else(|| ApiError::nf("Không tìm thấy phong cách KOC."))
}
async fn collect_koc(AxumPath((pid, id)): AxumPath<(String, String)>, Json(b): Json<Value>) -> J {
    need_product(&pid, &id)?;
    let urls: Vec<String> = b.get("urls").and_then(|u| u.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let max = b.get("max_videos").and_then(|m| m.as_u64()).unwrap_or(3) as usize;
    Ok(Json(koc::collect_and_analyze(&pid, &id, &urls, max, b.get("use_gemini").and_then(|g| g.as_bool()).unwrap_or(true)).await))
}
async fn analyze_koc_url(AxumPath((pid, id)): AxumPath<(String, String)>, Json(b): Json<Value>) -> J {
    let p = need_product(&pid, &id)?;
    let url = req_url(&b)?;
    let st = koc::analyze_koc_video(&url, &id, &p.name, b.get("use_gemini").and_then(|g| g.as_bool()).unwrap_or(true)).await
        .and_then(|st| koc::save_koc_style(&pid, &st).map(|_| st)).map_err(|e| ApiError::ise(format!("Lỗi phân tích video KOC: {e}")))?;
    Ok(Json(json!({"status": "ok", "style": st})))
}

pub(crate) async fn get_products(Query(q): Query<ProjectQuery>) -> Json<Value> {
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    Json(json!({ "products": prod::list_products(&proj) }))
}

pub(crate) async fn save_product(Query(q): Query<ProjectQuery>, Json(p): Json<prod::Product>) -> J {
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    ok_id(&proj)?;
    let p = if p.id.is_empty() { prod::Product { id: "prod_custom".into(), ..p } } else { p };
    check_new(&p)?;
    prod::save_product(&proj, &p)?;
    Ok(Json(json!({ "ok": true, "id": p.id })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        // dashboard-facing aliases (kept from Phase 0)
        .route("/data/projects/{project_id}/products", get(get_project_products))
        .route("/data/projects/{project_id}/products/{product_id}/review-flow", post(create_product_review_flow))
        .route("/data/projects/{project_id}/koc-styles", get(get_project_koc_styles))
        .route("/api/data/products", get(get_products).post(save_product))
        // Python-parity API
        .route("/projects/{project_id}/products", get(list_products).post(create_product))
        .route("/projects/{project_id}/products/collect-url", post(collect_product_url))
        .route("/projects/{project_id}/products/import-url", post(import_product_url))
        .route("/projects/{project_id}/products/{product_id}", get(get_product).put(update_product).delete(delete_product))
        .route("/projects/{project_id}/products/{product_id}/koc-styles", get(list_product_koc))
        .route("/projects/{project_id}/products/{product_id}/koc-styles/collect", post(collect_koc))
        .route("/projects/{project_id}/products/{product_id}/koc-styles/analyze-url", post(analyze_koc_url))
        .route("/projects/{project_id}/products/{product_id}/koc-styles/{style_id}", get(get_product_koc))
        .route("/projects/{project_id}/products/{product_id}/review-flow", post(create_product_review_flow))
        .route("/projects/{project_id}/koc-styles", get(get_project_koc_styles))
}
