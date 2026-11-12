use chrono::{DateTime, FixedOffset, Utc};
use serde_json::{json, Map, Value};

pub const MAX_ACCOUNTS: usize = 10;
/// Asia/Ho_Chi_Minh has no DST, so a fixed +07:00 offset is exact.
pub fn tz() -> FixedOffset { FixedOffset::east_opt(7 * 3600).unwrap() }
pub fn now_iso() -> String { Utc::now().with_timezone(&tz()).format("%Y-%m-%dT%H:%M:%S%.6f%:z").to_string() }
pub fn new_id(prefix: &str, n: usize) -> String { format!("{prefix}_{}", &uuid::Uuid::new_v4().simple().to_string()[..n]) }

/// Parse ISO-8601 (naive => +07:00), like Python `ensure_tz(fromisoformat(..))`.
pub fn parse_dt(s: &str) -> Option<DateTime<FixedOffset>> {
    if let Ok(d) = DateTime::parse_from_rfc3339(s) { return Some(d.with_timezone(&tz())); }
    for f in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M:%S"] {
        if let Ok(n) = chrono::NaiveDateTime::parse_from_str(s, f) {
            return n.and_local_timezone(tz()).single();
        }
    }
    None
}

pub fn obj(v: &mut Value) -> &mut Map<String, Value> {
    if !v.is_object() { *v = json!({}); }
    v.as_object_mut().unwrap()
}
fn dflt(m: &mut Map<String, Value>, k: &str, d: Value) { if m.get(k).map_or(true, |x| x.is_null() && !d.is_null()) { m.insert(k.into(), d); } }
fn req(m: &Map<String, Value>, ks: &[&str]) -> Result<(), String> {
    for k in ks { if !m.get(*k).map_or(false, |v| !v.is_null()) { return Err(format!("field required: {k}")); } }
    Ok(())
}
fn one_of(m: &Map<String, Value>, k: &str, allowed: &[&str]) -> Result<(), String> {
    match m.get(k).and_then(|v| v.as_str()) {
        Some(s) if allowed.contains(&s) => Ok(()),
        _ => Err(format!("invalid {k}: expected one of {allowed:?}")),
    }
}

pub const PLATFORMS: &[&str] = &["youtube", "tiktok", "facebook", "instagram", "douyin", "threads", "x"];
pub const POST_STATUSES: &[&str] = &["draft", "pending_approval", "approved", "scheduled", "publishing", "published", "failed", "cancelled"];

pub fn norm_account(mut v: Value) -> Result<Value, String> {
    let m = obj(&mut v);
    req(m, &["platform", "handle", "display_name"])?;
    one_of(m, "platform", PLATFORMS)?;
    dflt(m, "id", json!(new_id("acc", 10)));
    for (k, d) in [("avatar", json!("")), ("channel_url", json!("")), ("status", json!("manual")), ("groups", json!([])),
        ("default_hashtags", json!([])), ("default_caption_footer", json!("")), ("posting_windows", json!([])),
        ("daily_limit", json!(5)), ("min_gap_minutes", json!(120)), ("color", json!("#6366f1")),
        ("affiliate_default", json!({"link": "", "disclosure": false})), ("tags", json!([]))] { dflt(m, k, d); }
    dflt(m, "auth", json!({}));
    let a = obj(m.get_mut("auth").unwrap());
    for (k, d) in [("mode", json!("manual")), ("token_ref", Value::Null), ("scopes", json!([])), ("expires_at", Value::Null)] {
        if !a.contains_key(k) { a.insert(k.into(), d); }
    }
    Ok(v)
}

pub fn norm_target(mut v: Value) -> Result<Value, String> {
    let m = obj(&mut v);
    req(m, &["account_id", "platform"])?;
    one_of(m, "platform", PLATFORMS)?;
    dflt(m, "id", json!(new_id("tgt", 10)));
    for (k, d) in [("scheduled_at", Value::Null), ("caption", json!("")), ("hashtags", json!([])), ("title", json!("")),
        ("description", json!("")), ("tags", json!([])), ("privacy", json!("public")), ("made_for_kids", json!(false)),
        ("cover_time_s", json!(0.0)), ("cover_url", Value::Null), ("first_comment", json!("")), ("product_link", json!("")),
        ("disclosure_ad", json!(false)), ("ai_generated_label", json!(false)), ("comment_enabled", json!(true)),
        ("duet_enabled", json!(true)), ("stitch_enabled", json!(true)), ("category", json!("22")), ("status", json!("draft")),
        ("external_id", Value::Null), ("external_url", Value::Null), ("attempts", json!(0)), ("last_error", Value::Null),
        ("published_at", Value::Null)] { if !m.contains_key(k) { m.insert(k.into(), d); } }
    one_of(m, "status", POST_STATUSES)?;
    Ok(v)
}

pub fn norm_media(mut v: Value) -> Result<Value, String> {
    let m = obj(&mut v);
    req(m, &["path"])?;
    for (k, d) in [("platform_variant", Value::Null), ("kind", json!("video")), ("duration_s", json!(0.0)), ("thumb", json!(""))] {
        if !m.contains_key(k) { m.insert(k.into(), d); }
    }
    Ok(v)
}

pub fn norm_post(mut v: Value) -> Result<Value, String> {
    let m = obj(&mut v);
    req(m, &["project_id", "title"])?;
    dflt(m, "id", json!(new_id("post", 10)));
    for (k, d) in [("source_ref", Value::Null), ("render_job_id", Value::Null), ("idea_id", Value::Null), ("status", json!("draft"))] {
        if !m.contains_key(k) { m.insert(k.into(), d); }
    }
    one_of(m, "status", POST_STATUSES)?;
    let now = now_iso();
    dflt(m, "created_at", json!(now));
    dflt(m, "updated_at", json!(now));
    dflt(m, "approval", json!({}));
    let ap = obj(m.get_mut("approval").unwrap());
    for (k, d) in [("required", json!(true)), ("by", Value::Null), ("at", Value::Null), ("note", Value::Null)] {
        if !ap.contains_key(k) { ap.insert(k.into(), d); }
    }
    for key in ["media", "targets"] {
        let arr = m.remove(key).unwrap_or(json!([]));
        let mut out = vec![];
        for x in arr.as_array().cloned().unwrap_or_default() {
            out.push(if key == "media" { norm_media(x)? } else { norm_target(x)? });
        }
        m.insert(key.into(), Value::Array(out));
    }
    Ok(v)
}

pub fn norm_slot(mut v: Value) -> Result<Value, String> {
    let m = obj(&mut v);
    req(m, &["account_id", "rrule"])?;
    dflt(m, "id", json!(new_id("slot", 10)));
    for (k, d) in [("label", json!("")), ("template_hint", Value::Null)] { if !m.contains_key(k) { m.insert(k.into(), d); } }
    Ok(v)
}
pub fn norm_note(mut v: Value) -> Result<Value, String> {
    let m = obj(&mut v);
    req(m, &["date", "text"])?;
    dflt(m, "id", json!(new_id("note", 8)));
    dflt(m, "color", json!("#f59e0b"));
    Ok(v)
}
pub fn norm_rule(mut v: Value) -> Result<Value, String> {
    let m = obj(&mut v);
    req(m, &["name", "trigger"])?;
    one_of(m, "trigger", &["render_done", "idea_scripted", "schedule_gap"])?;
    dflt(m, "id", json!(new_id("rule", 8)));
    for (k, d) in [("enabled", json!(false)), ("conditions", json!({})), ("actions", json!({})), ("last_run_at", Value::Null),
        ("run_count", json!(0)), ("last_log", Value::Null)] { if !m.contains_key(k) { m.insert(k.into(), d); } }
    Ok(v)
}
pub fn norm_notification(mut v: Value) -> Result<Value, String> {
    let m = obj(&mut v);
    req(m, &["type", "title", "message"])?;
    dflt(m, "id", json!(new_id("notif", 8)));
    for (k, d) in [("created_at", json!(now_iso())), ("read", json!(false)), ("action_url", Value::Null), ("data", json!({}))] {
        if !m.contains_key(k) { m.insert(k.into(), d); }
    }
    Ok(v)
}
pub fn norm_template(mut v: Value) -> Result<Value, String> {
    let m = obj(&mut v);
    req(m, &["name"])?;
    dflt(m, "id", json!(new_id("tpl", 8)));
    let now = now_iso();
    for (k, d) in [("purpose", json!("general")), ("platforms", json!([])), ("caption_hook", json!("")), ("caption_body", json!("")),
        ("caption_cta", json!("")), ("hashtags", json!([])), ("first_comment", json!("")), ("footer", json!("")),
        ("disclosure", json!("")), ("created_at", json!(now)), ("updated_at", json!(now))] { if !m.contains_key(k) { m.insert(k.into(), d); } }
    Ok(v)
}
pub fn norm_snapshot(mut v: Value) -> Result<Value, String> {
    let m = obj(&mut v);
    req(m, &["post_id", "target_id", "platform"])?;
    for (k, d) in [("timestamp", json!(now_iso())), ("checkpoint", json!("custom")), ("views", json!(0)), ("likes", json!(0)),
        ("comments", json!(0)), ("shares", json!(0)), ("saves", json!(0)), ("watch_time_s", json!(0.0)),
        ("avg_retention_pct", json!(0.0)), ("subscribers_gained", json!(0)), ("ctr", json!(0.0)), ("revenue", json!(0.0)),
        ("raw", json!({}))] { if !m.contains_key(k) { m.insert(k.into(), d); } }
    Ok(v)
}

pub fn s<'a>(v: &'a Value, k: &str) -> &'a str { v.get(k).and_then(|x| x.as_str()).unwrap_or("") }
