//! Port of router.resolve_provider_for_model.
use super::models;

pub fn resolve_provider_for_model(model: &str, default_provider: &str) -> String {
    let m = model.trim().to_lowercase();
    if m.is_empty() {
        return default_provider.to_string();
    }
    if m.starts_with("mock") {
        return "mock".into();
    }
    if let Some(p) = models::get(model).and_then(|s| s["provider"].as_str().map(|s| s.to_lowercase())) {
        return p;
    }
    let has = |ks: &[&str]| ks.iter().any(|k| m.contains(k));
    if m.starts_with("fal-") || m.starts_with("fal/") {
        "fal".into()
    } else if m.starts_with("replicate") {
        "replicate".into()
    } else if has(&["pollinations", "pollen"]) {
        "pollinations".into()
    } else if has(&["cf-", "cloudflare"]) {
        "cloudflare".into()
    } else if has(&["local-", "mflux"]) {
        "local".into()
    } else if has(&["gemini", "veo", "imagen", "google"]) {
        "google".into()
    } else {
        default_provider.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolves_by_prefix() {
        assert_eq!(resolve_provider_for_model("mock-image", "google"), "mock");
        assert_eq!(resolve_provider_for_model("fal-ai/whatever-x", "google"), "fal");
        assert_eq!(resolve_provider_for_model("", "local"), "local");
        assert_eq!(resolve_provider_for_model("zzz-unknown", "google"), "google");
    }
}
