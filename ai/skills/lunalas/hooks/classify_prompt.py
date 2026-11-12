#!/usr/bin/env python3
"""UserPromptSubmit hook: pre-classify each prompt into a model tier.

Rule-based (no API call, no tokens spent). Prints a short routing directive
that Claude Code adds to the context of the turn, and appends one NDJSON
line to data/shared/token-router/log.ndjson (no prompt text is stored).
Never blocks: any failure exits 0 silently.

Tiers: native (no LLM) -> haiku -> sonnet -> opus.
Heuristics are adapted from madappgang task-complexity-router (keyword
tiers, file-count modifier) and claude-lean-skill (gates, plan nudge).
"""

import datetime
import json
import os
import re
import sys

TIERS = ["native", "haiku", "sonnet", "opus"]

# (regex, tier, weight). Vietnamese and English.
SIGNALS = [
    (r"\b(debug|gỡ lỗi|root cause|nguyên nhân|race condition|memory leak)\b", "opus", 3),
    (r"\b(architect|kiến trúc|design|thiết kế|trade.?off|security|bảo mật|audit|tối ưu|optimi[sz]e)\b", "opus", 3),
    (r"\b(critical|nghiêm trọng|production|production outage|mất dữ liệu|data corruption)\b", "opus", 2),
    (r"\b(implement|triển khai|refactor|tái cấu trúc|migrate|di chuyển|integrate|tích hợp|build|xây dựng)\b", "sonnet", 2),
    (r"\b(write tests?|viết test|add tests?|feature|tính năng)\b", "sonnet", 2),
    (r"\b(fix|sửa|bug|lỗi|update|cập nhật|rename|đổi tên|add|thêm)\b", "haiku", 1),
    (r"\b(comment|typo|docs?|readme|giải thích|explain|what does|là gì)\b", "haiku", 1),
    (r"\b(find|search|tìm|grep|list|liệt kê|show|hiển thị|format|git status|git log)\b", "native", 0),
]

MULTI_STEP = [
    r"\b(all|each|every|tất cả|mỗi|remaining|còn lại)\b.{0,30}\b(files?|endpoints?|components?|modules?|tests?|file|màn hình)\b",
    r"\b(step\s*\d|phase\s*\d|bước\s*\d|first.{0,20}then|trước.{0,20}sau)\b",
    r"\b(bulk|batch|across|throughout|xuyên suốt|hàng loạt)\b",
]

OVERRIDE = {
    "opus": r"\b(dùng|use|với|with)\s+opus\b",
    "sonnet": r"\b(dùng|use|với|with)\s+sonnet\b",
    "haiku": r"\b(dùng|use|với|with)\s+haiku\b",
}

GRAPH_PATHS = [
    "engine-rust/src/graphify-out/graph.json",
    "tool/dashboard/graphify-out/graph.json",
]

FILE_REF = re.compile(r"[\w./-]+\.(?:py|ts|tsx|js|jsx|rs|md|json|html|css)\b")

DIRECTIVE = {
    "native": "Use direct tools (Grep/Glob/Read/Bash). No subagent, no LLM reasoning needed.",
    "haiku": "Simple/linear. Answer or act directly; delegate pure lookups to an Agent with model=\"haiku\".",
    "sonnet": "Standard work. Stay on the current model; delegate reads/tests to model=\"haiku\" subagents. Run the gate (tests/lint) after edits.",
    "opus": "Hard reasoning. Plan first (short), then delegate mechanical steps to model=\"sonnet\"/\"haiku\" subagents. Escalate only the risky seams to Opus.",
}


def classify(prompt: str) -> tuple[str, list[str]]:
    text = prompt.lower()
    reasons: list[str] = []

    for tier, pattern in OVERRIDE.items():
        if re.search(pattern, text):
            return tier, [f"user override: {tier}"]

    scores = {t: 0 for t in TIERS}
    for pattern, tier, weight in SIGNALS:
        if re.search(pattern, text):
            scores[tier] += weight
            reasons.append(f"{tier}:{pattern[:24]}")

    level = 0
    # Long prompts are usually pasted logs/excerpts: keywords inside them must not force opus.
    if scores["opus"] >= 3 and len(prompt) <= 1500:
        level = 3
    elif scores["sonnet"] >= 2:
        level = 2
    elif scores["haiku"] >= 1:
        level = 1

    multi = sum(1 for p in MULTI_STEP if re.search(p, text))
    files = len(set(FILE_REF.findall(text)))
    if files >= 11:
        # File names in pasted logs/excerpts are not a task scope: raise to sonnet at most.
        level = max(level, 2)
        reasons.append("11+ files -> sonnet floor")
    elif len(prompt) > 800 and level < 3:
        level += 1
        reasons.append("long prompt +1")
    has_work = any(scores[t] for t in ("haiku", "sonnet", "opus"))
    if multi >= 1 and has_work and level < 2:  # pure lookups stay native
        level = 2
        reasons.append("multi-step -> sonnet")

    if level == 0 and not reasons:
        # No signal at all: default to sonnet (the router's safe default).
        return "sonnet", ["no signal, default sonnet"]
    return TIERS[level], reasons


def log(tier: str, reasons: list[str], prompt: str, root: str) -> None:
    try:
        folder = os.path.join(root, "data", "shared", "token-router")
        os.makedirs(folder, exist_ok=True)
        row = {
            "ts": datetime.datetime.now(datetime.timezone.utc).isoformat(),
            "tier": tier,
            "reasons": reasons[:6],
            "prompt_chars": len(prompt),
        }
        with open(os.path.join(folder, "log.ndjson"), "a", encoding="utf-8") as f:
            f.write(json.dumps(row, ensure_ascii=False) + "\n")
    except OSError:
        pass


def main() -> None:
    try:
        data = json.load(sys.stdin)
    except (json.JSONDecodeError, EOFError):
        return
    prompt = str(data.get("prompt", "")).strip()
    if not prompt or prompt.startswith("/"):
        return  # slash commands and empty input: leave alone

    tier, reasons = classify(prompt)
    root = os.environ.get("CLAUDE_PROJECT_DIR") or data.get("cwd") or os.getcwd()
    log(tier, reasons, prompt, root)

    # Display only: systemMessage is shown to the user and never added to model context,
    # so this line costs no model tokens.
    graphs = [p for p in GRAPH_PATHS if os.path.exists(os.path.join(root, p))]
    graph_status = "ready" if graphs else "not built"
    print(json.dumps({"systemMessage": f"🌾 lunalas | tier: {tier} | ponytail: on | graphify: {graph_status}"}, ensure_ascii=False))


if __name__ == "__main__":
    main()
