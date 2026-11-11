use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductInfo {
    pub id: String,
    pub title: String,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub highlights: Vec<String>,
    pub pain_points: Vec<String>,
    pub target_audience: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedScript {
    pub hook_title: String,
    pub target_duration_sec: u32,
    pub call_to_action: String,
    pub body_scenes: Vec<String>,
}

pub struct IntelligenceEngine;

impl IntelligenceEngine {
    /// Sinh kịch bản KOC AI theo định dạng hook 3s đầu chuẩn TikTok
    pub fn build_koc_prompt(product: &ProductInfo, style: &str) -> String {
        format!(
            "Hãy viết một kịch bản video TikTok 30s phong cách [{}] cho sản phẩm: {}\nĐiểm nổi bật: {:?}\nVấn đề giải quyết: {:?}",
            style, product.title, product.highlights, product.pain_points
        )
    }

    /// Kiểm tra vi phạm chính sách quảng cáo TikTok (Policy & Compliance Checker)
    pub fn check_policy_compliance(script_text: &str) -> Vec<String> {
        let banned_keywords = [
            "cam kết 100%",
            "trị dứt điểm",
            "tốt nhất thế giới",
            "chữa khỏi hoàn toàn",
            "thuốc thần",
        ];

        let mut violations = Vec::new();
        for kw in banned_keywords {
            if script_text.to_lowercase().contains(kw) {
                violations.push(format!("Từ cấm hoặc phóng đại quá mức: '{}'", kw));
            }
        }
        violations
    }
}

