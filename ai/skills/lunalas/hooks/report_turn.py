#!/usr/bin/env python3
"""Stop hook: show one lunalas line at the end of each turn.

Reads the last routing row written by classify_prompt.py and prints it as a
systemMessage. systemMessage is shown to the user only, so it costs no model
tokens. Never blocks: any failure exits 0 silently.
"""

import json
import os
import sys


def last_row(path: str) -> dict | None:
    try:
        with open(path, encoding="utf-8") as f:
            lines = f.readlines()
    except OSError:
        return None
    for line in reversed(lines):
        line = line.strip()
        if line:
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                return None
    return None


def main() -> None:
    try:
        data = json.load(sys.stdin)
    except (json.JSONDecodeError, EOFError):
        data = {}
    root = os.environ.get("CLAUDE_PROJECT_DIR") or data.get("cwd") or os.getcwd()
    row = last_row(os.path.join(root, "data", "shared", "token-router", "log.ndjson"))
    if not row:
        return
    print(json.dumps({
        "systemMessage": f"🌾 lunalas | end of turn | last tier: {row.get('tier')} | prompt chars: {row.get('prompt_chars')}"
    }, ensure_ascii=False))


if __name__ == "__main__":
    main()
