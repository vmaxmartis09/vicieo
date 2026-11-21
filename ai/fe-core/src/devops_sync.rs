use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

static RE_ENV_USE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:process\.env|import\.meta\.env)\.([A-Z][A-Z0-9_]{2,})\b").unwrap()
});
static RE_SECRET_VALUE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:sk-[a-zA-Z0-9]{20,}|ghp_[a-zA-Z0-9]{30,}|glpat-[a-zA-Z0-9\-_]{20,}|AIza[0-9A-Za-z\-_]{35}|xox[baprs]-[0-9a-zA-Z\-]{10,})").unwrap()
});

pub fn detect_preferred_browser() -> String {
    let candidates = [
        "/Applications/Ego Lite.app/Contents/MacOS/ego-browser",
        "/Applications/Ego.app/Contents/MacOS/ego-browser",
        "C:\\Users\\dtthoai\\.local\\share\\ego\\active_version_dir\\ego-browser.exe",
    ];
    for c in &candidates {
        if Path::new(c).exists() {
            return "ego-lite".to_string();
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let local_ego = format!("{home}/.local/share/ego/active_version_dir/ego-browser");
        if Path::new(&local_ego).exists() {
            return "ego-lite".to_string();
        }
    }
    if Command::new("which")
        .arg("ego-browser")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return "ego-lite".to_string();
    }
    "agy-builtin-browser".to_string()
}

pub fn run_browser_bridge() -> i32 {
    let engine = detect_preferred_browser();
    let out = json!({
        "engine": engine,
        "priority": ["1:ego-lite(protect-Dad-space)", "2:agy-builtin-browser(subagent:browser)", "3:playwright-mcp(140+devices)"],
        "protected_spaces": ["Dad"]
    });
    println!("@fe:browser{}", out);
    0
}

fn parse_env_keys(path: &Path) -> HashSet<String> {
    let mut keys = HashSet::new();
    if let Ok(raw) = fs::read_to_string(path) {
        for line in raw.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') {
                continue;
            }
            if let Some((k, _)) = t.split_once('=') {
                keys.insert(k.trim().to_string());
            }
        }
    }
    keys
}

pub fn run_env_guard(args: &[String]) -> i32 {
    let root = crate::router::detect_workspace_root();
    let target = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| root.clone());

    let mut used_keys: HashSet<String> = HashSet::new();
    let mut public_secret_leaks: Vec<String> = Vec::new();
    let mut hardcoded_secrets: Vec<String> = Vec::new();

    for entry in WalkDir::new(&target).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        let p_str = p.to_string_lossy().replace('\\', "/");
        if p_str.contains("node_modules")
            || p_str.contains("/.next")
            || p_str.contains("/target/")
            || p_str.contains("/.git/")
        {
            continue;
        }
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "ts" | "tsx" | "js" | "mjs" | "vue" | "svelte" | "astro") {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(p) {
            for cap in RE_ENV_USE.captures_iter(&raw) {
                if let Some(m) = cap.get(1) {
                    let key = m.as_str().to_string();
                    let is_pub = key.starts_with("NEXT_PUBLIC_")
                        || key.starts_with("VITE_")
                        || key.starts_with("NUXT_PUBLIC_");
                    let looks_secret = key.contains("SECRET")
                        || key.contains("PRIVATE")
                        || key.contains("SERVICE_ROLE")
                        || key.contains("PASSWORD");
                    if is_pub && looks_secret {
                        public_secret_leaks.push(format!("{p_str}:{key}"));
                    }
                    if key != "NODE_ENV" {
                        used_keys.insert(key);
                    }
                }
            }
            if RE_SECRET_VALUE.is_match(&raw) {
                hardcoded_secrets.push(p_str);
            }
        }
    }

    let env_local = parse_env_keys(&root.join(".env"))
        .into_iter()
        .chain(parse_env_keys(&root.join(".env.local")))
        .chain(parse_env_keys(&root.join("tool/dashboard/.env.local")))
        .collect::<HashSet<_>>();
    let env_example = parse_env_keys(&root.join(".env.example"))
        .into_iter()
        .chain(parse_env_keys(&root.join("tool/dashboard/.env.example")))
        .collect::<HashSet<_>>();

    let missing_in_example: Vec<String> = used_keys.difference(&env_example).cloned().collect();
    let ok = public_secret_leaks.is_empty() && hardcoded_secrets.is_empty();

    let out = json!({
        "ok": ok,
        "used_env_count": used_keys.len(),
        "defined_local_count": env_local.len(),
        "public_secret_leaks": public_secret_leaks,
        "hardcoded_secret_files": hardcoded_secrets,
        "undocumented_in_example": missing_in_example
    });
    println!("@fe:env{}", out);
    if ok { 0 } else { 1 }
}

pub fn run_sync(args: &[String]) -> i32 {
    let check_only = args.iter().any(|a| a == "--check");
    let root = crate::router::detect_workspace_root();

    let claude_agents = root.join(".claude/agents");
    let plugin_agents = root.join(".agents/plugins/fe-master-team/agents");
    let _ = fs::create_dir_all(&claude_agents);
    let _ = fs::create_dir_all(&plugin_agents);

    let mut drift = 0usize;
    let mut synced = 0usize;

    let agent_files = [
        "fe_senior_dev.md",
        "fe_qa_tester.md",
        "fe_code_auditor.md",
        "fe_ui_ux_critic.md",
        "fe_devops_deployer.md",
        "fe_design_scout.md",
    ];

    for name in &agent_files {
        let c_path = claude_agents.join(name);
        let p_path = plugin_agents.join(name);
        let c_raw = fs::read_to_string(&c_path).ok();
        let p_raw = fs::read_to_string(&p_path).ok();

        match (c_raw, p_raw) {
            (Some(c), Some(p)) if c == p => {}
            (Some(c), None) => {
                drift += 1;
                if !check_only {
                    let _ = fs::write(&p_path, c);
                    synced += 1;
                }
            }
            (None, Some(p)) => {
                drift += 1;
                if !check_only {
                    let _ = fs::write(&c_path, p);
                    synced += 1;
                }
            }
            (Some(c), Some(_)) => {
                drift += 1;
                if !check_only {
                    let _ = fs::write(&p_path, c);
                    synced += 1;
                }
            }
            (None, None) => {
                drift += 1;
            }
        }
    }

    // Sync core FE-Master skills between ai/skills and .claude/skills
    let core_skills = [
        "fe-tonadus",
        "nguyen-tac-tra-loi",
        "design-taste-frontend",
        "vicieo-signature-ui",
        "multi-stack-fe-senior",
        "web-genres-playbooks",
        "fe-seo-geo-master",
        "fe-qa-responsive-emu",
        "fe-code-quality-gate",
        "fe-multi-cloud-deploy",
    ];
    let ai_skills = root.join("ai/skills");
    let claude_skills = root.join(".claude/skills");
    for sk in &core_skills {
        let a_file = ai_skills.join(sk).join("SKILL.md");
        let c_file = claude_skills.join(sk).join("SKILL.md");
        let a_raw = fs::read_to_string(&a_file).ok();
        let c_raw = fs::read_to_string(&c_file).ok();
        match (a_raw, c_raw) {
            (Some(a), Some(c)) if a == c => {}
            (Some(a), _) => {
                drift += 1;
                if !check_only {
                    if let Some(parent) = c_file.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let _ = fs::write(&c_file, a);
                    synced += 1;
                }
            }
            (None, Some(c)) => {
                drift += 1;
                if !check_only {
                    if let Some(parent) = a_file.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let _ = fs::write(&a_file, c);
                    synced += 1;
                }
            }
            (None, None) => {
                drift += 1;
            }
        }
    }

    let ok = if check_only { drift == 0 } else { true };
    println!(
        "@fe:sync{}",
        json!({
            "mode": if check_only { "check" } else { "apply" },
            "ok": ok,
            "drift": drift,
            "synced": synced,
            "agents": agent_files.len(),
            "core_skills": core_skills.len()
        })
    );
    if check_only && drift > 0 { 1 } else { 0 }
}

pub fn run_update() -> i32 {
    let root = crate::router::detect_workspace_root();
    println!("🔄 Đang kéo bản cập nhật Elegans mới nhất từ GitHub (vmaxmartis09/elegans)...");

    let status = if cfg!(windows) {
        let ps_cmd = format!(
            "git clone --depth=1 https://github.com/vmaxmartis09/elegans.git \"$env:TEMP\\elegans-upd\"; & \"$env:TEMP\\elegans-upd\\install.ps1\" -TargetDir \"{}\"; Remove-Item -Recurse -Force \"$env:TEMP\\elegans-upd\"",
            root.display()
        );
        Command::new("powershell").args(["-Command", &ps_cmd]).status()
    } else {
        let sh_cmd = format!(
            "rm -rf /tmp/elegans-upd && git clone --depth=1 https://github.com/vmaxmartis09/elegans.git /tmp/elegans-upd && /tmp/elegans-upd/install.sh \"{}\" && rm -rf /tmp/elegans-upd",
            root.display()
        );
        Command::new("bash").args(["-c", &sh_cmd]).status()
    };

    match status {
        Ok(s) if s.success() => {
            println!("✨ @fe:update{{\"ok\":true,\"msg\":\"Elegans đã cập nhật thành công lên bản mới nhất!\"}}");
            0
        }
        _ => {
            eprintln!("❌ Lỗi khi cập nhật Elegans từ GitHub.");
            1
        }
    }
}
