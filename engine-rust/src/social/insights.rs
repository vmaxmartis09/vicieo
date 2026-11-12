//! Port of analytics/insights.py (AnalyticsEngine). All outputs are JSON shaped like the pydantic dumps.
use super::models::*;
use super::store::SocialStore;
use chrono::{Datelike, Duration, Timelike};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn median(v: &mut Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n == 0 { 0.0 } else if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 }
}
fn r(x: f64, d: i32) -> f64 { let k = 10f64.powi(d); (x * k).round() / k }
fn thousands(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() { if i > 0 && (s.len() - i) % 3 == 0 { out.push(','); } out.push(c); }
    if n < 0 { format!("-{out}") } else { out }
}
fn i(v: &Value, k: &str) -> i64 { v[k].as_i64().unwrap_or(0) }
fn f(v: &Value, k: &str) -> f64 { v[k].as_f64().unwrap_or(0.0) }

pub struct Engine { pub st: SocialStore }
impl Engine {
    pub fn new(project: &str) -> Result<Self, String> { Ok(Self { st: SocialStore::new(project).map_err(|e| e.to_string())? }) }

    pub fn kpis(&self, window: &str, platform: Option<&str>, account_id: Option<&str>) -> Value {
        let posts = self.st.list_posts(None, None, None, None, None, None);
        let m = self.st.list_all_latest_metrics();
        let now = chrono::Utc::now().with_timezone(&tz());
        let cutoff = match window { "7d" => Some(now - Duration::days(7)), "30d" => Some(now - Duration::days(30)), _ => None };
        let (mut views, mut likes, mut comments, mut shares, mut saves, mut subs, mut count) = (0i64, 0i64, 0i64, 0i64, 0i64, 0i64, 0i64);
        let (mut watch, mut rev) = (0.0, 0.0);
        let (mut rets, mut ctrs) = (vec![], vec![]);
        for p in &posts { for t in p["targets"].as_array().into_iter().flatten() {
            if platform.map_or(false, |x| s(t, "platform") != x) || account_id.map_or(false, |x| s(t, "account_id") != x) { continue; }
            if let (Some(c), Some(d)) = (cutoff, t["published_at"].as_str().and_then(parse_dt)) { if d < c { continue; } }
            let Some(sn) = m.get(s(t, "id")) else { if s(t, "status") == "published" { count += 1; } continue };
            count += 1;
            views += i(sn, "views"); likes += i(sn, "likes"); comments += i(sn, "comments"); shares += i(sn, "shares"); saves += i(sn, "saves");
            watch += f(sn, "watch_time_s"); subs += i(sn, "subscribers_gained"); rev += f(sn, "revenue");
            if f(sn, "avg_retention_pct") > 0.0 { rets.push(f(sn, "avg_retention_pct")); }
            if f(sn, "ctr") > 0.0 { ctrs.push(f(sn, "ctr")); }
        } }
        let er = if views > 0 { (likes + comments + shares + saves) as f64 / views.max(1) as f64 * 100.0 } else { 0.0 };
        let mean = |v: &Vec<f64>| if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 };
        json!({"total_views":views,"total_watch_time_s":watch,"avg_retention_pct":r(mean(&rets),2),"engagement_rate":r(er,2),"total_likes":likes,"total_comments":comments,
            "total_shares":shares,"total_saves":saves,"follower_gain":subs,"avg_ctr":r(mean(&ctrs),2),"affiliate_revenue":r(rev,2),"post_count":count})
    }

    pub fn golden_hours(&self) -> Value {
        let m = self.st.list_all_latest_metrics();
        let mut grid: Vec<Vec<Vec<f64>>> = vec![vec![vec![]; 24]; 7];
        for p in self.st.list_posts(None, None, None, None, None, None) { for t in p["targets"].as_array().into_iter().flatten() {
            if s(t, "status") != "published" { continue; }
            let Some(d) = t["published_at"].as_str().filter(|x| !x.is_empty()).or(t["scheduled_at"].as_str().filter(|x| !x.is_empty())).and_then(parse_dt) else { continue };
            grid[d.weekday().num_days_from_monday() as usize][d.hour() as usize].push(m.get(s(t, "id")).map_or(0.0, |x| f(x, "views")));
        } }
        let heur = |w: usize, h: usize| { let mut sc = 10.0; if h == 11 || h == 12 { sc += 35.0 } else if (19..=21).contains(&h) { sc += 55.0 } else if h == 8 || h == 9 { sc += 20.0 } if w >= 5 { sc += 15.0 } sc };
        let mut matrix = vec![];
        let mut slots: Vec<Value> = vec![];
        for w in 0..7 { let mut row = vec![]; for h in 0..24 {
            let mut vals = grid[w][h].clone();
            let (med, cnt, score) = if vals.is_empty() { (0.0, 0, heur(w, h)) } else { let med = median(&mut vals); (med, vals.len(), med * 0.7 + vals.len() as f64 * 5.0 + heur(w, h) * 0.3) };
            let info = json!({"weekday":w,"hour":h,"median_views":r(med,1),"sample_count":cnt,"score":r(score,1)});
            row.push(info.clone()); slots.push(info);
        } matrix.push(row); }
        slots.sort_by(|a, b| f(b, "score").partial_cmp(&f(a, "score")).unwrap()); // stable, like Python
        let mut wins: Vec<Value> = vec![];
        let mut seen = std::collections::HashSet::new();
        for sl in &slots {
            let (w, h) = (i(sl, "weekday"), i(sl, "hour"));
            if !seen.contains(&w) || wins.len() < 4 { wins.push(json!({"weekday":w,"start":format!("{h:02}:00"),"end":format!("{:02}:00", (h + 2).min(23))})); seen.insert(w); }
            if wins.len() >= 5 { break; }
        }
        json!({"matrix":matrix,"top_slots":slots.iter().take(10).collect::<Vec<_>>(),"recommended_windows":wins})
    }

    fn read(&self, parts: &[&str]) -> Option<Value> {
        let p = crate::workspace::project_path(&self.st.project_id, parts, false).ok()?;
        serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()
    }
    fn safe(id: &str) -> bool { !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') }

    pub fn creatives(&self) -> Value {
        let m = self.st.list_all_latest_metrics();
        let mut tpl: BTreeMap<String, (i64, i64, i64)> = BTreeMap::new();
        let mut bps: BTreeMap<String, (i64, i64, i64)> = BTreeMap::new();
        let mut tpl_order: Vec<String> = vec![];
        let mut bp_order: Vec<String> = vec![];
        for p in self.st.list_posts(None, None, None, None, None, None) {
            let mut tid = "unknown".to_string();
            if let Some(j) = p["render_job_id"].as_str().filter(|x| Self::safe(x)) {
                let d = self.read(&["runs", "render_queue", &format!("{j}.req.json")]).or_else(|| self.read(&["runs", "render_queue", &format!("{j}.json")]));
                if let Some(t) = d.and_then(|d| d["template_id"].as_str().map(String::from)) { tid = t; }
            }
            let mut bp = "none".to_string();
            if let Some(idea) = p["idea_id"].as_str().filter(|x| Self::safe(x)) {
                if let Some(d) = self.read(&["content", "ideas", &format!("{idea}.json")]).or_else(|| self.read(&["content", "ideas", &format!("idea_{idea}.json")])) {
                    bp = d["blueprint_ref"].as_str().filter(|x| !x.is_empty()).unwrap_or("none").to_string();
                    if tid == "unknown" { if let Some(t) = d["template_id"].as_str().filter(|x| !x.is_empty()) { tid = t.into(); } }
                }
            }
            let (mut v, mut it) = (0, 0);
            for t in p["targets"].as_array().into_iter().flatten() { if let Some(sn) = m.get(s(t, "id")) { v += i(sn, "views"); it += i(sn, "likes") + i(sn, "comments") + i(sn, "shares") + i(sn, "saves"); } }
            if !tpl.contains_key(&tid) { tpl_order.push(tid.clone()); }
            let e = tpl.entry(tid).or_default(); e.0 += 1; e.1 += v; e.2 += it;
            if bp != "none" { if !bps.contains_key(&bp) { bp_order.push(bp.clone()); } let e = bps.entry(bp).or_default(); e.0 += 1; e.1 += v; e.2 += it; }
        }
        let build = |order: &Vec<String>, map: &BTreeMap<String, (i64, i64, i64)>, key: &str| {
            let mut l: Vec<Value> = order.iter().map(|k| { let (c, v, it) = map[k]; json!({key:k,"post_count":c,"total_views":v,"avg_views":r(v as f64 / c.max(1) as f64, 1),"avg_er":r(it as f64 / v.max(1) as f64 * 100.0, 2)}) }).collect();
            l.sort_by(|a, b| f(b, "avg_views").partial_cmp(&f(a, "avg_views")).unwrap());
            l
        };
        json!({"templates":build(&tpl_order,&tpl,"template_id"),"blueprints":build(&bp_order,&bps,"blueprint_ref")})
    }

    pub fn outliers(&self) -> Value {
        let m = self.st.list_all_latest_metrics();
        let mut items = vec![];
        for p in self.st.list_posts(None, None, None, None, None, None) { for t in p["targets"].as_array().into_iter().flatten() {
            if s(t, "status") != "published" { continue; }
            items.push((s(&p, "id").to_string(), s(t, "id").to_string(), s(&p, "title").to_string(), s(t, "platform").to_string(), m.get(s(t, "id")).map_or(0, |x| i(x, "views"))));
        } }
        if items.is_empty() { return json!({"top_outliers":[],"underperforming":[]}); }
        let mut vs: Vec<f64> = items.iter().map(|x| x.4 as f64).collect();
        let med = median(&mut vs).max(1.0);
        let (mut top, mut under) = (vec![], vec![]);
        for (pid, tid, title, plat, v) in &items {
            let ratio = r(*v as f64 / med, 2);
            let mk = |kind: &str, h: &str| json!({"post_id":pid,"target_id":tid,"title":title,"platform":plat,"views":v,"benchmark_views":med,"ratio":ratio,"kind":kind,"hypothesis":h});
            if ratio >= 2.0 { top.push(mk("top_outlier", "Hook 3s cuốn hút, chủ đề đánh trúng insight khán giả và phát hành đúng khung giờ vàng tương tác cao.")); }
            else if ratio <= 0.5 && *v > 0 { under.push(mk("underperforming", "Nội dung mở đầu thiếu nhịp giật gân, đăng lệch khung giờ hoặc hình ảnh cover chưa nổi bật.")); }
        }
        top.sort_by_key(|x| std::cmp::Reverse(i(x, "views")));
        under.sort_by_key(|x| i(x, "views"));
        json!({"top_outliers":top,"underperforming":under})
    }

    pub fn learnings_md(&self) -> String {
        let k = self.kpis("30d", None, None);
        let g = self.golden_hours();
        let c = self.creatives();
        let o = self.outliers();
        let top_t = c["templates"][0]["template_id"].as_str().unwrap_or("tiktok-sales-v1").to_string();
        let slots: Vec<&Value> = g["top_slots"].as_array().unwrap().iter().take(3).collect();
        let slot_desc = if slots.is_empty() { "19:00 - 21:00 các ngày trong tuần".to_string() } else { slots.iter().map(|s| format!("Thứ {} lúc {:02}:00", i(s, "weekday") + 2, i(s, "hour"))).collect::<Vec<_>>().join(", ") };
        let now = chrono::Utc::now().with_timezone(&tz()).format("%Y-%m-%d");
        let ret = f(&k, "avg_retention_pct");
        let mut md = format!("# Báo cáo Phân tích & Đúc kết Tự học ({now})\n\n## 1. Chỉ số Tổng quan (30 ngày gần nhất)\n- **Tổng lượt xem:** {} views\n- **Tỷ lệ tương tác (ER):** {}%\n- **Thời gian xem tích lũy:** {} giờ\n- **Người theo dõi mới:** +{}\n- **Tổng số bài đã xuất bản:** {} video\n\n## 2. Khung Giờ Vàng Đề Xuất\n- Các khung giờ đạt median views và điểm tương tác cao nhất: **{slot_desc}**.\n- Đề xuất: Tập trung lên lịch các video bán hàng và KOC review vào các khung giờ này để đạt tỷ lệ phân phối thuật toán tối đa.\n\n## 3. Hiệu Quả Mẫu Video & Kịch Bản\n- **Template dẫn đầu:** `{top_t}` mang lại lượt xem trung bình và tỷ lệ chuyển đổi cao nhất.\n- Khuyến nghị Session A: Tiếp tục tối ưu hóa biến thể hook cho `{top_t}`.\n- Khuyến nghị Session B: Tăng cường các ý tưởng nội dung theo chủ đề giải đáp thắc mắc và so sánh trực diện.\n\n## 4. Phân Tích Bài Đột Phá & Kém Hiệu Quả\n- **Bài bùng nổ ({} bài):**\n",
            thousands(i(&k, "total_views")), f(&k, "engagement_rate"), r(f(&k, "total_watch_time_s") / 3600.0, 1), thousands(i(&k, "follower_gain")), i(&k, "post_count"), o["top_outliers"].as_array().unwrap().len());
        let tops = o["top_outliers"].as_array().unwrap();
        for x in tops.iter().take(3) { md += &format!("  - **{}** ({} views, {}x trung bình): {}\n", s(x, "title"), thousands(i(x, "views")), f(x, "ratio"), s(x, "hypothesis")); }
        if tops.is_empty() { md += "  - Chưa có bài đột phá vượt ngưỡng 2x trung bình.\n"; }
        md += "\n- **Bài cần cải thiện:**\n";
        let unders = o["underperforming"].as_array().unwrap();
        for x in unders.iter().take(3) { md += &format!("  - **{}** ({} views): {}\n", s(x, "title"), thousands(i(x, "views")), s(x, "hypothesis")); }
        if unders.is_empty() { md += "  - Không có bài viết nào dưới ngưỡng 0.5x trung bình.\n"; }
        md += "\n## 5. Kế hoạch Hành động Tuần Tới\n1. Áp dụng ngay bộ khung giờ vàng mới vào cấu hình đăng của các tài khoản chính.\n2. Tự động chuyển đổi các kịch bản đạt rating cao sang trạng thái dựng video.\n3. Duy trì nhịp đăng tối thiểu 1 video/ngày cho TikTok và YouTube Shorts.\n";
        let (hook, pace) = if ret > 0.0 && ret < 50.0 {
            (format!("Tỷ lệ giữ chân trung bình hiện tại ({ret}%) thấp hơn mốc kỳ vọng 50%. Tập sau bắt buộc phải có visual hook mạnh trong 3 giây đầu: Bi làm đổ vỡ, Koala nói câu hài hước, hoặc đặt câu hỏi thách đố người xem, tuyệt đối không mở đầu bằng lời chào chậm."),
             "Cắt ngắn các shot tĩnh, giữ nhịp chuyển cảnh dưới 3.0 giây/shot để giữ người xem không lướt qua.".to_string())
        } else if ret >= 50.0 {
            (format!("Tỷ lệ giữ chân trung bình rất tốt ({ret}%). Duy trì phong cách mở đầu trực diện, tập trung vào biểu cảm sinh động của Sóc Bi ngay khung hình đầu."),
             "Nhịp điệu phân cảnh 3.0–4.0s/shot đang hiệu quả, tiếp tục giữ tỷ lệ này.".to_string())
        } else {
            ("Chưa có đủ số liệu giữ chân thực tế. Khuyến nghị tập sau áp dụng công thức Hook 3s chuẩn: Hình ảnh động bất ngờ + câu thoại giật mình của Sóc Bi.".to_string(),
             "Thời lượng tối ưu: 30–60 giây cho TikTok/Shorts, nhịp cắt 2.5–3.5s/shot.".to_string())
        };
        md += &format!("\n## 6. Gợi Ý Cụ Thể Cho Tập Kế Tiếp\n- **Chiến lược Hook 3s đầu:** {hook}\n- **Nhịp độ & Thời lượng:** {pace} Thời lượng tập sau nên giới hạn 30–60 giây.\n- **Tương tác nhân vật:** Khóa cứng tương quan chiều cao Bi (100cm) và Koala (150cm), đẩy mạnh tình huống Koala ngô nghê cứu nguy cho Bi.\n- **Khung giờ phát hành tốt nhất:** {slot_desc}.\n- **Template ưu tiên:** `{top_t}`.\n");
        self.st.save_learnings_md(&md);
        md
    }

    pub fn recommendations(&self) -> Value {
        let c = self.creatives();
        let g = self.golden_hours();
        let mut tops: Vec<String> = c["templates"].as_array().unwrap().iter().filter(|t| s(t, "template_id") != "unknown").map(|t| s(t, "template_id").to_string()).collect();
        if tops.is_empty() { tops = vec!["tiktok-sales-v1".into(), "koc-review-v1".into(), "shorts-fact-v1".into()]; }
        let bps: Vec<&str> = c["blueprints"].as_array().unwrap().iter().filter(|b| s(b, "blueprint_ref") != "none").map(|b| s(b, "blueprint_ref")).take(5).collect();
        let ret = f(&self.kpis("30d", None, None), "avg_retention_pct");
        let (hs, ht, pace) = if ret > 0.0 && ret < 50.0 { ("aggressive_curiosity", "Tập sau cần mở đầu với hook tò mò giật gân trong 3s đầu do retention dưới 50%.", 2.8) }
            else if ret >= 50.0 { ("direct_story", "Retention đạt >50%, duy trì mở đầu trực diện vào cốt truyện của Bi.", 3.5) }
            else { ("standard_action", "Chưa có đủ số liệu retention, dùng hook hành động chuẩn của Sóc Bi.", 3.0) };
        let best: Vec<&Value> = g["top_slots"].as_array().unwrap().iter().take(5).collect();
        let recs = json!({"updated_at":now_iso(),"priority_templates_for_session_a":tops.iter().take(5).collect::<Vec<_>>(),"priority_blueprints_for_session_b":bps,
            "best_posting_slots":best,"recommended_posting_windows":g["recommended_windows"],
            "next_episode_suggestions":{"hook_strategy":hs,"hook_recommendation":ht,"target_duration_s":45,"recommended_shot_pacing_s":pace,"recommended_template":tops[0],
                "prime_window":best.first().map(|x| (*x).clone()).unwrap_or(json!({"weekday":3,"hour":19})),"character_ratio":"Sóc Bi 100cm / Koala 150cm","voice_profile":"soc-nho (-14 LUFS, OmniVoice)"}});
        self.st.save_recommendations_json(&recs);
        recs
    }

    pub fn daily_insights(&self, date: Option<&str>) -> Value {
        let now = chrono::Utc::now().with_timezone(&tz());
        let d = date.map(String::from).unwrap_or_else(|| now.format("%Y-%m-%d").to_string());
        let (k, g, c, o) = (self.kpis("30d", None, None), self.golden_hours(), self.creatives(), self.outliers());
        let m = self.st.list_all_latest_metrics();
        let mut by_day: BTreeMap<String, serde_json::Map<String, Value>> = BTreeMap::new();
        for n in 0..14 {
            let key = (now - Duration::days(13 - n)).format("%Y-%m-%d").to_string();
            by_day.insert(key, [("tiktok", 0), ("youtube", 0), ("facebook", 0), ("instagram", 0), ("total", 0)].into_iter().map(|(k, v)| (k.to_string(), json!(v))).collect());
        }
        for p in self.st.list_posts(None, None, None, None, None, None) { for t in p["targets"].as_array().into_iter().flatten() {
            if s(t, "status") != "published" { continue; }
            let pd: String = t["published_at"].as_str().filter(|x| !x.is_empty()).unwrap_or(s(&p, "created_at")).chars().take(10).collect();
            if let Some(day) = by_day.get_mut(&pd) {
                let v = m.get(s(t, "id")).map_or(0, |x| i(x, "views"));
                let pl = if ["tiktok", "youtube", "facebook", "instagram"].contains(&s(t, "platform")) { s(t, "platform") } else { "tiktok" };
                day.insert(pl.into(), json!(day[pl].as_i64().unwrap() + v));
                day.insert("total".into(), json!(day["total"].as_i64().unwrap() + v));
            }
        } }
        let data = json!({"date":d,"generated_at":now_iso(),"kpis":k,"views_by_day":by_day,"golden_hours":g,"creatives":c,"outliers":o});
        self.st.save_insights(&d, &data);
        self.learnings_md();
        self.recommendations();
        data
    }
}
