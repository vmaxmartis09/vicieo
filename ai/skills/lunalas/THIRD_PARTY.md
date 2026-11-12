# Third-party sources

lunalas adapts ideas and one hook from three MIT-licensed projects. Licences are kept in `vendor/`.

| Project | Licence | What lunalas uses |
|---|---|---|
| [CodeShuX/tokenwise](https://github.com/CodeShuX/tokenwise) | MIT | Measurement-first routing idea and log design. The installer is **not** used: it writes to `~/.claude`, outside this project. |
| [civillizard/claude-lean-skill](https://github.com/civillizard/claude-lean-skill) | MIT | Three-gate routing idea (gate, approach, blast radius). `hooks/task-model-guard.py` is copied unchanged. |
| [MadAppGang/claude-code](https://github.com/MadAppGang/claude-code) (plugins/multimodel/skills/task-complexity-router) | MIT | Keyword tiers, file-count modifier and escalation rule. |

Not vendored: TokenWise's `ab`, `report`, `summary` and `undo` skills. Lean's full
`SKILL.md` is kept only as a reference copy and is not loaded.
