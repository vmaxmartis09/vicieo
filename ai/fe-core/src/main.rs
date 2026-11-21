mod ast_slice;
mod bus_verdict;
mod devops_sync;
mod router;
mod s3_guard;

use std::env;
use std::process;

fn print_help() {
    println!(
        "fe-core v1.0.0 — 100% Rust Native Internal Engine for the 5-Agent FE-Master Team (Zero Python)
Commands:
  route [--prompt <text>] [--agy]         0-Token Prompt Router & Ponytail Matrix Injector
  slice <file> <start:end|--symbol name>  Surgical Slice Reader (20-40 lines, 0 bloat)
  check-edit [--file <path>]              S3 Real-time PostToolUse Guard (.tsx/.vue/.svelte/.astro/.ts/.css)
  ast-blast [<dir>] [--target <name>]     Multi-threaded AST Dependency & Blast-Radius Scanner
  sloplint [<dir_or_file>]                Anti-AI-Slop & ViciEO Signature UI Auditor
  seo-check [<dir>]                       Technical SEO, JSON-LD, OG, Sitemap & llms.txt Auditor
  qa-filter [<log_file>]                  Zero-Token Browser/Console/Network Log Compressor
  env-guard [<dir>]                       Zero-Leak .env Parity & Secret Guard
  bus <put|get|remember|recall|list> ...  Zero-Token Shared Bus & Living Memory
  browser-bridge                          Detect Ego Lite (protect Dad) -> fallback AGY Browser
  verdict <report.md> [--require ...]     Enforce @fe:verdict JSON & PNG Magic-Byte Evidence
  sync [--check]                          Sync .claude <-> .agents <-> ai/skills (0 drift)
  update                                  Pull & Update latest Elegans swarm from GitHub"
    );
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    if argv.len() < 2 {
        print_help();
        process::exit(0);
    }

    let cmd = argv[1].as_str();
    let rest = &argv[2..];

    let code = match cmd {
        "route" => router::run_route(rest),
        "slice" => ast_slice::run_slice(rest),
        "check-edit" => s3_guard::run_check_edit(rest),
        "ast-blast" => ast_slice::run_ast_blast(rest),
        "sloplint" => s3_guard::run_sloplint(rest),
        "seo-check" => s3_guard::run_seo_check(rest),
        "qa-filter" => bus_verdict::run_qa_filter(rest),
        "env-guard" => devops_sync::run_env_guard(rest),
        "bus" => bus_verdict::run_bus(rest),
        "browser-bridge" => devops_sync::run_browser_bridge(),
        "verdict" => bus_verdict::run_verdict(rest),
        "sync" => devops_sync::run_sync(rest),
        "update" | "upgrade" | "pull" => devops_sync::run_update(),
        "--help" | "-h" | "help" => {
            print_help();
            0
        }
        _ => {
            eprintln!("Unknown subcommand: {cmd}");
            print_help();
            1
        }
    };

    process::exit(code);
}
