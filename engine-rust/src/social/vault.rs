//! Token store for OAuth keys. Env first (`name` or `NAME`), then in-process map. Values never logged.
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

static EXTRA: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(Default::default);

pub fn get_key(name: &str) -> Option<String> {
    if let Some(v) = EXTRA.lock().ok()?.get(name) { return Some(v.clone()).filter(|v| !v.is_empty()); }
    std::env::var(name).or_else(|_| std::env::var(name.to_uppercase())).ok().filter(|v| !v.is_empty())
}
// shortcut: in-memory only (as Python persist_local=False), tokens vanish on restart; upgrade to providers key_vault when Track A lands it.
pub fn set_key(name: &str, value: &str) { if let Ok(mut m) = EXTRA.lock() { m.insert(name.into(), value.into()); } }

/// dry_run defaults to true; only an explicit SOCIAL_DRY_RUN=0|false|no disables it.
pub fn dry_run_default() -> bool {
    !matches!(std::env::var("SOCIAL_DRY_RUN").ok().as_deref().map(|s| s.to_lowercase()).as_deref(), Some("0") | Some("false") | Some("no"))
}
