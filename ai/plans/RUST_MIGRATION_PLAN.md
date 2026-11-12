# Kế hoạch hoàn tất migrate socnhotv (Python) → vicieo (Rust)

> Nguồn Python: `/Volumes/Data/socnhotv/tool/src/socnhotv/` (~42k dòng, ~240 endpoint FastAPI)
> Đích Rust: `/Volumes/Data/vicieo/engine-rust/src/` (~5.8k dòng, ~80 route Axum, 1 file `server.rs` 2180 dòng)
> Ngày lập: 2026-11-12

---

## 0. Hiện trạng thực tế (đã kiểm code, không phải ước lượng)

Con số "Rust đạt ~45%" là **quá lạc quan**. Phần đã chạy thật chủ yếu là đọc/ghi file JSON và stream file. Mọi thứ "sinh nội dung" đều là giả lập:

| Khu vực | Python | Rust hiện tại | Trạng thái thật |
|---|---|---|---|
| Graph runner | `core/engine/graph_runner.py` 3991 dòng: topo-sort DAG, truyền input giữa node, mock-policy, spend guard, key vault, cost | `run_project_graph` (server.rs:1538) lặp từng node, gắn ảnh PNG đầu tiên trong `characters/soc_nho/` và MP4 đầu tiên trong `output/` làm "kết quả" | **Giả hoàn toàn** — không gọi provider, không có thứ tự phụ thuộc |
| Providers | google, fal, replicate, pollinations, cloudflare, local, mock + router + key_vault + spend_guard + text_llm | `ai.rs` 45 dòng, chỉ Gemini text, không dùng ở đâu trong server | **~5%** |
| Video / HyperFrames | `video/` (hyperframes_engine 807, templates, episode, short_form, craft, lipsync, verify, queue) — 18 endpoint | `/video/render` trả `queued`, `/video/status` luôn `completed`; `video.rs` fallback ffmpeg ra màn đen 5s | **Giả** |
| Captions / audio | caption_adapter, word captions, mastering | `caption.rs`, `audio_mastering.rs` có khung, chưa nối route | **Khung** |
| Voice | `voice_routes.py` + `voice_say.py` + `dialogue_voice.py` (omnivoice clone/TTS) | list voice + stream file thật; `/voice/say`, `/voice/clone` trả URL sample cứng | **Đọc thật, sinh giả** |
| Intelligence | 31 endpoint: products, koc-styles, blueprints, patterns, channels, strategy, ideas, chat, compliance, node execute | 3 endpoint `/api/intel/*` trả JSON cứng; compliance = 5 từ khoá | **Giả** |
| Sources | 15 endpoint: download (yt-dlp), scenes, localize, transcript, script | không có | **0%** |
| Social Hub | 83 endpoint: store, queue, scheduler, planner, automation, adapters (tiktok/youtube/manual), analytics/insights, vault, video_library | ~8 route đọc/ghi JSON; `social.rs` publish trả chuỗi `ok` | **~10%** |
| Batch / Editor | batch (5) + editor decisions (3) | không có | **0%** |
| Workspace / project / character / graph IO / SSE / file stream | data_routes (34) + app.py | có, chạy thật | **~85%** |

Dashboard đang gọi trực tiếp các nhóm: `/projects/:p/video/*`, `/video/templates*`, `/projects/:p/intel/*`, `/projects/:p/products/*`, `/data/voices/:p/samples`, `/data/projects/:p/characters/:p/images*` → đây là ưu tiên cao vì UI đang hỏng/giả ở đó. Social Hub gần như chưa có UI gọi → ưu tiên thấp hơn.

**Lưu trữ:** Python dùng JSON file trong `data/` (không SQLite) → Rust phải đọc/ghi **đúng cùng schema file** để dữ liệu cũ dùng tiếp được.

**Quyết định kiến trúc đề xuất:** phần ML nặng (omnivoice TTS/clone, lipsync, matting, whisper) **không port sang Rust** — giữ thành sidecar Python gọi qua subprocess/HTTP. Rust làm orchestration, I/O, queue, provider HTTP, ffmpeg/Node render.

---

## 1. Vì sao phải có Phase 0 trước khi chạy song song

`server.rs` là 1 file 2180 dòng chứa mọi handler + router. Nếu 5 agent cùng sửa → conflict liên tục. `Cargo.toml` cũng là điểm nóng. Ngoài ra các track phụ thuộc nhau qua 2 interface: **text LLM** (Intel, Social captions, Script dùng) và **NodeExecutor** (Video, Voice đăng ký executor vào graph runner).

→ Phase 0 (1 agent, tuần tự, ~nửa ngày) tách module + chốt interface + thêm deps. Sau đó các track chỉ chạm thư mục riêng.

### Ranh giới file sau Phase 0

```
engine-rust/src/
  main.rs, cli.rs, workspace.rs, schema.rs        ← Phase 0 sở hữu, track khác KHÔNG sửa
  state.rs                                        ← AppState (tx SSE, job registry)  [Phase 0]
  routes/mod.rs                                   ← merge router các track            [Phase 0, mỗi track chỉ thêm 1 dòng .merge()]
  routes/core.rs      (workspace/project/character/files/graph IO)   ← Track 0/F
  routes/graph.rs     (run/cancel/events/estimate/validate/runs)     ← Track A
  routes/video.rs     routes/editor.rs                               ← Track B
  routes/voice.rs                                                    ← Track C
  routes/intel.rs     routes/sources.rs  routes/products.rs          ← Track D
  routes/social.rs                                                   ← Track E
  routes/batch.rs                                                    ← Track A
  engine/executor.rs   (trait NodeExecutor + registry)               ← Phase 0 định nghĩa, Track A hiện thực runner
  engine/graph_runner.rs engine/cost.rs engine/validator.rs          ← Track A
  providers/mod.rs     (trait TextLlm/ImageGen/VideoGen + mock)      ← Phase 0 định nghĩa, Track A hiện thực
  providers/{google,fal,replicate,pollinations,cloudflare,key_vault,spend_guard,router}.rs ← Track A
  video/**   caption.rs   audio_mastering.rs                         ← Track B
  voice/**   sidecar/voice_sidecar.py                                ← Track C
  intelligence/**  sources/**                                        ← Track D
  social/**                                                          ← Track E
  tests/parity/**                                                    ← Track F
```

Quy tắc xung đột:
1. Mỗi track làm trong **git worktree + branch riêng** (`mig/a-graph`, `mig/b-video`…).
2. Chỉ sửa file trong ranh giới của mình. Muốn đổi interface Phase 0 → ghi vào `ai/plans/INTERFACE_REQUESTS.md`, không tự sửa.
3. Dependency mới cho Cargo.toml: Phase 0 thêm sẵn; nếu thiếu, thêm **ở cuối block `[dependencies]` trong 1 commit riêng** để merge dễ.
4. `routes/mod.rs`: mỗi track chỉ thêm đúng 1 dòng `.merge(xxx::router())` (Phase 0 đã đặt sẵn dòng comment chỗ).
5. Mỗi track giữ nguyên **path + JSON shape** của Python (dashboard không phải sửa). Nếu dashboard cần sửa, chỉ ghi chú, không sửa `tool/dashboard/` (Track F/người dùng xử lý).
6. Thứ tự merge: Phase 0 → A → (B, C, D, E theo thứ tự xong) → F chạy lại parity sau mỗi merge.

---

## 2. Lộ trình

| Phase | Việc | Chạy |
|---|---|---|
| 0 | Commit WIP hiện tại, tách `server.rs`, chốt interface, deps, khung parity test | 1 agent, tuần tự |
| 1 | Track A (graph+providers) ∥ B (video) ∥ C (voice) ∥ D (intel+sources) ∥ E (social) ∥ F (parity audit) | 6 agent song song (hoặc 3+3 nếu muốn tiết kiệm) |
| 2 | Tích hợp: nối executor B/C vào runner A, chạy E2E 1 tập phim thật từ graph `soc-nho-tv` | 1 agent |
| 3 | Tắt hẳn backend Python, xoá route legacy trùng, cập nhật `ai/map/TOOLS.md` | 1 agent |

Track F (audit) có thể chạy **ngay cả khi Phase 0 chưa xong** vì chỉ đọc.

---

## 3. Prompt cho từng phase/track

Copy nguyên khối vào session mới (mỗi session 1 worktree). Các prompt đã ghi rõ ranh giới file.

### PHASE 0 — Tách module & chốt interface (chạy trước, một mình)

```
Bối cảnh: /Volumes/Data/vicieo/engine-rust là bản Rust (Axum 0.8) migrate từ backend Python
/Volumes/Data/socnhotv/tool/src/socnhotv. Đọc ai/plans/RUST_MIGRATION_PLAN.md mục 1.

Nhiệm vụ (không thêm tính năng, chỉ tái cấu trúc, hành vi HTTP phải giữ nguyên 100%):
1. Commit riêng các thay đổi WIP đang có trong engine-rust/src/server.rs trước khi tách.
2. Tách engine-rust/src/server.rs thành: state.rs (AppState), routes/mod.rs (build_router merge
   các sub-router, kèm dòng comment chỗ cho từng track), routes/{core,graph,video,voice,intel,
   products,sources,social,batch,editor}.rs. Handler hiện có chuyển nguyên trạng sang file tương ứng.
   Giữ OpenAPI/utoipa hoạt động.
3. Tạo engine/executor.rs:
   - struct NodeCtx { project_id, run_id, node_id, params: Value, inputs: HashMap<String, Value>,
     tx: broadcast::Sender<String>, cancel: CancellationToken, mock: bool }
   - struct NodeOutput { preview_type, preview_url: Option<String>, summary: String,
     cost_usd: f64, outputs: Value, artifacts: Vec<PathBuf> }
   - #[async_trait] trait NodeExecutor: Send+Sync { fn node_types(&self)->&'static [&'static str];
     async fn execute(&self, ctx:&NodeCtx)->anyhow::Result<NodeOutput>; }
   - ExecutorRegistry (HashMap type→Arc<dyn NodeExecutor>) + register(); một FallbackExecutor
     tái hiện đúng logic preview giả hiện tại để không vỡ UI.
4. Tạo providers/mod.rs với trait TextLlm (generate(prompt, system, model, json_mode)),
   ImageGen, VideoGen, mỗi trait có impl Mock. Chuyển ai.rs vào providers/google.rs (chỉ text).
   Hàm tiện ích pub async fn llm() -> Arc<dyn TextLlm> chọn theo env, fallback Mock.
5. Cargo.toml thêm sẵn: async-trait, tokio-util (CancellationToken), tempfile, regex, walkdir,
   sha2, base64, once_cell. Không thêm gì khác.
6. Tạo tests/parity/: script (bash hoặc Rust integration test) đọc danh sách endpoint từ
   tests/parity/endpoints.yaml, gọi :9000, so shape JSON với fixture. Chỉ dựng khung + 5 endpoint mẫu.
7. cargo build && cargo clippy sạch; chạy server, kiểm /health, /projects/soc-nho-tv/graph,
   /node-types, /projects/soc-nho-tv/run vẫn trả như trước.
Commit trên branch mig/phase0. Không sửa tool/dashboard.
```

### TRACK A — Graph runner thật + Provider gateway

```
Đọc ai/plans/RUST_MIGRATION_PLAN.md. Làm trên worktree branch mig/a-graph (base: mig/phase0).
Chỉ được sửa: engine-rust/src/engine/{graph_runner,cost,validator}.rs, engine-rust/src/providers/**,
engine-rust/src/routes/{graph,batch}.rs, thêm 1 dòng trong routes/mod.rs. KHÔNG sửa executor.rs trait
(muốn đổi → ghi ai/plans/INTERFACE_REQUESTS.md).

Port từ Python:
- core/engine/graph_runner.py (3991 dòng), run_manager.py, validator.py, batch_runner.py
- core/providers/{router,key_vault,spend_guard,google_provider,fal_provider,replicate_provider,
  pollinations_provider,cloudflare_provider,text_llm,mock_provider}.py
- core/models_registry, core/nodes/registry.py (danh mục node + capabilities)

Yêu cầu:
1. Runner: topo-sort DAG theo edges, phát hiện cycle, truyền outputs node cha → inputs node con
   (kể cả $ref "script"), chạy song song các nhánh độc lập (giới hạn concurrency), targetNodeId
   = chạy node đó + tổ tiên stale, cancel thật qua CancellationToken, ghi generations/pinned/state
   vào graph.json đúng schema Python, run record vào runs/ giống run_manager.py, SSE event giống hệt
   payload Python (so sánh field với dashboard store/slices/canvas).
2. Mock policy theo node/kind (image/video/voice/text) như graph_runner.py:349.
3. Providers: router resolve_provider_for_model giống router.py; key vault đọc cùng file/ENV như
   key_vault.py (không log key, redact_secrets); spend guard giới hạn USD/run; image_gen + video_gen
   qua fal (queue + poll), google (gemini text, imagen, veo), replicate, pollinations; tải kết quả
   về data/projects/<id>/output/... và trả URL /data/projects/<id>/files/...
4. Executors đăng ký: script, image_gen, image_edit, character, product, cast, video_gen,
   quality_check, policy, text. (hyperframes_render/captions/export thuộc Track B,
   voice_tts/voice_clone thuộc Track C — chỉ để FallbackExecutor.)
5. /projects/:p/estimate và /graph/cost tính giá thật theo bảng giá model.
6. Port batch: estimate-episodes, run-episodes, cancel, batch-runs.
7. Test: unit test topo-sort/cycle/cost; integration test chạy graph soc-nho-tv ở mock=true;
   chạy 1 node image_gen thật với key fal nếu có (nếu không có key → báo rõ, không giả kết quả).
Báo cáo: bảng endpoint/node đã port, cái nào còn thiếu.
```

### TRACK B — Video / HyperFrames / Captions / Editor

```
Đọc ai/plans/RUST_MIGRATION_PLAN.md. Worktree branch mig/b-video (base: mig/phase0).
Chỉ sửa: engine-rust/src/video/**, caption.rs, audio_mastering.rs, routes/{video,editor}.rs,
1 dòng routes/mod.rs, và data/shared/templates/_engine/** nếu cần runner Node.

Port từ Python: video/{hyperframes_engine,queue,verify,platforms,fallback_ffmpeg}.py,
video/templates/*, video/episode/*, video/short_form/*, video/craft/*,
core/adapters/{hyperframes_adapter,caption_adapter}.py, core/engine/edit_decisions.py,
core/api/routes/{video,editor}.py.

Yêu cầu:
1. Đủ 18 endpoint video + 3 editor, giữ path/shape: /video/platforms, /video/templates[/:tid
   [/validate|/preview|/preview.mp4|/preview.jpg]], /projects/:pid/video/render[/:job|/cancel|/events],
   /video/renders, /projects/:pid/video/episodes/:eid/{plan,build,status}, /video/craft/{hooks,lint}.
2. Render job queue thật (tokio task, trạng thái lưu file như queue.py, progress SSE, cancel kill
   process con). Bỏ /video/status giả.
3. HyperFrames: gọi Node headless renderer (hyperframes CLI / _engine runner) với params JSON;
   ghép đa phân cảnh, karaoke word-captions (từ timestamps voice), nhạc nền + ducking, xuất 9:16
   và 16:9 theo platforms.py. Fallback ffmpeg phải ghép đúng ảnh/clip của scene, không phải màn đen.
4. verify_video (ffprobe: duration, resolution, audio stream, loudness) sau render.
5. Hiện thực NodeExecutor cho hyperframes_render, captions, word_captions, export, scene_cut,
   handdraw; đăng ký trong video/mod.rs::register(registry).
6. Test: render template mẫu trong data/shared/templates ra mp4 hợp lệ; ffprobe pass.
Không đụng providers/ hay graph_runner.rs.
```

### TRACK C — Voice (TTS/clone qua sidecar)

```
Đọc ai/plans/RUST_MIGRATION_PLAN.md. Worktree branch mig/c-voice (base: mig/phase0).
Chỉ sửa: engine-rust/src/voice/**, routes/voice.rs, 1 dòng routes/mod.rs, và thư mục mới
engine-rust/sidecar/voice/ (Python, dùng .venv gốc — KHÔNG tạo venv mới).

Port: core/api/voice_routes.py, routes/voice_say.py, core/engine/dialogue_voice.py,
core/adapters/voice_adapter.py, video/episode/voice.py.

Quyết định: không port model omnivoice sang Rust. Viết sidecar Python mỏng (CLI JSON in/out hoặc
HTTP 127.0.0.1 cổng nội bộ) bọc code TTS/clone hiện có; Rust quản lý tiến trình, timeout, cache.

Yêu cầu:
1. /voice/say, /projects/:p/voice/say, /voice/clone, /voices/clone, /projects/:p/voice/clone,
   /voice/speakers (đọc từ voice_vault, bỏ list cứng), /data/voices/:id/samples sinh audio thật,
   lưu vào voice_vault/project output, trả URL + duration thật + word timestamps (cho karaoke).
2. Dialogue voice: nhiều nhân vật, map character→voice như graph_runner.py:1351 (báo lỗi rõ nếu
   nhân vật không có giọng).
3. Cache theo hash(text, voice, params).
4. NodeExecutor voice_tts, voice_clone.
5. Test: tổng hợp 1 câu tiếng Việt với giọng soc-nho, ffprobe duration > 0.
```

### TRACK D — Intelligence + Products + Sources

```
Đọc ai/plans/RUST_MIGRATION_PLAN.md. Worktree branch mig/d-intel (base: mig/phase0).
Chỉ sửa: engine-rust/src/intelligence/**, engine-rust/src/sources/**, routes/{intel,products,
sources}.rs, 1 dòng routes/mod.rs. Dùng LLM qua providers::llm() (trait từ Phase 0), KHÔNG sửa providers/.

Port: intelligence/* (analyzer, channel_analyze, channel_ingest, chat_assistant, compliance, ideas,
koc_styles, patterns, products, review_flow, script_gen, strategy, tiktok_product, node_runner),
core/engine/{content_policy,episode_script,script_parser}.py, sources/downloader.py,
core/api/routes/{intel,sources}.py.

Yêu cầu:
1. Đủ 31 endpoint intel + 15 sources, giữ path/shape. Thay 3 route /api/intel/* giả bằng logic thật
   (hoặc alias sang route mới).
2. Products: CRUD, collect-url / import-url (scrape TikTok Shop/Shopee: reqwest + scraper; nếu cần
   headless thì gọi Node/ego-browser qua subprocess), koc-styles collect/analyze-url.
3. Sources: tải video bằng yt-dlp (subprocess), cắt cảnh (ffmpeg scene detect), transcript (whisper
   sidecar dùng chung pattern với Track C — chỉ gọi, không sửa voice/), localize, script.
4. Blueprint/patterns: phân tích cấu trúc video Douyin/YouTube từ transcript + scenes bằng LLM,
   lưu đúng file Python đang lưu.
5. content_policy đầy đủ (policy packs data/shared), thay compliance 5 từ khoá.
6. Ideas generate/promote/schedule; chat assistant; review-flow.
7. Test với LLM mock + 1 lần thật nếu có GEMINI_API_KEY.
```

### TRACK E — Social Hub

```
Đọc ai/plans/RUST_MIGRATION_PLAN.md. Worktree branch mig/e-social (base: mig/phase0).
Chỉ sửa: engine-rust/src/social/**, routes/social.rs, 1 dòng routes/mod.rs.

Port: social/* (store, models, service, queue, scheduler, planner, automation, captions,
platform_rules, vault, video_library, analytics/*, adapters/{tiktok,youtube,manual,...}),
core/api/routes/social.py (83 endpoint).

Yêu cầu:
1. Liệt kê 83 endpoint Python → nhóm: accounts, posts, queue/schedule, approval, planner, templates,
   captions, library, analytics/insights, automation. Port theo thứ tự đó, giữ path/shape.
   Gộp/alias các route /api/social/* và /projects/:p/social/* hiện có.
2. Store đọc/ghi cùng file JSON với Python (dùng migrate.py làm tham chiếu schema); ghi atomic
   (tmp + rename) và lock theo project.
3. Scheduler: tokio interval chạy trong engine, lấy post đến hạn → adapter publish; retry/backoff;
   trạng thái Draft→Scheduled→Publishing→Published/Failed.
4. Adapters: YouTube Data API v3 upload (OAuth token từ vault), TikTok Content Posting API; manual
   adapter. Không tự động đăng thật khi test — có flag dry_run mặc định true.
5. Captions/hashtags qua providers::llm(). Analytics: ingest + insights như insights.py.
6. Không lưu token ra log. Test với dry_run.
```

### TRACK F — Parity audit & kiểm thử (read-only, chạy được ngay)

```
Nhiệm vụ chỉ đọc + viết test, không sửa code engine.
1. Trích toàn bộ endpoint FastAPI ở /Volumes/Data/socnhotv/tool/src/socnhotv/core/api/** (method,
   path, request/response model) và toàn bộ route Axum ở /Volumes/Data/vicieo/engine-rust/src/**.
2. Trích mọi path dashboard gọi trong /Volumes/Data/vicieo/tool/dashboard (app, components, store,
   lib, hooks) và rewrites trong next.config.
3. Ghi ai/plans/PARITY.md: bảng mỗi endpoint → {python có, rust có, rust thật/giả/thiếu,
   dashboard có gọi, track phụ trách A–E}. Đánh dấu route dashboard gọi mà Rust chưa có (=lỗi UI).
   Kiểm cả route Next.js trong tool/dashboard/app/api có proxy sang Python cũ không.
4. Bổ sung tests/parity/endpoints.yaml đầy đủ (sau khi Phase 0 merge); với endpoint đọc dữ liệu, so
   JSON shape Python (nếu chạy được backend Python cổng cũ) với Rust :9000.
5. Liệt kê node type trong core/nodes/registry.py vs /node-types Rust và executor nào còn Fallback.
Sau mỗi lần một track merge, chạy lại và cập nhật PARITY.md.
```

### PHASE 2 — Tích hợp E2E (sau khi A, B, C merge)

```
Merge mig/a-graph, mig/b-video, mig/c-voice (đã merge vào main). Chạy graph data/projects/
soc-nho-tv/graph.json end-to-end: script → character/image_gen → voice_tts → captions →
hyperframes_render → export, mock=false cho text, mock tuỳ chọn cho image/video theo key có sẵn.
Sửa lỗi ghép nối giữa executors (định dạng outputs/inputs), xác nhận mp4 cuối qua ffprobe và
mở xem được trên dashboard :3600. Cập nhật PARITY.md, ai/map/TOOLS.md.
```

---

## 4. Checklist "xong" cho mỗi track
- [ ] `cargo build --release` + `cargo clippy -- -D warnings` sạch
- [ ] Không còn JSON cứng/giả trong route của track (grep "mock"/hardcode list)
- [ ] Path + shape khớp Python (parity test pass)
- [ ] Dashboard màn hình liên quan chạy được (kiểm tay trên :3600)
- [ ] Không log secret; không ghi file ngoài `data/`
- [ ] PARITY.md cập nhật
