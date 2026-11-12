use super::prelude::*;

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}/products",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "List of catalog products", body = inline(Value))
    )
)]
pub(crate) async fn get_project_products(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let prod_dir = workspace::project_path(&project_id, &["products"], false).unwrap_or_default();
    let mut products = Vec::new();
    if prod_dir.exists() {
        if let Ok(entries) = fs::read_dir(prod_dir) {
            for entry in entries.flatten() {
                if entry.file_name() == "_trash" { continue; }
                let pfile = entry.path().join("product.json");
                if pfile.exists() {
                    if let Ok(txt) = fs::read_to_string(pfile) {
                        if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                            products.push(val);
                        }
                    }
                }
            }
        }
    }
    Json(json!(products))
}

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}/koc-styles",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "List available KOC review styles", body = inline(Value))
    )
)]
pub(crate) async fn get_project_koc_styles(AxumPath(_project_id): AxumPath<String>) -> Json<Value> {
    Json(json!([
        { "styleId": "koc_unboxing_tryon", "name": "Đập hộp & Thử đồ (Unboxing & Try-on)", "archetype": "unboxing_tryon" },
        { "styleId": "koc_problem_solution", "name": "Giải pháp phối đồ (Problem & Solution)", "archetype": "problem_solution" },
        { "styleId": "koc_outfit_check", "name": "Biến hình & Đánh giá (Outfit Check)", "archetype": "outfit_check" }
    ]))
}

#[utoipa::path(
    post,
    path = "/data/projects/{project_id}/products/{product_id}/review-flow",
    params(
        ("project_id" = String, Path, description = "Project Identifier"),
        ("product_id" = String, Path, description = "Product Identifier")
    ),
    request_body = inline(Value),
    responses(
        (status = 200, description = "Generate review flow graph for product", body = inline(Value))
    )
)]
pub(crate) async fn create_product_review_flow(
    AxumPath((project_id, product_id)): AxumPath<(String, String)>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let style_id = body.get("style_id").and_then(|v| v.as_str()).unwrap_or("koc_unboxing_tryon");
    let file_name = format!("graph.review-{}.json", style_id);
    let path = workspace::project_path(&project_id, &[&file_name], true).unwrap_or_default();
    
    // Save lightweight template if not exists
    if !path.exists() {
        let sample_graph = json!({
            "nodes": [],
            "edges": [],
            "productId": product_id,
            "styleId": style_id,
            "createdAt": chrono::Utc::now().to_rfc3339()
        });
        let _ = fs::write(&path, serde_json::to_string_pretty(&sample_graph).unwrap_or_default());
    }

    Json(json!({
        "ok": true,
        "file": file_name,
        "project": project_id,
        "productId": product_id,
        "styleId": style_id
    }))
}

pub(crate) async fn get_products(Query(q): Query<ProjectQuery>) -> Json<Value> {
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    let prod_dir = workspace::project_path(&proj, &["products"], false).unwrap_or_default();
    let mut products = Vec::new();
    if prod_dir.exists() {
        if let Ok(entries) = fs::read_dir(prod_dir) {
            for entry in entries.flatten() {
                let pfile = entry.path().join("product.json");
                if pfile.exists() {
                    if let Ok(txt) = fs::read_to_string(pfile) {
                        if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                            products.push(val);
                        }
                    }
                }
            }
        }
    }
    Json(json!({ "products": products }))
}

pub(crate) async fn save_product(Query(q): Query<ProjectQuery>, Json(body): Json<Value>) -> Json<Value> {
    let id = body.get("id").and_then(|v| v.as_str()).unwrap_or("prod_custom");
    let proj = q.project.unwrap_or_else(workspace::active_project_id);
    let dir = workspace::project_path(&proj, &["products", id], true).unwrap_or_default();
    let _ = fs::write(dir.join("product.json"), serde_json::to_string_pretty(&body).unwrap_or_default());
    Json(json!({ "ok": true, "id": id }))
}

pub(crate) async fn collect_product_url(
    AxumPath(project_id): AxumPath<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let url = payload.get("url").and_then(|u| u.as_str()).unwrap_or("");
    let products_dir = workspace::project_path(&project_id, &["products"], false).unwrap_or_default();
    let mut found = None;
    if products_dir.exists() {
        if let Ok(entries) = fs::read_dir(&products_dir) {
            for entry in entries.flatten() {
                let pj = entry.path().join("product.json");
                if pj.exists() {
                    if let Ok(txt) = fs::read_to_string(&pj) {
                        if let Ok(p) = serde_json::from_str::<Value>(&txt) {
                            found = Some(p);
                            break;
                        }
                    }
                }
            }
        }
    }
    let p = found.unwrap_or_else(|| json!({
        "id": "prod_auto_collected",
        "name": "Sản phẩm KOC Review Tự động",
        "price": 99000,
        "source_url": url,
        "cutout_images": []
    }));
    Json(json!({
        "product": p,
        "warnings": []
    }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/data/projects/{project_id}/products", get(get_project_products))
        .route("/projects/{project_id}/products/collect-url", post(collect_product_url))
        .route("/data/projects/{project_id}/products/{product_id}/review-flow", post(create_product_review_flow))
        .route("/data/projects/{project_id}/koc-styles", get(get_project_koc_styles))
        .route("/projects/{project_id}/products", get(get_project_products))
        .route("/api/data/products", get(get_products).post(save_product))
}
