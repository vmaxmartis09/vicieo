//! Social Hub routes (port of core/api/routes/social.py). Logic lives in `crate::social`.
use super::prelude::*;
use crate::social::{models::*, service::*, store::SocialStore, ApiErr, R};
use axum::body::Bytes;
use chrono::{Duration as CDur, Datelike};

type Q = Query<HashMap<String, String>>;
const DEFAULT_PROJECT: &str = "soc-nho-tv";

fn store(p: &str) -> Result<SocialStore, ApiErr> {
    if !is_valid_slug(p) { return Err(ApiErr::bad("invalid project id")); }
    SocialStore::new(p).map_err(|e| ApiErr::bad(e.to_string()))
}
fn svc(p: &str) -> Result<SocialService, ApiErr> {
    if !is_valid_slug(p) { return Err(ApiErr::bad("invalid project id")); }
    SocialService::new(p)
}
/// Lenient body: empty/invalid => `{}` (Python `Body(default={})`).
fn jb(b: &Bytes) -> Value { serde_json::from_slice(b).unwrap_or(json!({})) }
fn jb_strict(b: &Bytes) -> Result<Value, ApiErr> { serde_json::from_slice(b).map_err(|e| ApiErr::invalid(format!("invalid JSON body: {e}"))) }
fn qp<'a>(q: &'a HashMap<String, String>, k: &str) -> Option<&'a str> { q.get(k).map(|s| s.as_str()).filter(|s| !s.is_empty()) }
fn pid(q: &HashMap<String, String>) -> String { qp(q, "project_id").unwrap_or(DEFAULT_PROJECT).to_string() }
fn actor(b: &Value) -> String { b["actor"].as_str().unwrap_or("user").to_string() }
fn sstr(b: &Value, k: &str) -> String { b[k].as_str().unwrap_or("").to_string() }

fn dt_of(s: &str) -> Option<chrono::DateTime<chrono::FixedOffset>> { parse_dt(&s.replace('Z', "+00:00")) }

// ---------------- accounts ----------------
async fn list_accounts(AxumPath(p): AxumPath<String>) -> R { Ok(Json(json!(store(&p)?.list_accounts()))) }
async fn list_accounts_alias(Query(q): Q) -> R { list_accounts(AxumPath(pid(&q))).await }
async fn create_account(AxumPath(p): AxumPath<String>, b: Bytes) -> R {
    let st = store(&p)?;
    let acc = norm_account(jb_strict(&b)?).map_err(ApiErr::invalid)?;
    st.save_account(acc, true).map(|a| Json(a)).map_err(ApiErr::conflict)
}
async fn create_account_alias(Query(q): Q, b: Bytes) -> R { create_account(AxumPath(pid(&q)), b).await }
async fn get_account(AxumPath((p, id)): AxumPath<(String, String)>) -> R {
    store(&p)?.get_account(&id).map(Json).ok_or_else(|| ApiErr::not_found("Account not found"))
}
async fn update_account(AxumPath((p, id)): AxumPath<(String, String)>, b: Bytes) -> R {
    let mut acc = jb_strict(&b)?;
    obj(&mut acc).insert("id".into(), json!(id));
    let acc = norm_account(acc).map_err(ApiErr::invalid)?;
    store(&p)?.save_account(acc, false).map(Json).map_err(ApiErr::conflict)
}
async fn delete_account(AxumPath((p, id)): AxumPath<(String, String)>) -> R {
    if store(&p)?.delete_account(&id) { Ok(Json(json!({"status":"deleted","account_id":id}))) } else { Err(ApiErr::not_found("Account not found")) }
}
async fn accounts_stats(AxumPath(p): AxumPath<String>) -> R {
    let st = store(&p)?;
    let now = chrono::Utc::now().with_timezone(&tz());
    let sow = (now - CDur::days(now.weekday().num_days_from_monday() as i64)).date_naive().and_hms_opt(0, 0, 0).unwrap().and_local_timezone(tz()).single().unwrap();
    let today = now.format("%Y-%m-%d").to_string();
    let mut stats: Vec<Value> = st.list_accounts().iter().map(|a| json!({"account_id":a["id"],"posts_this_week":0,"posts_today":0,
        "daily_limit":a["daily_limit"],"quota_remaining_today":a["daily_limit"],"next_post_time":null,"last_posted_at":null})).collect();
    for post in st.list_posts(None, None, None, None, None, None) {
        for t in post["targets"].as_array().into_iter().flatten() {
            let Some(a) = stats.iter_mut().find(|x| x["account_id"] == t["account_id"]) else { continue };
            if let Some(pa) = t["published_at"].as_str().filter(|x| !x.is_empty()) {
                if a["last_posted_at"].as_str().map_or(true, |l| pa > l) { a["last_posted_at"] = json!(pa); }
            }
            if let Some(sa) = t["scheduled_at"].as_str().filter(|x| !x.is_empty()) {
                if let Some(d) = dt_of(sa) {
                    if d >= sow { a["posts_this_week"] = json!(a["posts_this_week"].as_i64().unwrap() + 1); }
                    if sa.get(..10) == Some(today.as_str()) { a["posts_today"] = json!(a["posts_today"].as_i64().unwrap() + 1); }
                    if d >= now && a["next_post_time"].as_str().map_or(true, |n| sa < n) { a["next_post_time"] = json!(sa); }
                }
            }
        }
    }
    for a in stats.iter_mut() {
        a["quota_remaining_today"] = json!((a["daily_limit"].as_i64().unwrap_or(5) - a["posts_today"].as_i64().unwrap()).max(0));
    }
    Ok(Json(json!(stats)))
}
async fn accounts_stats_alias(Query(q): Q) -> R { accounts_stats(AxumPath(pid(&q))).await }

// ---------------- templates ----------------
async fn list_templates(AxumPath(p): AxumPath<String>) -> R { Ok(Json(json!(store(&p)?.list_templates()))) }
async fn list_templates_alias(Query(q): Q) -> R { list_templates(AxumPath(pid(&q))).await }
async fn create_template(AxumPath(p): AxumPath<String>, b: Bytes) -> R { store(&p)?.save_template(jb_strict(&b)?).map(Json).map_err(ApiErr::invalid) }
async fn create_template_alias(Query(q): Q, b: Bytes) -> R { create_template(AxumPath(pid(&q)), b).await }
async fn update_template(AxumPath((p, id)): AxumPath<(String, String)>, b: Bytes) -> R {
    let mut t = jb_strict(&b)?;
    let m = obj(&mut t);
    m.insert("id".into(), json!(id));
    m.insert("updated_at".into(), json!(now_iso()));
    store(&p)?.save_template(t).map(Json).map_err(ApiErr::invalid)
}
async fn update_template_alias(AxumPath(id): AxumPath<String>, Query(q): Q, b: Bytes) -> R { update_template(AxumPath((pid(&q), id)), b).await }
async fn delete_template(AxumPath((p, id)): AxumPath<(String, String)>) -> R {
    if store(&p)?.delete_template(&id) { Ok(Json(json!({"status":"deleted","template_id":id}))) } else { Err(ApiErr::not_found("Template not found")) }
}
async fn delete_template_alias(AxumPath(id): AxumPath<String>, Query(q): Q) -> R { delete_template(AxumPath((pid(&q), id))).await }

// ---------------- posts ----------------
async fn list_posts(AxumPath(p): AxumPath<String>, Query(q): Q) -> R {
    Ok(Json(json!(store(&p)?.list_posts(qp(&q, "status"), qp(&q, "platform"), qp(&q, "account"), qp(&q, "from"), qp(&q, "to"), qp(&q, "q")))))
}
async fn create_post(AxumPath(p): AxumPath<String>, b: Bytes) -> R {
    let st = store(&p)?;
    let mut post = jb_strict(&b)?;
    obj(&mut post).insert("project_id".into(), json!(p));
    let saved = st.save_post(post).map_err(ApiErr::invalid)?;
    st.append_audit("api", "create_post", s(&saved, "id"), None, None, None, "Created via API");
    Ok(Json(saved))
}
async fn create_post_alias(Query(q): Q, b: Bytes) -> R { create_post(AxumPath(pid(&q)), b).await }
async fn list_posts_alias(Query(q): Q) -> R { list_posts(AxumPath(pid(&q)), Query(q.clone())).await }
async fn create_post_draft(AxumPath(p): AxumPath<String>, b: Bytes) -> R {
    let body = jb(&b);
    let st = store(&p)?;
    let platforms: Vec<String> = body["platforms"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_else(|| vec!["youtube".into()]);
    let id = crate::social::contracts::create_draft_post(&p, body["media_path"].as_str().unwrap_or(""), body["title"].as_str().unwrap_or("Untitled Draft"),
        body["caption"].as_str().unwrap_or(""), &platforms, body["source_ref"].as_str(), body["render_job_id"].as_str()).map_err(ApiErr::bad)?;
    Ok(Json(st.get_post(&id).unwrap_or(json!({"id":id,"status":"draft"}))))
}
async fn get_post(AxumPath((p, id)): AxumPath<(String, String)>) -> R { store(&p)?.get_post(&id).map(Json).ok_or_else(|| ApiErr::not_found("Post not found")) }
async fn patch_post(AxumPath((p, id)): AxumPath<(String, String)>, b: Bytes) -> R {
    let st = store(&p)?;
    let mut post = st.get_post(&id).ok_or_else(|| ApiErr::not_found("Post not found"))?;
    for (k, v) in jb_strict(&b)?.as_object().cloned().unwrap_or_default() {
        if post.get(&k).is_some() && !matches!(k.as_str(), "id" | "project_id" | "created_at") { post[&k] = v; }
    }
    let saved = st.save_post(post).map_err(ApiErr::invalid)?;
    st.append_audit("api", "patch_post", &id, None, None, None, "Fields updated via PATCH");
    Ok(Json(saved))
}
async fn delete_post(AxumPath((p, id)): AxumPath<(String, String)>) -> R {
    let st = store(&p)?;
    if !st.delete_post(&id) { return Err(ApiErr::not_found("Post not found")); }
    st.append_audit("api", "delete_post", &id, None, None, None, "");
    Ok(Json(json!({"status":"deleted","post_id":id})))
}
async fn post_submit(AxumPath((p, id)): AxumPath<(String, String)>, b: Bytes) -> R { let b = jb(&b); svc(&p)?.submit_for_approval(&id, &actor(&b), &sstr(&b, "note")).map(Json) }
async fn post_approve(AxumPath((p, id)): AxumPath<(String, String)>, b: Bytes) -> R { let b = jb(&b); svc(&p)?.approve(&id, &actor(&b), &sstr(&b, "note")).map(Json) }
async fn post_reject(AxumPath((p, id)): AxumPath<(String, String)>, b: Bytes) -> R { let b = jb(&b); svc(&p)?.reject(&id, &actor(&b), &sstr(&b, "reason")).map(Json) }
async fn post_schedule(AxumPath((p, id)): AxumPath<(String, String)>, b: Bytes) -> R {
    let b = jb_strict(&b)?;
    let times = b["targets"].as_object().cloned().unwrap_or_default();
    svc(&p)?.schedule(&id, &times, &actor(&b)).map(Json)
}
async fn post_cancel(AxumPath((p, id)): AxumPath<(String, String)>, b: Bytes) -> R { svc(&p)?.cancel(&id, &actor(&jb(&b))).map(Json) }
async fn post_duplicate(AxumPath((p, id)): AxumPath<(String, String)>, b: Bytes) -> R { svc(&p)?.duplicate(&id, &actor(&jb(&b))).map(Json) }
async fn target_time(AxumPath((p, id, tid)): AxumPath<(String, String, String)>, b: Bytes) -> R {
    let b = jb_strict(&b)?;
    let t = b["time"].as_str().or(b["scheduled_at"].as_str()).filter(|x| !x.is_empty()).ok_or_else(|| ApiErr::bad("Missing 'time' or 'scheduled_at' in payload"))?;
    svc(&p)?.update_target_time(&id, &tid, t, &actor(&b)).map(Json)
}

// ---------------- overview / bulk ----------------
async fn overview(AxumPath(p): AxumPath<String>) -> R {
    let st = store(&p)?;
    let accounts = st.list_accounts();
    let now = chrono::Utc::now().with_timezone(&tz());
    let (p24, p7) = (now + CDur::hours(24), now + CDur::days(7));
    let (mut up, mut over, mut failed, mut pend) = (vec![], vec![], vec![], vec![]);
    let mut dates: HashMap<String, std::collections::HashSet<String>> = accounts.iter().map(|a| (s(a, "id").to_string(), Default::default())).collect();
    for post in st.list_posts(None, None, None, None, None, None) {
        let thumb = post["media"][0]["thumb"].as_str().unwrap_or("").to_string();
        for t in post["targets"].as_array().into_iter().flatten() {
            let acc = accounts.iter().find(|a| s(a, "id") == s(t, "account_id"));
            let info = json!({"post_id":post["id"],"target_id":t["id"],"title":post["title"],"platform":t["platform"],"account_id":t["account_id"],
                "account_name":acc.map(|a| s(a,"display_name")).unwrap_or(s(t,"account_id")),"account_avatar":acc.map(|a| s(a,"avatar")).unwrap_or(""),
                "status":t["status"],"scheduled_at":t["scheduled_at"],"thumb":thumb,"caption":t["caption"]});
            let st_t = s(t, "status");
            if st_t == "failed" { failed.push(info); }
            else if st_t == "published" {}
            else if post["approval"]["required"] != json!(false) && (s(&post, "status") == "pending_approval" || st_t == "pending_approval") { pend.push(info); }
            else if let Some(d) = t["scheduled_at"].as_str().and_then(dt_of) {
                dates.entry(s(t, "account_id").to_string()).or_default().insert(d.format("%Y-%m-%d").to_string());
                if d < now { over.push(info); } else if d <= p24 { up.push(info); }
            }
        }
    }
    let mut expiring = vec![];
    for a in &accounts {
        if let Some(e) = a["auth"]["expires_at"].as_str().filter(|x| !x.is_empty()) {
            if let Some(d) = dt_of(e) { if now <= d && d <= p7 {
                expiring.push(json!({"id":a["id"],"display_name":a["display_name"],"platform":a["platform"],"expires_at":e,"days_left":(d-now).num_days().max(0)}));
            } }
        }
    }
    let mut empty = vec![];
    for off in 0..7 {
        let day = now + CDur::days(off);
        let (wd, ds) = (day.weekday().num_days_from_monday() as i64, day.format("%Y-%m-%d").to_string());
        for a in &accounts {
            if dates.get(s(a, "id")).map_or(true, |d| !d.contains(&ds)) {
                for w in a["posting_windows"].as_array().into_iter().flatten().filter(|w| w["weekday"].as_i64() == Some(wd)) {
                    empty.push(json!({"date":ds,"weekday":wd,"account_id":a["id"],"account_name":a["display_name"],"platform":a["platform"],"start":w["start"],"end":w["end"]}));
                }
            }
        }
    }
    let n = empty.len();
    empty.truncate(15);
    Ok(Json(json!({"summary":{"upcoming_24h_count":up.len(),"overdue_count":over.len(),"failed_count":failed.len(),"pending_approval_count":pend.len(),
        "token_expiring_count":expiring.len(),"empty_slots_7d_count":n},"upcoming_24h":up,"overdue":over,"failed":failed,"pending_approval":pend,
        "token_expiring_accounts":expiring,"empty_slots_7d":empty})))
}
async fn overview_alias(Query(q): Q) -> R { overview(AxumPath(pid(&q))).await }

async fn bulk(AxumPath(p): AxumPath<String>, b: Bytes) -> R {
    let st = store(&p)?;
    let body = jb_strict(&b)?;
    let action = body["action"].as_str().ok_or_else(|| ApiErr::invalid("field required: action"))?.to_string();
    let params = &body["params"];
    let now = chrono::Utc::now().with_timezone(&tz());
    let mut results = vec![];
    for tid in body["ids"].as_array().ok_or_else(|| ApiErr::invalid("field required: ids"))?.iter().filter_map(|x| x.as_str()) {
        // re-read each time: earlier actions may have rewritten the post
        let posts = st.list_posts(None, None, None, None, None, None);
        let Some(mut post) = posts.into_iter().find(|p| p["targets"].as_array().map_or(false, |t| t.iter().any(|t| s(t, "id") == tid))) else {
            results.push(json!({"id":tid,"success":false,"error":"Target not found"})); continue };
        let idx = post["targets"].as_array().unwrap().iter().position(|t| s(t, "id") == tid).unwrap();
        let pidv = s(&post, "id").to_string();
        let res: Result<(), String> = (|| {
            let t = &mut post["targets"][idx];
            let note;
            let audit = match action.as_str() {
                "approve" => {
                    t["status"] = json!(if t["scheduled_at"].as_str().map_or(false, |x| !x.is_empty()) { "scheduled" } else { "approved" });
                    let ts = post["targets"].as_array().unwrap();
                    if ts.iter().all(|o| matches!(s(o, "status"), "scheduled" | "approved" | "published")) {
                        post["status"] = json!(if ts.iter().any(|o| o["scheduled_at"].as_str().map_or(false, |x| !x.is_empty())) { "scheduled" } else { "approved" });
                    }
                    note = "Bulk approved".to_string(); "bulk_approve"
                }
                "reschedule" => {
                    if let Some(nt) = params["new_time"].as_str().filter(|x| !x.is_empty()) { t["scheduled_at"] = json!(nt); }
                    else if !params["shift_hours"].is_null() || !params["shift_days"].is_null() {
                        let mut base = t["scheduled_at"].as_str().and_then(dt_of).unwrap_or(now);
                        if base < now { base = now; }
                        let g = |k: &str| params[k].as_i64().or_else(|| params[k].as_str().and_then(|x| x.parse().ok())).unwrap_or(0);
                        t["scheduled_at"] = json!((base + CDur::hours(g("shift_hours")) + CDur::days(g("shift_days"))).to_rfc3339());
                    }
                    t["status"] = json!("scheduled");
                    note = format!("Rescheduled to {}", t["scheduled_at"].as_str().unwrap_or("None")); "bulk_reschedule"
                }
                "duplicate_to" => {
                    let aid = params["account_id"].as_str().filter(|x| !x.is_empty()).ok_or("Missing target account_id")?;
                    let acc = st.get_account(aid).ok_or_else(|| format!("Account {aid} not found"))?;
                    let mut nt = t.clone();
                    nt["id"] = json!(new_id("tgt", 10)); nt["account_id"] = json!(aid); nt["platform"] = acc["platform"].clone();
                    nt["status"] = json!("draft");
                    for k in ["scheduled_at", "external_id", "external_url"] { nt[k] = Value::Null; }
                    post["targets"].as_array_mut().unwrap().push(nt);
                    note = format!("Duplicated to {aid}"); "bulk_duplicate"
                }
                "cancel" => { t["status"] = json!("cancelled"); note = "Bulk cancelled".into(); "bulk_cancel" }
                "delete_draft" => {
                    post["targets"].as_array_mut().unwrap().remove(idx);
                    note = "Draft deleted".into(); "bulk_delete_draft"
                }
                "retry" => {
                    t["status"] = json!("scheduled"); t["last_error"] = Value::Null; t["attempts"] = json!(0);
                    t["scheduled_at"] = json!((now + CDur::minutes(5)).to_rfc3339());
                    note = "Bulk retried".into(); "bulk_retry"
                }
                other => return Err(format!("Unsupported action '{other}'")),
            };
            if action == "delete_draft" && post["targets"].as_array().unwrap().is_empty() { st.delete_post(&pidv); }
            else { st.save_post(post.clone())?; }
            st.append_audit("user", audit, &pidv, Some(tid), None, None, &note);
            Ok(())
        })();
        results.push(match res { Ok(()) => json!({"id":tid,"success":true,"error":null}), Err(e) => json!({"id":tid,"success":false,"error":e}) });
    }
    Ok(Json(json!({"results": results})))
}
async fn bulk_alias(Query(q): Q, b: Bytes) -> R { bulk(AxumPath(pid(&q)), b).await }

// ---------------- calendar / slots / notes / helpers ----------------
async fn calendar(AxumPath(p): AxumPath<String>, Query(q): Q) -> R {
    let st = store(&p)?;
    let (from, to, view) = (qp(&q, "from"), qp(&q, "to"), qp(&q, "view").unwrap_or("month"));
    let accounts = st.list_accounts();
    let metrics = st.list_all_latest_metrics();
    let mut items = vec![];
    for post in st.list_posts(None, None, None, from, to, None) {
        let thumb = post["media"][0]["thumb"].as_str().unwrap_or("").to_string();
        for t in post["targets"].as_array().into_iter().flatten() {
            let acc = accounts.iter().find(|a| s(a, "id") == s(t, "account_id"));
            items.push(json!({"post_id":post["id"],"target_id":t["id"],"title":post["title"],"platform":t["platform"],"account_id":t["account_id"],
                "account_name":acc.map(|a| s(a,"display_name")).unwrap_or(s(t,"account_id")),"scheduled_at":t["scheduled_at"],"status":t["status"],
                "caption":t["caption"],"thumb":thumb,"views":metrics.get(s(t,"id")).map(|m| m["views"].clone()).unwrap_or(json!(0))}));
        }
    }
    let mut empty = vec![];
    if view == "week" && from.is_some() && to.is_some() {
        for a in &accounts { for w in a["posting_windows"].as_array().into_iter().flatten() {
            empty.push(json!({"account_id":a["id"],"platform":a["platform"],"weekday":w["weekday"],"start":w["start"],"end":w["end"],
                "label":format!("Khung giờ vàng ({}-{})", s(w,"start"), s(w,"end"))}));
        } }
    }
    Ok(Json(json!({"view":view,"from":from,"to":to,"items":items,"empty_slots":empty,"notes":st.list_notes(from,to),"slots":st.list_slots(None)})))
}
async fn list_slots(AxumPath(p): AxumPath<String>, Query(q): Q) -> R { Ok(Json(json!(store(&p)?.list_slots(qp(&q, "account_id"))))) }
async fn put_slots(AxumPath(p): AxumPath<String>, b: Bytes) -> R {
    let items: Result<Vec<Value>, String> = jb_strict(&b)?.as_array().cloned().ok_or_else(|| ApiErr::invalid("expected array"))?.into_iter().map(norm_slot).collect();
    let items = items.map_err(ApiErr::invalid)?;
    store(&p)?.save_slots(&items);
    Ok(Json(json!(items)))
}
async fn list_notes(AxumPath(p): AxumPath<String>, Query(q): Q) -> R { Ok(Json(json!(store(&p)?.list_notes(qp(&q, "from"), qp(&q, "to"))))) }
async fn put_notes(AxumPath(p): AxumPath<String>, b: Bytes) -> R {
    let items: Result<Vec<Value>, String> = jb_strict(&b)?.as_array().cloned().ok_or_else(|| ApiErr::invalid("expected array"))?.into_iter().map(norm_note).collect();
    let items = items.map_err(ApiErr::invalid)?;
    store(&p)?.save_notes(&items);
    Ok(Json(json!(items)))
}
async fn schedule_idea(AxumPath((p, idea)): AxumPath<(String, String)>, b: Bytes) -> R {
    let b = jb_strict(&b)?;
    let aid = b["account_id"].as_str().filter(|x| !x.is_empty()).ok_or_else(|| ApiErr::bad("Missing account_id in payload"))?;
    svc(&p)?.schedule_idea(&idea, aid, b["at"].as_str()).map(Json)
}
async fn suggest_time(AxumPath(p): AxumPath<String>, b: Bytes) -> R {
    let b = jb_strict(&b)?;
    let aid = b["account_id"].as_str().filter(|x| !x.is_empty()).ok_or_else(|| ApiErr::bad("Missing account_id in payload"))?;
    Ok(Json(json!({"account_id":aid,"suggested_time":svc(&p)?.suggest_time(aid, b["after"].as_str())})))
}
async fn caption_variants_route(AxumPath(_p): AxumPath<String>, b: Bytes) -> R {
    let b = jb_strict(&b)?;
    let strs = |k: &str, d: &[&str]| -> Vec<String> { b[k].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_else(|| d.iter().map(|x| x.to_string()).collect()) };
    Ok(Json(json!({"variants": caption_variants(&sstr(&b,"title"), &sstr(&b,"caption"), &strs("hashtags", &[]), &strs("platforms", &["tiktok","youtube"]), &sstr(&b,"product_link"), &sstr(&b,"footer"))})))
}
async fn audit(AxumPath(p): AxumPath<String>, Query(q): Q) -> R {
    Ok(Json(json!(store(&p)?.get_audit_log(qp(&q, "post_id"), qp(&q, "limit").and_then(|x| x.parse().ok()).unwrap_or(100)))))
}
async fn queue_route(AxumPath(p): AxumPath<String>, Query(q): Q) -> R {
    let st = store(&p)?;
    let n = |k: &str, d: usize| qp(&q, k).and_then(|x| x.parse::<usize>().ok()).unwrap_or(d);
    let page = n("page", 1);
    let ps = n("page_size", 100);
    if page < 1 || !(1..=500).contains(&ps) { return Err(ApiErr::invalid("page>=1, 1<=page_size<=500")); }
    let bucket = if q.contains_key("bucket") { qp(&q, "bucket") } else { Some("upcoming") };
    Ok(Json(crate::social::queue::get_social_queue(&st, &crate::social::queue::QueueParams { bucket, account_id: qp(&q, "account"), platform: qp(&q, "platform"),
        from: qp(&q, "from"), group_by: qp(&q, "group_by"), sort: qp(&q, "sort"), page, page_size: ps })))
}
async fn queue_alias(Query(q): Q) -> R { queue_route(AxumPath(pid(&q)), Query(q.clone())).await }
fn dry(q: &HashMap<String, String>) -> bool { match qp(q, "dry_run") { Some("false") | Some("0") => false, Some(_) => true, None => crate::social::vault::dry_run_default() } }

async fn oauth_connect(AxumPath((p, id)): AxumPath<(String, String)>, Query(q): Q) -> R {
    let acc = store(&p)?.get_account(&id).ok_or_else(|| ApiErr::not_found("Account not found"))?;
    let redirect = qp(&q, "redirect_uri").unwrap_or("http://localhost:3000/api/auth/callback");
    Ok(Json(json!({"account_id":id,"platform":acc["platform"],"connect_url":crate::social::adapters::connect_url(s(&acc,"platform"), &id, redirect)})))
}
async fn oauth_callback(AxumPath(platform): AxumPath<String>, Query(q): Q) -> R {
    let code = qp(&q, "code").ok_or_else(|| ApiErr::invalid("code is required"))?;
    let redirect = qp(&q, "redirect_uri").unwrap_or("http://localhost:3000/api/auth/callback");
    let t = crate::social::adapters::exchange_code(&platform, code, redirect).await.map_err(ApiErr::bad)?;
    let state = qp(&q, "state");
    if let Some(sid) = state {
        if !sid.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') { return Err(ApiErr::bad("invalid state")); }
        crate::social::vault::set_key(&format!("oauth_{platform}_{sid}_access"), t["access_token"].as_str().unwrap_or(""));
        if let Some(r) = t["refresh_token"].as_str() { crate::social::vault::set_key(&format!("oauth_{platform}_{sid}_refresh"), r); }
    }
    Ok(Json(json!({"status":"connected","platform":platform,"account_id":state})))
}
async fn refresh_token(AxumPath((p, id)): AxumPath<(String, String)>) -> R {
    let acc = store(&p)?.get_account(&id).ok_or_else(|| ApiErr::not_found("Account not found"))?;
    Ok(Json(json!({"status":"refreshed","details":crate::social::adapters::refresh(&acc).await})))
}
async fn validate_target(AxumPath((p, pid, tid)): AxumPath<(String, String, String)>) -> R {
    let st = store(&p)?;
    let post = st.get_post(&pid).ok_or_else(|| ApiErr::not_found("Post not found"))?;
    let t = post["targets"].as_array().and_then(|a| a.iter().find(|t| s(t, "id") == tid)).ok_or_else(|| ApiErr::not_found("Target not found"))?;
    let mode = st.get_account(s(t, "account_id")).and_then(|a| a["auth"]["mode"].as_str().map(String::from)).unwrap_or("manual".into());
    Ok(Json(json!({"target_id":tid,"issues":crate::social::adapters::validate(s(t,"platform"), &mode, t, post["media"].get(0))})))
}
async fn publish_now(AxumPath((p, pid)): AxumPath<(String, String)>, Query(q): Q) -> R {
    let st = store(&p)?;
    let mut post = st.get_post(&pid).ok_or_else(|| ApiErr::not_found("Post not found"))?;
    if post["approval"]["required"] != json!(false) && !matches!(s(&post, "status"), "approved" | "scheduled") {
        return Err(ApiErr::conflict("Bài viết cần được phê duyệt trước khi xuất bản (approval required)"));
    }
    let dry_run = dry(&q);
    let mut results = vec![];
    for i in 0..post["targets"].as_array().map_or(0, |a| a.len()) {
        let t = post["targets"][i].clone();
        if s(&t, "status") == "published" { continue; }
        let acc = st.get_account(s(&t, "account_id")).unwrap_or_else(|| json!({"id":t["account_id"],"platform":t["platform"],"handle":t["account_id"],"display_name":t["account_id"],"auth":{"mode":"manual"}}));
        let res = crate::social::adapters::publish(&crate::social::adapters::Ctx { project_id: &p, post_id: &pid, dry_run }, &acc, &t, post["media"].get(0)).await;
        let tg = &mut post["targets"][i];
        if res.status == "published" {
            tg["status"] = json!("published"); tg["external_id"] = json!(res.external_id); tg["external_url"] = json!(res.url); tg["published_at"] = json!(now_iso());
            st.append_audit("api", "publish_now_success", &pid, Some(s(&t, "id")), None, Some("published"), if dry_run { "dry_run" } else { "" });
        } else {
            tg["status"] = json!("failed"); tg["last_error"] = json!(res.error);
            st.append_audit("api", "publish_now_failed", &pid, Some(s(&t, "id")), None, None, res.error.as_deref().unwrap_or(""));
        }
        results.push(json!({"target_id":t["id"],"platform":t["platform"],"status":tg["status"],"url":res.url,"error":res.error}));
    }
    if post["targets"].as_array().unwrap().iter().all(|t| s(t, "status") == "published") { post["status"] = json!("published"); }
    let saved = st.save_post(post).map_err(ApiErr::invalid)?;
    Ok(Json(json!({"post_id":pid,"status":saved["status"],"dry_run":dry_run,"targets":results})))
}
async fn mark_published(AxumPath((p, pid, tid)): AxumPath<(String, String, String)>, b: Bytes) -> R {
    let st = store(&p)?;
    let mut post = st.get_post(&pid).ok_or_else(|| ApiErr::not_found("Post not found"))?;
    let b = jb_strict(&b)?;
    let url = b["url"].as_str().filter(|x| !x.is_empty()).or(b["external_url"].as_str()).unwrap_or("").to_string();
    let idx = post["targets"].as_array().and_then(|a| a.iter().position(|t| s(t, "id") == tid)).ok_or_else(|| ApiErr::not_found("Target not found"))?;
    let tg = &mut post["targets"][idx];
    tg["status"] = json!("published"); tg["external_url"] = json!(url); tg["published_at"] = json!(now_iso());
    if post["targets"].as_array().unwrap().iter().all(|t| s(t, "status") == "published") { post["status"] = json!("published"); }
    st.save_post(post).map_err(ApiErr::invalid)?;
    st.append_audit("user", "manual_mark_published", &pid, Some(&tid), None, Some("published"), &format!("Manual URL: {url}"));
    Ok(Json(json!({"post_id":pid,"target_id":tid,"status":"published","url":url})))
}
async fn scheduler_status(AxumPath(p): AxumPath<String>) -> R {
    let st = store(&p)?;
    Ok(Json(json!({"heartbeat":crate::social::scheduler::get_heartbeat(&p),"scheduled_count":st.list_posts(Some("scheduled"),None,None,None,None,None).len(),"recent_activity":st.get_audit_log(None,10)})))
}
/// Manual run of one scheduler cycle (extra; useful with ?dry_run=true).
async fn scheduler_tick(AxumPath(p): AxumPath<String>, Query(q): Q) -> R {
    store(&p)?;
    let d = dry(&q);
    Ok(Json(json!({"dry_run":d,"results":crate::social::scheduler::tick(&p, None, d).await})))
}
fn pkg_err(e: String) -> ApiErr { ApiErr::not_found(e) }
async fn package_info(AxumPath((p, pid, plat)): AxumPath<(String, String, String)>) -> R {
    store(&p)?;
    if let Some(i) = crate::social::adapters::get_manual_package_info(&p, &pid, &plat) { return Ok(Json(i)); }
    crate::social::adapters::create_manual_package(&p, &pid, &plat).map(Json).map_err(pkg_err)
}
async fn package_generate(AxumPath((p, pid, plat)): AxumPath<(String, String, String)>) -> R {
    store(&p)?;
    crate::social::adapters::create_manual_package(&p, &pid, &plat).map(Json).map_err(pkg_err)
}
/// Route is `/packages/{post_id}/{platform_zip}` where the last segment ends with `.zip` (axum can't mix param+suffix).
async fn package_dispatch(AxumPath((p, pid, last)): AxumPath<(String, String, String)>) -> axum::response::Response {
    use axum::response::IntoResponse;
    if let Some(plat) = last.strip_suffix(".zip") {
        let r = match crate::social::adapters::export_package_zip(&p, &pid, plat) {
            Ok(z) => Ok(z),
            Err(_) => crate::social::adapters::create_manual_package(&p, &pid, plat).and_then(|i| Ok(std::path::PathBuf::from(i["zip_path"].as_str().unwrap_or("")))),
        };
        return match r.and_then(|z| std::fs::read(&z).map(|b| (z, b)).map_err(|e| e.to_string())) {
            Ok((z, bytes)) => ([("content-type", "application/zip".to_string()), ("content-disposition", format!("attachment; filename=\"{}\"", z.file_name().unwrap().to_string_lossy()))], bytes).into_response(),
            Err(e) => pkg_err(e).into_response(),
        };
    }
    package_info(AxumPath((p, pid, last))).await.into_response()
}

// ---------------- automation rules ----------------
async fn list_rules(AxumPath(p): AxumPath<String>) -> R { Ok(Json(json!(store(&p)?.list_automation_rules()))) }
async fn save_rule(AxumPath(p): AxumPath<String>, b: Bytes) -> R {
    let r = norm_rule(jb_strict(&b)?).map_err(ApiErr::invalid)?;
    store(&p)?.save_automation_rule(r.clone());
    Ok(Json(r))
}
async fn toggle_rule(AxumPath((p, id)): AxumPath<(String, String)>) -> R {
    let st = store(&p)?;
    let mut r = st.list_automation_rules().into_iter().find(|r| s(r, "id") == id).ok_or_else(|| ApiErr::not_found("Rule not found"))?;
    r["enabled"] = json!(r["enabled"] != json!(true));
    st.save_automation_rule(r.clone());
    Ok(Json(json!({"id":id,"enabled":r["enabled"]})))
}
async fn delete_rule(AxumPath((p, id)): AxumPath<(String, String)>) -> R {
    if store(&p)?.delete_automation_rule(&id) { Ok(Json(json!({"status":"deleted","id":id}))) } else { Err(ApiErr::not_found("Rule not found")) }
}

// ---------------- library / plans / captions ----------------
async fn videos(AxumPath(p): AxumPath<String>) -> R { store(&p)?; Ok(Json(json!(crate::social::video_library::list_videos(&p)))) }
async fn videos_alias(Query(q): Q) -> R { videos(AxumPath(pid(&q))).await }
async fn library(AxumPath(p): AxumPath<String>, Query(q): Q) -> R {
    store(&p)?;
    Ok(Json(json!(crate::social::video_library::list_library(&p, qp(&q, "filter"), qp(&q, "account"), qp(&q, "platform"), qp(&q, "q")))))
}
async fn library_alias(Query(q): Q) -> R { library(AxumPath(pid(&q)), Query(q.clone())).await }
async fn plan_preview(AxumPath(p): AxumPath<String>, b: Bytes) -> R { store(&p)?; crate::social::planner::preview_plan(&p, &jb_strict(&b)?).map(Json) }
async fn plan_preview_alias(Query(q): Q, b: Bytes) -> R { plan_preview(AxumPath(pid(&q)), b).await }
async fn plan_commit(AxumPath(p): AxumPath<String>, b: Bytes) -> R { store(&p)?; crate::social::planner::commit_plan(&p, &jb_strict(&b)?).map(Json) }
async fn plan_commit_alias(Query(q): Q, b: Bytes) -> R { plan_commit(AxumPath(pid(&q)), b).await }
async fn captions_gen(AxumPath(p): AxumPath<String>, b: Bytes) -> R { store(&p)?; crate::social::captions::generate(&p, &jb_strict(&b)?).await.map(Json) }
async fn captions_alias(Query(q): Q, b: Bytes) -> R { captions_gen(AxumPath(pid(&q)), b).await }

// ---------------- analytics ----------------
fn eng(p: &str) -> Result<crate::social::insights::Engine, ApiErr> { store(p)?; crate::social::insights::Engine::new(p).map_err(ApiErr::bad) }
async fn an_overview(AxumPath(p): AxumPath<String>, Query(q): Q) -> R {
    let e = eng(&p)?;
    let daily = e.daily_insights(None);
    // Python computes kpis for the requested window/filters, then daily insights separately.
    let kpis = e.kpis(qp(&q, "window").unwrap_or("30d"), qp(&q, "platform"), qp(&q, "account_id"));
    Ok(Json(json!({"kpis":kpis,"views_by_day":daily["views_by_day"],"outliers":daily["outliers"]})))
}
async fn an_overview_alias(Query(q): Q) -> R { an_overview(AxumPath(pid(&q)), Query(q.clone())).await }
async fn an_heatmap(AxumPath(p): AxumPath<String>) -> R { Ok(Json(eng(&p)?.golden_hours())) }
async fn an_creatives(AxumPath(p): AxumPath<String>) -> R { Ok(Json(eng(&p)?.creatives())) }
async fn an_outliers(AxumPath(p): AxumPath<String>) -> R { Ok(Json(eng(&p)?.outliers())) }
async fn an_learnings(AxumPath(p): AxumPath<String>) -> R { let e = eng(&p)?; Ok(Json(json!({"learnings_md":e.learnings_md(),"recommendations":e.recommendations()}))) }
async fn an_generate(AxumPath(p): AxumPath<String>) -> R {
    let e = eng(&p)?;
    let d = e.daily_insights(None);
    Ok(Json(json!({"status":"ok","date":d["date"],"kpis":d["kpis"],"learnings_md":e.learnings_md(),"recommendations":e.recommendations()})))
}
async fn an_collect(AxumPath(p): AxumPath<String>) -> R { Ok(Json(json!({"status":"collected","count":crate::social::collectors::collect_all(&store(&p)?).await}))) }
async fn an_apply_windows(AxumPath(p): AxumPath<String>, b: Bytes) -> R {
    let st = store(&p)?;
    let b = jb_strict(&b)?;
    let aid = b["account_id"].as_str().filter(|x| !x.is_empty());
    let mut wins = b["windows"].clone();
    if !wins.as_array().map_or(false, |a| !a.is_empty()) { wins = eng(&p)?.golden_hours()["recommended_windows"].clone(); }
    for w in wins.as_array().cloned().unwrap_or_default() {
        if !(w["weekday"].as_i64().map_or(false, |d| (0..=6).contains(&d)) && w["start"].is_string() && w["end"].is_string()) { return Err(ApiErr::invalid("invalid posting window")); }
    }
    let mut updated = vec![];
    for mut a in st.list_accounts().into_iter().filter(|a| aid.map_or(true, |x| s(a, "id") == x)) {
        a["posting_windows"] = wins.clone();
        st.save_account(a.clone(), false).map_err(ApiErr::conflict)?;
        updated.push(s(&a, "id").to_string());
    }
    st.append_audit("user", "apply_golden_windows", "all", None, None, None, &format!("Updated accounts: {updated:?}"));
    Ok(Json(json!({"status":"applied","updated_accounts":updated,"windows":wins})))
}
async fn an_manual(AxumPath(p): AxumPath<String>, b: Bytes) -> R {
    let st = store(&p)?;
    let b = jb_strict(&b)?;
    if b["target_id"].as_str().map_or(true, |x| x.is_empty()) { return Err(ApiErr::bad("target_id is required")); }
    let n = |k: &str| b[k].as_i64().or_else(|| b[k].as_f64().map(|x| x as i64)).unwrap_or(0);
    let fl = |k: &str| b[k].as_f64().unwrap_or(0.0);
    let snap = norm_snapshot(json!({"post_id":b["post_id"].as_str().unwrap_or(""),"target_id":b["target_id"],"platform":b["platform"].as_str().unwrap_or("youtube"),"checkpoint":"manual",
        "views":n("views"),"likes":n("likes"),"comments":n("comments"),"shares":n("shares"),"saves":n("saves"),"watch_time_s":fl("watch_time_s"),
        "avg_retention_pct":fl("avg_retention_pct"),"subscribers_gained":n("subscribers_gained"),"ctr":fl("ctr"),"revenue":fl("revenue")})).map_err(ApiErr::invalid)?;
    st.append_metrics_snapshot(&snap);
    Ok(Json(json!({"status":"saved","snapshot":snap})))
}
async fn an_import_csv(AxumPath(p): AxumPath<String>, b: Bytes) -> R {
    let st = store(&p)?;
    let b = jb_strict(&b)?;
    let text = b["csv_text"].as_str().unwrap_or("");
    let plat = b["platform"].as_str().unwrap_or("youtube");
    if text.trim().is_empty() { return Err(ApiErr::bad("csv_text is required")); }
    let records = crate::social::collectors::parse_csv(text);
    let posts = st.list_posts(None, None, None, None, None, None);
    let mut matched = 0;
    for rec in &records {
        let title = s(rec, "title").trim().to_lowercase();
        if title.is_empty() { continue; }
        let Some(post) = posts.iter().find(|p| { let pt = s(p, "title").to_lowercase(); title.contains(&pt) || pt.contains(&title) }) else { continue };
        let ts = post["targets"].as_array().cloned().unwrap_or_default();
        let Some(t) = ts.iter().find(|t| s(t, "platform") == plat).or(ts.first()) else { continue };
        let mut snap = json!({"post_id":post["id"],"target_id":t["id"],"platform":t["platform"],"checkpoint":"manual","raw":rec});
        for k in ["views","likes","comments","shares","saves","watch_time_s","avg_retention_pct","subscribers_gained","ctr","revenue"] { snap[k] = rec[k].clone(); }
        st.append_metrics_snapshot(&norm_snapshot(snap).map_err(ApiErr::invalid)?);
        matched += 1;
    }
    Ok(Json(json!({"status":"imported","total_records":records.len(),"matched_count":matched,"records":records.iter().take(10).collect::<Vec<_>>()})))
}

async fn automation_run(AxumPath(p): AxumPath<String>, Query(q): Q) -> R {
    store(&p)?;
    match qp(&q, "rule_id") {
        Some(id) => crate::social::automation::test_run_rule(&p, id).map(|r| Json(json!({"reports":[r]}))).map_err(ApiErr::not_found),
        None => crate::social::automation::run_rules(&p).map(|r| Json(json!({"reports":r}))).map_err(ApiErr::bad),
    }
}
async fn migrate_route(AxumPath(p): AxumPath<String>, Query(q): Q) -> R {
    store(&p)?;
    crate::social::migrate::migrate_legacy_publishing(&p, qp(&q, "force") == Some("true")).map(Json).map_err(ApiErr::bad)
}

async fn ping() -> Json<Value> { Json(json!({"status":"ok","module":"social"})) }

// ---------------- notifications / automation rules (store-only parts) ----------------
async fn list_notifs(AxumPath(p): AxumPath<String>, Query(q): Q) -> R {
    Ok(Json(json!(store(&p)?.list_notifications(qp(&q, "unread_only") == Some("true"), qp(&q, "limit").and_then(|x| x.parse().ok()).unwrap_or(50)))))
}
async fn read_notif(AxumPath((p, id)): AxumPath<(String, String)>) -> R {
    if store(&p)?.mark_notification_read(&id) { Ok(Json(json!({"status":"ok"}))) } else { Err(ApiErr::not_found("Notification not found")) }
}
async fn clear_notifs(AxumPath(p): AxumPath<String>) -> R { store(&p)?.clear_notifications(); Ok(Json(json!({"status":"cleared"}))) }

pub fn router() -> Router<AppState> {
    crate::social::scheduler::spawn();
    let pp = "/projects/{project_id}/social";
    Router::new()
        // legacy project publishing JSON (kept for dashboard)
        .route("/projects/{project_id}/publishing", get(get_project_publishing))
        .route("/projects/{project_id}/publishing/schedule", post(save_project_scheduled_post))
        .route("/social/_ping", get(ping))
        .route(&format!("{pp}/automation/run"), post(automation_run))
        .route(&format!("{pp}/migrate"), post(migrate_route))
        .route(&format!("{pp}/analytics/overview"), get(an_overview)).route("/api/social/analytics", get(an_overview_alias))
        .route(&format!("{pp}/analytics/heatmap"), get(an_heatmap))
        .route(&format!("{pp}/analytics/apply-golden-windows"), post(an_apply_windows))
        .route(&format!("{pp}/analytics/creatives"), get(an_creatives))
        .route(&format!("{pp}/analytics/outliers"), get(an_outliers))
        .route(&format!("{pp}/analytics/learnings"), get(an_learnings))
        .route(&format!("{pp}/analytics/generate-insights"), post(an_generate))
        .route(&format!("{pp}/analytics/collect"), post(an_collect))
        .route(&format!("{pp}/analytics/manual-metrics"), post(an_manual))
        .route(&format!("{pp}/analytics/import-csv"), post(an_import_csv))
        .route(&format!("{pp}/videos"), get(videos)).route("/social/videos", get(videos_alias))
        .route(&format!("{pp}/library"), get(library)).route("/social/library", get(library_alias))
        .route(&format!("{pp}/plans/preview"), post(plan_preview)).route("/social/plans/preview", post(plan_preview_alias))
        .route(&format!("{pp}/plans/commit"), post(plan_commit)).route("/social/plans/commit", post(plan_commit_alias))
        .route(&format!("{pp}/captions/generate"), post(captions_gen)).route("/social/captions/generate", post(captions_alias))
        .route(&format!("{pp}/accounts/{{account_id}}/connect"), get(oauth_connect))
        .route("/social/oauth/callback/{platform}", get(oauth_callback))
        .route(&format!("{pp}/accounts/{{account_id}}/refresh"), post(refresh_token))
        .route(&format!("{pp}/posts/{{post_id}}/targets/{{target_id}}/validate"), post(validate_target))
        .route(&format!("{pp}/posts/{{post_id}}/publish-now"), post(publish_now))
        .route(&format!("{pp}/posts/{{post_id}}/targets/{{target_id}}/mark-published"), post(mark_published))
        .route(&format!("{pp}/scheduler/status"), get(scheduler_status))
        .route(&format!("{pp}/scheduler/tick"), post(scheduler_tick))
        .route(&format!("{pp}/packages/{{post_id}}/{{platform}}"), get(package_dispatch))
        .route(&format!("{pp}/packages/{{post_id}}/{{platform}}/generate"), post(package_generate))
        .route(&format!("{pp}/automation/rules"), get(list_rules).post(save_rule))
        .route(&format!("{pp}/automation/rules/{{rule_id}}/toggle"), post(toggle_rule))
        .route(&format!("{pp}/automation/rules/{{rule_id}}"), delete(delete_rule))
        .route(&format!("{pp}/queue"), get(queue_route))
        .route("/social/queue", get(queue_alias))
        .route("/api/social/schedule", get(queue_alias))
        .route(&format!("{pp}/overview"), get(overview)).route("/social/overview", get(overview_alias))
        .route(&format!("{pp}/targets/bulk"), post(bulk)).route("/social/targets/bulk", post(bulk_alias))
        .route(&format!("{pp}/accounts"), get(list_accounts).post(create_account))
        .route("/social/accounts", get(list_accounts_alias).post(create_account_alias))
        .route("/api/social/accounts", get(list_accounts_alias).post(create_account_alias))
        .route(&format!("{pp}/accounts/stats"), get(accounts_stats)).route("/social/accounts/stats", get(accounts_stats_alias))
        .route(&format!("{pp}/accounts/{{account_id}}"), get(get_account).put(update_account).delete(delete_account))
        .route(&format!("{pp}/templates"), get(list_templates).post(create_template))
        .route("/social/templates", get(list_templates_alias).post(create_template_alias))
        .route(&format!("{pp}/templates/{{template_id}}"), put(update_template).delete(delete_template))
        .route("/social/templates/{template_id}", put(update_template_alias).delete(delete_template_alias))
        .route(&format!("{pp}/posts"), get(list_posts).post(create_post))
        .route("/social/posts", get(list_posts_alias).post(create_post_alias))
        .route("/api/social/posts", get(list_posts_alias).post(create_post_alias))
        .route(&format!("{pp}/posts/draft"), post(create_post_draft))
        .route(&format!("{pp}/posts/{{post_id}}"), get(get_post).patch(patch_post).delete(delete_post))
        .route(&format!("{pp}/posts/{{post_id}}/submit"), post(post_submit))
        .route(&format!("{pp}/posts/{{post_id}}/approve"), post(post_approve))
        .route(&format!("{pp}/posts/{{post_id}}/reject"), post(post_reject))
        .route(&format!("{pp}/posts/{{post_id}}/schedule"), post(post_schedule))
        .route(&format!("{pp}/posts/{{post_id}}/cancel"), post(post_cancel))
        .route(&format!("{pp}/posts/{{post_id}}/duplicate"), post(post_duplicate))
        .route(&format!("{pp}/posts/{{post_id}}/targets/{{target_id}}/time"), put(target_time))
        .route(&format!("{pp}/calendar"), get(calendar))
        .route(&format!("{pp}/slots"), get(list_slots).put(put_slots))
        .route(&format!("{pp}/notes"), get(list_notes).put(put_notes))
        .route(&format!("{pp}/ideas/{{idea_id}}/schedule"), post(schedule_idea))
        .route(&format!("{pp}/suggest-time"), post(suggest_time))
        .route(&format!("{pp}/caption-variants"), post(caption_variants_route))
        .route(&format!("{pp}/audit"), get(audit))
        .route(&format!("{pp}/notifications"), get(list_notifs).delete(clear_notifs))
        .route(&format!("{pp}/notifications/{{notif_id}}/read"), post(read_notif))
}

// legacy handlers kept from the pre-port file
pub(crate) async fn get_project_publishing(AxumPath(project_id): AxumPath<String>) -> Json<Value> {
    let p = workspace::project_path(&project_id, &["publishing", "schedule.json"], false).unwrap_or_default();
    Json(fs::read_to_string(&p).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(json!({"channels":[],"posts":[],"schedule":[]})))
}
pub(crate) async fn save_project_scheduled_post(AxumPath(project_id): AxumPath<String>, Json(body): Json<Value>) -> Json<Value> {
    let p = workspace::project_path(&project_id, &["publishing", "schedule.json"], true).unwrap_or_default();
    let mut raw: Value = fs::read_to_string(&p).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(json!({"channels":[],"posts":[]}));
    if let Some(a) = raw.get_mut("posts").and_then(|x| x.as_array_mut()) { a.push(body); } else { raw["posts"] = json!([body]); }
    let _ = crate::social::store::atomic_write_json(&p, &raw);
    Json(json!({"ok": true}))
}
