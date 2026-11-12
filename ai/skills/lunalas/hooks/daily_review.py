#!/usr/bin/env python3
"""Daily review of lunalas routing log. Writes a markdown report, edits nothing else.

Usage: python3 daily_review.py [YYYY-MM-DD]   (default: today, UTC)
"""

import collections
import datetime
import json
import os
import sys

ROOT = os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd()
LOG = os.path.join(ROOT, "data", "shared", "token-router", "log.ndjson")
OUT_DIR = os.path.join(ROOT, "data", "shared", "token-router", "daily")


def load(day: str) -> list[dict]:
    rows = []
    if not os.path.exists(LOG):
        return rows
    with open(LOG, encoding="utf-8") as f:
        for line in f:
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            if str(row.get("ts", "")).startswith(day):
                rows.append(row)
    return rows


def render(day: str, rows: list[dict]) -> str:
    tiers = collections.Counter(r.get("tier", "?") for r in rows)
    total = sum(tiers.values())
    reasons = collections.Counter(
        reason for r in rows for reason in r.get("reasons", [])
    )
    lines = [f"# lunalas daily review — {day}", "", f"Prompts routed: {total}"]
    if total == 0:
        lines += ["", "No routed prompts for this day."]
        return "\n".join(lines) + "\n"

    lines += ["", "## Tier distribution", "", "| tier | count | share |", "|---|---:|---:|"]
    for tier in ["native", "haiku", "sonnet", "opus"]:
        n = tiers.get(tier, 0)
        lines.append(f"| {tier} | {n} | {n / total:.0%} |")
    cheap = (tiers.get("native", 0) + tiers.get("haiku", 0)) / total
    lines += ["", f"Cheap share (native + haiku): {cheap:.0%}."]

    lines += ["", "## Most frequent reasons", ""]
    lines += [f"- `{reason}` × {n}" for reason, n in reasons.most_common(10)] or ["- none"]

    lines += [
        "",
        "## Suggestions (manual review)",
        "",
        "- If opus is above 25%, check whether the opus signals are too broad.",
        "- If cheap share is below 40%, check whether haiku signals are too narrow.",
        "- Record each escalation (a task that needed a higher tier) in this file by hand. Escalations are the only ground truth.",
    ]
    return "\n".join(lines) + "\n"


def main() -> None:
    day = sys.argv[1] if len(sys.argv) > 1 else datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d")
    report = render(day, load(day))
    os.makedirs(OUT_DIR, exist_ok=True)
    path = os.path.join(OUT_DIR, f"{day}.md")
    with open(path, "w", encoding="utf-8") as f:
        f.write(report)
    print(path)


if __name__ == "__main__":
    main()
