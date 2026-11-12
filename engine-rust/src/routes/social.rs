use super::prelude::*;

pub(crate) async fn get_project_publishing(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let sched_path = workspace::project_path(&project_id, &["publishing", "schedule.json"], false).unwrap_or_default();
    if sched_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sched_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    Json(json!({ "channels": [], "posts": [], "schedule": [] }))
}

pub(crate) async fn save_project_scheduled_post(AxumPath(project_id): AxumPath<String>, Json(body): Json<Value>) -> Json<Value> {
    let sched_path = workspace::project_path(&project_id, &["publishing", "schedule.json"], true).unwrap_or_default();
    let mut current_posts: Vec<Value> = Vec::new();
    let mut raw_val = json!({ "channels": [], "posts": [] });
    if sched_path.exists() {
        if let Ok(txt) = fs::read_to_string(&sched_path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                if let Some(arr) = val.get("posts").and_then(|p| p.as_array()) {
                    current_posts = arr.clone();
                }
                raw_val = val;
            }
        }
    }
    current_posts.push(body.clone());
    if let Some(obj) = raw_val.as_object_mut() {
        obj.insert("posts".to_string(), json!(current_posts));
    }
    let _ = fs::write(&sched_path, serde_json::to_string_pretty(&raw_val).unwrap_or_default());
    Json(json!({ "ok": true }))
}

pub(crate) async fn get_social_accounts(Query(q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "accounts": [] }))
}

pub(crate) async fn save_social_account(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({ "ok": true, "account": body }))
}

pub(crate) async fn get_social_posts(Query(q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "posts": [] }))
}

pub(crate) async fn create_social_post(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({ "ok": true, "postId": format!("post_{}", chrono::Utc::now().timestamp()) }))
}

pub(crate) async fn get_schedule(Query(q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "schedule": [] }))
}

pub(crate) async fn save_schedule(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({ "ok": true }))
}

pub(crate) async fn get_analytics(Query(q): Query<ProjectQuery>) -> Json<Value> {
    Json(json!({ "views": 0, "likes": 0, "shares": 0, "platform": "all" }))
}

// Video & HyperFrames Handlers

pub(crate) async fn get_project_social_posts(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let path = workspace::project_path(&project_id, &["publishing", "posts.json"], false).unwrap_or_default();
    if path.exists() {
        if let Ok(txt) = fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    Json(json!([]))
}

pub(crate) async fn create_project_social_post(
    AxumPath(project_id): AxumPath<String>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let path = workspace::project_path(&project_id, &["publishing", "posts.json"], true).unwrap_or_default();
    let mut posts = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<Vec<Value>>(&t).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let mut new_post = body.clone();
    let post_id = format!("post_{}", chrono::Utc::now().timestamp_millis());
    if new_post.get("id").is_none() {
        new_post["id"] = json!(post_id);
    }
    new_post["projectId"] = json!(project_id);
    posts.push(new_post.clone());
    let _ = fs::write(&path, serde_json::to_string_pretty(&posts).unwrap_or_default());
    Json(new_post)
}

pub(crate) async fn get_project_social_accounts(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let path = workspace::project_path(&project_id, &["accounts", "accounts.json"], false).unwrap_or_default();
    if path.exists() {
        if let Ok(txt) = fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                return Json(val);
            }
        }
    }
    Json(json!([
        { "id": "acc_tiktok_01", "platform": "tiktok", "name": "TikTok Channel", "connected": true, "followers": 12500 },
        { "id": "acc_youtube_01", "platform": "youtube", "name": "YouTube Shorts", "connected": true, "subscribers": 38400 }
    ]))
}

pub(crate) async fn get_project_social_templates(AxumPath(_project_id): AxumPath<String>) -> Json<Value> {
    Json(json!([
        { "id": "tmpl_standard_caption", "title": "Tiêu đề chuẩn TikTok SEO", "format": "{hook} #xuhuong #review" }
    ]))
}

pub(crate) async fn get_project_social_stats(AxumPath(_project_id): AxumPath<String>) -> Json<Value> {
    Json(json!({
        "totalPosts": 12,
        "viewsTotal": 158400,
        "engagementRate": 0.084
    }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/projects/{project_id}/publishing", get(get_project_publishing))
        .route("/projects/{project_id}/publishing/schedule", post(save_project_scheduled_post))
        .route("/projects/{project_id}/social/posts", get(get_project_social_posts).post(create_project_social_post))
        .route("/projects/{project_id}/social/accounts", get(get_project_social_accounts))
        .route("/projects/{project_id}/social/templates", get(get_project_social_templates))
        .route("/projects/{project_id}/social/stats", get(get_project_social_stats))
        .route("/api/social/accounts", get(get_social_accounts).post(save_social_account))
        .route("/api/social/posts", get(get_social_posts).post(create_social_post))
        .route("/api/social/schedule", get(get_schedule).post(save_schedule))
        .route("/api/social/analytics", get(get_analytics))
}
