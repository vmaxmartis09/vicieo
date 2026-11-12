
## Track D (mig/d-intel) → Track A: `providers/google.rs`
- Default Gemini model `gemini-2.5-flash` returns 404 ("no longer available to new users"); the Python backend used `gemini-3.8-flash` (`WATCH_GEMINI_MODEL`), which the API no longer lists (503/unavailable); the alias `gemini-flash-latest` works. Please change the default and honour `GEMINI_MODEL`/`WATCH_GEMINI_MODEL`.
- `generate_text` swallows API errors and returns the literal `"No content returned"`; `TextLlm::generate` should return `Err` so callers can fall back. Track D currently treats any unparsable reply as failure and falls back to deterministic output.
- `GeminiText` ignores `system` and `json_mode`; Track D prepends the system text to the prompt and strips ``` fences itself. Map them to `systemInstruction` / `responseMimeType: application/json`.
- Track D passes the model explicitly (`intelligence::util::llm_model()`), so no change is required for Track D to work.
