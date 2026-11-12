//! API key lookup: process env -> repo .env.local -> .env (same order and var names as key_vault.py,
//! minus the in-RAM store). Keys are never logged; use `redact` on any text that may reach logs/SSE.
use crate::workspace;
use std::collections::HashMap;

/// (vault name, env vars, min length)
const SPECS: &[(&str, &[&str], usize)] = &[
    ("gemini", &["GEMINI_API_KEY", "GOOGLE_API_KEY"], 20),
    ("fal", &["FAL_KEY"], 16),
    ("openai", &["OPENAI_API_KEY"], 16),
    ("replicate", &["REPLICATE_API_TOKEN"], 16),
    ("pollinations", &["POLLINATIONS_API_KEY", "POLLINATIONS_TOKEN"], 8),
    ("cloudflare", &["CLOUDFLARE_API_TOKEN"], 16),
];
const PLACEHOLDERS: &[&str] = &["your_", "your-", "placeholder", "xxxx", "change_me", "insert_", "<", ">"];

pub fn normalize(provider: &str) -> &'static str {
    match provider.trim().to_lowercase().as_str() {
        "google" | "gemini" => "gemini",
        "fal" | "fal.ai" => "fal",
        "openai" => "openai",
        "replicate" => "replicate",
        "pollinations" => "pollinations",
        "cloudflare" => "cloudflare",
        _ => "local",
    }
}

fn parse_env(path: &std::path::Path) -> HashMap<String, String> {
    let mut m = HashMap::new();
    if let Ok(t) = std::fs::read_to_string(path) {
        for l in t.lines() {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = l.split_once('=') {
                m.insert(k.trim().to_string(), v.trim().trim_matches('"').trim_matches('\'').to_string());
            }
        }
    }
    m
}

pub fn raw_key(provider: &str) -> String {
    let norm = normalize(provider);
    let Some((_, vars, _)) = SPECS.iter().find(|s| s.0 == norm) else { return String::new() };
    for v in *vars {
        if let Ok(x) = std::env::var(v) {
            if !x.trim().is_empty() {
                return x.trim().to_string();
            }
        }
    }
    let root = workspace::repo_root();
    for f in [".env.local", ".env"] {
        let parsed = parse_env(&root.join(f));
        for v in *vars {
            if let Some(x) = parsed.get(*v).filter(|x| !x.is_empty()) {
                return x.clone();
            }
        }
    }
    String::new()
}

/// Validated key or a user-facing error (never includes the key).
pub fn require_key(provider: &str) -> anyhow::Result<String> {
    let norm = normalize(provider);
    let raw = raw_key(norm);
    let min = SPECS.iter().find(|s| s.0 == norm).map(|s| s.2).unwrap_or(0);
    let low = raw.to_lowercase();
    if raw.is_empty() {
        anyhow::bail!("Chưa có API key cho '{}' (biến môi trường, .env.local hoặc .env ở gốc repo).", norm);
    }
    if !raw.is_ascii() || raw.contains(' ') || raw.len() < min || PLACEHOLDERS.iter().any(|m| low.contains(m)) {
        anyhow::bail!("API key của '{}' không giống khóa thật (chữ giữ chỗ / quá ngắn).", norm);
    }
    Ok(raw)
}

pub fn has_valid_key(provider: &str) -> bool {
    require_key(provider).is_ok()
}

/// Replace every known secret (and `key=...`/Bearer patterns) in `text`.
pub fn redact(text: &str) -> String {
    let mut out = text.to_string();
    for (name, _, _) in SPECS {
        let k = raw_key(name);
        if k.len() >= 6 {
            out = out.replace(&k, "***");
        }
    }
    let re = regex::Regex::new(r"(?i)(key=|bearer\s+|key\s+)[A-Za-z0-9_\-:.]{12,}").unwrap();
    re.replace_all(&out, "${1}***").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redacts_patterns() {
        assert!(!redact("failed url?key=abcdefghijklmnop1234").contains("abcdefghijklmnop"));
    }
}
