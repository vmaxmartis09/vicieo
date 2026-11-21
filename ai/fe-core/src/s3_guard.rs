use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct RuleViolation {
    pub file: String,
    pub line: usize,
    pub code: &'static str,
    pub severity: &'static str, // "BLOCK" | "WARN"
    pub msg: String,
    pub fix: &'static str,
}

static RE_ARBITRARY_FONT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\btext-\[(?:9|10|10\.5)px\]").unwrap());
static RE_HALF_STEP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:p|px|py|pt|pb|pl|pr|m|mx|my|mt|mb|ml|mr|gap|space-x|space-y)-(?:0\.5|1\.5|2\.5|3\.5|7|9|11)\b").unwrap()
});
static RE_ARBITRARY_SPACE_RADIUS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:p|px|py|m|gap|rounded)-\[\d+px\]").unwrap()
});
static RE_LUCIDE_IMPORT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"from\s+['"](?:lucide-react|react-icons[^'"]*|@heroicons/[^'"]*)['"]"#).unwrap()
});
static RE_WINDOW_CONFIRM: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bwindow\.(?:confirm|alert|prompt)\s*\(").unwrap());
static RE_BARE_STORE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\buse(?:Studio|Social|App|Crm)?Store\s*\(\s*\)").unwrap());
static RE_TRANSITION_ALL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\btransition-all\b|transition\s*:\s*all\b").unwrap());
static RE_AI_PURPLE_SLOP: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)#7c3aed|#ec4899|\bfrom-purple-\d+\s+to-(?:pink|blue|indigo)-\d+").unwrap());
static RE_DIV_ONCLICK: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<(?:div|span)\b[^>]*\bonClick=").unwrap());
static RE_IMG_TAG: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<img\b([^>]*)>").unwrap());
static RE_PUBLIC_SECRET: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(?:NEXT_PUBLIC|VITE|NUXT_PUBLIC)_[A-Z0-9_]*(?:SECRET|PRIVATE_KEY|SERVICE_ROLE|PASSWORD|API_SECRET)\b").unwrap()
});
static RE_FAKE_KPI_SLOP: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\+12\.5%|Welcome back,\s|10,000\+\s+teams|Quietly trusted by").unwrap());
static RE_TASTE_COPY_SLOP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(?:supercharge your|unlock the power of|seamlessly integrate|elevate your workflow|cutting-edge solution|all-in-one platform)\b").unwrap()
});
static RE_EYEBROW_SLOP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\buppercase\b[^\n]*\btracking-(?:widest|wider|\[\d)").unwrap()
});
static RE_LATEX_LEAK: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\\rightarrow\$|\$\\leftarrow\$").unwrap());

pub fn scan_fe_code(
    file_path: &str,
    new_snippet: &str,
    full_file: &str,
) -> (Vec<RuleViolation>, Vec<RuleViolation>) {
    let mut blocks = Vec::new();
    let mut warns = Vec::new();

    let norm = file_path.replace('\\', "/");
    let ext = Path::new(&norm)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if !matches!(ext, "tsx" | "ts" | "jsx" | "vue" | "svelte" | "astro" | "css") {
        return (blocks, warns);
    }

    let is_ui_file = matches!(ext, "tsx" | "jsx" | "vue" | "svelte" | "astro" | "css");
    let is_vicieo_dashboard = norm.contains("tool/dashboard/");

    for (idx, line) in new_snippet.lines().enumerate() {
        let ln = idx + 1;
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        // [VICIEO-UI-01] Token & Concentric Geometry
        if is_ui_file && RE_ARBITRARY_FONT.is_match(line) {
            blocks.push(RuleViolation {
                file: norm.clone(),
                line: ln,
                code: "VICIEO-UI-01",
                severity: "BLOCK",
                msg: "Cỡ chữ 9px/10px bị cấm (tối thiểu 11px text-caption).".to_string(),
                fix: "Dùng token chuẩn: text-caption (11px), text-footnote (12px), text-body (13px)",
            });
        }

        if is_ui_file && is_vicieo_dashboard && RE_HALF_STEP.is_match(line) {
            blocks.push(RuleViolation {
                file: norm.clone(),
                line: ln,
                code: "VICIEO-UI-01B",
                severity: "BLOCK",
                msg: "Vi phạm lưới 4px (bước lẻ .5 hoặc 7/9/11).".to_string(),
                fix: "Dùng bước chia hết cho 4px: gap-1 (4px), gap-2 (8px), p-3 (12px), p-4 (16px)",
            });
        }

        if is_ui_file && is_vicieo_dashboard && RE_ARBITRARY_SPACE_RADIUS.is_match(line) && !line.contains("min-h-[44px]") {
            blocks.push(RuleViolation {
                file: norm.clone(),
                line: ln,
                code: "VICIEO-UI-01C",
                severity: "BLOCK",
                msg: "Dùng giá trị pixel tự chế [..px] cho spacing hoặc border-radius.".to_string(),
                fix: "Dùng rounded-xs (4), rounded-ctl (8), rounded-card (12), rounded-panel (16), rounded-sheet (24)",
            });
        }

        // [VICIEO-UI-02] Anti-AI-Slop & Morphicons
        if is_vicieo_dashboard && RE_LUCIDE_IMPORT.is_match(line) {
            blocks.push(RuleViolation {
                file: norm.clone(),
                line: ln,
                code: "VICIEO-UI-02",
                severity: "BLOCK",
                msg: "Cấm import lucide-react / react-icons / @heroicons.".to_string(),
                fix: "Import icon từ '@/components/ui/icons' hoặc dùng <MorphIcon />",
            });
        }

        if is_ui_file && RE_TRANSITION_ALL.is_match(line) {
            blocks.push(RuleViolation {
                file: norm.clone(),
                line: ln,
                code: "VICIEO-UI-02B",
                severity: "BLOCK",
                msg: "Cấm dùng transition-all (gây layout thrash & mùi AI-Slop).".to_string(),
                fix: "Chỉ định rõ transition-colors, transition-opacity hoặc transition-transform (150-250ms)",
            });
        }

        if is_ui_file && RE_AI_PURPLE_SLOP.is_match(line) {
            blocks.push(RuleViolation {
                file: norm.clone(),
                line: ln,
                code: "VICIEO-UI-02C",
                severity: "BLOCK",
                msg: "Phát hiện bảng màu tím-hồng AI-Slop (#7C3AED / #EC4899 / purple gradient).".to_string(),
                fix: "Dùng CSS Variable token của dự án (bg-card, text-foreground, brand < 3%)",
            });
        }

        if is_ui_file && (RE_FAKE_KPI_SLOP.is_match(line) || RE_LATEX_LEAK.is_match(line)) {
            blocks.push(RuleViolation {
                file: norm.clone(),
                line: ln,
                code: "VICIEO-UI-02D",
                severity: "BLOCK",
                msg: "Phát hiện dữ liệu KPI/Social Proof giả ('+12.5%' / '10,000+ teams' / 'Welcome back') hoặc lọt chuỗi LaTeX ra UI.".to_string(),
                fix: "Chỉ hiển thị dữ liệu thật từ store/API và ký tự Unicode chuẩn",
            });
        }

        if is_ui_file && RE_TASTE_COPY_SLOP.is_match(line) {
            blocks.push(RuleViolation {
                file: norm.clone(),
                line: ln,
                code: "TASTE-WAVE6-COPY",
                severity: "BLOCK",
                msg: "Phát hiện văn phong quảng cáo AI sáo rỗng (Leonxlnx/taste-skill Wave 6 Copy Tell: supercharge/unlock/seamlessly/elevate).".to_string(),
                fix: "Viết câu khẳng định thực tế kèm số liệu/bằng chứng thật của sản phẩm (taste-skill references/copy.md)",
            });
        }

        if is_ui_file && RE_EYEBROW_SLOP.is_match(line) {
            warns.push(RuleViolation {
                file: norm.clone(),
                line: ln,
                code: "TASTE-WAVE2-EYEBROW",
                severity: "WARN",
                msg: "Phát hiện chữ Eyebrow/Kicker in hoa giãn cách (Leonxlnx/taste-skill Wave 2 & 4 Tell: uppercase + tracking-wider/widest).".to_string(),
                fix: "Dùng sentence case ở cỡ chữ text-caption/text-footnote đọc được tự nhiên, hoặc gộp vào tiêu đề",
            });
        }

        // [VICIEO-UI-03] Smart Interaction & A11y
        if RE_WINDOW_CONFIRM.is_match(line) {
            blocks.push(RuleViolation {
                file: norm.clone(),
                line: ln,
                code: "VICIEO-UI-03",
                severity: "BLOCK",
                msg: "Cấm dùng window.confirm / window.alert.".to_string(),
                fix: "Dùng hộp xác nhận mọc sát icon: await confirm({ tone, title, message }) từ '@/lib/confirm'",
            });
        }

        if is_ui_file && RE_DIV_ONCLICK.is_match(line) && !line.contains("role=") && !line.contains("tabIndex") {
            warns.push(RuleViolation {
                file: norm.clone(),
                line: ln,
                code: "VICIEO-UI-03B",
                severity: "WARN",
                msg: "Thẻ <div>/<span> có onClick nhưng thiếu ngữ nghĩa bàn phím & A11y.".to_string(),
                fix: "Chuyển sang thẻ <button type=\"button\"> hoặc thêm role=\"button\" tabIndex={0} onKeyDown",
            });
        }

        if is_ui_file {
            if let Some(cap) = RE_IMG_TAG.captures(line) {
                let attrs = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                if !attrs.contains("loading=") {
                    blocks.push(RuleViolation {
                        file: norm.clone(),
                        line: ln,
                        code: "VICIEO-UI-03C",
                        severity: "BLOCK",
                        msg: "Thẻ <img> thiếu loading=\"lazy\".".to_string(),
                        fix: "Thêm loading=\"lazy\" decoding=\"async\" alt=\"...\" hoặc dùng <Img />",
                    });
                }
            }
        }

        // [VICIEO-UI-04] State & Re-render Guard
        if RE_BARE_STORE.is_match(line) {
            blocks.push(RuleViolation {
                file: norm.clone(),
                line: ln,
                code: "VICIEO-UI-04",
                severity: "BLOCK",
                msg: "Gọi Zustand store trần không có selector (gây re-render toàn bộ cây UI).".to_string(),
                fix: "Dùng selector hẹp: useStudioStore(useShallow((s) => ({ ... })))",
            });
        }

        // [FE-SEC-05] Public Secret Exposure
        if RE_PUBLIC_SECRET.is_match(line) {
            blocks.push(RuleViolation {
                file: norm.clone(),
                line: ln,
                code: "FE-SEC-05",
                severity: "BLOCK",
                msg: "Nguy cơ lộ Secret Key ra Client Bundle qua biến NEXT_PUBLIC_ / VITE_ / NUXT_PUBLIC_.".to_string(),
                fix: "Bỏ tiền tố PUBLIC và chỉ đọc biến bí mật ở Server / Route Handler / Edge Function",
            });
        }
    }

    // Check file length budget & Elegans Co-located Architecture
    let total_lines = full_file.lines().count();
    let base_name = Path::new(&norm)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if base_name.starts_with("use") && total_lines > 150 {
        warns.push(RuleViolation {
            file: norm.clone(),
            line: total_lines,
            code: "ELEGANS-LOC-HOOK",
            severity: "WARN",
            msg: format!("Custom Hook dài {total_lines} dòng (vượt ngưỡng Elegans 150 dòng)."),
            fix: "Tách logic thuần sang utils/ hoặc chia nhỏ sub-hooks",
        });
    } else if is_ui_file && total_lines > 250 {
        blocks.push(RuleViolation {
            file: norm.clone(),
            line: total_lines,
            code: "ELEGANS-LOC-COMP",
            severity: "BLOCK",
            msg: format!("Component dài {total_lines} dòng (vượt ngưỡng Elegans 250 dòng). Vi phạm nguyên tắc bảo trì & tách bạch trách nhiệm."),
            fix: "Tách sub-components vào components/ và logic vào hooks/ theo chuẩn Elegans Co-location",
        });
    }

    (blocks, warns)
}

pub fn run_check_edit(args: &[String]) -> i32 {
    let mut file_path = String::new();
    let mut new_snippet = String::new();

    if let Some(pos) = args.iter().position(|a| a == "--file") {
        if let Some(f) = args.get(pos + 1) {
            file_path = f.clone();
            new_snippet = fs::read_to_string(&file_path).unwrap_or_default();
        }
    } else {
        let mut raw = String::new();
        let _ = io::stdin().read_to_string(&mut raw);
        if let Ok(v) = serde_json::from_str::<Value>(&raw) {
            let input = v.get("tool_input").or_else(|| v.get("input")).unwrap_or(&v);
            file_path = input
                .get("file_path")
                .or_else(|| input.get("Path"))
                .or_else(|| input.get("path"))
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            new_snippet = input
                .get("content")
                .or_else(|| input.get("Content"))
                .or_else(|| input.get("new_string"))
                .or_else(|| input.get("Replacement"))
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
        }
    }

    if file_path.is_empty() {
        return 0;
    }
    let full_file = fs::read_to_string(&file_path).unwrap_or_else(|_| new_snippet.clone());
    if new_snippet.is_empty() {
        new_snippet = full_file.clone();
    }

    let (blocks, warns) = scan_fe_code(&file_path, &new_snippet, &full_file);
    if blocks.is_empty() && warns.is_empty() {
        return 0;
    }

    let packet = json!({
        "file": file_path,
        "blocks": blocks.len(),
        "warns": warns.len(),
        "items": blocks.iter().chain(warns.iter()).take(8).collect::<Vec<_>>()
    });
    println!("@fe:s3{}", packet);

    if !blocks.is_empty() {
        return 2;
    }
    0
}

pub fn run_sloplint(args: &[String]) -> i32 {
    let target = args.first().map(|s| s.as_str()).unwrap_or(".");
    let mut total_files = 0usize;
    let mut all_blocks = Vec::new();
    let mut all_warns = Vec::new();

    for entry in WalkDir::new(target).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        let path_str = p.to_string_lossy();
        if path_str.contains("node_modules")
            || path_str.contains("/.next")
            || path_str.contains("/dist/")
            || path_str.contains("/target/")
        {
            continue;
        }
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "tsx" | "jsx" | "vue" | "svelte" | "astro" | "css") {
            continue;
        }
        total_files += 1;
        if let Ok(content) = fs::read_to_string(p) {
            let (b, w) = scan_fe_code(&path_str, &content, &content);
            all_blocks.extend(b);
            all_warns.extend(w);
        }
    }

    let penalty = (all_blocks.len() * 8 + all_warns.len() * 2).min(100);
    let human_score = 100usize.saturating_sub(penalty);
    let status = if all_blocks.is_empty() { "PASS" } else { "FAIL" };

    let out = json!({
        "status": status,
        "files_scanned": total_files,
        "slop_blocks": all_blocks.len(),
        "warns": all_warns.len(),
        "human_craft_score": human_score,
        "top_violations": all_blocks.iter().chain(all_warns.iter()).take(12).collect::<Vec<_>>()
    });
    println!("@fe:sloplint{}", out);
    if all_blocks.is_empty() { 0 } else { 1 }
}

pub fn run_seo_check(args: &[String]) -> i32 {
    let target = args.first().map(|s| s.as_str()).unwrap_or(".");
    let mut has_metadata = false;
    let mut has_jsonld = false;
    let mut has_sitemap = false;
    let mut has_robots = false;
    let mut has_llms_txt = false;
    let mut page_files = 0usize;

    for entry in WalkDir::new(target).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        let path_str = p.to_string_lossy().replace('\\', "/");
        if path_str.contains("node_modules") || path_str.contains("/.next") {
            continue;
        }
        let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if fname.starts_with("sitemap.") {
            has_sitemap = true;
        }
        if fname.starts_with("robots.") {
            has_robots = true;
        }
        if fname == "llms.txt" || path_str.contains("llms.txt") {
            has_llms_txt = true;
        }
        if matches!(
            fname,
            "page.tsx" | "layout.tsx" | "index.vue" | "app.vue" | "+page.svelte" | "index.astro"
        ) {
            page_files += 1;
            if let Ok(raw) = fs::read_to_string(p) {
                if raw.contains("generateMetadata")
                    || raw.contains("export const metadata")
                    || raw.contains("useSeoMeta")
                    || raw.contains("<svelte:head>")
                    || raw.contains("<meta")
                {
                    has_metadata = true;
                }
                if raw.contains("application/ld+json") {
                    has_jsonld = true;
                }
            }
        }
    }

    let mut missing = Vec::new();
    if !has_metadata {
        missing.push("metadata(title/desc/og)");
    }
    if !has_jsonld {
        missing.push("json-ld(application/ld+json)");
    }
    if !has_sitemap {
        missing.push("sitemap.xml");
    }
    if !has_robots {
        missing.push("robots.txt");
    }
    if !has_llms_txt {
        missing.push("llms.txt(GEO-AI-Search)");
    }

    let score = 100usize.saturating_sub(missing.len() * 15);
    let out = json!({
        "pages": page_files,
        "seo_score": score,
        "has": {
            "metadata": has_metadata,
            "json_ld": has_jsonld,
            "sitemap": has_sitemap,
            "robots": has_robots,
            "llms_txt": has_llms_txt
        },
        "missing": missing
    });
    println!("@fe:seo{}", out);
    0
}
