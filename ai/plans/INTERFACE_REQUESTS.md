
## Track C (voice) — register executor
`engine/executor.rs` `ExecutorRegistry::with_defaults()` needs one line at the registration point:
`crate::routes::voice_impl::register(&mut r);`  (registers `voice_tts`, `voice_clone`).
The voice module is mounted via `#[path = "../voice/mod.rs"] pub mod voice_impl;` in routes/mod.rs (main.rs is Phase-0 owned).
