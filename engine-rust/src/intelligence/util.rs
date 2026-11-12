use crate::workspace;
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// Error carrying an HTTP status, rendered as FastAPI-style `{"detail": ...}`.
#[derive(Debug)]
pub struct ApiError(pub u16, pub String);
pub type ApiResult<T> = std::result::Result<T, ApiError>;

impl ApiError {
    pub fn nf(m: impl Into<String>) -> Self { Self(404, m.into()) }
    pub fn bad(m: impl Into<String>) -> Self { Self(400, m.into()) }
    pub fn unproc(m: impl Into<String>) -> Self { Self(422, m.into()) }
    pub fn ise(m: impl Into<String>) -> Self { Self(500, m.into()) }
}
impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self { Self(500, e.to_string()) }
}
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let code = axum::http::StatusCode::from_u16(self.0).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        (code, axum::Json(serde_json::json!({ "detail": self.1 }))).into_response()
    }
}

/// Ids that become path segments must be plain slugs (no traversal).
pub fn safe_id(s: &str) -> Result<&str> {
    if !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) && !s.contains("..") {
        Ok(s)
    } else {
        Err(anyhow!("invalid id: {s}"))
    }
}

/// data/projects/<id>/<parts...>; `ensure` creates the parent directory.
pub fn pp(project_id: &str, parts: &[&str], ensure: bool) -> PathBuf {
    if safe_id(project_id).is_err() || parts.iter().any(|p| p.contains("..")) {
        return PathBuf::from("/nonexistent-invalid-path");
    }
    workspace::project_path(project_id, parts, ensure).unwrap_or_else(|_| PathBuf::from("/nonexistent-invalid-path"))
}

pub fn read_json(p: &std::path::Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(p).ok()?).ok()
}

pub fn write_json<T: Serialize>(p: &std::path::Path, v: &T) -> Result<()> {
    if let Some(d) = p.parent() { fs::create_dir_all(d)?; }
    fs::write(p, serde_json::to_string_pretty(v)?)?;
    Ok(())
}

pub fn now_iso() -> String { chrono::Utc::now().to_rfc3339() }

/// rapidfuzz `fuzz.ratio` equivalent (0-100): 2*LCS/(la+lb).
pub fn fuzz_ratio(a: &str, b: &str) -> f64 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() && b.is_empty() { return 100.0; }
    let mut prev = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        let mut cur = vec![0usize; b.len() + 1];
        for j in 1..=b.len() {
            cur[j] = if a[i - 1] == b[j - 1] { prev[j - 1] + 1 } else { prev[j].max(cur[j - 1]) };
        }
        prev = cur;
    }
    200.0 * prev[b.len()] as f64 / (a.len() + b.len()) as f64
}

/// Format like Python `f"{x:,.0f}"`.
pub fn fmt_thousands(x: f64) -> String {
    let s = format!("{:.0}", x.abs());
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 { out.push(','); }
        out.push(c);
    }
    if x < 0.0 { format!("-{out}") } else { out }
}

pub fn s(v: &Value, k: &str) -> String { v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string() }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ratio_and_fmt() {
        assert_eq!(fuzz_ratio("abc", "abc"), 100.0);
        assert!(fuzz_ratio("abcd", "abxd") > 70.0);
        assert_eq!(fmt_thousands(1234567.0), "1,234,567");
        assert_eq!(fmt_thousands(199.0), "199");
        assert!(safe_id("../x").is_err());
    }
}

/// Slug used for ids (strip Vietnamese diacritics, non-alnum -> '_').
pub fn slug(s: &str) -> String {
    let t: String = s.replace('đ', "d").replace('Đ', "D").chars().flat_map(|c| {
        let n = match c {
            'à'|'á'|'ả'|'ã'|'ạ'|'ă'|'ằ'|'ắ'|'ẳ'|'ẵ'|'ặ'|'â'|'ầ'|'ấ'|'ẩ'|'ẫ'|'ậ' => 'a',
            'À'|'Á'|'Ả'|'Ã'|'Ạ'|'Ă'|'Ằ'|'Ắ'|'Ẳ'|'Ẵ'|'Ặ'|'Â'|'Ầ'|'Ấ'|'Ẩ'|'Ẫ'|'Ậ' => 'a',
            'è'|'é'|'ẻ'|'ẽ'|'ẹ'|'ê'|'ề'|'ế'|'ể'|'ễ'|'ệ'|'È'|'É'|'Ẻ'|'Ẽ'|'Ẹ'|'Ê'|'Ề'|'Ế'|'Ể'|'Ễ'|'Ệ' => 'e',
            'ì'|'í'|'ỉ'|'ĩ'|'ị'|'Ì'|'Í'|'Ỉ'|'Ĩ'|'Ị' => 'i',
            'ò'|'ó'|'ỏ'|'õ'|'ọ'|'ô'|'ồ'|'ố'|'ổ'|'ỗ'|'ộ'|'ơ'|'ờ'|'ớ'|'ở'|'ỡ'|'ợ'|'Ò'|'Ó'|'Ỏ'|'Õ'|'Ọ'|'Ô'|'Ồ'|'Ố'|'Ổ'|'Ỗ'|'Ộ'|'Ơ'|'Ờ'|'Ớ'|'Ở'|'Ỡ'|'Ợ' => 'o',
            'ù'|'ú'|'ủ'|'ũ'|'ụ'|'ư'|'ừ'|'ứ'|'ử'|'ữ'|'ự'|'Ù'|'Ú'|'Ủ'|'Ũ'|'Ụ'|'Ư'|'Ừ'|'Ứ'|'Ử'|'Ữ'|'Ự' => 'u',
            'ỳ'|'ý'|'ỷ'|'ỹ'|'ỵ'|'Ỳ'|'Ý'|'Ỷ'|'Ỹ'|'Ỵ' => 'y',
            c => c,
        };
        n.to_lowercase().collect::<Vec<_>>()
    }).collect();
    let r = regex::Regex::new(r"[^a-z0-9]+").unwrap().replace_all(&t, "_").trim_matches('_').to_string();
    if r.is_empty() { "x".into() } else { r }
}

/// Text model for Track D calls: GEMINI_MODEL / WATCH_GEMINI_MODEL, else the stable `gemini-flash-latest` alias (version-pinned names get retired).
pub fn llm_model() -> String {
    ["GEMINI_MODEL", "WATCH_GEMINI_MODEL"].iter().find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty())).unwrap_or_else(|| "gemini-flash-latest".into())
}

/// Run the active LLM and parse a JSON object/array out of the reply (code fences tolerated). None on any failure.
pub async fn ask_json(system: &str, prompt: &str) -> Option<Value> {
    let full = format!("{system}\n\n{prompt}");
    let raw = crate::providers::llm().generate(&full, Some(system), Some(&llm_model()), true).await.ok()?;
    let t = raw.trim();
    let t = t.strip_prefix("```json").or_else(|| t.strip_prefix("```")).unwrap_or(t);
    let t = t.strip_suffix("```").unwrap_or(t).trim();
    serde_json::from_str(t).ok()
}
