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
    let accounts = st.list_accounts();
    let media = sstr(&body, "media_path");
    let targets: Vec<Value> = platforms.iter().map(|pl| {
        let acc = accounts.iter().find(|a| s(a, "platform") == pl);
        json!({"account_id": acc.map(|a| s(a, "id")).unwrap_or("manual"), "platform": pl, "caption": sstr(&body, "caption"), "title": body["title"].as_str().unwrap_or("Untitled Draft")})
    }).collect();
    let post = json!({"project_id": p, "title": body["title"].as_str().unwrap_or("Untitled Draft"),
        "media": if media.is_empty() { json!([]) } else { json!([{"path": media}]) },
        "source_ref": body["source_ref"], "render_job_id": body["render_job_id"], "targets": targets});
    let saved = st.save_post(post).map_err(ApiErr::invalid)?;
    st.append_audit("contracts", "create_draft", s(&saved, "id"), None, None, Some("draft"), "");
    Ok(Json(saved))
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
    let pp = "/projects/{project_id}/social";
    Router::new()
        // legacy project publishing JSON (kept for dashboard)
        .route("/projects/{project_id}/publishing", get(get_project_publishing))
        .route("/projects/{project_id}/publishing/schedule", post(save_project_scheduled_post))
        .route("/social/_ping", get(ping))
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
