//! /settings/api-keys — read/write API keys in the repo's .env.local (values are never returned).
use super::prelude::*;
use super::graph::{api_err, ApiErr};
use crate::providers::key_vault::{self, normalize, parse_env, SPECS};

fn label(norm: &str) -> &'static str {
    match norm {
        "gemini" => "Google Gemini & Veo",
        "fal" => "fal.ai (FLUX & Kling/LTX Video Queue)",
        "openai" => "OpenAI (GPT / Whisper)",
        "replicate" => "Replicate",
        "pollinations" => "Pollinations.ai (Pollen Key - Tùy chọn)",
        _ => "Cloudflare Workers AI",
    }
}

fn status(norm: &str) -> Value {
    let spec = SPECS.iter().find(|s| s.0 == norm).expect("known provider");
    json!({ "provider": norm, "label": label(norm), "envVars": spec.1, "configured": key_vault::has_valid_key(norm), "updatedAt": Value::Null })
}

/// Format check mirroring key_vault.validate_key_format: ASCII, no spaces, min length, not a placeholder.
fn check_format(norm: &str, key: &str) -> Result<String, ApiErr> {
    let spec = SPECS.iter().find(|s| s.0 == norm).expect("known provider");
    let cleaned = key.trim().trim_matches(|c| c == '"' || c == '\'').to_string();
    if cleaned.is_empty() {
        return Ok(String::new());
    }
    let low = cleaned.to_lowercase();
    if !cleaned.is_ascii() || cleaned.chars().any(char::is_whitespace) {
        return Err(api_err(StatusCode::BAD_REQUEST, "Khóa API chứa ký tự không hợp lệ (không phải ASCII hoặc có khoảng trắng)."));
    }
    if cleaned.len() < spec.2 || ["your_", "your-", "placeholder", "xxxx", "change_me", "insert_", "<", ">"].iter().any(|p| low.contains(p)) {
        return Err(api_err(StatusCode::BAD_REQUEST, "Khóa API quá ngắn hoặc là chữ giữ chỗ."));
    }
    Ok(cleaned)
}

/// Live check against the provider's free auth endpoint. Providers without one get format-only checks.
async fn verify_live(norm: &str, key: &str) -> Result<Value, ApiErr> {
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(15)).build().map_err(|e| api_err(StatusCode::BAD_GATEWAY, e.to_string()))?;
    let req = match norm {
        "gemini" => client.get("https://generativelanguage.googleapis.com/v1beta/models").query(&[("pageSize", "1")]).header("x-goog-api-key", key),
        "fal" => client.get("https://rest.alpha.fal.ai/models").query(&[("limit", "1")]).header("Authorization", format!("Key {key}")),
        "replicate" => client.get("https://api.replicate.com/v1/account").header("Authorization", format!("Bearer {key}")),
        "openai" => client.get("https://api.openai.com/v1/models").header("Authorization", format!("Bearer {key}")),
        _ => return Ok(json!({ "provider": norm, "valid": true, "checkedLive": false })),
    };
    let r = req.send().await.map_err(|e| api_err(StatusCode::BAD_GATEWAY, key_vault::redact(&e.to_string())))?;
    let code = r.status().as_u16();
    if code == 401 || code == 403 || (norm == "gemini" && code >= 400) {
        return Err(api_err(StatusCode::BAD_REQUEST, format!("Khóa {norm} không hợp lệ (HTTP {code}).")));
    }
    Ok(json!({ "provider": norm, "valid": true, "checkedLive": true }))
}

/// Write or remove one key in .env.local, keeping the other entries (0600 like key_vault.py).
fn write_key(norm: &str, key: &str) -> Result<(), ApiErr> {
    write_key_at(&crate::workspace::repo_root().join(".env.local"), norm, key)
}

fn write_key_at(path: &std::path::Path, norm: &str, key: &str) -> Result<(), ApiErr> {
    let io = |e: std::io::Error| api_err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    let mut vals: std::collections::BTreeMap<String, String> = parse_env(path).into_iter().collect();
    let primary = SPECS.iter().find(|s| s.0 == norm).map(|s| s.1[0]).unwrap_or_default();
    if key.is_empty() {
        vals.remove(primary);
    } else {
        vals.insert(primary.to_string(), key.to_string());
    }
    let mut lines = vec!["# Khóa API cục bộ của AI Video Studio (gitignore, không đẩy lên git)".to_string()];
    lines.extend(vals.iter().map(|(k, v)| format!("{k}=\"{v}\"")));
    std::fs::write(path, lines.join("\n") + "\n").map_err(io)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_key_at;

    #[test]
    fn writes_then_clears_one_key_keeping_others() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join(".env.local");
        std::fs::write(&f, "OTHER_VAR=\"keep\"\n").unwrap();
        write_key_at(&f, "fal", "fal-test-value-0123456789").unwrap();
        let t = std::fs::read_to_string(&f).unwrap();
        assert!(t.contains("FAL_KEY=\"fal-test-value-0123456789\"") && t.contains("OTHER_VAR=\"keep\""), "{t}");
        write_key_at(&f, "fal", "").unwrap();
        let t = std::fs::read_to_string(&f).unwrap();
        assert!(!t.contains("FAL_KEY") && t.contains("OTHER_VAR"), "{t}");
    }
}

async fn get_status() -> Json<Value> {
    Json(json!({ "providers": SPECS.iter().map(|s| status(s.0)).collect::<Vec<_>>() }))
}

async fn set_key(Json(b): Json<Value>) -> Result<Json<Value>, ApiErr> {
    let norm = normalize(b["provider"].as_str().unwrap_or(""));
    if norm == "local" {
        return Err(api_err(StatusCode::BAD_REQUEST, "Nhà cung cấp không được hỗ trợ"));
    }
    let key = check_format(norm, b["apiKey"].as_str().unwrap_or(""))?;
    if b["verifyLive"].as_bool().unwrap_or(false) && !key.is_empty() {
        verify_live(norm, &key).await?;
    }
    write_key(norm, &key)?;
    let mut st = status(norm);
    if b["verifyLive"].as_bool().unwrap_or(false) && st["configured"] == true {
        st["verifiedLive"] = json!(true);
    }
    Ok(Json(st))
}

async fn verify(Json(b): Json<Value>) -> Result<Json<Value>, ApiErr> {
    let provider = b["provider"].as_str().unwrap_or("").trim().to_lowercase();
    let norm = normalize(&provider);
    if norm == "local" {
        return Err(api_err(StatusCode::BAD_REQUEST, "Nhà cung cấp không được hỗ trợ"));
    }
    let key = b["apiKey"].as_str().unwrap_or("");
    let key = if key.trim().is_empty() { key_vault::raw_key(norm) } else { check_format(norm, key)? };
    if key.is_empty() {
        return Err(api_err(StatusCode::BAD_REQUEST, format!("Chưa có khóa cho {norm}")));
    }
    let detail = verify_live(norm, &key).await?;
    Ok(Json(json!({ "provider": norm, "valid": true, "detail": detail })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/settings/api-keys", get(get_status).post(set_key))
        .route("/settings/api-keys/verify", post(verify))
}
