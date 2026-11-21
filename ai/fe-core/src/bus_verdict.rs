use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct FeVerdict {
    pub id: String,
    pub ag: String,      // "senior_dev" | "qa_tester" | "code_auditor" | "ui_ux_critic" | "devops_deployer"
    pub st: String,      // "PASS" | "FAIL" | "BLOCKED" | "BRIEF_READY"
    #[serde(default)]
    pub p0: usize,
    #[serde(default)]
    pub p1: usize,
    #[serde(default)]
    pub fix: Vec<String>,
    #[serde(default)]
    pub not_tested: Vec<String>,
    #[serde(default)]
    pub evidence: Vec<String>,
}

pub fn run_bus(args: &[String]) -> i32 {
    let root = crate::router::detect_workspace_root();
    let bus_dir = root.join("ai/state/bus");
    let _ = fs::create_dir_all(&bus_dir);
    let mem_file = root.join("ai/state/memory.jsonl");

    let sub = args.first().map(|s| s.as_str()).unwrap_or("list");
    match sub {
        "put" => {
            if args.len() < 4 {
                eprintln!("Usage: fe-core bus put <task_id> <key> <json_or_val>");
                return 1;
            }
            let task_id = &args[1];
            let key = &args[2];
            let val_raw = &args[3];
            let file = bus_dir.join(format!("{task_id}.json"));
            let mut doc: Value = fs::read_to_string(&file)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(|| json!({}));
            let parsed_val: Value =
                serde_json::from_str(val_raw).unwrap_or_else(|_| Value::String(val_raw.clone()));
            doc[key] = parsed_val;
            doc["updated_at"] = Value::String(Utc::now().to_rfc3339());
            let _ = fs::write(&file, serde_json::to_string_pretty(&doc).unwrap_or_default());
            println!(
                "@fe:bus{{\"op\":\"put\",\"task\":\"{task_id}\",\"key\":\"{key}\",\"ref\":\"bus://{task_id}/{key}\"}}"
            );
            0
        }
        "get" => {
            if args.len() < 3 {
                eprintln!("Usage: fe-core bus get <task_id> <key>");
                return 1;
            }
            let task_id = &args[1];
            let key = &args[2];
            let file = bus_dir.join(format!("{task_id}.json"));
            if let Ok(raw) = fs::read_to_string(&file) {
                if let Ok(doc) = serde_json::from_str::<Value>(&raw) {
                    if let Some(v) = doc.get(key) {
                        println!("{}", v);
                        return 0;
                    }
                }
            }
            println!("null");
            0
        }
        "remember" => {
            let note = args.get(1).cloned().unwrap_or_default();
            let tag = args.get(2).cloned().unwrap_or_else(|| "general".to_string());
            let entry = json!({
                "at": Utc::now().to_rfc3339(),
                "tag": tag,
                "lesson": note
            });
            let mut line = serde_json::to_string(&entry).unwrap_or_default();
            line.push('\n');
            use std::io::Write;
            if let Ok(mut f) = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&mem_file)
            {
                let _ = f.write_all(line.as_bytes());
            }
            println!("@fe:mem{{\"op\":\"saved\",\"tag\":\"{tag}\"}}");
            0
        }
        "recall" => {
            let kw = args.get(1).map(|s| s.to_lowercase()).unwrap_or_default();
            let mut hits = Vec::new();
            if let Ok(raw) = fs::read_to_string(&mem_file) {
                for l in raw.lines().rev() {
                    if kw.is_empty() || l.to_lowercase().contains(&kw) {
                        if let Ok(v) = serde_json::from_str::<Value>(l) {
                            hits.push(v);
                        }
                    }
                    if hits.len() >= 5 {
                        break;
                    }
                }
            }
            println!("@fe:mem{}", json!({ "query": kw, "hits": hits }));
            0
        }
        _ => {
            let mut tasks = Vec::new();
            if let Ok(rd) = fs::read_dir(&bus_dir) {
                for e in rd.flatten() {
                    if let Some(name) = e.path().file_stem().and_then(|s| s.to_str()) {
                        tasks.push(name.to_string());
                    }
                }
            }
            println!("@fe:bus{}", json!({ "active_tasks": tasks }));
            0
        }
    }
}

pub fn run_qa_filter(args: &[String]) -> i32 {
    let raw = if let Some(f) = args.first() {
        fs::read_to_string(f).unwrap_or_default()
    } else {
        let mut s = String::new();
        let _ = io::stdin().read_to_string(&mut s);
        s
    };

    let mut console_errors = Vec::new();
    let mut net_errors = Vec::new();
    for line in raw.lines() {
        let lc = line.to_lowercase();
        if lc.contains("error") || lc.contains("uncaught") || lc.contains("hydration") {
            if console_errors.len() < 5 {
                console_errors.push(line.chars().take(120).collect::<String>());
            }
        }
        if lc.contains(" 404 ") || lc.contains(" 500 ") || lc.contains(" 502 ") || lc.contains("cors") {
            if net_errors.len() < 5 {
                net_errors.push(line.chars().take(120).collect::<String>());
            }
        }
    }

    let st = if console_errors.is_empty() && net_errors.is_empty() {
        "PASS"
    } else {
        "FAIL"
    };
    let out = json!({
        "st": st,
        "console_err_count": console_errors.len(),
        "net_err_count": net_errors.len(),
        "console_sample": console_errors,
        "net_sample": net_errors
    });
    println!("@fe:qa{}", out);
    if st == "PASS" { 0 } else { 1 }
}

fn is_valid_png(path: &Path) -> bool {
    if let Ok(bytes) = fs::read(path) {
        return bytes.len() > 8 && bytes[0..4] == [0x89, 0x50, 0x4E, 0x47];
    }
    false
}

pub fn run_verdict(args: &[String]) -> i32 {
    if args.iter().any(|a| a == "--schema") {
        println!(
            "@fe:verdict{{\"id\":\"T01\",\"ag\":\"senior_dev|qa_tester|code_auditor|ui_ux_critic|devops_deployer\",\"st\":\"PASS|FAIL|BLOCKED\",\"p0\":0,\"p1\":0,\"fix\":[],\"not_tested\":[],\"evidence\":[\"screenshots/desktop.png\"]}}"
        );
        return 0;
    }

    let file = match args.first().filter(|a| !a.starts_with("--")) {
        Some(f) => f,
        None => {
            eprintln!("Usage: fe-core verdict <report.md> [--require code_auditor,qa_tester,ui_ux_critic]");
            return 1;
        }
    };

    let raw = match fs::read_to_string(file) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("@fe:verdict_err{{\"msg\":\"Cannot read {file}: {e}\"}}");
            return 1;
        }
    };
    let base_dir = Path::new(file).parent().unwrap_or_else(|| Path::new("."));

    let mut verdicts: Vec<FeVerdict> = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(json_part) = trimmed.strip_prefix("@fe:verdict") {
            if let Ok(v) = serde_json::from_str::<FeVerdict>(json_part) {
                verdicts.push(v);
            }
        }
    }

    let req_str = args
        .iter()
        .position(|a| a == "--require")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("");
    let required: Vec<&str> = req_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut errors = Vec::new();
    for v in &verdicts {
        if v.st == "PASS" && (v.p0 > 0 || !v.fix.is_empty() || !v.not_tested.is_empty()) {
            errors.push(format!("{}: PASS cannot have p0/fix/not_tested", v.ag));
        }
        for ev in &v.evidence {
            let p = base_dir.join(ev);
            if !p.exists() && !Path::new(ev).exists() {
                errors.push(format!("{}: missing evidence file {}", v.ag, ev));
            } else {
                let real_p = if p.exists() { p } else { Path::new(ev).to_path_buf() };
                if real_p.extension().and_then(|e| e.to_str()) == Some("png") && !is_valid_png(&real_p) {
                    errors.push(format!("{}: corrupted PNG magic bytes in {}", v.ag, ev));
                }
            }
        }
    }

    for r in &required {
        match verdicts.iter().rev().find(|v| v.ag == *r) {
            Some(v) if v.st == "PASS" => {}
            Some(v) => errors.push(format!("required agent {r} verdict is {}", v.st)),
            None => errors.push(format!("missing verdict from required agent {r}")),
        }
    }

    if verdicts.is_empty() {
        errors.push("no @fe:verdict packet found".to_string());
    }

    let ok = errors.is_empty();
    println!(
        "@fe:gate{}",
        json!({
            "ok": ok,
            "verdicts_found": verdicts.len(),
            "required": required,
            "errors": errors
        })
    );
    if ok { 0 } else { 1 }
}
