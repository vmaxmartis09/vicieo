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
