use super::prelude::*;

#[utoipa::path(
    get,
    path = "/data/projects/{project_id}/policy",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    responses(
        (status = 200, description = "Get project content compliance policy", body = inline(Value))
    )
)]
pub(crate) async fn get_project_policy(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let policy_path = workspace::project_path(&project_id, &["policy.json"], false).unwrap_or_default();
    if policy_path.exists() {
        if let Ok(content) = fs::read_to_string(&policy_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                return Json(val);
            }
        }
    }
    Json(json!({
        "projectId": project_id,
        "rules": []
    }))
}

#[utoipa::path(
    put,
    path = "/data/projects/{project_id}/policy",
    params(
        ("project_id" = String, Path, description = "Project Identifier")
    ),
    request_body = inline(Value),
    responses(
        (status = 200, description = "Save project content compliance policy", body = inline(Value))
    )
)]
pub(crate) async fn save_project_policy(AxumPath(project_id): AxumPath<String>, Json(body): Json<Value>) -> Json<Value> {
    let policy_path = workspace::project_path(&project_id, &["policy.json"], true).unwrap_or_default();
    let _ = fs::write(policy_path, serde_json::to_string_pretty(&body).unwrap_or_default());
    Json(body)
}

#[utoipa::path(
    post,
    path = "/api/intel/ideas",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Generate KOC ideas and video hooks", body = inline(Value))
    )
)]
pub(crate) async fn generate_ideas(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({
        "ideas": [
            { "title": "Bí quyết chọn áo cardigan mùa thu cực xinh", "hook": "Đừng mua cardigan nếu chưa biết điều này!", "score": 9.2 },
            { "title": "Phối đồ phong cách tối giản thanh lịch", "hook": "3 cách phối đồ chuẩn Hàn Quốc chỉ với 1 chiếc áo", "score": 8.8 }
        ]
    }))
}

#[utoipa::path(
    post,
    path = "/api/intel/script",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Generate scene-by-scene KOC script", body = inline(Value))
    )
)]
pub(crate) async fn generate_script(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({
        "script": {
            "title": "Kịch bản review sản phẩm",
            "scenes": [
                { "scene": 1, "duration": 3.0, "narration": "Chiếc áo này đang làm mưa làm gió tuần qua..." },
                { "scene": 2, "duration": 5.0, "narration": "Chất vải dày dặn, form dáng cực kỳ tôn eo." }
            ]
        }
    }))
}

#[utoipa::path(
    post,
    path = "/api/intel/compliance",
    request_body = inline(Value),
    responses(
        (status = 200, description = "Validate script policy compliance", body = inline(Value))
    )
)]
pub(crate) async fn check_compliance(Json(body): Json<Value>) -> Json<Value> {
    let text = body.get("text").and_then(|v| v.as_str()).unwrap_or("");
    let violations = crate::intelligence::IntelligenceEngine::check_policy_compliance(text);
    Json(json!({ "compliant": violations.is_empty(), "violations": violations }))
}

// Voice Handlers

pub(crate) async fn chat_with_assistant(
    AxumPath(project_id): AxumPath<String>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let msg = body.get("message").and_then(|m| m.as_str()).unwrap_or("");
    let reply = format!(
        "Studio AI Assistant [Dự án {}]: Đã nhận chỉ thị '{}'. Hệ thống đang đồng bộ cấu trúc nhân vật và kịch bản phân cảnh.",
        project_id, msg
    );
    Json(json!({ "reply": reply, "action": Value::Null }))
}

pub(crate) async fn get_policy_packs_handler() -> Json<Value> {
    let path = workspace::shared_path(&["policy_packs.json"], false).unwrap_or_default();
    if path.exists() {
        if let Ok(txt) = fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    Json(json!([]))
}

pub(crate) async fn generate_script_ai(
    AxumPath(_project_id): AxumPath<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let prompt = payload.get("prompt").and_then(|p| p.as_str()).unwrap_or("");
    let md = format!(
        "# Kịch bản tạo bởi AI\n\n**Ý tưởng:** {}\n\nCảnh 1:\nBi: Chào các bạn nhỏ!\n(Bi vẫy tay tươi cười chào khán giả)\n\nCảnh 2:\nBi: Hôm nay chúng mình cùng học điều thú vị nhé!\n(Bi hào hứng chỉ tay vào bảng khám phá)\n\nCảnh 3:\nBi: Hẹn gặp lại các bạn ở tập sau!\n(Bi chào tạm biệt và nhảy múa)",
        prompt
    );
    Json(json!({ "mode": "gemini", "markdown": md }))
}

pub(crate) async fn check_project_policy(
    AxumPath(project_id): AxumPath<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let text = payload.get("text").and_then(|t| t.as_str()).unwrap_or("");
    let forbidden = ["giết", "đánh nhau", "máu me", "chửi thề", "khốn nạn", "khiêu dâm"];
    let mut violations = Vec::new();
    let lower = text.to_lowercase();
    for term in forbidden {
        if lower.contains(term) {
            violations.push(json!({
                "ruleId": "content_safety",
                "category": "An toàn nội dung",
                "term": term,
                "severity": "block"
            }));
        }
    }
    Json(json!({
        "ok": violations.is_empty(),
        "violations": violations,
        "checkedTextLength": text.len(),
        "projectId": project_id
    }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/data/projects/{project_id}/ai/script", post(generate_script_ai))
        .route("/data/projects/{project_id}/policy/check", post(check_project_policy))
        .route("/data/projects/{project_id}/policy", get(get_project_policy).put(save_project_policy))
        .route("/projects/{project_id}/ai/script", post(generate_script_ai))
        .route("/projects/{project_id}/ai/character", post(chat_with_assistant))
        .route("/projects/{project_id}/chat", post(chat_with_assistant))
        .route("/data/policy-packs", get(get_policy_packs_handler))
        .route("/api/intel/ideas", post(generate_ideas))
        .route("/api/intel/script", post(generate_script))
        .route("/api/intel/compliance", post(check_compliance))
}
