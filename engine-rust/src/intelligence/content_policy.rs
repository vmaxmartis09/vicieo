//! Project content policy + checker (port of core/engine/content_policy.py): rules, platform packs, imports, URL fetch, check.
use super::util::*;
use crate::workspace;
use regex::Regex;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn rule(id: &str, cat: &str, label: &str, sev: &str, terms: &[&str], note: &str) -> Value {
    json!({"id": id, "category": cat, "label": label, "severity": sev, "enabled": true, "terms": terms, "note": note})
}

pub fn default_policy() -> Value {
    json!({"name": "Chính sách nội dung trẻ em (Made for Kids)", "mode": "standard",
        "scopes": {"dialogue": true, "caption": true, "script": true, "prompt": true}, "allowlist": [],
        "rules": [
            rule("violence", "Bạo lực", "Bạo lực, gây thương tích", "block", &["giết", "đánh nhau", "đấm", "đá vào mặt", "máu me", "đổ máu", "tra tấn", "dao găm", "súng", "bắn chết", "kill", "gore", "blood"], "Nội dung dành cho trẻ 3–7 tuổi: không mô tả hoặc cổ xuý bạo lực."),
            rule("scary", "Gây sợ hãi", "Kinh dị, đáng sợ", "warn", &["ma quỷ", "kinh dị", "xác chết", "rùng rợn", "ác mộng kinh hoàng", "horror"], "Tránh gây ám ảnh; nếu cần (vd. bài học sợ bóng tối) phải xử lý nhẹ nhàng."),
            rule("adult", "Người lớn", "Nội dung người lớn", "block", &["khiêu dâm", "sex", "làm tình", "nude", "khỏa thân", "18+"], ""),
            rule("profanity", "Thô tục", "Chửi thề, lăng mạ", "block", &["đồ ngu", "ngu ngốc", "đm", "đ.m", "chết tiệt", "khốn nạn", "đồ điên", "stupid", "idiot", "damn"], "Tránh lời lẽ xúc phạm; dùng cách nói ôn hòa."),
            rule("hate", "Phân biệt đối xử", "Kỳ thị, miệt thị", "block", &["kỳ thị", "miệt thị", "đồ béo", "đồ lùn", "đồ xấu xí", "dân tộc thấp kém"], ""),
            rule("danger", "Hành vi nguy hiểm", "Hành vi trẻ có thể bắt chước gây nguy hiểm", "warn", &["nhảy từ trên cao", "chơi với lửa", "uống thuốc một mình", "đi một mình ra đường", "đùa với điện", "leo lên mái nhà"], ""),
            rule("commercial", "Quảng cáo", "Quảng cáo, mua bán, thương hiệu", "warn", &["mua ngay", "đặt hàng", "giảm giá", "khuyến mãi", "link bio", "đăng ký kênh ngay"], "Made for Kids hạn chế kêu gọi hành động thương mại."),
            rule("personal", "Thông tin cá nhân", "Thông tin cá nhân / liên hệ", "block", &["số điện thoại", "địa chỉ nhà", "zalo", "facebook của bé", "gmail.com"], ""),
        ]})
}

fn policy_file(pid: &str) -> PathBuf { pp(pid, &["policy.json"], false) }

pub fn load_policy(pid: &str) -> Value {
    let mut base = default_policy();
    if let Some(Value::Object(data)) = read_json(&policy_file(pid)) {
        if data.get("rules").is_some_and(|r| r.is_array()) {
            let mut scopes = base["scopes"].as_object().cloned().unwrap_or_default();
            if let Some(Value::Object(sc)) = data.get("scopes") { for (k, v) in sc { scopes.insert(k.clone(), v.clone()); } }
            for (k, v) in &data { if k != "scopes" { base[k] = v.clone(); } }
            base["scopes"] = Value::Object(scopes);
        }
    }
    base
}

fn st(v: &Value, k: &str) -> String { v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string() }
fn strs(v: Option<&Value>) -> Vec<String> { v.and_then(|x| x.as_array()).map(|a| a.iter().map(|t| t.as_str().map(String::from).unwrap_or_else(|| t.to_string()).trim().to_string()).filter(|t| !t.is_empty()).collect()).unwrap_or_default() }

pub fn save_policy(pid: &str, policy: &Value) -> anyhow::Result<Value> {
    let d = default_policy();
    let mut scopes = Map::new();
    for k in d["scopes"].as_object().unwrap().keys() { scopes.insert(k.clone(), json!(policy.get("scopes").and_then(|s| s.get(k)).and_then(|v| v.as_bool()).unwrap_or(true))); }
    let mut rules: Vec<Value> = vec![];
    for r in policy.get("rules").and_then(|r| r.as_array()).cloned().unwrap_or_default() {
        let n = rules.len() + 1;
        let cat = Some(st(&r, "category")).filter(|c| !c.is_empty()).unwrap_or_else(|| "Khác".into());
        let mut o = json!({"id": Some(st(&r, "id")).filter(|c| !c.is_empty()).unwrap_or(format!("rule_{n}")), "category": cat,
            "label": Some(st(&r, "label")).filter(|c| !c.is_empty()).unwrap_or(cat.clone()).to_string().replace("\u{0}", ""), "severity": if st(&r, "severity") == "warn" { "warn" } else { "block" },
            "enabled": r.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true), "terms": strs(r.get("terms")), "note": st(&r, "note")});
        if r.get("source").and_then(|v| v.as_str()).is_some_and(|s| !s.is_empty()) { o["source"] = json!(st(&r, "source")); }
        rules.push(o);
    }
    let refs: Vec<Value> = policy.get("references").and_then(|r| r.as_array()).map(|a| a.iter().filter(|r| r.get("url").and_then(|u| u.as_str()).is_some_and(|u| !u.is_empty())).map(|r| {
        let mut m = Map::new();
        for k in ["url", "title", "kind", "fetchedAt", "excerpt"] { if let Some(v) = r.get(k) { m.insert(k.into(), json!(v.as_str().map(String::from).unwrap_or_else(|| v.to_string()))); } }
        Value::Object(m) }).collect()).unwrap_or_default();
    let docs: Vec<Value> = policy.get("documents").and_then(|r| r.as_array()).map(|a| a.iter().filter(|d| d.get("file").is_some_and(|f| f.as_str().is_some_and(|f| !f.is_empty()))).map(|d| {
        let mut m = Map::new();
        for k in ["id", "name", "file", "chars", "sha", "sections", "importedAt"] { if let Some(v) = d.get(k) { m.insert(k.into(), v.clone()); } }
        Value::Object(m) }).collect()).unwrap_or_default();
    let clean = json!({"name": Some(st(policy, "name")).filter(|n| !n.is_empty()).unwrap_or_else(|| st(&d, "name")), "mode": if st(policy, "mode") == "strict" { "strict" } else { "standard" },
        "scopes": scopes, "allowlist": strs(policy.get("allowlist")), "references": refs, "documents": docs, "rules": rules});
    write_json(&pp(pid, &["policy.json"], true), &clean)?;
    Ok(clean)
}

pub fn merge_extra_terms(policy: &Value, extra: &[String], severity: &str) -> Value {
    let terms: Vec<String> = extra.iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
    if terms.is_empty() { return policy.clone(); }
    let mut out = policy.clone();
    out["rules"].as_array_mut().unwrap().push(json!({"id": "node_extra", "category": "Riêng node", "label": "Từ khoá bổ sung của node", "severity": if severity == "warn" { "warn" } else { "block" }, "enabled": true, "terms": terms, "note": ""}));
    out
}

fn packs_dir() -> PathBuf { workspace::shared_path(&["policy_packs"], false).unwrap_or_default() }

pub fn list_packs() -> Vec<Value> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(packs_dir()).map(|rd| rd.flatten().map(|e| e.path()).collect()).unwrap_or_default();
    files.sort();
    files.into_iter().filter(|f| f.extension().is_some_and(|e| e == "json")).filter_map(|f| {
        let p = read_json(&f)?;
        if !p.is_object() { return None; }
        let stem = f.file_stem()?.to_string_lossy().to_string();
        Some(json!({"id": p.get("id").cloned().unwrap_or(json!(stem)), "name": p.get("name").cloned().unwrap_or(json!(stem)), "platform": st(&p, "platform"), "sourceUrl": st(&p, "sourceUrl"),
            "description": st(&p, "description"), "ruleCount": p.get("rules").and_then(|r| r.as_array()).map(|r| r.len()).unwrap_or(0)}))
    }).collect()
}

pub fn load_pack(id: &str) -> Option<Value> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-') { return None; }
    read_json(&packs_dir().join(format!("{id}.json")))
}

pub fn merge_rules(policy: &Value, rules: &[Value], source: &str) -> Value {
    let mut out = policy.clone();
    let arr = out["rules"].as_array_mut().unwrap();
    for r in rules {
        let rid = format!("{source}:{}", r.get("id").map(|v| v.as_str().map(String::from).unwrap_or_else(|| v.to_string())).unwrap_or_else(|| "None".into()));
        let mut item = r.clone();
        item["id"] = json!(rid); item["source"] = json!(source);
        match arr.iter().position(|x| x.get("id").and_then(|i| i.as_str()) == Some(&rid)) { Some(i) => arr[i] = item, None => arr.push(item) }
    }
    out
}

pub fn import_pack(policy: &Value, pack: &Value) -> Value {
    let rules = pack.get("rules").and_then(|r| r.as_array()).cloned().unwrap_or_default();
    let mut out = merge_rules(policy, &rules, &format!("pack-{}", pack.get("id").and_then(|v| v.as_str()).unwrap_or("x")));
    let mut refs: Vec<Value> = out.get("references").and_then(|r| r.as_array()).cloned().unwrap_or_default();
    refs.retain(|r| r.get("url") != pack.get("sourceUrl"));
    if pack.get("sourceUrl").is_some_and(|u| u.as_str().is_some_and(|u| !u.is_empty())) { refs.push(json!({"url": pack["sourceUrl"], "title": st(pack, "name"), "kind": "pack", "fetchedAt": ""})); }
    out["references"] = json!(refs);
    out
}

fn norm(s: &str) -> String { s.to_lowercase() }

pub fn parse_policy_text(filename: &str, text: &str) -> Result<Vec<Value>, String> {
    let mut rules: Vec<Value> = vec![];
    fn mk(rules: &mut Vec<Value>, category: &str, terms: Vec<String>, sev: &str) {
        let terms: Vec<String> = terms.into_iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
        if terms.is_empty() { return; }
        let mut rid = Regex::new(r"[^a-z0-9]+").unwrap().replace_all(&norm(category), "_").trim_matches('_').to_string();
        if rid.is_empty() { rid = format!("nhom_{}", rules.len() + 1); }
        if let Some(r) = rules.iter_mut().find(|r| r["id"] == rid) { r["terms"].as_array_mut().unwrap().extend(terms.into_iter().map(Value::from)); return; }
        rules.push(json!({"id": rid, "category": category, "label": category, "severity": sev, "enabled": true, "terms": terms, "note": ""}));
    }
    let sev_of = |r: &Value| if st(r, "severity") == "warn" { "warn" } else { "block" };
    if filename.to_lowercase().ends_with(".json") {
        let data: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if let Some(rs) = data.get("rules").and_then(|r| r.as_array()) {
            for r in rs {
                let cat = Some(st(r, "category")).filter(|c| !c.is_empty()).or(Some(st(r, "label")).filter(|c| !c.is_empty())).unwrap_or_else(|| "Tải lên".into());
                mk(&mut rules, &cat, strs(r.get("terms")), sev_of(r));
                if let Some(l) = rules.last_mut() { let lab = st(r, "label"); if !lab.is_empty() { l["label"] = json!(lab); } l["note"] = json!(st(r, "note")); }
            }
        } else if let Some(items) = data.as_array() {
            for it in items {
                match it {
                    Value::String(s) => mk(&mut rules, "Tải lên", vec![s.clone()], "block"),
                    Value::Object(_) => mk(&mut rules, &Some(st(it, "category")).filter(|c| !c.is_empty()).unwrap_or_else(|| "Tải lên".into()), strs(it.get("terms")), sev_of(it)),
                    _ => {}
                }
            }
        }
        return Ok(rules);
    }
    let mut cat = "Tải lên".to_string();
    let split = Regex::new(r"[,;|]").unwrap();
    for raw in text.replace("\r\n", "\n").split('\n') {
        let line = raw.trim().trim_start_matches('\u{feff}');
        if line.is_empty() { continue; }
        if line.starts_with('#') { let c = line.trim_start_matches('#').trim(); if !c.is_empty() { cat = c.to_string(); } continue; }
        if let Some((head, tail)) = line.split_once(':') {
            if !line.starts_with("http://") && !line.starts_with("https://") && !head.is_empty() && head.chars().count() <= 40 {
                mk(&mut rules, head.trim(), split.split(tail).map(String::from).collect(), "block");
                continue;
            }
        }
        let body = line.trim_start_matches(|c| matches!(c, '-' | '*' | '•' | ' '));
        mk(&mut rules, &cat, if line.contains(',') || line.contains(';') || line.contains('|') { split.split(body).map(String::from).collect() } else { vec![body.to_string()] }, "block");
    }
    Ok(rules)
}

pub fn looks_like_prose(text: &str) -> bool {
    let unified = text.replace("\r\n", "\n");
    let lines: Vec<&str> = unified.split('\n').map(str::trim).filter(|l| !l.is_empty()).collect();
    if lines.is_empty() { return false; }
    let wordy = lines.iter().filter(|l| l.split_whitespace().count() >= 8).count();
    text.chars().count() > 4000 && wordy as f64 / lines.len() as f64 > 0.4
}

pub fn extract_sections(text: &str, limit: usize) -> Vec<String> {
    let re = Regex::new(r"^(#{1,4})\s+(.+)$").unwrap();
    let mut out: Vec<String> = vec![];
    for raw in text.replace("\r\n", "\n").split('\n') {
        let line = raw.trim().replace('\\', "");
        let Some(m) = re.captures(&line) else { continue };
        let t = Regex::new(r"[*_`]+").unwrap().replace_all(&m[2], "").trim_matches(|c| c == ' ' || c == '-' || c == '=' || c == '#').to_string();
        let low = t.to_lowercase();
        if !t.is_empty() && !out.contains(&t) && !["source", "thu thap", "ngay thu"].iter().any(|p| low.starts_with(p)) { out.push(t.chars().take(120).collect()); }
        if out.len() >= limit { break; }
    }
    out
}

pub fn add_document(pid: &str, name: &str, text: &str) -> anyhow::Result<Value> {
    let stem_raw = std::path::Path::new(name).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let stem = Regex::new(r"[^a-z0-9]+").unwrap().replace_all(&slug(&stem_raw), "-").trim_matches('-').to_string();
    let stem = if stem.is_empty() { "chinh-sach".to_string() } else { stem };
    let fn_ = format!("{stem}.md");
    let p = pp(pid, &["policy", &fn_], true);
    std::fs::write(&p, text)?;
    let doc = json!({"id": stem, "name": stem_raw, "file": format!("policy/{fn_}"), "chars": text.chars().count(), "sha": format!("{:x}", Sha256::digest(text.as_bytes()))[..16].to_string(),
        "sections": extract_sections(text, 60), "importedAt": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()});
    let mut pol = load_policy(pid);
    let mut docs: Vec<Value> = pol.get("documents").and_then(|d| d.as_array()).cloned().unwrap_or_default();
    docs.retain(|d| d.get("id") != doc.get("id"));
    docs.push(doc.clone());
    pol["documents"] = json!(docs);
    save_policy(pid, &pol)?;
    Ok(doc)
}

pub fn read_documents(pid: &str, policy: &Value, max_chars: usize) -> String {
    let parts: Vec<String> = policy.get("documents").and_then(|d| d.as_array()).map(|a| a.iter().filter_map(|d| {
        let f = st(d, "file");
        if f.contains("..") { return None; }
        let t = std::fs::read_to_string(pp(pid, &[&f], false)).ok()?;
        Some(format!("## {}\n{}", st(d, "name"), t.replace('\\', "")))
    }).collect()).unwrap_or_default();
    parts.join("\n\n").chars().take(max_chars).collect()
}

pub fn add_reference(policy: &Value, r: Value) -> Value {
    let mut out = policy.clone();
    let mut refs: Vec<Value> = out.get("references").and_then(|x| x.as_array()).cloned().unwrap_or_default();
    refs.retain(|x| x.get("url") != r.get("url"));
    refs.push(r);
    out["references"] = json!(refs);
    out
}

fn is_word(c: char) -> bool { c.is_alphanumeric() || c == '_' }

/// Policy check over {source: text}; whole-word, case-insensitive, diacritic-preserving matching.
pub fn check_texts(texts: &[(String, String)], policy: &Value) -> Value {
    let allow: Vec<Vec<char>> = policy.get("allowlist").and_then(|a| a.as_array()).map(|a| a.iter().filter_map(|x| x.as_str()).filter(|x| !x.is_empty()).map(|x| norm(x).chars().collect()).collect()).unwrap_or_default();
    let mut found: Vec<Value> = vec![];
    for (source, raw) in texts {
        if raw.is_empty() { continue; }
        let rc: Vec<char> = raw.chars().collect();
        let (mut nc, mut idx): (Vec<char>, Vec<usize>) = (vec![], vec![]);
        for (i, c) in rc.iter().enumerate() { for l in c.to_lowercase() { nc.push(l); idx.push(i); } }
        for rule in policy.get("rules").and_then(|r| r.as_array()).into_iter().flatten() {
            if !rule.get("enabled").and_then(|e| e.as_bool()).unwrap_or(true) { continue; }
            for term in rule.get("terms").and_then(|t| t.as_array()).into_iter().flatten().filter_map(|t| t.as_str()) {
                let nt: Vec<char> = norm(term).trim().chars().collect();
                if nt.is_empty() || nt.len() > nc.len() { continue; }
                for m in 0..=(nc.len() - nt.len()) {
                    if nc[m..m + nt.len()] != nt[..] { continue; }
                    let end = m + nt.len();
                    if (m > 0 && is_word(nc[m - 1])) || (end < nc.len() && is_word(nc[end])) { continue; }
                    let win: Vec<char> = nc[m.saturating_sub(40)..(end + 40).min(nc.len())].to_vec();
                    if allow.iter().any(|a| !a.is_empty() && win.windows(a.len()).any(|w| w == &a[..]) && a.windows(nt.len()).any(|w| w == &nt[..])) { continue; }
                    let (s0, e0) = (idx[m], idx[end - 1] + 1);
                    let (cs, ce) = (s0.saturating_sub(24), (e0 + 24).min(rc.len()));
                    found.push(json!({"source": source, "ruleId": rule.get("id"), "category": rule.get("category"), "label": rule.get("label"), "severity": rule.get("severity").cloned().unwrap_or(json!("block")),
                        "term": rc[s0..e0].iter().collect::<String>(), "snippet": rc[cs..ce].iter().collect::<String>().replace('\n', " "), "note": rule.get("note").cloned().unwrap_or(json!(""))}));
                }
            }
        }
    }
    let mut order: Vec<(String, String, String)> = vec![];
    let mut best: std::collections::HashMap<(String, String, String), Value> = Default::default();
    for v in found {
        let key = (st(&v, "source"), st(&v, "term").to_lowercase(), st(&v, "snippet"));
        match best.get(&key) {
            None => { order.push(key.clone()); best.insert(key, v); }
            Some(old) if st(&v, "severity") == "block" && st(old, "severity") != "block" => { best.insert(key, v); }
            _ => {}
        }
    }
    let violations: Vec<Value> = order.into_iter().map(|k| best.remove(&k).unwrap()).collect();
    let strict = st(policy, "mode") == "strict";
    let blocking = violations.iter().filter(|v| strict || st(v, "severity") == "block").count();
    json!({"passed": blocking == 0, "violations": violations, "blockCount": blocking, "warnCount": violations.len() - blocking, "checkedSources": texts.iter().map(|(k, _)| k).collect::<Vec<_>>()})
}

pub fn format_report(r: &Value, limit: usize) -> String {
    let v = r["violations"].as_array().cloned().unwrap_or_default();
    if v.is_empty() { return "Chính sách: ĐẠT — không có từ ngữ vi phạm.".into(); }
    let head = format!("Chính sách: {} — {} vi phạm chặn, {} cảnh báo.", if r["passed"] == true { "CẢNH BÁO" } else { "KHÔNG ĐẠT" }, r["blockCount"], r["warnCount"]);
    let mut lines: Vec<String> = v.iter().take(limit).map(|x| format!("- [{}] {}: “{}” trong {} …{}…", st(x, "severity").to_uppercase(), st(x, "category"), st(x, "term"), st(x, "source"), st(x, "snippet"))).collect();
    if v.len() > limit { lines.push(format!("… và {} mục khác.", v.len() - limit)); }
    format!("{head}\n{}", lines.join("\n"))
}

async fn is_public_host(host: &str) -> bool {
    let Ok(addrs) = tokio::net::lookup_host((host, 443)).await else { return false };
    let addrs: Vec<_> = addrs.collect();
    !addrs.is_empty() && addrs.iter().all(|a| match a.ip() {
        std::net::IpAddr::V4(v) => !(v.is_private() || v.is_loopback() || v.is_link_local() || v.is_broadcast() || v.is_unspecified() || v.is_multicast() || v.octets()[0] == 0 || v.octets()[0] >= 240),
        std::net::IpAddr::V6(v) => !(v.is_loopback() || v.is_unspecified() || v.is_multicast() || (v.segments()[0] & 0xfe00) == 0xfc00 || (v.segments()[0] & 0xffc0) == 0xfe80),
    })
}

/// Public https policy page -> title + plain text. Blocks internal addresses (SSRF), re-checking every redirect hop.
pub async fn fetch_policy_page(url: &str) -> Result<Value, String> {
    let max_bytes = 1_500_000usize;
    let client = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).timeout(std::time::Duration::from_secs(12))
        .user_agent("Mozilla/5.0 (compatible; PolicyFetcher/1.0)").build().map_err(|e| e.to_string())?;
    let mut cur = url::Url::parse(url).map_err(|_| "Chỉ hỗ trợ đường dẫn https.".to_string())?;
    for _ in 0..4 {
        if cur.scheme() != "https" || cur.host_str().is_none() { return Err("Chỉ hỗ trợ đường dẫn https.".into()); }
        if !is_public_host(cur.host_str().unwrap()).await { return Err("Không truy cập được địa chỉ này (nội bộ hoặc không phân giải được).".into()); }
        let r = client.get(cur.clone()).header("Accept-Language", "vi,en;q=0.8").send().await.map_err(|e| e.to_string())?;
        if r.status().is_redirection() {
            let loc = r.headers().get("location").and_then(|l| l.to_str().ok()).ok_or("redirect không hợp lệ")?;
            cur = cur.join(loc).map_err(|e| e.to_string())?;
            continue;
        }
        let bytes = r.bytes().await.map_err(|e| e.to_string())?;
        let page = String::from_utf8_lossy(&bytes[..bytes.len().min(max_bytes)]).to_string();
        let title = Regex::new(r"(?is)<title[^>]*>(.*?)</title>").unwrap().captures(&page).map(|c| html_unescape(&c[1]).trim().to_string()).unwrap_or_else(|| cur.host_str().unwrap().to_string());
        let body = Regex::new(r"(?is)<(script|style|noscript|svg)[^>]*>.*?</(script|style|noscript|svg)>").unwrap().replace_all(&page, " ").to_string();
        let body = Regex::new(r"(?is)<br\s*/?>|</p>|</li>|</h\d>|</div>").unwrap().replace_all(&body, "\n").to_string();
        let text = html_unescape(&Regex::new(r"<[^>]+>").unwrap().replace_all(&body, " "));
        let text = Regex::new(r"[ \t\r\x0c\x0b]+").unwrap().replace_all(&text, " ").to_string();
        let text = Regex::new(r"\n\s*\n+").unwrap().replace_all(&text, "\n\n").trim().to_string();
        return Ok(json!({"url": url, "title": title, "text": text.chars().take(60000).collect::<String>(), "chars": text.chars().count()}));
    }
    Err("Quá nhiều lần chuyển hướng.".into())
}

fn html_unescape(s: &str) -> String { s.replace("&nbsp;", " ").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'").replace("&amp;", "&") }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_whole_word_and_allowlist() {
        let pol = default_policy();
        let r = check_texts(&vec![("dialogue".into(), "Bé không được giết ai. Mình đi mua ngay nhé".into())], &pol);
        assert_eq!(r["blockCount"], 1);
        assert_eq!(r["warnCount"], 1);
        assert_eq!(r["passed"], false);
        // "tạo" must not trip "tao"-like substrings; word boundary: "súng" inside "súngg" is not a hit
        assert_eq!(check_texts(&vec![("t".into(), "súngg".into())], &pol)["violations"].as_array().unwrap().len(), 0);
        let mut p2 = pol.clone();
        p2["allowlist"] = json!(["đánh nhau với bệnh tật"]);
        assert_eq!(check_texts(&vec![("t".into(), "Cùng đánh nhau với bệnh tật".into())], &p2)["blockCount"], 0);
    }
    #[test]
    fn parse_text_and_pack_merge() {
        let r = parse_policy_text("a.txt", "# Nhóm A\nxấu, tệ\nNhóm B: x; y").unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0]["terms"].as_array().unwrap().len(), 2);
        let m = merge_rules(&default_policy(), &r, "upload-a");
        assert!(m["rules"].as_array().unwrap().iter().any(|x| x["id"] == "upload-a:nh_m_a"));
    }
}
