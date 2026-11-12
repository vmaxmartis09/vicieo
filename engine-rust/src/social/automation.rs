use super::contracts::create_draft_post;
use super::models::*;
use super::service::SocialService;
use super::store::SocialStore;
use crate::workspace;
use chrono::Duration;
use serde_json::{json, Value};

fn read(p: &std::path::Path) -> Option<Value> { serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok() }

pub fn run_rules(project: &str) -> Result<Vec<Value>, String> {
    let st = SocialStore::new(project).map_err(|e| e.to_string())?;
    Ok(st.list_automation_rules().into_iter().filter(|r| r["enabled"] == json!(true)).map(|r| execute(project, r)).collect())
}
pub fn test_run_rule(project: &str, id: &str) -> Result<Value, String> {
    let st = SocialStore::new(project).map_err(|e| e.to_string())?;
    let r = st.list_automation_rules().into_iter().find(|r| s(r, "id") == id).ok_or_else(|| format!("Rule {id} not found"))?;
    Ok(execute(project, r))
}

fn execute(project: &str, mut rule: Value) -> Value {
    let st = SocialStore::new(project).unwrap();
    let mut actions: Vec<String> = vec![];
    let res: Result<Vec<String>, String> = match s(&rule, "trigger") {
        "render_done" => render_done(project, &rule, &mut actions),
        "idea_scripted" => idea_scripted(project, &mut actions),
        "schedule_gap" => schedule_gap(project, &rule, &mut actions).map(|_| vec![]),
        _ => Ok(vec![]),
    };
    rule["last_run_at"] = json!(now_iso());
    match res {
        Ok(created) => {
            rule["run_count"] = json!(rule["run_count"].as_i64().unwrap_or(0) + 1);
            rule["last_log"] = json!(format!("Thành công: {}", if actions.is_empty() { "Không có mục mới phù hợp điều kiện".to_string() } else { actions.join(", ") }));
            st.save_automation_rule(rule.clone());
            json!({"rule_id":rule["id"],"rule_name":rule["name"],"status":"success","actions":actions,"created_post_ids":created})
        }
        Err(e) => {
            rule["last_log"] = json!(format!("Lỗi: {e}"));
            st.save_automation_rule(rule.clone());
            json!({"rule_id":rule["id"],"rule_name":rule["name"],"status":"error","error":e})
        }
    }
}

fn render_done(project: &str, rule: &Value, actions: &mut Vec<String>) -> Result<Vec<String>, String> {
    let st = SocialStore::new(project).map_err(|e| e.to_string())?;
    let svc = SocialService::new(project).map_err(|e| e.1)?;
    let qd = workspace::project_path(project, &["runs", "render_queue"], false).map_err(|e| e.to_string())?;
    let mut created = vec![];
    if !qd.exists() { return Ok(created); }
    let prefix = rule["conditions"]["template_id_prefix"].as_str().unwrap_or("");
    let exact = rule["conditions"]["template_id"].as_str().filter(|x| !x.is_empty());
    let auto = rule["actions"]["auto_approve"] == json!(true);
    let existing: std::collections::HashSet<String> = st.list_posts(None, None, None, None, None, None).iter().filter_map(|p| p["render_job_id"].as_str().map(String::from)).collect();
    let mut files: Vec<_> = std::fs::read_dir(&qd).map_err(|e| e.to_string())?.flatten().map(|e| e.path()).filter(|p| { let n = p.file_name().unwrap().to_string_lossy().to_string(); n.starts_with("render_") && n.ends_with(".req.json") }).collect();
    files.sort();
    for rf in files {
        let job_id = rf.file_name().unwrap().to_string_lossy().trim_end_matches(".req.json").to_string();
        if existing.contains(&job_id) { continue; }
        let Some(req) = read(&rf) else { continue };
        let tpl = req["template_id"].as_str().unwrap_or("");
        if exact.map_or(false, |e| tpl != e) || (!prefix.is_empty() && !tpl.starts_with(prefix)) { continue; }
        let Some(job) = read(&qd.join(format!("{job_id}.json"))) else { continue };
        if job["status"] != "done" { continue; }
        let Some(outputs) = job["outputs"].as_object().filter(|o| !o.is_empty()) else { continue };
        let media = outputs.values().next().and_then(|v| v.as_str()).unwrap_or("");
        let platforms: Vec<String> = rule["actions"]["platforms"].as_array().filter(|a| !a.is_empty()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_else(|| outputs.keys().cloned().collect());
        let title = req["output_name"].as_str().filter(|x| !x.is_empty()).map(String::from).unwrap_or_else(|| format!("Video {tpl}"));
        let pid = create_draft_post(project, media, &title, &format!("Video mới tự động render từ mẫu #{tpl}!"), &platforms, None, Some(&job_id))?;
        created.push(pid.clone());
        actions.push(format!("Tạo bài nháp {pid} từ render {job_id}"));
        if let Some(mut post) = st.get_post(&pid) {
            for t in post["targets"].as_array_mut().into_iter().flatten() { t["scheduled_at"] = json!(svc.suggest_time(s(t, "account_id"), None)); }
            let post = st.save_post(post)?;
            if auto {
                svc.submit_for_approval(&pid, "automation", "").map_err(|e| e.1)?;
                svc.approve(&pid, "automation_rule", "Tự động duyệt theo quy tắc").map_err(|e| e.1)?;
                let times: serde_json::Map<String, Value> = post["targets"].as_array().unwrap().iter().map(|t| (s(t, "id").to_string(), t["scheduled_at"].as_str().map(|x| json!(x)).unwrap_or(json!(now_iso())))).collect();
                svc.schedule(&pid, &times, "automation_rule").map_err(|e| e.1)?;
                actions.push(format!("Tự động duyệt và lên lịch {pid}"));
            } else {
                svc.submit_for_approval(&pid, "automation", "Chờ người dùng duyệt từ Render").map_err(|e| e.1)?;
                actions.push(format!("Gửi bài {pid} vào hàng chờ duyệt"));
            }
        }
        let _ = st.add_notification(json!({"type":if auto { "publish_success" } else { "manual_package_due" },"title":"Đã tạo bài viết từ video render",
            "message":format!("Bản render '{title}' ({job_id}) đã được tạo bài viết và đặt lịch."),"action_url":format!("/social?tab={}", if auto { "calendar" } else { "approvals" }),"data":{"post_id":pid,"job_id":job_id}}));
    }
    Ok(created)
}

fn idea_scripted(project: &str, actions: &mut Vec<String>) -> Result<Vec<String>, String> {
    let st = SocialStore::new(project).map_err(|e| e.to_string())?;
    let dir = workspace::project_path(project, &["content", "ideas"], false).map_err(|e| e.to_string())?;
    let mut created = vec![];
    if !dir.exists() { return Ok(created); }
    let existing: std::collections::HashSet<String> = st.list_posts(None, None, None, None, None, None).iter().filter_map(|p| p["idea_id"].as_str().map(String::from)).collect();
    let mut files: Vec<_> = std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten().map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "json")).collect();
    files.sort();
    for f in files {
        let Some(d) = read(&f) else { continue };
        let Some(idea) = d["id"].as_str().filter(|x| !x.is_empty()) else { continue };
        if existing.contains(idea) || d["status"] != "scripted" { continue; }
        let title = d["title"].as_str().map(String::from).unwrap_or_else(|| format!("Ý tưởng {idea}"));
        let plats: Vec<String> = d["platform_targets"].as_array().filter(|a| !a.is_empty()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_else(|| vec!["tiktok".into(), "youtube".into()]);
        let pid = create_draft_post(project, "", &title, &format!("{title}\n\n#socnhotv #viral"), &plats, None, None)?;
        if let Some(mut p) = st.get_post(&pid) { p["idea_id"] = json!(idea); st.save_post(p)?; }
        actions.push(format!("Tạo bài nháp {pid} từ kịch bản {idea}"));
        created.push(pid);
    }
    Ok(created)
}

fn schedule_gap(project: &str, rule: &Value, actions: &mut Vec<String>) -> Result<(), String> {
    let st = SocialStore::new(project).map_err(|e| e.to_string())?;
    let min = rule["conditions"]["min_weekly_posts"].as_i64().unwrap_or(3);
    let now = chrono::Utc::now().with_timezone(&tz());
    let nw = now + Duration::days(7);
    let n = st.list_posts(None, None, None, None, None, None).iter().flat_map(|p| p["targets"].as_array().cloned().unwrap_or_default())
        .filter(|t| matches!(s(t, "status"), "scheduled" | "approved") && t["scheduled_at"].as_str().and_then(parse_dt).map_or(false, |d| now <= d && d <= nw)).count() as i64;
    if n < min {
        let _ = st.add_notification(json!({"type":"schedule_gap","title":"Cảnh báo: Lịch xuất bản tuần tới còn trống",
            "message":format!("Tuần tới chỉ có {n} bài được lên lịch (mục tiêu tối thiểu: {min} bài). Hãy chọn thêm kịch bản để lấp lịch trống!"),"action_url":"/social?tab=calendar","data":{"upcoming_count":n,"target_min":min}}));
        actions.push(format!("Phát hiện thiếu bài ({n}/{min}), đã gửi cảnh báo lấp lịch trống"));
    }
    Ok(())
}
