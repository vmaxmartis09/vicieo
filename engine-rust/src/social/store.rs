use super::models::*;
use crate::workspace;
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

static LOCKS: Lazy<Mutex<HashMap<String, Arc<Mutex<()>>>>> = Lazy::new(Default::default);

pub fn atomic_write_json(path: &Path, data: &Value) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(dir)?;
    let tmp = dir.join(format!(".tmp_{}_{}", path.file_name().unwrap().to_string_lossy(), uuid::Uuid::new_v4().simple()));
    let res = (|| {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(serde_json::to_string_pretty(data)?.as_bytes())?;
        f.sync_all()?;
        fs::rename(&tmp, path)
    })();
    if res.is_err() { let _ = fs::remove_file(&tmp); }
    res
}
pub fn read_json(path: &Path, default: Value) -> Value {
    fs::read_to_string(path).ok().filter(|t| !t.trim().is_empty()).and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(default)
}

#[derive(Clone)]
pub struct SocialStore {
    pub project_id: String,
    pub pub_dir: PathBuf,
    pub posts_dir: PathBuf,
    pub metrics_dir: PathBuf,
    pub insights_dir: PathBuf,
    pub analytics_dir: PathBuf,
    mu: Arc<Mutex<()>>,
}

impl SocialStore {
    pub fn new(project_id: &str) -> anyhow::Result<Self> {
        let pp = |p: &[&str]| -> anyhow::Result<PathBuf> {
            let d = workspace::project_path(project_id, p, false)?;
            fs::create_dir_all(&d)?;
            Ok(d)
        };
        let pub_dir = pp(&["publishing"])?;
        let posts_dir = pp(&["publishing", "posts"])?;
        pp(&["publishing", "inbox", "_done"])?;
        let analytics_dir = pp(&["analytics"])?;
        let metrics_dir = pp(&["analytics", "metrics"])?;
        let insights_dir = pp(&["analytics", "insights"])?;
        let mu = LOCKS.lock().unwrap().entry(project_id.to_string()).or_default().clone();
        Ok(Self { project_id: project_id.into(), pub_dir, posts_dir, metrics_dir, insights_dir, analytics_dir, mu })
    }

    /// Per-project in-process mutex + flock on `.store.lock` (same file Python locks). Not reentrant: never nest.
    fn locked<T>(&self, f: impl FnOnce() -> T) -> T {
        let _g = self.mu.lock().unwrap_or_else(|e| e.into_inner());
        let lf = fs::OpenOptions::new().create(true).append(true).open(self.pub_dir.join(".store.lock")).ok();
        if let Some(l) = &lf { unsafe { libc::flock(l.as_raw_fd(), libc::LOCK_EX) }; }
        let r = f();
        if let Some(l) = &lf { unsafe { libc::flock(l.as_raw_fd(), libc::LOCK_UN) }; }
        r
    }

    fn list_file(&self, name: &str, norm: fn(Value) -> Result<Value, String>) -> Vec<Value> {
        self.locked(|| self.list_file_unlocked(name, norm))
    }
    fn list_file_unlocked(&self, name: &str, norm: fn(Value) -> Result<Value, String>) -> Vec<Value> {
        read_json(&self.pub_dir.join(name), json!([])).as_array().cloned().unwrap_or_default()
            .into_iter().filter_map(|x| norm(x).ok()).collect()
    }
    fn write_list(&self, name: &str, items: &[Value]) {
        let _ = atomic_write_json(&self.pub_dir.join(name), &Value::Array(items.to_vec()));
    }
    fn upsert(&self, name: &str, norm: fn(Value) -> Result<Value, String>, item: &Value, replace_pos: bool) {
        self.locked(|| {
            let mut items = self.list_file_unlocked(name, norm);
            match items.iter().position(|x| s(x, "id") == s(item, "id")) {
                Some(i) if replace_pos => items[i] = item.clone(),
                Some(_) => { items.retain(|x| s(x, "id") != s(item, "id")); items.push(item.clone()) }
                None => items.push(item.clone()),
            }
            self.write_list(name, &items);
        })
    }
    fn remove(&self, name: &str, norm: fn(Value) -> Result<Value, String>, id: &str) -> bool {
        self.locked(|| {
            let mut items = self.list_file_unlocked(name, norm);
            let n = items.len();
            items.retain(|x| s(x, "id") != id);
            if items.len() < n { self.write_list(name, &items); true } else { false }
        })
    }

    // --- accounts ---
    pub fn list_accounts(&self) -> Vec<Value> { self.list_file("accounts.json", norm_account) }
    pub fn get_account(&self, id: &str) -> Option<Value> { self.list_accounts().into_iter().find(|a| s(a, "id") == id) }
    pub fn save_account(&self, acc: Value, enforce_limit: bool) -> Result<Value, String> {
        let acc = norm_account(acc)?;
        self.locked(|| {
            let mut items = self.list_file_unlocked("accounts.json", norm_account);
            let h = s(&acc, "handle").trim().to_lowercase();
            if !h.is_empty() && items.iter().any(|a| s(a, "id") != s(&acc, "id") && s(a, "platform") == s(&acc, "platform") && s(a, "handle").trim().to_lowercase() == h) {
                return Err(format!("Tài khoản với handle '{}' trên nền tảng {} đã tồn tại.", s(&acc, "handle"), s(&acc, "platform")));
            }
            match items.iter().position(|a| s(a, "id") == s(&acc, "id")) {
                Some(i) => items[i] = acc.clone(),
                None => {
                    if enforce_limit && items.len() >= MAX_ACCOUNTS {
                        return Err(format!("Tối đa {MAX_ACCOUNTS} tài khoản mỗi dự án (MAX_ACCOUNTS = {MAX_ACCOUNTS})"));
                    }
                    items.push(acc.clone());
                }
            }
            self.write_list("accounts.json", &items);
            Ok(acc)
        })
    }
    pub fn delete_account(&self, id: &str) -> bool { self.remove("accounts.json", norm_account, id) }

    // --- posts ---
    pub fn list_posts(&self, status: Option<&str>, platform: Option<&str>, account_id: Option<&str>, from: Option<&str>, to: Option<&str>, q: Option<&str>) -> Vec<Value> {
        let mut posts: Vec<Value> = self.locked(|| {
            let Ok(rd) = fs::read_dir(&self.posts_dir) else { return vec![] };
            rd.flatten().filter(|e| e.path().extension().map_or(false, |x| x == "json"))
                .filter_map(|e| norm_post(read_json(&e.path(), Value::Null)).ok()).collect()
        });
        posts.sort_by(|a, b| s(b, "updated_at").cmp(s(a, "updated_at")));
        let tg = |p: &Value| p["targets"].as_array().cloned().unwrap_or_default();
        posts.into_iter().filter(|p| {
            let ts = tg(p);
            if let Some(st) = status { if s(p, "status") != st && !ts.iter().any(|t| s(t, "status") == st) { return false; } }
            if let Some(pl) = platform { if !ts.iter().any(|t| s(t, "platform") == pl) { return false; } }
            if let Some(a) = account_id { if !ts.iter().any(|t| s(t, "account_id") == a) { return false; } }
            if from.is_some() || to.is_some() {
                let ok = ts.iter().any(|t| {
                    let d = t.get("scheduled_at").and_then(|x| x.as_str()).filter(|x| !x.is_empty()).unwrap_or(s(p, "created_at"));
                    let d = d.get(..10).unwrap_or(d);
                    from.map_or(true, |f| d >= f) && to.map_or(true, |e| d <= e)
                });
                if !ok { return false; }
            }
            if let Some(q) = q {
                let q = q.to_lowercase();
                if !(s(p, "title").to_lowercase().contains(&q) || ts.iter().any(|t| s(t, "caption").to_lowercase().contains(&q))) { return false; }
            }
            true
        }).collect()
    }
    fn post_path(&self, id: &str) -> Option<PathBuf> {
        // ids come from URLs: refuse path separators
        (!id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')).then(|| self.posts_dir.join(format!("{id}.json")))
    }
    pub fn get_post(&self, id: &str) -> Option<Value> {
        let p = self.post_path(id)?;
        self.locked(|| norm_post(read_json(&p, Value::Null)).ok())
    }
    pub fn save_post(&self, mut post: Value) -> Result<Value, String> {
        post = norm_post(post)?;
        post["updated_at"] = json!(now_iso());
        let p = self.post_path(s(&post, "id")).ok_or("invalid post id")?;
        self.locked(|| atomic_write_json(&p, &post)).map_err(|e| e.to_string())?;
        Ok(post)
    }
    pub fn delete_post(&self, id: &str) -> bool {
        let Some(p) = self.post_path(id) else { return false };
        self.locked(|| p.exists() && fs::remove_file(&p).is_ok())
    }

    // --- slots / notes / rules / notifications ---
    pub fn list_slots(&self, account_id: Option<&str>) -> Vec<Value> {
        self.list_file("slots.json", norm_slot).into_iter().filter(|x| account_id.map_or(true, |a| s(x, "account_id") == a)).collect()
    }
    pub fn save_slots(&self, slots: &[Value]) { self.locked(|| self.write_list("slots.json", slots)) }
    pub fn list_notes(&self, from: Option<&str>, to: Option<&str>) -> Vec<Value> {
        self.list_file("notes.json", norm_note).into_iter().filter(|n| from.map_or(true, |f| s(n, "date") >= f) && to.map_or(true, |t| s(n, "date") <= t)).collect()
    }
    pub fn save_notes(&self, notes: &[Value]) { self.locked(|| self.write_list("notes.json", notes)) }

    pub fn list_automation_rules(&self) -> Vec<Value> {
        self.locked(|| {
            let path = self.pub_dir.join("automation_rules.json");
            if !path.exists() || read_json(&path, Value::Null).is_null() {
                let d: Vec<Value> = [
                    json!({"id":"rule_render_auto","name":"Tự động tạo bài nháp khi Render video xong","enabled":false,"trigger":"render_done",
                        "conditions":{"template_id_prefix":"tiktok-sales-"},"actions":{"create_draft":true,"platforms":["tiktok","reels"],"auto_approve":false}}),
                    json!({"id":"rule_schedule_gap","name":"Cảnh báo lấp lịch trống tuần tới","enabled":true,"trigger":"schedule_gap",
                        "conditions":{"min_weekly_posts":3},"actions":{"notify":true}}),
                ].into_iter().filter_map(|x| norm_rule(x).ok()).collect();
                self.write_list("automation_rules.json", &d);
                return d;
            }
            self.list_file_unlocked("automation_rules.json", norm_rule)
        })
    }
    pub fn save_automation_rules(&self, r: &[Value]) { self.locked(|| self.write_list("automation_rules.json", r)) }
    pub fn save_automation_rule(&self, rule: Value) {
        let mut rules = self.list_automation_rules();
        match rules.iter().position(|r| s(r, "id") == s(&rule, "id")) { Some(i) => rules[i] = rule, None => rules.push(rule) }
        self.save_automation_rules(&rules);
    }
    pub fn delete_automation_rule(&self, id: &str) -> bool {
        let mut rules = self.list_automation_rules();
        let n = rules.len();
        rules.retain(|r| s(r, "id") != id);
        if rules.len() < n { self.save_automation_rules(&rules); true } else { false }
    }
    pub fn list_notifications(&self, unread_only: bool, limit: usize) -> Vec<Value> {
        let mut n: Vec<Value> = self.list_file("notifications.json", norm_notification).into_iter().filter(|n| !unread_only || n["read"] != json!(true)).collect();
        n.sort_by(|a, b| s(b, "created_at").cmp(s(a, "created_at")));
        n.truncate(limit);
        n
    }
    pub fn add_notification(&self, n: Value) -> Result<Value, String> {
        let n = norm_notification(n)?;
        self.locked(|| {
            let mut items = self.list_file_unlocked("notifications.json", norm_notification);
            items.insert(0, n.clone());
            items.truncate(100);
            self.write_list("notifications.json", &items);
        });
        Ok(n)
    }
    pub fn mark_notification_read(&self, id: &str) -> bool {
        self.locked(|| {
            let mut items = self.list_file_unlocked("notifications.json", norm_notification);
            let mut ch = false;
            for n in items.iter_mut() { if s(n, "id") == id || id == "all" { n["read"] = json!(true); ch = true; } }
            if ch { self.write_list("notifications.json", &items); }
            ch
        })
    }
    pub fn clear_notifications(&self) { self.locked(|| self.write_list("notifications.json", &[])) }

    // --- templates (file is {version, description, templates:{id:{..}}} or a list) ---
    fn templates_unlocked(&self) -> Vec<Value> {
        match read_json(&self.pub_dir.join("templates.json"), json!({})) {
            Value::Array(a) => a.into_iter().filter_map(|x| norm_template(x).ok()).collect(),
            Value::Object(o) => {
                let t = o.get("templates").and_then(|t| t.as_object()).cloned().unwrap_or(o);
                t.into_iter().filter(|(_, v)| v.is_object()).filter_map(|(k, mut v)| {
                    v.as_object_mut().unwrap().insert("id".into(), json!(k)); // Python: {"id": k, **v} => v wins
                    norm_template(v).ok()
                }).collect()
            }
            _ => vec![],
        }
    }
    fn write_templates(&self, t: &[Value]) {
        let map: serde_json::Map<String, Value> = t.iter().map(|x| (s(x, "id").to_string(), x.clone())).collect();
        let _ = atomic_write_json(&self.pub_dir.join("templates.json"),
            &json!({"version":"1.0.0","description":"Kho mẫu nội dung mạng xã hội","templates":map}));
    }
    pub fn list_templates(&self) -> Vec<Value> { self.locked(|| self.templates_unlocked()) }
    pub fn save_template(&self, t: Value) -> Result<Value, String> {
        let t = norm_template(t)?;
        self.locked(|| {
            let mut items = self.templates_unlocked();
            match items.iter().position(|x| s(x, "id") == s(&t, "id")) { Some(i) => items[i] = t.clone(), None => items.push(t.clone()) }
            self.write_templates(&items);
        });
        Ok(t)
    }
    pub fn delete_template(&self, id: &str) -> bool {
        self.locked(|| {
            let mut items = self.templates_unlocked();
            let n = items.len();
            items.retain(|x| s(x, "id") != id);
            if items.len() < n { self.write_templates(&items); true } else { false }
        })
    }

    // --- audit ---
    pub fn append_audit(&self, actor: &str, action: &str, post_id: &str, target_id: Option<&str>, from: Option<&str>, to: Option<&str>, note: &str) {
        let e = json!({"ts":now_iso(),"actor":actor,"action":action,"post_id":post_id,"target_id":target_id,"from_status":from,"to_status":to,"note":note});
        self.locked(|| {
            if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(self.pub_dir.join("audit.jsonl")) {
                let _ = writeln!(f, "{e}");
            }
        });
    }
    pub fn get_audit_log(&self, post_id: Option<&str>, limit: usize) -> Vec<Value> {
        let txt = self.locked(|| fs::read_to_string(self.pub_dir.join("audit.jsonl")).unwrap_or_default());
        let v: Vec<Value> = txt.lines().filter_map(|l| serde_json::from_str::<Value>(l.trim()).ok())
            .filter(|r| post_id.map_or(true, |p| s(r, "post_id") == p)).collect();
        v[v.len().saturating_sub(limit)..].to_vec()
    }

    // --- metrics / insights ---
    fn metric_path(&self, target_id: &str) -> Option<PathBuf> {
        (!target_id.is_empty() && target_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')).then(|| self.metrics_dir.join(format!("{target_id}.jsonl")))
    }
    pub fn append_metrics_snapshot(&self, snap: &Value) {
        let Some(p) = self.metric_path(s(snap, "target_id")) else { return };
        self.locked(|| if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(p) { let _ = writeln!(f, "{snap}"); });
    }
    pub fn get_metrics_snapshots(&self, target_id: &str) -> Vec<Value> {
        let Some(p) = self.metric_path(target_id) else { return vec![] };
        let txt = self.locked(|| fs::read_to_string(p).unwrap_or_default());
        txt.lines().filter_map(|l| serde_json::from_str::<Value>(l.trim()).ok()).filter_map(|v| norm_snapshot(v).ok()).collect()
    }
    pub fn list_all_latest_metrics(&self) -> HashMap<String, Value> {
        self.locked(|| {
            let mut out = HashMap::new();
            if let Ok(rd) = fs::read_dir(&self.metrics_dir) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.extension().map_or(true, |x| x != "jsonl") { continue; }
                    let txt = fs::read_to_string(&p).unwrap_or_default();
                    if let Some(v) = txt.lines().filter(|l| !l.trim().is_empty()).last().and_then(|l| serde_json::from_str::<Value>(l).ok()).and_then(|v| norm_snapshot(v).ok()) {
                        out.insert(p.file_stem().unwrap().to_string_lossy().into(), v);
                    }
                }
            }
            out
        })
    }
    pub fn save_insights(&self, date: &str, data: &Value) { if date.chars().all(|c| c.is_ascii_digit() || c == '-') { self.locked(|| { let _ = atomic_write_json(&self.insights_dir.join(format!("{date}.json")), data); }) } }
    pub fn get_insights(&self, date: &str) -> Option<Value> {
        if !date.chars().all(|c| c.is_ascii_digit() || c == '-') { return None; }
        let p = self.insights_dir.join(format!("{date}.json"));
        self.locked(|| p.exists().then(|| read_json(&p, Value::Null)))
    }
    pub fn save_learnings_md(&self, c: &str) { self.locked(|| { let _ = fs::write(self.analytics_dir.join("learnings.md"), c); }) }
    pub fn get_learnings_md(&self) -> String {
        fs::read_to_string(self.analytics_dir.join("learnings.md")).unwrap_or_else(|_| "# Bài học tuần này\n\nChưa có báo cáo phân tích mới.".into())
    }
    pub fn save_recommendations_json(&self, d: &Value) { self.locked(|| { let _ = atomic_write_json(&self.analytics_dir.join("recommendations.json"), d); }) }
    pub fn get_recommendations_json(&self) -> Value { self.locked(|| read_json(&self.analytics_dir.join("recommendations.json"), json!({}))) }
}
