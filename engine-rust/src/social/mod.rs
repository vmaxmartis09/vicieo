//! Social Hub (Track E): port of socnhotv.social. Data stays as `serde_json::Value` normalised to the
//! pydantic defaults so files written by Python and Rust round-trip with no field loss.
pub mod models;
pub mod service;
pub mod store;

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
