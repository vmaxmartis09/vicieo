
## Track C (voice) — register executor
`engine/executor.rs` `ExecutorRegistry::with_defaults()` needs one line at the registration point:
`crate::routes::voice_impl::register(&mut r);`  (registers `voice_tts`, `voice_clone`).
The voice module is mounted via `#[path = "../voice/mod.rs"] pub mod voice_impl;` in routes/mod.rs (main.rs is Phase-0 owned).


## Track D (mig/d-intel) → Track A: `providers/google.rs`
- Default Gemini model `gemini-2.5-flash` returns 404 ("no longer available to new users"); the Python backend used `gemini-3.8-flash` (`WATCH_GEMINI_MODEL`), which the API no longer lists (503/unavailable); the alias `gemini-flash-latest` works. Please change the default and honour `GEMINI_MODEL`/`WATCH_GEMINI_MODEL`.
- `generate_text` swallows API errors and returns the literal `"No content returned"`; `TextLlm::generate` should return `Err` so callers can fall back. Track D currently treats any unparsable reply as failure and falls back to deterministic output.
- `GeminiText` ignores `system` and `json_mode`; Track D prepends the system text to the prompt and strips ``` fences itself. Map them to `systemInstruction` / `responseMimeType: application/json`.
- Track D passes the model explicitly (`intelligence::util::llm_model()`), so no change is required for Track D to work.

# Interface requests (Track A → Phase 0 owners)

1. `NodeCtx.params` semantics: Track A passes the RESOLVED input map (node params overlaid with wired port values), and
   `ctx.inputs` holds raw upstream outputs keyed by source node id. Tracks B/C should read wired ports (`voice_clips`,
   `scenes`, `captions`, ...) from `ctx.params`. Artifact values are `{kind, uri (project-relative), url, provider, model}`.
2. `engine/mod.rs` now declares `cost`, `graph_runner`, `validator` (needed to compile; not an executor.rs change).
3. Registration: Track A registers its executors in `graph_runner::registry_with_track_a()` (called per run) instead of editing
   `ExecutorRegistry::with_defaults()`. When B/C merge, add `crate::video::register(&mut r)` / `crate::voice::register(&mut r)` to
   `with_defaults()`; B/C must not register the Track A types (script, image_gen, image_edit, character, product, cast,
   video_gen, quality_check, policy, text).
4. `workspace::project_path(.., ensure=true)` only creates the PARENT of the last segment; Track A creates leaf directories itself.
