use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct PonytailMatrix {
    pub design_pre: &'static str,
    pub dev_code: &'static str,
    pub code_audit: &'static str,
    pub qa_test: &'static str,
    pub ui_judge: &'static str,
    pub devops: &'static str,
}

#[derive(Debug, Serialize)]
pub struct SurgicalTarget {
    pub file: String,
    pub reason: String,
    pub slice_cmd: String,
}

#[derive(Debug, Serialize)]
pub struct RoutePacket {
    pub lane: String,
    pub stack: String,
    pub genre: String,
    pub seo: String,
    pub browser: String,
    pub pony: PonytailMatrix,
    pub agents: Vec<String>,
    pub skills: Vec<String>,
    pub surgical_files: Vec<SurgicalTarget>,
    pub solution_archetype: String,
    pub execution_plan: Vec<String>,
    pub reuse: Vec<String>,
}

pub fn detect_workspace_root() -> PathBuf {
    if let Ok(dir) = std::env::var("CLAUDE_PROJECT_DIR") {
        let p = PathBuf::from(dir);
        if p.exists() {
            return p;
        }
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut cur = cwd.as_path();
    loop {
        if cur.join("ai").exists() || cur.join(".claude").exists() || cur.join(".git").exists() {
            return cur.to_path_buf();
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => return cwd,
        }
    }
}

pub fn detect_stack(root: &Path, prompt_lc: &str) -> String {
    if prompt_lc.contains("nuxt") {
        return "nuxt3-vue3".to_string();
    }
    if prompt_lc.contains("vue") {
        return "vue3".to_string();
    }
    if prompt_lc.contains("svelte") {
        return "svelte5-kit".to_string();
    }
    if prompt_lc.contains("astro") {
        return "astro5".to_string();
    }
    if prompt_lc.contains("next") || prompt_lc.contains("rsc") || prompt_lc.contains("app router") {
        return "nextjs15-react19".to_string();
    }

    let pkg_candidates = [
        root.join("tool/dashboard/package.json"),
        root.join("package.json"),
    ];
    for pkg in &pkg_candidates {
        if let Ok(raw) = fs::read_to_string(pkg) {
            if raw.contains("\"nuxt\"") {
                return "nuxt3-vue3".to_string();
            }
            if raw.contains("\"svelte\"") {
                return "svelte5-kit".to_string();
            }
            if raw.contains("\"astro\"") {
                return "astro5".to_string();
            }
            if raw.contains("\"next\"") {
                return "nextjs15-react19".to_string();
            }
            if raw.contains("\"vue\"") {
                return "vue3".to_string();
            }
            if raw.contains("\"react\"") {
                return "react19".to_string();
            }
        }
    }
    "nextjs15-react19".to_string()
}

pub fn classify_genre(prompt_lc: &str) -> (&'static str, &'static str) {
    if prompt_lc.contains("crm")
        || prompt_lc.contains("erp")
        || prompt_lc.contains("kanban")
        || prompt_lc.contains("datatable")
        || prompt_lc.contains("bảng dữ liệu")
        || prompt_lc.contains("quản lý khách hàng")
    {
        return ("crm-erp-saas", "internal");
    }
    if prompt_lc.contains("shop")
        || prompt_lc.contains("ecommerce")
        || prompt_lc.contains("e-commerce")
        || prompt_lc.contains("giỏ hàng")
        || prompt_lc.contains("checkout")
        || prompt_lc.contains("stripe")
        || prompt_lc.contains("payos")
        || prompt_lc.contains("bán hàng")
    {
        return ("ecommerce-marketplace", "full-public-seo");
    }
    if prompt_lc.contains("3d")
        || prompt_lc.contains("webgl")
        || prompt_lc.contains("three.js")
        || prompt_lc.contains("r3f")
        || prompt_lc.contains("gsap")
        || prompt_lc.contains("scrollytelling")
        || prompt_lc.contains("awwwards")
    {
        return ("creative-3d-scrollytelling", "full-public-seo");
    }
    if prompt_lc.contains("landing")
        || prompt_lc.contains("hero")
        || prompt_lc.contains("marketing")
        || prompt_lc.contains("trang chủ")
        || prompt_lc.contains("waitlist")
        || prompt_lc.contains("pricing")
    {
        return ("landing-growth-funnel", "full-public-seo");
    }
    if prompt_lc.contains("blog")
        || prompt_lc.contains("docs")
        || prompt_lc.contains("lms")
        || prompt_lc.contains("khóa học")
        || prompt_lc.contains("tin tức")
        || prompt_lc.contains("mdx")
    {
        return ("content-blog-docs-lms", "full-public-seo+llms.txt");
    }
    if prompt_lc.contains("canvas")
        || prompt_lc.contains("node")
        || prompt_lc.contains("reactflow")
        || prompt_lc.contains("xyflow")
        || prompt_lc.contains("studio")
    {
        return ("interactive-canvas-studio", "internal");
    }
    if prompt_lc.contains("chat")
        || prompt_lc.contains("feed")
        || prompt_lc.contains("social")
        || prompt_lc.contains("cộng đồng")
        || prompt_lc.contains("messenger")
    {
        return ("social-chat-realtime", "hybrid-seo");
    }
    ("enterprise-dashboard-bi", "internal")
}

pub fn classify_lane(prompt_lc: &str) -> &'static str {
    if prompt_lc.contains("tiếp quản")
        || prompt_lc.contains("take over")
        || prompt_lc.contains("làm chủ")
        || prompt_lc.contains("khám bệnh")
        || prompt_lc.contains("đọc hiểu dự án")
    {
        return "FE-MASTER-BUILD";
    }

    let is_deploy = prompt_lc.contains("deploy")
        || prompt_lc.contains("vercel")
        || prompt_lc.contains("cloudflare")
        || prompt_lc.contains("wrangler")
        || prompt_lc.contains("netlify")
        || prompt_lc.contains(".env")
        || prompt_lc.contains("đóng gói");
    if is_deploy {
        return "FE-DEPLOY";
    }

    let is_audit_or_qa = prompt_lc.contains("review")
        || prompt_lc.contains("audit")
        || prompt_lc.contains("kiểm định")
        || prompt_lc.contains("sloplint")
        || prompt_lc.contains("test responsive")
        || prompt_lc.contains("qa ");
    if is_audit_or_qa {
        return "FE-AUDIT-QA";
    }

    let is_dev = prompt_lc.contains("build")
        || prompt_lc.contains("tạo")
        || prompt_lc.contains("dựng")
        || prompt_lc.contains("code")
        || prompt_lc.contains("implement")
        || prompt_lc.contains("refactor")
        || prompt_lc.contains("redesign")
        || prompt_lc.contains("thiết kế")
        || prompt_lc.contains("sửa ")
        || prompt_lc.contains("fix ")
        || prompt_lc.contains("component")
        || prompt_lc.contains("page")
        || prompt_lc.contains("trang ")
        || prompt_lc.contains("dashboard")
        || prompt_lc.contains("crm")
        || prompt_lc.contains("landing")
        || prompt_lc.contains("seo");

    if is_dev && prompt_lc.len() > 24 {
        return "FE-MASTER-BUILD";
    }
    "FE-THUNDER"
}

pub fn find_reusable_candidates(root: &Path, prompt_lc: &str) -> Vec<String> {
    let mut out = Vec::new();
    let search_dirs = [
        root.join("tool/dashboard/components/ui"),
        root.join("tool/dashboard/lib"),
        root.join("tool/dashboard/store"),
        root.join("src/components/ui"),
        root.join("components/ui"),
    ];

    let keywords: Vec<&str> = prompt_lc
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 4)
        .take(8)
        .collect();

    for dir in &search_dirs {
        if !dir.exists() {
            continue;
        }
        for entry in WalkDir::new(dir).max_depth(2).into_iter().flatten() {
            if !entry.file_type().is_file() {
                continue;
            }
            let p = entry.path();
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "tsx" | "ts" | "vue" | "svelte" | "astro") {
                continue;
            }
            let rel = p
                .strip_prefix(root)
                .unwrap_or(p)
                .to_string_lossy()
                .replace('\\', "/");
            let name_lc = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            if keywords.iter().any(|k| name_lc.contains(k))
                || matches!(
                    name_lc.as_str(),
                    "icons" | "button" | "card" | "confirm" | "preview" | "motion" | "reveal"
                )
            {
                if !out.contains(&rel) {
                    out.push(rel);
                }
            }
            if out.len() >= 6 {
                return out;
            }
        }
    }
    out
}

pub fn select_skills(lane: &str, stack: &str, genre: &str, seo: &str) -> Vec<String> {
    let mut s = vec![
        "fe-tonadus".to_string(),
        "nguyen-tac-tra-loi".to_string(),
    ];
    if lane == "FE-THUNDER" {
        return s;
    }
    s.push("design-taste-frontend".to_string());
    s.push("vicieo-signature-ui".to_string());
    s.push("multi-stack-fe-senior".to_string());
    if !genre.is_empty() {
        s.push(format!("web-genres-playbooks#{genre}"));
    }
    if seo.contains("seo") {
        s.push("fe-seo-geo-master".to_string());
    }
    if stack.contains("vue") || stack.contains("nuxt") {
        s.push("vue3-nuxt-official".to_string());
    }
    if lane == "FE-DEPLOY" {
        s.push("fe-multi-cloud-deploy".to_string());
    } else {
        s.push("fe-code-quality-gate".to_string());
        s.push("fe-qa-responsive-emu".to_string());
    }
    s
}

pub fn determine_agents(lane: &str, prompt_lc: &str) -> Vec<String> {
    let needs_scout = prompt_lc.contains("video")
        || prompt_lc.contains("tiktok")
        || prompt_lc.contains("reels")
        || prompt_lc.contains("shorts")
        || prompt_lc.contains("scout")
        || prompt_lc.contains("reverse")
        || prompt_lc.contains("detect")
        || prompt_lc.contains("revert")
        || prompt_lc.contains("bóc tách")
        || prompt_lc.contains("tham khảo web")
        || prompt_lc.contains("học hỏi");

    let is_takeover = prompt_lc.contains("tiếp quản")
        || prompt_lc.contains("take over")
        || prompt_lc.contains("làm chủ")
        || prompt_lc.contains("khám bệnh")
        || prompt_lc.contains("đọc hiểu dự án");

    let mut agents = match lane {
        "FE-DEPLOY" => vec![
            "fe_code_auditor".to_string(),
            "fe_devops_deployer".to_string(),
        ],
        "FE-AUDIT-QA" => {
            if prompt_lc.contains("ui") || prompt_lc.contains("slop") || prompt_lc.contains("thẩm mỹ") || prompt_lc.contains("design") {
                vec![
                    "fe_ui_ux_critic".to_string(),
                    "fe_code_auditor".to_string(),
                    "fe_qa_tester".to_string(),
                ]
            } else {
                vec![
                    "fe_code_auditor".to_string(),
                    "fe_qa_tester".to_string(),
                ]
            }
        }
        "FE-MASTER-BUILD" => {
            if is_takeover {
                vec![
                    "fe_senior_dev".to_string(),
                    "fe_code_auditor".to_string(),
                    "fe_ui_ux_critic".to_string(),
                    "fe_qa_tester".to_string(),
                ]
            } else {
                let needs_design = prompt_lc.contains("mới")
                    || prompt_lc.contains("tạo")
                    || prompt_lc.contains("dựng")
                    || prompt_lc.contains("thiết kế")
                    || prompt_lc.contains("redesign")
                    || prompt_lc.contains("landing")
                    || prompt_lc.contains("ui")
                    || prompt_lc.contains("giao diện");
                if needs_design {
                    vec![
                        "fe_ui_ux_critic".to_string(),
                        "fe_senior_dev".to_string(),
                        "fe_code_auditor".to_string(),
                        "fe_qa_tester".to_string(),
                    ]
                } else {
                    vec![
                        "fe_senior_dev".to_string(),
                        "fe_code_auditor".to_string(),
                        "fe_qa_tester".to_string(),
                    ]
                }
            }
        }
        _ => {
            // FE-THUNDER
            if prompt_lc.contains("sửa") || prompt_lc.contains("fix") || prompt_lc.contains("đổi") {
                vec![
                    "fe_senior_dev".to_string(),
                    "fe_code_auditor".to_string(),
                ]
            } else {
                vec!["fe_senior_dev".to_string()]
            }
        }
    };

    if needs_scout && !agents.contains(&"fe_design_scout".to_string()) {
        agents.insert(0, "fe_design_scout".to_string());
    }
    agents
}

pub fn find_surgical_targets(root: &Path, prompt_lc: &str) -> Vec<SurgicalTarget> {
    let mut targets = Vec::new();
    let search_roots = [
        root.join("tool/dashboard"),
        root.join("src"),
        root.join("components"),
        root.join("app"),
        root.join("pages"),
    ];

    let stop_words: std::collections::HashSet<&str> = [
        "cho", "tôi", "làm", "sửa", "trang", "thêm", "của", "vào", "được", "không", "một", "các",
        "này", "với", "nhé", "luôn", "giúp", "muốn", "và", "tra", "biến", "trình", "bản", "hãy", "xem",
        "the", "for", "with", "from", "that", "this", "please", "make", "fix", "update", "change"
    ].iter().cloned().collect();

    let keywords: Vec<String> = prompt_lc
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !stop_words.contains(*w))
        .take(10)
        .map(|s| s.to_string())
        .collect();

    if keywords.is_empty() {
        return targets;
    }

    for dir in &search_roots {
        if !dir.exists() {
            continue;
        }
        for entry in WalkDir::new(dir).max_depth(5).into_iter().flatten() {
            if !entry.file_type().is_file() {
                continue;
            }
            let p = entry.path();
            let p_str = p.to_string_lossy().replace('\\', "/");
            if p_str.contains("node_modules")
                || p_str.contains("/.next")
                || p_str.contains("/dist/")
                || p_str.contains("/build/")
            {
                continue;
            }
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "tsx" | "ts" | "jsx" | "js" | "vue" | "svelte" | "astro") {
                continue;
            }

            let rel = p.strip_prefix(root).unwrap_or(p).to_string_lossy().replace('\\', "/");
            let file_name = p.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

            for kw in &keywords {
                if file_name.contains(kw) {
                    targets.push(SurgicalTarget {
                        file: rel.clone(),
                        reason: format!("File name matches keyword '{kw}'"),
                        slice_cmd: format!("fe-core slice \"{rel}\" 1:60"),
                    });
                    break;
                }
            }

            if targets.len() >= 4 {
                return targets;
            }
        }
    }
    targets
}

pub fn determine_solution_archetype(genre: &str, prompt_lc: &str, _stack: &str) -> (String, Vec<String>) {
    if prompt_lc.contains("tiếp quản")
        || prompt_lc.contains("take over")
        || prompt_lc.contains("làm chủ")
        || prompt_lc.contains("khám bệnh")
        || prompt_lc.contains("đọc hiểu dự án")
    {
        return (
            "Elegans Takeover & Health Audit: Full-Spectrum Architecture Inspection, Component Size Audit (> 250 loc), Anti-Slop Check & Prioritized Action Plan.".to_string(),
            vec![
                "1. Quét toàn bộ kiến trúc & phân tích AST dependencies".to_string(),
                "2. Soi component phình to & code bừa bãi theo chuẩn Elegans Co-location".to_string(),
                "3. Lập Bản đồ Hành động Phẫu thuật (Top 3-5 việc cần làm & cải thiện ngay)".to_string(),
            ],
        );
    }

    let plan = vec![
        "1. Surgical target slice inspection (fe-core slice)".to_string(),
        "2. Implement changes with Zero-Slop ViciEO UI Craft & Two-Tier State".to_string(),
        "3. Gatekeeper verification (fe-core check-edit + qa-filter)".to_string(),
    ];

    if prompt_lc.contains("filter") || prompt_lc.contains("tìm kiếm") || prompt_lc.contains("sort") {
        (
            "Two-Tier State: URL searchParams (shareable) + nuqs / useSearchParams with React Suspense boundary".to_string(),
            plan,
        )
    } else if prompt_lc.contains("dialog") || prompt_lc.contains("modal") || prompt_lc.contains("popup") {
        (
            "Accessible Dialog: Radix/shadcn Dialog, focus trap, Escape hotkey, scroll locked body, animated via EnterOnce".to_string(),
            plan,
        )
    } else if prompt_lc.contains("card") || prompt_lc.contains("danh sách") {
        (
            "Framed-Sheet: bg-[#18181B], border-[#27272A], concentric radius (p-4 r-xl -> child r-lg), hover subtle border".to_string(),
            plan,
        )
    } else if prompt_lc.contains("expand") || prompt_lc.contains("accordion") || prompt_lc.contains("chi tiết") {
        (
            "CSS Grid Reveal: 0fr -> 1fr height transition (no JS DOM measurement, pure GPU compositor)".to_string(),
            plan,
        )
    } else if prompt_lc.contains("checkout") || prompt_lc.contains("thanh toán") || prompt_lc.contains("payos") || prompt_lc.contains("stripe") {
        (
            "Checkout Engine: PayOS / Stripe Server Action + Optimistic UI + Idempotency Key token".to_string(),
            plan,
        )
    } else {
        (
            format!("ViciEO Signature UI Craft for {genre}: Flat-at-Rest, <3% Brand Accent, Concentric Radius & Morphicons"),
            plan,
        )
    }
}

pub fn run_route(args: &[String]) -> i32 {
    let root = detect_workspace_root();
    let is_agy = args.iter().any(|a| a == "--agy");
    let mut prompt = String::new();
    let mut conv_id = String::new();

    if let Some(pos) = args.iter().position(|a| a == "--prompt") {
        if let Some(val) = args.get(pos + 1) {
            prompt = val.clone();
        }
    } else {
        let mut stdin_raw = String::new();
        let _ = io::stdin().read_to_string(&mut stdin_raw);
        if !stdin_raw.trim().is_empty() {
            if let Ok(v) = serde_json::from_str::<Value>(&stdin_raw) {
                prompt = v
                    .get("prompt")
                    .or_else(|| v.get("lastUserInput"))
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                conv_id = v
                    .get("conversationId")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
            } else {
                prompt = stdin_raw;
            }
        }
    }

    // Deduplicate for Antigravity PreInvocation hook
    if is_agy {
        let state_dir = root.join("ai/state");
        let _ = fs::create_dir_all(&state_dir);
        let state_file = state_dir.join("format-reminder-state.json");
        let mut hasher = Sha256::new();
        hasher.update(format!("{conv_id}|{prompt}").as_bytes());
        let key = format!("{:x}", hasher.finalize());

        if let Ok(raw) = fs::read_to_string(&state_file) {
            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                if v.get("key").and_then(|k| k.as_str()) == Some(&key) {
                    println!("{{}}");
                    return 0;
                }
            }
        }
        let _ = fs::write(
            &state_file,
            serde_json::to_string(&json!({ "key": key })).unwrap_or_default(),
        );
    }

    let prompt_lc = prompt.to_lowercase();
    let lane = classify_lane(&prompt_lc);
    let stack = detect_stack(&root, &prompt_lc);
    let (genre, seo) = classify_genre(&prompt_lc);
    let reuse = find_reusable_candidates(&root, &prompt_lc);
    let skills = select_skills(lane, &stack, genre, seo);
    let browser = crate::devops_sync::detect_preferred_browser();
    let agents = determine_agents(lane, &prompt_lc);
    let surgical_files = find_surgical_targets(&root, &prompt_lc);
    let (solution_archetype, execution_plan) = determine_solution_archetype(genre, &prompt_lc, &stack);

    let packet = RoutePacket {
        lane: lane.to_string(),
        stack,
        genre: genre.to_string(),
        seo: seo.to_string(),
        browser,
        pony: PonytailMatrix {
            design_pre: "ART-DISCIPLINED",
            dev_code: "SURGICAL-SENIOR",
            code_audit: "ULTRA-RUTHLESS",
            qa_test: "FULL-COVERAGE/0-BLOAT",
            ui_judge: "ZERO-SLOP-JUDGE",
            devops: "ZERO-CONFIG-SAFE",
        },
        agents,
        skills,
        surgical_files,
        solution_archetype,
        execution_plan,
        reuse,
    };

    let packet_json = serde_json::to_string(&packet).unwrap_or_else(|_| "{}".to_string());
    let primary_agent = packet.agents.first().map(|s| s.as_str()).unwrap_or("fe_senior_dev");
    let directive = if lane == "FE-THUNDER" {
        format!("@fe:ctx{packet_json}\n🏷️ [Elegans :: {primary_agent}]\n[FE-THUNDER] Mở đầu bằng `🏷️ [Elegans :: {primary_agent}]` kèm `> ⚡ [FE-THUNDER]`. Xưng hô `Ngài Vmax` / `con`. Trả lời thẳng trọng tâm, không gọi cổng nặng.")
    } else {
        let _ = Regex::new(r"https?://\S+");
        format!(
            "@fe:ctx{packet_json}\n🏷️ [Elegans :: {primary_agent}]\n[{lane}] Mở đầu bằng `🏷️ [Elegans :: {primary_agent}]` kèm `> 🎯 [{lane} · {}]`. Xưng hô `Ngài Vmax` / `con`. Tuân thủ Ma trận Ponytail 6 cấp, ViciEO Signature UI Craft, Chuẩn Elegans Co-located Component Architecture & truyền dẫn `@fe:packet` 0-token.",
            packet.stack
        )
    };

    if is_agy {
        let out = json!({
            "injectSteps": [
                {
                    "systemMessage": {
                        "systemMessage": directive
                    }
                }
            ]
        });
        println!("{}", out);
    } else {
        println!("{}", directive);
    }
    0
}
