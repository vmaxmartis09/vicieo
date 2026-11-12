# PARITY — Python (FastAPI) vs Rust (Axum) vs Dashboard

Sinh tự động bởi `tests/parity/tools/gen_parity.py` (chạy lại sau mỗi track merge). Track A–E là heuristic theo path — chỉnh tay nếu sai.

- Python: 234 endpoint · Rust: 275 · Dashboard path gọi: 106
- **Lỗi UI (dashboard gọi, Rust thiếu): 0**

## Dashboard → Rust :9000 rewrites (next.config.mjs)

`/projects/*`, `/data/*`, `/voices/*`, `/health`, `/node-types`, `/custom-node-types`, `/models`, `/providers`, `/graph/*`, `/video/*`, `/voice/*`

Không có rewrite: `/settings/*`, `/api/voice/preview` (Rust có route), `/core/*` (Next route = snapshot Vercel read-only, KHÔNG proxy Python), social client gọi cổng `:8103`.
Route Next.js `app/api/*` (toolbox, templates): xử lý cục bộ trong Next, không proxy sang Python (xem mục dưới).

## Lỗi UI — dashboard gọi nhưng Rust thiếu

| Method | Path | Python | Track | Gọi từ |
|---|---|---|---|---|

## Bảng đầy đủ

| Method | Path | Python | Rust | Rust thật/giả/thiếu | Dashboard | Track |
|---|---|---|---|---|---|---|
| GET | `/api/data/assets` |  | ✓ | thật |  | D |
| GET | `/api/data/characters` |  | ✓ | thật |  | B |
| POST | `/api/data/characters` |  | ✓ | thật |  | B |
| GET | `/api/data/library` |  | ✓ | thật |  | D |
| GET | `/api/data/products` |  | ✓ | thật |  | D |
| POST | `/api/data/products` |  | ✓ | thật |  | D |
| POST | `/api/intel/compliance` |  | ✓ | giả |  | E |
| POST | `/api/intel/ideas` |  | ✓ | giả |  | E |
| POST | `/api/intel/script` |  | ✓ | giả |  | E |
| GET | `/api/social/accounts` |  | ✓ | thật |  | C |
| POST | `/api/social/accounts` |  | ✓ | thật |  | C |
| GET | `/api/social/analytics` |  | ✓ | thật |  | C |
| GET | `/api/social/posts` |  | ✓ | thật |  | C |
| POST | `/api/social/posts` |  | ✓ | thật |  | C |
| GET | `/api/social/schedule` |  | ✓ | thật |  | C |
| GET | `/api/v1/info` |  | ✓ | thật |  | E |
| GET | `/api/v1/projects` |  | ✓ | thật |  | D |
| GET | `/api/voice/preview` |  | ✓ | thật | ✓ | B |
| GET | `/custom-node-types` | ✓ | ✓ | thật | ✓ | A |
| POST | `/custom-node-types` | ✓ | ✓ | thật | ✓ | A |
| DELETE | `/custom-node-types/{}` | ✓ | ✓ | thật | ✓ | A |
| GET | `/data/policy-packs` | ✓ | ✓ | giả | ✓ | D |
| GET | `/data/projects` | ✓ | ✓ | thật | ✓ | D |
| POST | `/data/projects` | ✓ | ✓ | thật | ✓ | D |
| PUT | `/data/projects/active` | ✓ |  | thiếu |  | D |
| GET | `/data/projects/{}` | ✓ | ✓ | thật | ✓ | D |
| PUT | `/data/projects/{}` | ✓ | ✓ | thật | ✓ | D |
| POST | `/data/projects/{}/ai/character` | ✓ | ✓ | giả | ✓ | D |
| POST | `/data/projects/{}/ai/script` | ✓ | ✓ | giả | ✓ | D |
| GET | `/data/projects/{}/artifacts/{}` |  | ✓ | thật |  | A |
| GET | `/data/projects/{}/channels` | ✓ | ✓ | thật | ✓ | D |
| PUT | `/data/projects/{}/channels` | ✓ | ✓ | thật | ✓ | D |
| GET | `/data/projects/{}/characters` | ✓ | ✓ | thật | ✓ | B |
| POST | `/data/projects/{}/characters` | ✓ | ✓ | thật | ✓ | B |
| DELETE | `/data/projects/{}/characters/{}` | ✓ | ✓ | thật | ✓ | B |
| PUT | `/data/projects/{}/characters/{}` | ✓ | ✓ | thật | ✓ | B |
| POST | `/data/projects/{}/characters/{}/images` | ✓ | ✓ | giả | ✓ | B |
| DELETE | `/data/projects/{}/characters/{}/images/{}` | ✓ | ✓ | giả | ✓ | B |
| POST | `/data/projects/{}/episodes` | ✓ | ✓ | thật | ✓ | D |
| GET | `/data/projects/{}/episodes/{}/shot-pack` | ✓ | ✓ | thật | ✓ | D |
| GET | `/data/projects/{}/files/{}` | ✓ | ✓ | thật | ✓ | D |
| GET | `/data/projects/{}/graph` |  | ✓ | thật |  | A |
| POST | `/data/projects/{}/graph` |  | ✓ | thật |  | A |
| GET | `/data/projects/{}/koc-styles` | ✓ | ✓ | giả | ✓ | D |
| GET | `/data/projects/{}/library` | ✓ | ✓ | thật | ✓ | D |
| POST | `/data/projects/{}/library/upload` | ✓ | ✓ | thật | ✓ | D |
| * | `/data/projects/{}/output/final_render.mp4` |  |  | thiếu | ✓ | D |
| GET | `/data/projects/{}/policy` | ✓ | ✓ | thật | ✓ | D |
| PUT | `/data/projects/{}/policy` | ✓ | ✓ | thật | ✓ | D |
| POST | `/data/projects/{}/policy/check` | ✓ | ✓ | thật |  | D |
| POST | `/data/projects/{}/policy/fetch-url` | ✓ |  | thiếu |  | D |
| POST | `/data/projects/{}/policy/import` | ✓ |  | thiếu |  | D |
| POST | `/data/projects/{}/policy/import-pack` | ✓ |  | thiếu |  | D |
| GET | `/data/projects/{}/products` |  | ✓ | giả |  | D |
| POST | `/data/projects/{}/products/{}/review-flow` | ✓ | ✓ | giả | ✓ | D |
| POST | `/data/projects/{}/script-data/compile-all` | ✓ |  | thiếu |  | D |
| GET | `/data/projects/{}/script-data/{}` | ✓ | ✓ | thật | ✓ | D |
| POST | `/data/projects/{}/scripts` | ✓ | ✓ | thật | ✓ | D |
| GET | `/data/voices/{}/sample` |  | ✓ | thật |  | B |
| GET | `/data/voices/{}/samples` |  | ✓ | thật | ✓ | B |
| POST | `/data/voices/{}/samples` | ✓ | ✓ | giả | ✓ | B |
| GET | `/data/voices/{}/{}` | ✓ | ✓ | thật |  | B |
| GET | `/graph` |  | ✓ | thật |  | A |
| POST | `/graph` |  | ✓ | thật |  | A |
| POST | `/graph/cancel` |  | ✓ | thật |  | A |
| POST | `/graph/cost` |  | ✓ | giả |  | A |
| GET | `/graph/events` |  | ✓ | thật |  | A |
| POST | `/graph/run` |  | ✓ | thật |  | A |
| POST | `/graph/validate` | ✓ | ✓ | thật | ✓ | A |
| GET | `/health` | ✓ | ✓ | thật | ✓ | E |
| GET | `/intel/_ping` | ✓ | ✓ | giả |  | E |
| GET | `/models` | ✓ | ✓ | giả |  | E |
| * | `/models{}` |  |  | thiếu | ✓ | E |
| GET | `/node-types` | ✓ | ✓ | giả | ✓ | A |
| GET | `/policy-packs` |  | ✓ | giả |  | D |
| GET | `/projects` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/active` |  | ✓ | thật | ✓ | D |
| POST | `/projects/active` |  | ✓ | thật | ✓ | D |
| PUT | `/projects/active` | ✓ | ✓ | thật | ✓ | D |
| * | `/projects/{}` |  |  | thiếu | ✓ | D |
| POST | `/projects/{}/ai/character` |  | ✓ | giả |  | D |
| POST | `/projects/{}/ai/script` | ✓ | ✓ | giả | ✓ | D |
| GET | `/projects/{}/artifacts/{}` | ✓ | ✓ | thật |  | A |
| GET | `/projects/{}/batch-runs` | ✓ | ✓ | thật |  | D |
| GET | `/projects/{}/batch-runs/{}` | ✓ | ✓ | thật |  | D |
| GET | `/projects/{}/characters` | ✓ |  | thiếu |  | B |
| POST | `/projects/{}/characters` | ✓ |  | thiếu |  | B |
| DELETE | `/projects/{}/characters/{}` | ✓ |  | thiếu |  | B |
| POST | `/projects/{}/chat` | ✓ | ✓ | giả | ✓ | D |
| GET | `/projects/{}/data-catalog` | ✓ |  | thiếu |  | D |
| GET | `/projects/{}/editor/{}` | ✓ | ✓ | thật |  | B |
| POST | `/projects/{}/editor/{}/decisions` | ✓ | ✓ | thật |  | B |
| POST | `/projects/{}/editor/{}/render` | ✓ | ✓ | thật |  | B |
| POST | `/projects/{}/estimate` | ✓ | ✓ | giả | ✓ | A |
| POST | `/projects/{}/estimate-episodes` | ✓ | ✓ | thật |  | A |
| GET | `/projects/{}/events` | ✓ | ✓ | thật | ✓ | A |
| * | `/projects/{}/files/${path.replace(/^\/+/,` |  |  | thiếu | ✓ | D |
| * | `/projects/{}/files/${rel.replace(/^\/+/,` |  |  | thiếu | ✓ | D |
| * | `/projects/{}/files/${uriOrUrl.replace(/^\/+/,` |  |  | thiếu | ✓ | D |
| GET | `/projects/{}/files/{}` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/graph` | ✓ | ✓ | thật | ✓ | A |
| POST | `/projects/{}/graph` | ✓ | ✓ | thật | ✓ | A |
| POST | `/projects/{}/graph/validate` | ✓ |  | thiếu |  | A |
| POST | `/projects/{}/intel/analyze/{}` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/intel/blueprints` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/intel/blueprints/{}` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/intel/channels` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/intel/channels` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/intel/channels/user-data` | ✓ | ✓ | thật |  | D |
| GET | `/projects/{}/intel/channels/{}` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/intel/chat` | ✓ | ✓ | thật |  | D |
| POST | `/projects/{}/intel/compliance/check` | ✓ | ✓ | thật |  | D |
| POST | `/projects/{}/intel/generate-script` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/intel/ideas` | ✓ | ✓ | thật |  | D |
| POST | `/projects/{}/intel/ideas/generate` | ✓ | ✓ | thật | ✓ | D |
| PATCH | `/projects/{}/intel/ideas/{}` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/intel/ideas/{}/promote` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/intel/ideas/{}/schedule` | ✓ | ✓ | thật | ✓ | D |
| * | `/projects/{}/intel/ideas{}` |  |  | thiếu | ✓ | D |
| POST | `/projects/{}/intel/nodes/execute` | ✓ | ✓ | thật |  | D |
| GET | `/projects/{}/intel/patterns` | ✓ | ✓ | thật |  | D |
| * | `/projects/{}/intel/patterns{}` |  |  | thiếu | ✓ | D |
| GET | `/projects/{}/intel/strategy` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/intel/strategy` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/koc-styles` |  | ✓ | giả |  | D |
| GET | `/projects/{}/library` |  | ✓ | thật |  | D |
| POST | `/projects/{}/library/upload` | ✓ |  | thiếu |  | D |
| GET | `/projects/{}/outputs` | ✓ | ✓ | giả | ✓ | A |
| GET | `/projects/{}/policy` |  | ✓ | thật |  | D |
| PUT | `/projects/{}/policy` |  | ✓ | thật |  | D |
| POST | `/projects/{}/policy/check` |  | ✓ | thật |  | D |
| POST | `/projects/{}/policy/fetch-url` |  | ✓ | thật |  | D |
| POST | `/projects/{}/policy/import` |  | ✓ | thật |  | D |
| POST | `/projects/{}/policy/import-pack` |  | ✓ | thật |  | D |
| GET | `/projects/{}/products` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/products` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/products/collect-url` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/products/import-url` | ✓ | ✓ | thật | ✓ | D |
| DELETE | `/projects/{}/products/{}` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/products/{}` | ✓ | ✓ | thật | ✓ | D |
| PUT | `/projects/{}/products/{}` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/products/{}/koc-styles` | ✓ | ✓ | giả |  | D |
| POST | `/projects/{}/products/{}/koc-styles/analyze-url` | ✓ | ✓ | thật |  | D |
| POST | `/projects/{}/products/{}/koc-styles/collect` | ✓ | ✓ | thật |  | D |
| GET | `/projects/{}/products/{}/koc-styles/{}` | ✓ | ✓ | thật |  | D |
| POST | `/projects/{}/products/{}/review-flow` |  | ✓ | giả |  | D |
| GET | `/projects/{}/publishing` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/publishing` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/publishing/schedule` |  | ✓ | thật | ✓ | D |
| POST | `/projects/{}/resolve-preview` | ✓ |  | thiếu |  | D |
| POST | `/projects/{}/run` | ✓ | ✓ | giả | ✓ | A |
| POST | `/projects/{}/run-episodes` | ✓ | ✓ | giả |  | A |
| POST | `/projects/{}/run-episodes/cancel` | ✓ | ✓ | thật |  | A |
| POST | `/projects/{}/run/cancel` | ✓ | ✓ | thật | ✓ | A |
| GET | `/projects/{}/runs` | ✓ | ✓ | giả | ✓ | A |
| GET | `/projects/{}/runs/active` | ✓ | ✓ | giả | ✓ | A |
| DELETE | `/projects/{}/runs/{}` | ✓ | ✓ | giả | ✓ | A |
| GET | `/projects/{}/runs/{}` | ✓ | ✓ | giả | ✓ | A |
| GET | `/projects/{}/script-data/{}` |  | ✓ | thật |  | D |
| GET | `/projects/{}/social/accounts` | ✓ | ✓ | giả | ✓ | C |
| POST | `/projects/{}/social/accounts` | ✓ | ✓ | thật | ✓ | C |
| GET | `/projects/{}/social/accounts/stats` | ✓ | ✓ | thật |  | C |
| DELETE | `/projects/{}/social/accounts/{}` | ✓ | ✓ | giả | ✓ | C |
| GET | `/projects/{}/social/accounts/{}` | ✓ | ✓ | thật | ✓ | C |
| PUT | `/projects/{}/social/accounts/{}` | ✓ | ✓ | thật | ✓ | C |
| GET | `/projects/{}/social/accounts/{}/connect` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/accounts/{}/refresh` | ✓ | ✓ | giả |  | C |
| POST | `/projects/{}/social/analytics/apply-golden-windows` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/analytics/collect` | ✓ | ✓ | giả |  | C |
| GET | `/projects/{}/social/analytics/creatives` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/analytics/generate-insights` | ✓ | ✓ | thật |  | C |
| GET | `/projects/{}/social/analytics/heatmap` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/analytics/import-csv` | ✓ | ✓ | thật |  | C |
| GET | `/projects/{}/social/analytics/learnings` | ✓ | ✓ | giả |  | C |
| POST | `/projects/{}/social/analytics/manual-metrics` | ✓ | ✓ | thật |  | C |
| GET | `/projects/{}/social/analytics/outliers` | ✓ | ✓ | thật |  | C |
| GET | `/projects/{}/social/analytics/overview` | ✓ | ✓ | thật |  | C |
| GET | `/projects/{}/social/audit` | ✓ | ✓ | giả |  | C |
| GET | `/projects/{}/social/automation/rules` | ✓ | ✓ | giả |  | C |
| POST | `/projects/{}/social/automation/rules` | ✓ | ✓ | thật |  | C |
| DELETE | `/projects/{}/social/automation/rules/{}` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/automation/rules/{}/toggle` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/automation/run` | ✓ | ✓ | thật |  | A |
| GET | `/projects/{}/social/calendar` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/caption-variants` | ✓ | ✓ | thật |  | B |
| POST | `/projects/{}/social/captions/generate` | ✓ | ✓ | thật |  | B |
| POST | `/projects/{}/social/ideas/{}/schedule` | ✓ | ✓ | thật |  | C |
| GET | `/projects/{}/social/library` | ✓ | ✓ | giả |  | C |
| POST | `/projects/{}/social/migrate` | ✓ | ✓ | thật |  | C |
| GET | `/projects/{}/social/notes` | ✓ | ✓ | giả |  | C |
| PUT | `/projects/{}/social/notes` | ✓ | ✓ | thật |  | C |
| DELETE | `/projects/{}/social/notifications` | ✓ | ✓ | giả |  | C |
| GET | `/projects/{}/social/notifications` | ✓ | ✓ | giả |  | C |
| POST | `/projects/{}/social/notifications/{}/read` | ✓ | ✓ | giả |  | C |
| GET | `/projects/{}/social/overview` | ✓ | ✓ | thật |  | C |
| GET | `/projects/{}/social/packages/{}/{}` | ✓ | ✓ | thật |  | C |
| GET | `/projects/{}/social/packages/{}/{}.zip` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/packages/{}/{}/generate` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/plans/commit` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/plans/preview` | ✓ | ✓ | thật |  | C |
| GET | `/projects/{}/social/posts` | ✓ | ✓ | giả |  | C |
| POST | `/projects/{}/social/posts` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/posts/draft` | ✓ | ✓ | thật |  | C |
| DELETE | `/projects/{}/social/posts/{}` | ✓ | ✓ | thật |  | C |
| GET | `/projects/{}/social/posts/{}` | ✓ | ✓ | thật |  | C |
| PATCH | `/projects/{}/social/posts/{}` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/posts/{}/approve` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/posts/{}/cancel` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/posts/{}/duplicate` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/posts/{}/publish-now` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/posts/{}/reject` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/posts/{}/schedule` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/posts/{}/submit` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/posts/{}/targets/{}/mark-published` | ✓ | ✓ | thật |  | C |
| PUT | `/projects/{}/social/posts/{}/targets/{}/time` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/posts/{}/targets/{}/validate` | ✓ | ✓ | thật |  | C |
| GET | `/projects/{}/social/queue` | ✓ | ✓ | thật | ✓ | C |
| GET | `/projects/{}/social/scheduler/status` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/scheduler/tick` |  | ✓ | thật |  | C |
| GET | `/projects/{}/social/slots` | ✓ | ✓ | giả |  | C |
| PUT | `/projects/{}/social/slots` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/suggest-time` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/targets/bulk` | ✓ | ✓ | thật |  | C |
| GET | `/projects/{}/social/templates` | ✓ | ✓ | giả | ✓ | C |
| POST | `/projects/{}/social/templates` | ✓ | ✓ | thật | ✓ | C |
| DELETE | `/projects/{}/social/templates/{}` | ✓ | ✓ | giả | ✓ | C |
| PUT | `/projects/{}/social/templates/{}` | ✓ | ✓ | thật | ✓ | C |
| GET | `/projects/{}/social/videos` | ✓ | ✓ | giả |  | B |
| GET | `/projects/{}/sources` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/sources` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/sources/jobs/{}` | ✓ | ✓ | thật |  | D |
| DELETE | `/projects/{}/sources/{}` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/sources/{}` | ✓ | ✓ | thật | ✓ | D |
| PATCH | `/projects/{}/sources/{}` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/sources/{}/file/{}` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/sources/{}/localize` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/sources/{}/localize/{}` | ✓ | ✓ | thật |  | D |
| POST | `/projects/{}/sources/{}/scenes` | ✓ | ✓ | giả | ✓ | D |
| GET | `/projects/{}/sources/{}/script` | ✓ | ✓ | thật | ✓ | D |
| PUT | `/projects/{}/sources/{}/script` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/sources/{}/transcribe` |  | ✓ | giả |  | D |
| GET | `/projects/{}/sources/{}/transcript` | ✓ | ✓ | thật | ✓ | D |
| PUT | `/projects/{}/sources/{}/transcript` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/spend` | ✓ |  | thiếu |  | A |
| POST | `/projects/{}/video/episodes/{}/build` | ✓ | ✓ | thật | ✓ | B |
| POST | `/projects/{}/video/episodes/{}/plan` | ✓ | ✓ | thật | ✓ | B |
| GET | `/projects/{}/video/episodes/{}/status` | ✓ | ✓ | thật | ✓ | B |
| POST | `/projects/{}/video/render` | ✓ | ✓ | thật | ✓ | B |
| GET | `/projects/{}/video/render/{}` | ✓ | ✓ | thật | ✓ | B |
| POST | `/projects/{}/video/render/{}/cancel` | ✓ | ✓ | giả | ✓ | B |
| GET | `/projects/{}/video/render/{}/events` | ✓ | ✓ | thật | ✓ | A |
| GET | `/projects/{}/video/renders` | ✓ | ✓ | giả |  | B |
| * | `/projects/{}/video/renders{}` |  |  | thiếu | ✓ | B |
| POST | `/projects/{}/voice/clone` | ✓ | ✓ | thật | ✓ | B |
| POST | `/projects/{}/voice/preview` | ✓ |  | thiếu |  | B |
| POST | `/projects/{}/voice/say` | ✓ | ✓ | thật | ✓ | B |
| * | `/projects/{}/{}` |  |  | thiếu | ✓ | D |
| GET | `/providers` | ✓ | ✓ | giả | ✓ | E |
| GET | `/settings/api-keys` | ✓ | ✓ | giả | ✓ | E |
| POST | `/settings/api-keys` | ✓ | ✓ | thật | ✓ | E |
| POST | `/settings/api-keys/verify` | ✓ | ✓ | thật | ✓ | E |
| GET | `/social/_ping` | ✓ | ✓ | giả |  | C |
| GET | `/social/accounts` |  | ✓ | thật |  | C |
| POST | `/social/accounts` |  | ✓ | thật |  | C |
| GET | `/social/accounts/stats` | ✓ | ✓ | thật |  | C |
| POST | `/social/captions/generate` | ✓ | ✓ | thật |  | B |
| GET | `/social/library` | ✓ | ✓ | thật |  | C |
| GET | `/social/oauth/callback/{}` | ✓ | ✓ | thật |  | C |
| GET | `/social/overview` | ✓ | ✓ | thật |  | C |
| POST | `/social/plans/commit` | ✓ | ✓ | thật |  | C |
| POST | `/social/plans/preview` | ✓ | ✓ | thật |  | C |
| GET | `/social/posts` |  | ✓ | thật |  | C |
| POST | `/social/posts` |  | ✓ | thật |  | C |
| GET | `/social/queue` | ✓ | ✓ | thật |  | C |
| POST | `/social/targets/bulk` | ✓ | ✓ | thật |  | C |
| GET | `/social/templates` | ✓ | ✓ | thật |  | C |
| POST | `/social/templates` | ✓ | ✓ | thật |  | C |
| DELETE | `/social/templates/{}` | ✓ | ✓ | thật |  | C |
| PUT | `/social/templates/{}` | ✓ | ✓ | thật |  | C |
| GET | `/social/videos` | ✓ | ✓ | thật |  | B |
| GET | `/sources/_ping` | ✓ | ✓ | giả |  | E |
| GET | `/video/_ping` | ✓ | ✓ | giả |  | B |
| POST | `/video/craft/hooks` | ✓ | ✓ | thật | ✓ | B |
| POST | `/video/craft/lint` | ✓ | ✓ | thật | ✓ | B |
| GET | `/video/platforms` | ✓ | ✓ | giả |  | B |
| POST | `/video/render` |  | ✓ | thật |  | B |
| GET | `/video/renders` |  | ✓ | thật |  | B |
| GET | `/video/templates` | ✓ | ✓ | giả | ✓ | B |
| GET | `/video/templates/{}` | ✓ | ✓ | giả | ✓ | B |
| POST | `/video/templates/{}/preview` | ✓ | ✓ | thật | ✓ | B |
| GET | `/video/templates/{}/preview.jpg` | ✓ | ✓ | thật | ✓ | B |
| GET | `/video/templates/{}/preview.mp4` | ✓ | ✓ | thật | ✓ | B |
| POST | `/video/templates/{}/validate` | ✓ | ✓ | giả | ✓ | B |
| * | `/video/templates{}` |  |  | thiếu | ✓ | B |
| POST | `/voice/clone` |  | ✓ | thật |  | B |
| POST | `/voice/say` |  | ✓ | thật |  | B |
| GET | `/voice/speakers` |  | ✓ | thật |  | B |
| GET | `/voices` | ✓ | ✓ | thật | ✓ | B |
| * | `/voices/${voiceId` |  |  | thiếu | ✓ | B |
| POST | `/voices/clone` | ✓ | ✓ | thật | ✓ | B |
| GET | `/voices/jobs/{}` | ✓ |  | thiếu |  | B |
| GET | `/voices/{}/sample` | ✓ | ✓ | thật | ✓ | B |
| GET | `/voices/{}/{}` |  | ✓ | thật |  | B |
| * | `/{}` |  |  | thiếu | ✓ | E |

## Node types (core/nodes vs Rust /node-types)

Python 32 · Rust node_types.json 0 · executor thật đăng ký: 18

| type | Python | Rust /node-types | Executor |
|---|---|---|---|
| bg_remove | ✓ | thiếu | Fallback |
| blueprint_analyze | ✓ | thiếu | Fallback |
| captions | ✓ | thiếu | thật |
| cast | ✓ | thiếu | thật |
| channel_insight | ✓ | thiếu | Fallback |
| character | ✓ | thiếu | thật |
| character_image | ✓ | thiếu | Fallback |
| depth_layers | ✓ | thiếu | Fallback |
| export | ✓ | thiếu | thật |
| handdraw | ✓ | thiếu | thật |
| hyperframes_render | ✓ | thiếu | thật |
| image | ✓ | thiếu | Fallback |
| image_edit | ✓ | thiếu | thật |
| image_gen | ✓ | thiếu | thật |
| localize | ✓ | thiếu | Fallback |
| platform_export | ✓ | thiếu | Fallback |
| policy | ✓ | thiếu | thật |
| product | ✓ | thiếu | thật |
| quality_check | ✓ | thiếu | thật |
| scene | ✓ | thiếu | Fallback |
| scene_cut | ✓ | thiếu | thật |
| script | ✓ | thiếu | thật |
| script_from_blueprint | ✓ | thiếu | Fallback |
| source_video | ✓ | thiếu | Fallback |
| template_render | ✓ | thiếu | Fallback |
| text | ✓ | thiếu | thật |
| transition | ✓ | thiếu | Fallback |
| video_gen | ✓ | thiếu | thật |
| vision_analyze | ✓ | thiếu | Fallback |
| voice_clone | ✓ | thiếu | thật |
| voice_tts | ✓ | thiếu | thật |
| word_captions | ✓ | thiếu | thật |
