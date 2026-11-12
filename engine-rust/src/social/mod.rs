//! Social Hub (Track E): port of socnhotv.social. Data stays as `serde_json::Value` normalised to the
//! pydantic defaults so files written by Python and Rust round-trip with no field loss.
pub mod adapters;
pub mod automation;
pub mod captions;
pub mod collectors;
pub mod contracts;
pub mod insights;
pub mod migrate;
pub mod models;
pub mod planner;
pub mod platform_rules;
pub mod queue;
pub mod scheduler;
pub mod service;
pub mod store;
pub mod video_library;
pub mod vault;

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// FastAPI-shaped error: `{"detail": msg}`.
#[derive(Debug)]
pub struct ApiErr(pub StatusCode, pub String);
impl IntoResponse for ApiErr {
    fn into_response(self) -> axum::response::Response {
        (self.0, Json(json!({ "detail": self.1 }))).into_response()
    }
}
impl ApiErr {
    pub fn not_found(m: impl Into<String>) -> Self { Self(StatusCode::NOT_FOUND, m.into()) }
    pub fn bad(m: impl Into<String>) -> Self { Self(StatusCode::BAD_REQUEST, m.into()) }
    pub fn invalid(m: impl Into<String>) -> Self { Self(StatusCode::UNPROCESSABLE_ENTITY, m.into()) }
    pub fn conflict(m: impl Into<String>) -> Self { Self(StatusCode::CONFLICT, m.into()) }
}
pub type R<T = serde_json::Value> = Result<Json<T>, ApiErr>;

#[cfg(test)]
mod tests {
    use super::{models::*, scheduler, service::SocialService, store::SocialStore};
    use serde_json::{json, Value};

    /// Throwaway project under data/projects, removed on drop.
    struct Tmp(String);
    impl Tmp { fn new() -> Self { Self(format!("zz-social-test-{}", uuid::Uuid::new_v4().simple())) } }
    impl Drop for Tmp { fn drop(&mut self) { let _ = std::fs::remove_dir_all(crate::workspace::data_root().join("projects").join(&self.0)); } }

    fn seed(st: &SocialStore, platform: &str, mode: &str, scheduled: &str, extra: Value) -> (String, String) {
        let acc = st.save_account(json!({"platform":platform,"handle":format!("@{}", uuid::Uuid::new_v4().simple()),"display_name":"T","auth":{"mode":mode}}), false).unwrap();
        let mut target = json!({"account_id":acc["id"],"platform":platform,"scheduled_at":scheduled,"status":"scheduled","caption":"hi"});
        if let Some(o) = extra.as_object() { for (k, v) in o { target[k] = v.clone(); } }
        let post = st.save_post(json!({"project_id":st.project_id,"title":"T","status":"scheduled","approval":{"required":false},"media":[{"path":"x.mp4"}],"targets":[target]})).unwrap();
        (s(&post, "id").into(), s(&post["targets"][0], "id").into())
    }

    #[test]
    fn state_machine_requires_approval_then_schedules() {
        let t = Tmp::new();
        let svc = SocialService::new(&t.0).unwrap();
        let post = svc.store.save_post(json!({"project_id":t.0,"title":"a","targets":[{"account_id":"a","platform":"tiktok"}]})).unwrap();
        let id = s(&post, "id").to_string();
        let times = serde_json::Map::from_iter([("tiktok".to_string(), json!("2030-01-01T10:00:00+07:00"))]);
        assert!(svc.schedule(&id, &times, "u").is_err(), "must be approved first");
        svc.submit_for_approval(&id, "u", "").unwrap();
        svc.approve(&id, "u", "").unwrap();
        let p = svc.schedule(&id, &times, "u").unwrap();
        assert_eq!((s(&p, "status"), s(&p["targets"][0], "status")), ("scheduled", "scheduled"));
        assert!(svc.submit_for_approval(&id, "u", "").is_err(), "scheduled -> pending_approval is invalid");
    }

    #[test]
    fn store_reads_python_written_template_dict_and_defaults() {
        let t = Tmp::new();
        let st = SocialStore::new(&t.0).unwrap();
        std::fs::write(st.pub_dir.join("templates.json"), r#"{"templates":{"t1":{"name":"N"}}}"#).unwrap();
        let tpl = st.list_templates();
        assert_eq!((s(&tpl[0], "id"), s(&tpl[0], "purpose")), ("t1", "general"));
        assert!(st.save_account(json!({"platform":"nope","handle":"h","display_name":"d"}), false).is_err());
    }

    #[tokio::test]
    async fn scheduler_dry_run_publishes_due_and_never_calls_network() {
        let t = Tmp::new();
        let st = SocialStore::new(&t.0).unwrap();
        let (pid, _) = seed(&st, "youtube", "oauth", "2020-01-01T10:00:00+07:00", json!({"title":"Hello"}));
        let (future_pid, _) = seed(&st, "youtube", "oauth", "2099-01-01T10:00:00+07:00", json!({"title":"Later"}));
        let r = scheduler::tick(&t.0, None, true).await;
        assert_eq!(r.len(), 1);
        let p = st.get_post(&pid).unwrap();
        assert_eq!((s(&p, "status"), s(&p["targets"][0], "status")), ("published", "published"));
        assert!(s(&p["targets"][0], "external_url").starts_with("dry-run://"));
        assert_eq!(s(&st.get_post(&future_pid).unwrap(), "status"), "scheduled");
    }

    #[tokio::test]
    async fn scheduler_backoff_then_permanent_failure() {
        let t = Tmp::new();
        let st = SocialStore::new(&t.0).unwrap();
        // youtube without title => validation error => failed publish result
        let (pid, _) = seed(&st, "youtube", "oauth", "2020-01-01T10:00:00+07:00", json!({"caption":""}));
        let now = parse_dt("2020-01-02T00:00:00+07:00").unwrap();
        scheduler::tick(&t.0, Some(now), true).await;
        let p = st.get_post(&pid).unwrap();
        assert_eq!((s(&p["targets"][0], "status"), p["targets"][0]["attempts"].as_i64()), ("scheduled", Some(1)));
        assert_eq!(s(&p["targets"][0], "scheduled_at").get(..16), Some("2020-01-02T00:02"), "2 min backoff");
        let mut later = now;
        for _ in 0..2 { later += chrono::Duration::hours(1); scheduler::tick(&t.0, Some(later), true).await; }
        let p = st.get_post(&pid).unwrap();
        assert_eq!((s(&p["targets"][0], "status"), s(&p, "status")), ("failed", "failed"));
        assert!(!st.list_notifications(false, 50).is_empty());
    }
}
