//! Studio canvas nodes: channel_insight, blueprint_analyze, script_from_blueprint, compliance_check.
use super::analyzer::analyze_video;
use super::channels::*;
use super::compliance::check_and_clean_script;
use super::contracts::*;
use super::products::*;
use super::script_gen::{generate_complete_script, ScriptOpts};
use super::util::*;
use serde_json::{json, Value};

fn pick(inp: &Value, p: &Value, ks: &[&str]) -> String {
    ks.iter().find_map(|k| inp.get(*k).and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty()).map(|s| s.trim().to_string()))
        .or_else(|| ks.iter().find_map(|k| p.get(*k).and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty()).map(|s| s.trim().to_string()))).unwrap_or_default()
}
fn pretty(v: &impl serde::Serialize) -> String { serde_json::to_string_pretty(v).unwrap_or_default() }

pub async fn execute_intel_node(pid: &str, node_type: &str, params: Option<&Value>, inputs: Option<&Value>) -> ApiResult<Value> {
    let (empty, p, inp) = (json!({}), params.unwrap_or(&json!({})).clone(), inputs.unwrap_or(&json!({})).clone());
    let _ = empty;
    match node_type {
        "channel_insight" => {
            let url = pick(&inp, &p, &["channel_url"]);
            let name = pick(&inp, &p, &["channel_name"]);
            let user_videos = inp.get("user_videos").or_else(|| p.get("user_videos")).and_then(|v| v.as_array()).filter(|a| !a.is_empty()).cloned();
            let meta = if let Some(vids) = user_videos {
                ingest_user_channel_data(pid, &if url.is_empty() { format!("https://custom/{}", if name.is_empty() { "user_channel" } else { &name }) } else { url }, if name.is_empty() { "Kênh người dùng cung cấp" } else { &name }, &vids)?
            } else if !url.is_empty() {
                ingest_channel(pid, &url, p.get("max_videos").and_then(|v| v.as_u64()).unwrap_or(50) as u32, 0).await?
            } else { return Err(ApiError::bad("Node 'channel_insight' yêu cầu 'channel_url' hoặc 'user_videos' của kênh.")); };
            let a = analyze_channel(pid, &s(&meta, "id"))?;
            let (st, pillars) = (&a["stats"], a["pillars"].as_array().cloned().unwrap_or_default());
            let f = |k: &str| st[k].as_f64().unwrap_or(0.0);
            let summary = format!("### Phân tích Kênh: {}\n- **Trạng thái**: {}\n- **Tổng video phân tích**: {}\n- **Lượt xem trung vị (Median Views)**: {}\n- **Lượt xem trung bình (Avg Views)**: {}\n- **Thời lượng trung bình**: {} giây\n- **Tỷ lệ Shorts/Longs**: {:.0}% Shorts / {:.0}% Video dài\n- **Nhịp đăng**: {}\n- **Pillars chính**: {}\n- **Giữ chân**: {}\n",
                a["channel_meta"]["name"].as_str().unwrap_or("N/A"), a["data_label"].as_str().unwrap_or("DỮ LIỆU THẬT"), f("total_videos") as i64, fmt_thousands(f("median_views")), fmt_thousands(f("avg_views")),
                f("avg_duration_s"), f("shorts_ratio") * 100.0, f("longs_ratio") * 100.0, st["upload_cadence"].as_str().unwrap_or("N/A"),
                pillars.iter().take(3).map(|x| s(x, "name")).collect::<Vec<_>>().join(", "), st["retention_note"].as_str().unwrap_or("Không có số liệu bịa đặt"));
            Ok(json!({"blueprint": pretty(&a["blueprint"]), "pillars": pretty(&pillars), "summary": summary, "raw_analysis": a}))
        }
        "blueprint_analyze" => {
            let sid = pick(&inp, &p, &["video", "source_id"]);
            if sid.is_empty() { return Err(ApiError::bad("Node 'blueprint_analyze' yêu cầu đầu vào 'video' hoặc 'source_id'.")); }
            let bp = analyze_video(pid, &sid).await?;
            Ok(json!({"blueprint": pretty(&bp), "pacing_cuts_per_10s": bp.pacing_cuts_per_10s, "suggested_template_id": bp.suggested_template_id.clone().unwrap_or_else(|| "koc-review-v1".into()),
                      "hook": bp.hook, "cta": bp.cta, "beats_count": bp.beats.len(), "data_label": bp.data_label}))
        }
        "script_from_blueprint" => {
            let raw = inp.get("blueprint").or_else(|| p.get("blueprint"));
            let blueprint = raw.and_then(|b| b.as_str()).filter(|s| !s.trim().is_empty()).and_then(|b| serde_json::from_str::<VideoBlueprint>(b).ok().or_else(|| load_video_blueprint(pid, b.trim())))
                .or_else(|| raw.filter(|b| b.is_object()).and_then(|b| serde_json::from_value(b.clone()).ok()));
            let product_id = pick(&inp, &p, &["product_id"]);
            let product = match Some(product_id).filter(|i| !i.is_empty()).and_then(|i| get_product(pid, &i)).or_else(|| list_products(pid).into_iter().next()) {
                Some(x) => x,
                None => {
                    let mut x = Product::new(&format!("prod_demo_{pid}"), "[MẪU] Sản phẩm Trải nghiệm KOC");
                    x.brand = "SocNho Brand".into(); x.price = 199000.0; x.usp = vec!["Tiện dụng".into(), "Thiết kế thông minh".into()];
                    x.pain_points = vec!["Bất tiện hàng ngày".into()]; x.compliance_notes = "[MẪU] Sản phẩm demo tự động".into();
                    save_product(pid, &x)?; x
                }
            };
            let ch = { let c = pick(&inp, &p, &["character", "character_name"]); if c.is_empty() { "LyLy".to_string() } else { c } };
            let voice = { let v = p.get("voice_id").and_then(|v| v.as_str()).unwrap_or("soc-nho").trim().to_string(); if v.is_empty() { "soc-nho".to_string() } else { v } };
            let gt = p.get("generator_type").and_then(|v| v.as_str()).unwrap_or("koc_review");
            let r = generate_complete_script(pid, &product, &ScriptOpts { generator_type: gt, blueprint: blueprint.as_ref(), character_name: &ch, voice_id: &voice, ..Default::default() })?;
            Ok(json!({"script": pretty(&r["lines"]), "full_text": r["full_text"], "render_request": pretty(&r["render_request"]), "template_id": r["template_id"],
                      "compliance_warnings": r["compliance_warnings"], "hook_lab": r["hook_lab"]}))
        }
        "compliance_check" => {
            let text = pick(&inp, &p, &["text"]);
            let r = check_and_clean_script(&text, p.get("is_affiliate").and_then(|v| v.as_bool()).unwrap_or(true));
            Ok(json!({"clean_text": r.modified_text, "passed": r.passed, "warnings": r.warnings, "violations": r.violations, "disclosure_added": r.disclosure_added}))
        }
        other => Err(ApiError::bad(format!("Loại node không xác định trong intelligence node runner: '{other}'"))),
    }
}
