//! Studio chat assistant for the intelligence stream (keyword-routed; same intents/replies as the Python version).
use super::channels::*;
use super::compliance::check_and_clean_script;
use super::ideas::generate_ideas_batch;
use super::products::*;
use super::script_gen::{generate_complete_script, ScriptOpts};
use super::strategy::{generate_channel_strategy, load_channel_strategy};
use super::util::*;
use regex::Regex;
use serde_json::{json, Value};

fn any(t: &str, ks: &[&str]) -> bool { ks.iter().any(|k| t.contains(k)) }

pub async fn handle_intel_chat(pid: &str, prompt: &str, _ctx: Option<&Value>) -> ApiResult<Value> {
    let p = prompt.trim();
    let low = p.to_lowercase();
    let found_url = Regex::new(r"https?://[^\s]+").unwrap().find(p).map(|m| m.as_str().trim_end_matches(['.', ',', ')']).to_string());
    let mut actions: Vec<Value> = vec![];

    if any(&low, &["tuân thủ", "compliance", "từ cấm", "vi phạm", "kiểm tra kịch bản"]) {
        let text = match Regex::new(r#"(?s)["“'](.*?)["”']"#).unwrap().captures(p) {
            Some(c) => c[1].trim().to_string(),
            None => Regex::new(r"(?i)^(kiểm tra\s+)?(tuân thủ|compliance|từ cấm|chính sách)(\s+kịch bản)?[:\s\-]*").unwrap().replace(p, "").trim().to_string(),
        };
        if text.chars().count() < 5 {
            return Ok(json!({"reply": "**[Kiểm tra Tuân thủ Chính sách Quảng cáo]**\n\nVui lòng cung cấp đoạn văn hoặc kịch bản cần kiểm tra. Ví dụ:\n> *Kiểm tra tuân thủ: \"Sản phẩm này cam kết chữa khỏi 100% bệnh đau lưng và tốt nhất Việt Nam.\"*",
                "mode": "intel_compliance", "actions": [], "data_label": "DỮ LIỆU THẬT"}));
        }
        let r = check_and_clean_script(&text, true);
        let mut reply = format!("### Kết quả Kiểm tra Tuân thủ: {}\n\n- **Trạng thái**: {}\n- **Nhãn Tiếp thị (#affiliate)**: {}\n\n",
            if r.passed { "✅ HỢP LỆ" } else { "⚠️ PHÁT HIỆN VI PHẠM" },
            if r.passed { "Không có vi phạm chính sách".to_string() } else { format!("Có {} cảnh báo", r.violations.len()) },
            if r.disclosure_added { "Đã tự động bổ sung" } else { "Đã có sẵn" });
        let mut details = vec![];
        for v in &r.violations {
            details.push(format!("- **{} ({})**: {}", v.severity, v.category, v.message));
            if !v.suggested_replacement.is_empty() { details.push(format!("  *Thay thế đề xuất*: `{}`", v.suggested_replacement)); }
        }
        if !details.is_empty() { reply += &format!("#### Chi tiết vi phạm phát hiện:\n{}\n\n", details.join("\n")); }
        reply += &format!("#### Bản văn bản đã được làm sạch và chuẩn hoá:\n```text\n{}\n```", r.modified_text);
        return Ok(json!({"reply": reply, "mode": "intel_compliance", "actions": [], "data_label": "DỮ LIỆU THẬT", "compliance_result": r}));
    }

    if any(&low, &["kênh", "channel", "đối thủ", "nghiên cứu kênh"]) && (found_url.is_some() || any(&low, &["phân tích", "ingest", "học"])) {
        let Some(url) = found_url else {
            return Ok(json!({"reply": "**[Nghiên cứu Kênh Đối thủ]**\n\nHãy dán đường dẫn kênh YouTube hoặc TikTok công khai bạn muốn nghiên cứu. Ví dụ:\n> *Phân tích kênh https://www.youtube.com/@SocNhoTV*\n\nHệ thống sẽ bóc tách: lượt xem trung vị thực tế, tỷ lệ Shorts/Longs, lịch đăng, cụm chủ đề (Pillars) và công thức tiêu đề viral.",
                "mode": "intel_channel", "actions": [], "data_label": "DỮ LIỆU THẬT"}));
        };
        let res: Result<(Value, Value), String> = async {
            let meta = ingest_channel(pid, &url, 30, 0).await.map_err(|e| e.to_string())?;
            let a = analyze_channel(pid, &s(&meta, "id")).map_err(|e| e.1)?;
            Ok((meta, a))
        }.await;
        return Ok(match res {
            Ok((meta, a)) => {
                let st = &a["stats"];
                let f = |k: &str| st[k].as_f64().unwrap_or(0.0);
                let mut reply = format!("### 📊 Phân tích Kênh: {}\n\n- **Nguồn dữ liệu**: {} (Trích xuất công khai qua yt-dlp)\n- **Tổng video phân tích**: {}\n- **Lượt xem trung vị (Median Views)**: {} view\n- **Lượt xem trung bình**: {} view\n- **Phân bổ định dạng**: {:.0}% Shorts / {:.0}% Video dài\n- **Lịch đăng**: {} (Giờ hiển thị: {})\n- **Tỷ lệ giữ chân**: {}\n\n#### 🎯 Các Pillars nội dung chủ đạo:\n",
                    meta["name"].as_str().unwrap_or(&s(&meta, "id")), meta["data_label"].as_str().unwrap_or("DỮ LIỆU THẬT"), f("total_videos") as i64, fmt_thousands(f("median_views")), fmt_thousands(f("avg_views")),
                    f("shorts_ratio") * 100.0, f("longs_ratio") * 100.0, st["upload_cadence"].as_str().unwrap_or("N/A"), a["schedule_heatmap_status"].as_str().unwrap_or("N/A"), st["retention_note"].as_str().unwrap_or("Không có số liệu bịa đặt"));
                for pi in a["pillars"].as_array().into_iter().flatten().take(4) { reply += &format!("- **{}**: {}% số video ({}% tổng lượt xem)\n", s(pi, "name"), pi["video_pct"].as_f64().unwrap_or(0.0), pi["view_pct"].as_f64().unwrap_or(0.0)); }
                reply += "\n#### 📝 Mẫu tiêu đề viral phổ biến:\n";
                for t in a["title_patterns"].as_array().into_iter().flatten().take(3) { reply += &format!("- `{}`\n", t.as_str().unwrap_or("")); }
                actions.push(json!({"type": "add_node", "nodeType": "channel_insight", "params": {"channel_url": url}, "summaryVi": format!("Thêm node Phân tích Kênh ({}) vào canvas", meta["name"].as_str().unwrap_or("Đối thủ"))}));
                json!({"reply": reply, "mode": "intel_channel", "actions": actions, "data_label": "DỮ LIỆU THẬT", "channel_id": meta["id"], "analysis": a})
            }
            Err(e) => json!({"reply": format!("❌ Lỗi khi tải dữ liệu kênh '{url}': {e}. Bạn có thể cung cấp danh sách video thủ công qua API user-data."), "mode": "intel_channel", "actions": [], "data_label": "MẪU"}),
        });
    }

    if any(&low, &["ý tưởng", "idea", "chủ đề", "gợi ý", "nội dung mới"]) && !any(&low, &["viết kịch bản", "tạo kịch bản", "kịch bản chi tiết"]) {
        let count = Regex::new(r"\b(\d+)\s*(ý tưởng|chủ đề|video)\b").unwrap().captures(&low).and_then(|c| c[1].parse::<usize>().ok()).map(|n| n.min(30)).unwrap_or(10);
        let ideas = generate_ideas_batch(pid, count);
        let mut reply = format!("### 💡 Gợi ý {} Ý Tưởng Video KOC & Hoạt Hình Sóc Nhỏ\n- **Bảo vệ chống sao chép**: Đã lọc qua RapidFuzz guardrail (< 70% trùng lặp đối thủ)\n- **Trạng thái**: Đã ghi vào `content/ideas/` sẵn sàng lên lịch đăng\n\n", ideas.len());
        for (i, id) in ideas.iter().take(count).enumerate() { reply += &format!("{}. **{}** (Độ ưu tiên: ⭐{} | Template: `{}`)\n", i + 1, id.title, id.priority, id.template_id.clone().unwrap_or_else(|| "None".into())); }
        actions.push(json!({"type": "open_tab", "tab": "ideas", "summaryVi": "Mở danh sách ý tưởng nội dung"}));
        return Ok(json!({"reply": reply, "mode": "intel_ideas", "actions": actions, "data_label": "DỮ LIỆU THẬT", "ideas": ideas.iter().take(count).collect::<Vec<_>>()}));
    }

    if any(&low, &["kịch bản", "script", "viết review", "bán hàng"]) || (low.contains("koc") && any(&low, &["viết", "tạo", "soạn", "làm", "review", "chạy"])) {
        let gt = if any(&low, &["bán hàng", "sales", "tiktok shop"]) { "tiktok_sales" } else if any(&low, &["so sánh", "compare"]) { "affiliate_compare" } else { "koc_review" };
        let ch = if low.contains("neau") { "Neau" } else if low.contains("sóc nhỏ") || low.contains("soc nho") || low.contains("bi") { "Sóc Nhỏ" } else { "LyLy" };
        let prods = list_products(pid);
        let prod = prods.iter().find(|p| low.contains(&p.name.to_lowercase())).or(prods.first()).cloned().unwrap_or_else(|| {
            let mut x = Product::new(&format!("prod_demo_{pid}"), "[MẪU] Kem Chống Nắng Quang Phổ Rộng Sóc Nhỏ");
            x.brand = "Sóc Nhỏ Lab".into(); x.price = 289000.0; x.usp = vec!["Kháng nước 80 phút".into(), "Kiềm dầu 12 tiếng".into()];
            x.pain_points = vec!["Kem bết dính và chảy vệt trắng".into()]; x.compliance_notes = "[MẪU] Dữ liệu sản phẩm mẫu".into();
            let _ = save_product(pid, &x); x
        });
        let r = generate_complete_script(pid, &prod, &ScriptOpts { generator_type: gt, character_name: ch, voice_id: "soc-nho", ..Default::default() })?;
        let mut reply = format!("### 🎬 Kịch bản KOC: {}\n- **Định dạng**: `{gt}` | **Nhân vật**: {ch} | **Giọng đọc**: `soc-nho` (-14 LUFS)\n- **Template gắn kết**: `{}`\n- **Tuân thủ chính sách**: {}\n\n#### 🎯 Thử nghiệm Hook Lab (Video Craft A/B):\n",
            prod.name, s(&r, "template_id"), if r["compliance_passed"] == true { "✅ Đã kiểm duyệt" } else { "⚠️ Cần lưu ý" });
        for h in r["hook_lab"].as_array().into_iter().flatten() { reply += &format!("- **Phương án {}** ({}) ⭐ {:.0}/100: \"{}\"\n", s(h, "variant_id"), s(h, "archetype"), h["score"].as_f64().unwrap_or(0.0), s(h, "text")); }
        reply += "\n#### 📜 Kịch bản chi tiết theo phân cảnh:\n";
        for (i, l) in r["lines"].as_array().into_iter().flatten().enumerate() { reply += &format!("{}. **[{}]** ({}s): \"{}\"\n", i + 1, s(l, "role").to_uppercase(), l["duration_s"], s(l, "text")); }
        actions.push(json!({"type": "add_script", "scriptId": r["script_id"], "title": format!("Review {}", prod.name), "templateId": r["template_id"], "summaryVi": format!("Lưu kịch bản {} vào danh sách kịch bản dự án", prod.name)}));
        return Ok(json!({"reply": reply, "mode": "intel_script", "actions": actions, "data_label": "DỮ LIỆU THẬT", "script_result": r}));
    }

    if any(&low, &["sản phẩm", "product", "hàng hóa", "giỏ hàng"]) {
        if let Some(url) = found_url.as_ref().filter(|_| any(&low, &["nhập", "import", "thêm"])) {
            let ex = import_product_from_url(url).await;
            let slug_t = Regex::new(r"[^a-zA-Z0-9]+").unwrap().replace_all(ex["title"].as_str().unwrap_or("item"), "_").to_string();
            let mut np = Product::new(&format!("prod_{}", slug_t.chars().take(16).collect::<String>().to_lowercase()), ex["title"].as_str().filter(|t| !t.is_empty()).unwrap_or("Sản phẩm mới"));
            np.price = ex["price"].as_f64().unwrap_or(0.0);
            if let Some(d) = ex["description"].as_str().filter(|d| !d.is_empty()) { np.usp = vec![d.to_string()]; }
            np.affiliate_links.other = url.clone(); np.compliance_notes = "Nhập từ OpenGraph công khai".into();
            save_product(pid, &np)?;
            return Ok(json!({"reply": format!("✅ Đã nhập sản phẩm từ URL công khai:\n- **Tên**: {}\n- **Giá**: {} {}\n- **Mã ID**: `{}`\n\nBạn có thể yêu cầu: *“Viết kịch bản KOC review cho sản phẩm {}”*.", np.name, fmt_thousands(np.price), np.currency, np.id, np.name),
                "mode": "intel_product", "actions": [], "data_label": "DỮ LIỆU THẬT", "product": np}));
        }
        let prods = list_products(pid);
        if prods.is_empty() {
            return Ok(json!({"reply": "Hiện tại chưa có sản phẩm nào trong dự án. Bạn có thể dán đường dẫn sản phẩm công khai hoặc nhập thông tin sản phẩm mới.", "mode": "intel_product", "actions": [], "data_label": "DỮ LIỆU THẬT"}));
        }
        let mut reply = format!("### 🛍️ Danh sách Sản phẩm KOC ({} sản phẩm):\n", prods.len());
        for p in &prods { reply += &format!("- **{}** (`{}`): {} {} | USP: {}\n", p.name, p.id, fmt_thousands(p.price), p.currency, if p.usp.is_empty() { "N/A".into() } else { p.usp.iter().take(2).cloned().collect::<Vec<_>>().join(", ") }); }
        return Ok(json!({"reply": reply, "mode": "intel_product", "actions": [], "data_label": "DỮ LIỆU THẬT", "products": prods}));
    }

    if any(&low, &["chiến lược", "strategy", "định hướng", "kpi"]) {
        let strat = match load_channel_strategy(pid) { Some(s) => s, None => generate_channel_strategy(pid, &[])? };
        let mut reply = format!("### 🎯 Chiến Lược Kênh & Định Vị Nội Dung Sóc Nhỏ TV\n\n**Điểm khác biệt độc quyền**:\n{}\n\n#### 📌 Phân bổ 4 Pillars Nội Dung:\n", s(&strat, "differentiation"));
        for pi in strat["our_pillars"].as_array().into_iter().flatten() { reply += &format!("- **{}** ({}%): {}\n", s(pi, "name"), pi["target_ratio"], s(pi, "description")); }
        let k = &strat["kpi_30d"];
        reply += &format!("\n#### 📈 Mục tiêu Kế hoạch 30 Ngày ({})\n- Lượt xem mục tiêu: {}\n- Người đăng ký mới: {}\n- Lượt click affiliate: {}\n",
            k["note"].as_str().unwrap_or("[MẪU]"), fmt_thousands(k["target_total_views"].as_f64().unwrap_or(0.0)), fmt_thousands(k["target_subscribers"].as_f64().unwrap_or(0.0)), fmt_thousands(k["target_affiliate_clicks"].as_f64().unwrap_or(0.0)));
        return Ok(json!({"reply": reply, "mode": "intel_strategy", "actions": [], "data_label": "DỮ LIỆU THẬT", "strategy": strat}));
    }

    Ok(json!({"reply": "**[Trợ Lý Nghiên Cứu Kênh & Kịch Bản KOC SocNhoTV]**\n\nTôi có thể hỗ trợ bạn trực tiếp các tác vụ sau:\n1. **Phân tích Kênh Đối Thủ**: *“Phân tích kênh https://youtube.com/@...”*\n2. **Sinh Kịch Bản KOC**: *“Viết kịch bản KOC review cho sản phẩm Kem Chống Nắng”*\n3. **Kiểm Tra Tuân Thủ**: *“Kiểm tra tuân thủ: Sản phẩm này cam kết chữa khỏi 100%...”*\n4. **Gợi Ý Ý Tưởng**: *“Gợi ý 10 ý tưởng video KOC mới”*\n5. **Quản Lý Sản Phẩm**: *“Danh sách sản phẩm”* hoặc *“Nhập sản phẩm từ https://...”*\n6. **Chiến Lược Kênh**: *“Xem chiến lược kênh và định hướng nội dung”*",
        "mode": "intel_help", "actions": [], "data_label": "DỮ LIỆU THẬT"}))
}
