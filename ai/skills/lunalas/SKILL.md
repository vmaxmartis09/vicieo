---
name: lunalas
description: Lunalas — prompt-routing agent that picks the cheapest model tier able to do each task (native / Haiku / Sonnet / Opus), delegates mechanical work to subagents, escalates only risky seams, and learns daily from its own routing log to sharpen its reflexes. Use when routing, model selection, token cost, or "tiết kiệm token" comes up, or when reviewing lunalas' daily report.
---

# Lunalas

Lunalas is the routing layer for this project. It runs on every prompt through a
`UserPromptSubmit` hook, which classifies the prompt and injects a one-line
directive. It does not call any API to classify, so classification costs 0 tokens.

## Tiers

| Tier | Model | Use for | Thinking |
|---|---|---|---|
| native | none | find, grep, list, format, git status | none |
| haiku | Haiku | lookups, comments, typos, renames, short explanations | minimal, ~1–2K output |
| sonnet | Sonnet | features, refactors, tests, multi-file edits with a clear pattern | medium, ~4–8K output |
| opus | Opus | root-cause debugging, architecture, security, trade-offs, 11+ files | high, ~16–32K thinking |

Default when there is no signal: sonnet. User override (`dùng opus`, `use haiku`)
always wins.

## Rules

1. Haiku or native for lookups. Delegate pure reads to `Agent` with `model="haiku"`.
2. Sonnet is the default for standard work. Run the gate (tests, lint, build) after edits.
3. Opus plans first, in a short plan. Mechanical steps go to Sonnet or Haiku subagents.
   Opus reviews only the high-blast seams, at most 2 per task.
4. Escalate one tier when the same step fails twice. Never drop a tier for security, data or production work.
5. Keep the reply short. Say what was skipped and what was not verified.

## Daily learning loop

Each prompt appends one line to `data/shared/token-router/log.ndjson`. The log stores
the tier, the reasons and the prompt length. It never stores prompt text.

Run `python3 ai/skills/lunalas/hooks/daily_review.py` once a day. It writes
`data/shared/token-router/daily/YYYY-MM-DD.md` with:
- tier distribution, and the share of work that stayed on native/haiku,
- the most frequent reasons, and reasons that never fire (candidates to delete),
- suggested threshold changes, based on escalation counts you record by hand.

The review only suggests changes. It never edits the classifier. A change is made
by a person, then checked with `python3 -m unittest discover -s ai/skills/lunalas/tests`.

## Files

- `hooks/classify_prompt.py` — UserPromptSubmit classifier and logger.
- `hooks/daily_review.py` — daily summary of the log.
- `hooks/task-model-guard.py` — PreToolUse hook on `Agent`, suggests a model when none is set (from claude-lean-skill).
- `tests/` — self-checks for classification and review output.
- `vendor/` — licences and source copies of the three upstream projects. See `THIRD_PARTY.md`.
