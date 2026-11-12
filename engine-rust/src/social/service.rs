use super::models::*;
use super::store::SocialStore;
use super::ApiErr;
use chrono::{Datelike, Duration, TimeZone, Timelike};
use serde_json::{json, Value};

fn valid_next(cur: &str) -> &'static [&'static str] {
    match cur {
        "draft" => &["pending_approval", "cancelled"],
        "pending_approval" => &["approved", "draft", "cancelled"],
        "approved" => &["scheduled", "publishing", "draft", "cancelled"],
        "scheduled" => &["publishing", "approved", "draft", "cancelled"],
        "publishing" => &["published", "failed"],
        "failed" => &["scheduled", "draft", "cancelled"],
        "cancelled" => &["draft"],
        _ => &[],
    }
}
fn verify(cur: &str, to: &str) -> Result<(), ApiErr> {
    if valid_next(cur).contains(&to) { Ok(()) } else { Err(ApiErr::conflict(format!("Cannot transition post status from '{cur}' to '{to}'"))) }
}

pub struct SocialService { pub store: SocialStore }

impl SocialService {
    pub fn new(project_id: &str) -> Result<Self, ApiErr> {
        Ok(Self { store: SocialStore::new(project_id).map_err(|e| ApiErr::bad(e.to_string()))? })
    }
    fn post(&self, id: &str) -> Result<Value, ApiErr> { self.store.get_post(id).ok_or_else(|| ApiErr::not_found(format!("Post {id} not found"))) }
    fn save(&self, p: Value) -> Result<Value, ApiErr> { self.store.save_post(p).map_err(ApiErr::invalid) }
    fn each_target(p: &mut Value, mut f: impl FnMut(&mut Value)) { for t in p["targets"].as_array_mut().into_iter().flatten() { f(t) } }

    pub fn submit_for_approval(&self, id: &str, actor: &str, note: &str) -> Result<Value, ApiErr> {
        let mut p = self.post(id)?;
        let old = s(&p, "status").to_string();
        verify(&old, "pending_approval")?;
        p["status"] = json!("pending_approval");
        if !note.is_empty() { p["approval"]["note"] = json!(note); }
        Self::each_target(&mut p, |t| if matches!(s(t, "status"), "draft" | "cancelled") { t["status"] = json!("pending_approval") });
        let p = self.save(p)?;
        self.store.append_audit(actor, "submit_approval", id, None, Some(&old), Some("pending_approval"), note);
        Ok(p)
    }
    pub fn approve(&self, id: &str, actor: &str, note: &str) -> Result<Value, ApiErr> {
        let mut p = self.post(id)?;
        let old = s(&p, "status").to_string();
        verify(&old, "approved")?;
        p["status"] = json!("approved");
        p["approval"]["by"] = json!(actor);
        p["approval"]["at"] = json!(now_iso());
        if !note.is_empty() { p["approval"]["note"] = json!(note); }
        Self::each_target(&mut p, |t| if s(t, "status") == "pending_approval" { t["status"] = json!("approved") });
        let p = self.save(p)?;
        self.store.append_audit(actor, "approve", id, None, Some(&old), Some("approved"), note);
        Ok(p)
    }
    pub fn reject(&self, id: &str, actor: &str, reason: &str) -> Result<Value, ApiErr> {
        let mut p = self.post(id)?;
        let old = s(&p, "status").to_string();
        verify(&old, "draft")?;
        p["status"] = json!("draft");
        p["approval"]["note"] = json!(if reason.is_empty() { "Rejected".to_string() } else { format!("Rejected: {reason}") });
        Self::each_target(&mut p, |t| if s(t, "status") == "pending_approval" { t["status"] = json!("draft") });
        let p = self.save(p)?;
        self.store.append_audit(actor, "reject", id, None, Some(&old), Some("draft"), reason);
        Ok(p)
    }
    /// `times`: target_id or platform -> ISO time.
    pub fn schedule(&self, id: &str, times: &serde_json::Map<String, Value>, actor: &str) -> Result<Value, ApiErr> {
        let mut p = self.post(id)?;
        let old = s(&p, "status").to_string();
        if p["approval"]["required"] != json!(false) && old != "approved" {
            return Err(ApiErr::conflict("Post requires approval before scheduling (current status is not approved)"));
        }
        verify(&old, "scheduled")?;
        Self::each_target(&mut p, |t| {
            let v = times.get(s(t, "id")).or_else(|| times.get(s(t, "platform"))).cloned();
            if let Some(v) = v { t["scheduled_at"] = v; }
            if t["scheduled_at"].as_str().map_or(false, |x| !x.is_empty()) { t["status"] = json!("scheduled"); }
        });
        p["status"] = json!("scheduled");
        let p = self.save(p)?;
        self.store.append_audit(actor, "schedule", id, None, Some(&old), Some("scheduled"), "");
        Ok(p)
    }
    pub fn update_target_time(&self, id: &str, tid: &str, new_iso: &str, actor: &str) -> Result<Value, ApiErr> {
        let mut p = self.post(id)?;
        let mut old = Value::Null;
        let mut found = false;
        let mut published = false;
        Self::each_target(&mut p, |t| if s(t, "id") == tid {
            found = true;
            if s(t, "status") == "published" { published = true; return; }
            old = t["scheduled_at"].clone();
            t["scheduled_at"] = json!(new_iso);
        });
        if !found { return Err(ApiErr::not_found(format!("Target {tid} not found on post {id}"))); }
        if published { return Err(ApiErr::conflict("Cannot reschedule an already published target")); }
        let p = self.save(p)?;
        let o = if old.is_null() { "None".to_string() } else { old.as_str().unwrap_or("").to_string() };
        self.store.append_audit(actor, "reschedule_target", id, Some(tid), None, None, &format!("From {o} to {new_iso}"));
        Ok(p)
    }
    pub fn cancel(&self, id: &str, actor: &str) -> Result<Value, ApiErr> {
        let mut p = self.post(id)?;
        let old = s(&p, "status").to_string();
        verify(&old, "cancelled")?;
        p["status"] = json!("cancelled");
        Self::each_target(&mut p, |t| if s(t, "status") != "published" { t["status"] = json!("cancelled") });
        let p = self.save(p)?;
        self.store.append_audit(actor, "cancel", id, None, Some(&old), Some("cancelled"), "");
        Ok(p)
    }
    pub fn duplicate(&self, id: &str, actor: &str) -> Result<Value, ApiErr> {
        let mut d = self.post(id)?;
        d["id"] = json!(new_id("post", 10));
        d["title"] = json!(format!("{} (Copy)", s(&d, "title")));
        d["status"] = json!("draft");
        d["approval"] = json!({"required": true, "by": null, "at": null, "note": null});
        d["created_at"] = json!(now_iso());
        Self::each_target(&mut d, |t| {
            t["id"] = json!(new_id("tgt", 10));
            t["status"] = json!("draft");
            for k in ["external_id", "external_url", "last_error", "published_at"] { t[k] = Value::Null; }
            t["attempts"] = json!(0);
        });
        let d = self.save(d)?;
        self.store.append_audit(actor, "duplicate", s(&d, "id"), None, None, None, &format!("Cloned from {id}"));
        Ok(d)
    }

    pub fn suggest_time(&self, account_id: &str, after: Option<&str>) -> String {
        let now = chrono::Utc::now().with_timezone(&tz());
        let Some(acc) = self.store.get_account(account_id) else { return (now + Duration::hours(2)).to_rfc3339() };
        let base = after.and_then(parse_dt).unwrap_or(now);
        let mut taken = vec![];
        for p in self.store.list_posts(None, None, Some(account_id), None, None, None) {
            for t in p["targets"].as_array().into_iter().flatten() {
                if s(t, "account_id") == account_id && matches!(s(t, "status"), "scheduled" | "publishing" | "published") {
                    if let Some(d) = parse_dt(s(t, "scheduled_at")) { taken.push(d); }
                }
            }
        }
        let limit = acc["daily_limit"].as_i64().filter(|n| *n > 0).unwrap_or(5);
        let wins = acc["posting_windows"].as_array().cloned().unwrap_or_default();
        let cand = base + Duration::minutes(15);
        for off in 0..14 {
            let day = (cand + Duration::days(off)).date_naive();
            if taken.iter().filter(|t| t.date_naive() == day).count() as i64 >= limit { continue; }
            let wd = day.weekday().num_days_from_monday() as i64;
            let mut starts: Vec<String> = wins.iter().filter(|w| w["weekday"].as_i64() == Some(wd)).map(|w| s(w, "start").to_string()).collect();
            if starts.is_empty() { starts.push("19:00".into()); }
            for st in starts {
                let mut it = st.split(':').map(|x| x.parse::<u32>().unwrap_or(0));
                let (h, m) = (it.next().unwrap_or(19), it.next().unwrap_or(0));
                let Some(slot) = tz().with_ymd_and_hms(day.year(), day.month(), day.day(), h, m, 0).single() else { continue };
                if slot <= base { continue; }
                if !taken.iter().any(|t| (*t - slot).num_seconds().abs() < 7200) { return slot.to_rfc3339(); }
            }
        }
        (base + Duration::hours(2)).to_rfc3339()
    }

    pub fn schedule_idea(&self, idea_id: &str, account_id: &str, at: Option<&str>) -> Result<Value, ApiErr> {
        let acc = self.store.get_account(account_id).ok_or_else(|| ApiErr::not_found(format!("Account {account_id} not found")))?;
        if !idea_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') { return Err(ApiErr::bad("invalid idea id")); }
        let idea: Value = crate::workspace::project_path(&self.store.project_id, &["content", "ideas", &format!("{idea_id}.json")], false).ok()
            .and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(json!({}));
        let title = idea["title"].as_str().map(String::from).unwrap_or_else(|| format!("Nội dung từ ý tưởng {idea_id}"));
        let when = at.map(String::from).unwrap_or_else(|| self.suggest_time(account_id, None));
        let post = json!({
            "project_id": self.store.project_id, "title": title, "idea_id": idea_id,
            "targets": [{"account_id": acc["id"], "platform": acc["platform"], "scheduled_at": when, "title": title,
                "caption": format!("Kịch bản cho ý tưởng: {title}"), "hashtags": acc["default_hashtags"]}],
        });
        let p = self.save(post)?;
        self.store.append_audit("service", "schedule_idea", s(&p, "id"), None, None, None, &format!("Linked to idea {idea_id}"));
        Ok(p)
    }
}

pub fn caption_variants(title: &str, caption: &str, hashtags: &[String], platforms: &[String], link: &str, footer: &str) -> Value {
    let tags = hashtags.iter().map(|t| format!("#{}", t.trim_start_matches('#'))).collect::<Vec<_>>().join(" ");
    let cut = |s: String, n: usize| s.chars().take(n).collect::<String>();
    let mut out = serde_json::Map::new();
    let tail = |mut b: String, footer: &str, tags: &str| { if !footer.is_empty() { b += &format!("\n\n{footer}") } if !tags.is_empty() { b += &format!("\n\n{tags}") } b };
    for plat in platforms {
        let pl = plat.to_lowercase();
        let head = format!("{title}\n\n{caption}").trim().to_string();
        let v = match pl.as_str() {
            "tiktok" => {
                let mut b = head;
                if !link.is_empty() { b += &format!("\n\n🛒 Mua ngay tại giỏ hàng: {link} #affiliate #ad"); }
                json!({"caption": cut(tail(b, footer, &tags), 2200), "hashtags": hashtags, "disclosure_ad": !link.is_empty()})
            }
            "youtube" => {
                let mut b = format!("{caption}\n\n{footer}").trim().to_string();
                if !link.is_empty() { b += &format!("\n\n🔗 Link sản phẩm: {link}"); }
                if !tags.is_empty() { b += &format!("\n\n{tags}"); }
                json!({"title": cut(title.into(), 100), "description": cut(b, 5000), "tags": hashtags.iter().take(15).collect::<Vec<_>>(), "privacy": "public", "made_for_kids": false})
            }
            "instagram" => {
                let mut b = head;
                if !link.is_empty() { b += &format!("\n\n(Quảng cáo / Hợp tác) Link trong bio: {link}"); }
                json!({"caption": cut(tail(b, footer, &tags), 2200), "hashtags": hashtags.iter().take(30).collect::<Vec<_>>()})
            }
            "facebook" => {
                let mut b = head;
                if !link.is_empty() { b += &format!("\n\n👉 Xem chi tiết sản phẩm tại: {link}"); }
                json!({"caption": cut(tail(b, footer, &tags), 5000), "hashtags": hashtags})
            }
            _ => {
                let mut b = format!("{title} - {caption}");
                if !tags.is_empty() { b += &format!(" {tags}"); }
                json!({"caption": cut(b, 1000), "hashtags": hashtags})
            }
        };
        out.insert(pl, v);
    }
    Value::Object(out)
}

#[allow(dead_code)]
fn _unused(t: chrono::DateTime<chrono::FixedOffset>) -> u32 { t.hour() }
