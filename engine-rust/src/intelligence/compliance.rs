//! Commercial-ad compliance cleaner (port of intelligence/compliance.py).
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

const MEDICAL: &[&str] = &[r"chữa khỏi\s*(?:100%|hoàn toàn|triệt để)", r"trị dứt điểm", r"cam kết khỏi bệnh", r"thay thế thuốc chữa bệnh", r"thần dược", r"thuốc tiên", r"chữa ung thư", r"chữa bách bệnh", r"đặc trị\s+[a-zA-ZÀ-ỹ0-9_]+", r"khỏi hẳn\s*(?:ngay|sau)", r"vĩnh viễn không tái phát"];
const ABSOLUTE: &[&str] = &[r"cam kết\s*(?:tuyệt đối|100%|vĩnh viễn)", r"hiệu quả tức thì sau\s*(?:1|2|3)\s*giây", r"tốt nhất thế giới", r"số 1 việt nam", r"an toàn tuyệt đối", r"cam kết hiệu quả 100%", r"hoàn tiền 100% nếu không khỏi", r"duy nhất tại việt nam"];
const DEFAME: &[&str] = &[r"tẩy chay\s+[a-zA-ZÀ-ỹ0-9_]+", r"hàng rác rưởi", r"lừa đảo trắng trợn", r"đừng mua đồ của\s+[a-zA-ZÀ-ỹ0-9_]+"];
const SCARCITY: &[&str] = &[r"chỉ còn duy nhất\s*(?:1|2|3)\s*(?:suất|cái|chiếc|đơn)", r"hôm nay duy nhất", r"cơ hội cuối cùng trong đời", r"chỉ dành cho\s*(?:1|2|3)\s*người đầu tiên"];

struct Group { category: &'static str, msg: &'static str, severity: &'static str, repl: &'static str, pats: Vec<(&'static str, Regex)> }

static GROUPS: Lazy<Vec<Group>> = Lazy::new(|| {
    let mk = |category, msg, severity, repl, src: &'static [&'static str]| Group {
        category, msg, severity, repl,
        pats: src.iter().map(|p| (*p, Regex::new(&format!("(?i){p}")).unwrap())).collect(),
    };
    vec![
        mk("medical_claim", "Phát hiện tuyên bố y tế bị cấm theo chính sách quảng cáo", "CRITICAL", "hỗ trợ cải thiện", MEDICAL),
        mk("absolute_claim", "Phát hiện cam kết tuyệt đối bị cấm", "CRITICAL", "mang lại trải nghiệm hiệu quả", ABSOLUTE),
        mk("defamation", "Phát hiện ngôn từ hạ thấp thương hiệu khác", "CRITICAL", "sản phẩm thông thường khác", DEFAME),
        mk("fake_scarcity", "Cảnh báo tính khan hiếm giả tạo", "WARNING", "số lượng ưu đãi có hạn", SCARCITY),
    ]
});

#[derive(Debug, Clone, Serialize)]
pub struct Violation { pub category: String, pub pattern: String, pub severity: String, pub message: String, pub suggested_replacement: String }

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceResult { pub passed: bool, pub warnings: Vec<String>, pub modified_text: String, pub violations: Vec<Violation>, pub disclosure_added: bool }

pub fn check_and_clean_script(text: &str, is_affiliate: bool) -> ComplianceResult {
    let (mut warnings, mut violations, mut clean, mut disclosure_added) = (vec![], vec![], text.to_string(), false);
    for g in GROUPS.iter() {
        for (src, re) in &g.pats {
            if re.is_match(&clean) {
                let msg = if g.severity == "WARNING" { format!("{}: '{}'", g.msg, src) } else { format!("{}: '{}'", g.msg, src) };
                warnings.push(msg.clone());
                violations.push(Violation { category: g.category.into(), pattern: (*src).into(), severity: g.severity.into(), message: msg, suggested_replacement: g.repl.into() });
                clean = re.replace_all(&clean, g.repl).to_string();
            }
        }
    }
    if is_affiliate {
        let low = clean.to_lowercase();
        if !["#affiliate", "#quangcao", "#tiepthilienket", "#sponsored"].iter().any(|t| low.contains(t)) {
            clean.push_str("\n\n(Video có gắn link tiếp thị liên kết #affiliate #quangcao)");
            warnings.push("Đã tự động bổ sung nhãn tiếp thị liên kết (#affiliate #quangcao).".into());
            disclosure_added = true;
        }
    }
    let critical = violations.iter().any(|v| v.severity == "CRITICAL");
    let passed = !critical && !warnings.iter().any(|w| w.contains("bị cấm"));
    ComplianceResult { passed, warnings, modified_text: clean.trim().to_string(), violations, disclosure_added }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flags_and_cleans() {
        let r = check_and_clean_script("Cam kết chữa khỏi 100% và tốt nhất thế giới", true);
        assert!(!r.passed && r.disclosure_added && r.violations.len() == 2);
        assert!(r.modified_text.contains("hỗ trợ cải thiện") && r.modified_text.contains("#affiliate"));
        assert!(check_and_clean_script("Áo đẹp", false).passed);
    }
}
