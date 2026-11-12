# PARITY — Python (FastAPI) vs Rust (Axum) vs Dashboard

Sinh tự động bởi `tests/parity/tools/gen_parity.py` (chạy lại sau mỗi track merge). Track A–E là heuristic theo path — chỉnh tay nếu sai.

- Python: 234 endpoint · Rust: 101 · Dashboard path gọi: 106
- **Lỗi UI (dashboard gọi, Rust thiếu): 78**

## Dashboard → Rust :9000 rewrites (next.config.mjs)

`/projects/*`, `/data/*`, `/voices/*`, `/health`, `/node-types`, `/custom-node-types`, `/models`, `/providers`, `/graph/*`, `/video/*`, `/voice/*`

Không có rewrite: `/settings/*`, `/api/voice/preview` (Rust có route), `/core/*` (Next route = snapshot Vercel read-only, KHÔNG proxy Python), social client gọi cổng `:8103`.
Route Next.js `app/api/*` (toolbox, templates): xử lý cục bộ trong Next, không proxy sang Python (xem mục dưới).

## Lỗi UI — dashboard gọi nhưng Rust thiếu

| Method | Path | Python | Track | Gọi từ |
|---|---|---|---|---|
| POST | `/data/projects` | ✓ | D | lib/repositories/library.ts |
| PUT | `/data/projects/{}` | ✓ | D | lib/repositories/core.ts |
| POST | `/data/projects/{}/ai/character` | ✓ | D | lib/repositories/character-assets.ts |
| POST | `/data/projects/{}/characters/{}/images` | ✓ | B | lib/repositories/character-assets.ts |
| DELETE | `/data/projects/{}/characters/{}/images/{}` | ✓ | B | lib/repositories/character-assets.ts |
| GET | `/data/projects/{}/episodes/{}/shot-pack` | ✓ | D | components/canvas/node-body/components/ShotPackLink.tsx |
| * | `/data/projects/{}/output/final_render.mp4` | ✗ | D | components/studio/create-video-modal/api.ts |
| POST | `/data/voices/{}/samples` | ✓ | B | lib/repositories/character-assets.ts |
| * | `/models{}` | ✗ | E | lib/api/models.ts |
| * | `/projects/{}` | ✗ | D | components/canvas/node-body/components/EpisodeOutputPreview.tsx |
| * | `/projects/{}/files/${path.replace(/^\/+/,` | ✗ | D | components/project/ProductsManager.tsx |
| * | `/projects/{}/files/${rel.replace(/^\/+/,` | ✗ | D | components/canvas/ProductNodeCard.tsx |
| * | `/projects/{}/files/${uriOrUrl.replace(/^\/+/,` | ✗ | D | lib/api/nodes.ts |
| POST | `/projects/{}/intel/analyze/{}` | ✓ | D | lib/api/intel.ts |
| GET | `/projects/{}/intel/blueprints` | ✓ | D | lib/api/intel.ts |
| GET | `/projects/{}/intel/blueprints/{}` | ✓ | D | lib/api/intel.ts |
| GET | `/projects/{}/intel/channels` | ✓ | D | lib/api/intel.ts |
| POST | `/projects/{}/intel/channels` | ✓ | D | lib/api/intel.ts |
| GET | `/projects/{}/intel/channels/{}` | ✓ | D | lib/api/intel.ts |
| POST | `/projects/{}/intel/generate-script` | ✓ | D | lib/api/intel.ts |
| POST | `/projects/{}/intel/ideas/generate` | ✓ | D | lib/api/intel.ts |
| PATCH | `/projects/{}/intel/ideas/{}` | ✓ | D | lib/api/intel.ts |
| POST | `/projects/{}/intel/ideas/{}/promote` | ✓ | D | lib/api/intel.ts |
| POST | `/projects/{}/intel/ideas/{}/schedule` | ✓ | D | lib/api/intel.ts |
| * | `/projects/{}/intel/ideas{}` | ✗ | D | lib/api/intel.ts |
| * | `/projects/{}/intel/patterns{}` | ✗ | D | lib/api/intel.ts |
| GET | `/projects/{}/intel/strategy` | ✓ | D | lib/api/intel.ts |
| POST | `/projects/{}/intel/strategy` | ✓ | D | lib/api/intel.ts |
| POST | `/projects/{}/products` | ✓ | D | lib/api/intel.ts |
| POST | `/projects/{}/products/import-url` | ✓ | D | lib/api/intel.ts |
| DELETE | `/projects/{}/products/{}` | ✓ | D | lib/api/intel.ts |
| GET | `/projects/{}/products/{}` | ✓ | D | lib/api/intel.ts |
| PUT | `/projects/{}/products/{}` | ✓ | D | lib/api/intel.ts |
| POST | `/projects/{}/publishing` | ✓ | D | lib/repositories/core.ts |
| DELETE | `/projects/{}/runs/{}` | ✓ | A | lib/core/coreClient.ts |
| GET | `/projects/{}/runs/{}` | ✓ | A | lib/core/coreClient.ts |
| POST | `/projects/{}/social/accounts` | ✓ | C | components/social/hub/Accounts/AccountsView.tsx |
| DELETE | `/projects/{}/social/accounts/{}` | ✓ | C | components/social/hub/Accounts/AccountsView.tsx |
| GET | `/projects/{}/social/accounts/{}` | ✓ | C | components/social/hub/Accounts/AccountsView.tsx |
| PUT | `/projects/{}/social/accounts/{}` | ✓ | C | components/social/hub/Accounts/AccountsView.tsx |
| GET | `/projects/{}/social/queue` | ✓ | C | components/social/hub/Calendar/CalendarView.tsx |
| POST | `/projects/{}/social/templates` | ✓ | C | components/social/hub/Templates/TemplatesView.tsx |
| DELETE | `/projects/{}/social/templates/{}` | ✓ | C | components/social/hub/Templates/TemplatesView.tsx |
| PUT | `/projects/{}/social/templates/{}` | ✓ | C | components/social/hub/Templates/TemplatesView.tsx |
| GET | `/projects/{}/sources` | ✓ | D | lib/api/sources.ts |
| POST | `/projects/{}/sources` | ✓ | D | lib/api/sources.ts |
| DELETE | `/projects/{}/sources/{}` | ✓ | D | lib/api/sources.ts |
| GET | `/projects/{}/sources/{}` | ✓ | D | lib/api/sources.ts |
| PATCH | `/projects/{}/sources/{}` | ✓ | D | lib/api/sources.ts |
| GET | `/projects/{}/sources/{}/file/{}` | ✓ | D | lib/api/sources.ts |
| POST | `/projects/{}/sources/{}/localize` | ✓ | D | lib/api/sources.ts |
| POST | `/projects/{}/sources/{}/scenes` | ✓ | D | lib/api/sources.ts |
| GET | `/projects/{}/sources/{}/script` | ✓ | D | lib/api/sources.ts |
| PUT | `/projects/{}/sources/{}/script` | ✓ | D | lib/api/sources.ts |
| GET | `/projects/{}/sources/{}/transcript` | ✓ | D | lib/api/sources.ts |
| PUT | `/projects/{}/sources/{}/transcript` | ✓ | D | lib/api/sources.ts |
| POST | `/projects/{}/video/episodes/{}/build` | ✓ | B | lib/api/video.ts |
| POST | `/projects/{}/video/episodes/{}/plan` | ✓ | B | lib/api/video.ts |
| GET | `/projects/{}/video/episodes/{}/status` | ✓ | B | lib/api/video.ts |
| POST | `/projects/{}/video/render` | ✓ | B | lib/api/video.ts |
| GET | `/projects/{}/video/render/{}` | ✓ | B | lib/api/video.ts |
| POST | `/projects/{}/video/render/{}/cancel` | ✓ | B | lib/api/video.ts |
| GET | `/projects/{}/video/render/{}/events` | ✓ | A | lib/api/video.ts |
| * | `/projects/{}/video/renders{}` | ✗ | B | lib/api/video.ts |
| * | `/projects/{}/{}` | ✗ | D | components/studio/tabs/jobs-cost-tab/components/JobQueueCard.tsx, components/stu |
| GET | `/settings/api-keys` | ✓ | E | app/[locale]/settings/page.tsx, components/navigation/user-bubble/hooks/useUserB |
| POST | `/settings/api-keys` | ✓ | E | app/[locale]/settings/page.tsx, components/navigation/user-bubble/hooks/useUserB |
| POST | `/settings/api-keys/verify` | ✓ | E | app/[locale]/settings/page.tsx |
| POST | `/video/craft/hooks` | ✓ | B | components/studio/create-video-modal/api.ts |
| POST | `/video/craft/lint` | ✓ | B | components/studio/create-video-modal/api.ts |
| GET | `/video/templates/{}` | ✓ | B | lib/api/video.ts |
| POST | `/video/templates/{}/preview` | ✓ | B | lib/api/video.ts |
| GET | `/video/templates/{}/preview.jpg` | ✓ | B | components/studio/create-video-modal/steps/Step1TemplatePicker.tsx, components/t |
| GET | `/video/templates/{}/preview.mp4` | ✓ | B | components/studio/create-video-modal/steps/Step1TemplatePicker.tsx, components/t |
| POST | `/video/templates/{}/validate` | ✓ | B | lib/api/video.ts |
| * | `/video/templates{}` | ✗ | B | lib/api/video.ts |
| * | `/voices/${voiceId` | ✗ | B | components/canvas/node-body/components/VoiceHoverSpeaker.tsx |
| * | `/{}` | ✗ | E | components/canvas/CharacterHero.tsx, components/canvas/ProductNodeCard.tsx, comp |

## Bảng đầy đủ

| Method | Path | Python | Rust | Rust thật/giả/thiếu | Dashboard | Track |
|---|---|---|---|---|---|---|
| GET | `/api/data/assets` |  | ✓ | thật |  | D |
| GET | `/api/data/characters` |  | ✓ | thật |  | B |
| POST | `/api/data/characters` |  | ✓ | thật |  | B |
| GET | `/api/data/library` |  | ✓ | thật |  | D |
| GET | `/api/data/products` |  | ✓ | thật |  | D |
| POST | `/api/data/products` |  | ✓ | thật |  | D |
| POST | `/api/intel/compliance` |  | ✓ | thật |  | E |
| POST | `/api/intel/ideas` |  | ✓ | thật |  | E |
| POST | `/api/intel/script` |  | ✓ | thật |  | E |
| GET | `/api/social/accounts` |  | ✓ | thật |  | C |
| POST | `/api/social/accounts` |  | ✓ | thật |  | C |
| GET | `/api/social/analytics` |  | ✓ | thật |  | C |
| GET | `/api/social/posts` |  | ✓ | thật |  | C |
| POST | `/api/social/posts` |  | ✓ | thật |  | C |
| GET | `/api/social/schedule` |  | ✓ | thật |  | C |
| POST | `/api/social/schedule` |  | ✓ | thật |  | C |
| GET | `/api/v1/info` |  | ✓ | thật |  | E |
| GET | `/api/v1/projects` |  | ✓ | thật |  | D |
| GET | `/api/voice/preview` |  | ✓ | thật | ✓ | B |
| GET | `/custom-node-types` | ✓ | ✓ | thật | ✓ | A |
| POST | `/custom-node-types` | ✓ | ✓ | thật | ✓ | A |
| DELETE | `/custom-node-types/{}` | ✓ | ✓ | thật | ✓ | A |
| GET | `/data/policy-packs` | ✓ | ✓ | thật | ✓ | D |
| GET | `/data/projects` | ✓ | ✓ | thật | ✓ | D |
| POST | `/data/projects` | ✓ |  | thiếu | ✓ | D |
| PUT | `/data/projects/active` | ✓ |  | thiếu |  | D |
| GET | `/data/projects/{}` | ✓ | ✓ | thật | ✓ | D |
| PUT | `/data/projects/{}` | ✓ |  | thiếu | ✓ | D |
| POST | `/data/projects/{}/ai/character` | ✓ |  | thiếu | ✓ | D |
| POST | `/data/projects/{}/ai/script` | ✓ | ✓ | thật | ✓ | D |
| GET | `/data/projects/{}/artifacts/{}` |  | ✓ | thật |  | A |
| GET | `/data/projects/{}/channels` | ✓ | ✓ | thật | ✓ | D |
| PUT | `/data/projects/{}/channels` | ✓ | ✓ | thật | ✓ | D |
| GET | `/data/projects/{}/characters` | ✓ | ✓ | thật | ✓ | B |
| POST | `/data/projects/{}/characters` | ✓ | ✓ | thật | ✓ | B |
| DELETE | `/data/projects/{}/characters/{}` | ✓ | ✓ | thật | ✓ | B |
| PUT | `/data/projects/{}/characters/{}` | ✓ | ✓ | thật | ✓ | B |
| POST | `/data/projects/{}/characters/{}/images` | ✓ |  | thiếu | ✓ | B |
| DELETE | `/data/projects/{}/characters/{}/images/{}` | ✓ |  | thiếu | ✓ | B |
| POST | `/data/projects/{}/episodes` | ✓ | ✓ | thật | ✓ | D |
| GET | `/data/projects/{}/episodes/{}/shot-pack` | ✓ |  | thiếu | ✓ | D |
| GET | `/data/projects/{}/files/{}` | ✓ | ✓ | thật | ✓ | D |
| GET | `/data/projects/{}/graph` |  | ✓ | giả |  | A |
| POST | `/data/projects/{}/graph` |  | ✓ | giả |  | A |
| GET | `/data/projects/{}/koc-styles` | ✓ | ✓ | thật | ✓ | D |
| GET | `/data/projects/{}/library` | ✓ | ✓ | thật | ✓ | D |
| POST | `/data/projects/{}/library/upload` | ✓ | ✓ | thật | ✓ | D |
| * | `/data/projects/{}/output/final_render.mp4` |  |  | thiếu | ✓ | D |
| GET | `/data/projects/{}/policy` | ✓ | ✓ | thật | ✓ | D |
| PUT | `/data/projects/{}/policy` | ✓ | ✓ | thật | ✓ | D |
| POST | `/data/projects/{}/policy/check` | ✓ | ✓ | thật |  | D |
| POST | `/data/projects/{}/policy/fetch-url` | ✓ |  | thiếu |  | D |
| POST | `/data/projects/{}/policy/import` | ✓ |  | thiếu |  | D |
| POST | `/data/projects/{}/policy/import-pack` | ✓ |  | thiếu |  | D |
| GET | `/data/projects/{}/products` |  | ✓ | thật |  | D |
| POST | `/data/projects/{}/products/{}/review-flow` | ✓ | ✓ | thật | ✓ | D |
| POST | `/data/projects/{}/script-data/compile-all` | ✓ |  | thiếu |  | D |
| GET | `/data/projects/{}/script-data/{}` | ✓ | ✓ | thật | ✓ | D |
| POST | `/data/projects/{}/scripts` | ✓ | ✓ | thật | ✓ | D |
| GET | `/data/voices/{}/sample` |  | ✓ | thật |  | B |
| POST | `/data/voices/{}/samples` | ✓ |  | thiếu | ✓ | B |
| GET | `/data/voices/{}/{}` | ✓ | ✓ | thật |  | B |
| GET | `/graph` |  | ✓ | giả |  | A |
| POST | `/graph` |  | ✓ | giả |  | A |
| POST | `/graph/cancel` |  | ✓ | giả |  | A |
| POST | `/graph/cost` |  | ✓ | giả |  | A |
| GET | `/graph/events` |  | ✓ | giả |  | A |
| POST | `/graph/run` |  | ✓ | giả |  | A |
| POST | `/graph/validate` | ✓ | ✓ | giả | ✓ | A |
| GET | `/health` | ✓ | ✓ | thật | ✓ | E |
| GET | `/intel/_ping` | ✓ |  | thiếu |  | E |
| GET | `/models` | ✓ | ✓ | giả |  | E |
| * | `/models{}` |  |  | thiếu | ✓ | E |
| GET | `/node-types` | ✓ | ✓ | giả | ✓ | A |
| GET | `/projects` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/active` |  | ✓ | thật | ✓ | D |
| POST | `/projects/active` |  | ✓ | thật | ✓ | D |
| PUT | `/projects/active` | ✓ | ✓ | thật | ✓ | D |
| * | `/projects/{}` |  |  | thiếu | ✓ | D |
| POST | `/projects/{}/ai/character` |  | ✓ | thật |  | D |
| POST | `/projects/{}/ai/script` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/artifacts/{}` | ✓ | ✓ | thật |  | A |
| GET | `/projects/{}/batch-runs` | ✓ |  | thiếu |  | D |
| GET | `/projects/{}/batch-runs/{}` | ✓ |  | thiếu |  | D |
| GET | `/projects/{}/characters` | ✓ |  | thiếu |  | B |
| POST | `/projects/{}/characters` | ✓ |  | thiếu |  | B |
| DELETE | `/projects/{}/characters/{}` | ✓ |  | thiếu |  | B |
| POST | `/projects/{}/chat` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/data-catalog` | ✓ |  | thiếu |  | D |
| GET | `/projects/{}/editor/{}` | ✓ |  | thiếu |  | B |
| POST | `/projects/{}/editor/{}/decisions` | ✓ |  | thiếu |  | B |
| POST | `/projects/{}/editor/{}/render` | ✓ |  | thiếu |  | B |
| POST | `/projects/{}/estimate` | ✓ | ✓ | giả | ✓ | A |
| POST | `/projects/{}/estimate-episodes` | ✓ |  | thiếu |  | A |
| GET | `/projects/{}/events` | ✓ | ✓ | giả | ✓ | A |
| * | `/projects/{}/files/${path.replace(/^\/+/,` |  |  | thiếu | ✓ | D |
| * | `/projects/{}/files/${rel.replace(/^\/+/,` |  |  | thiếu | ✓ | D |
| * | `/projects/{}/files/${uriOrUrl.replace(/^\/+/,` |  |  | thiếu | ✓ | D |
| GET | `/projects/{}/files/{}` | ✓ | ✓ | thật | ✓ | D |
| GET | `/projects/{}/graph` | ✓ | ✓ | giả | ✓ | A |
| POST | `/projects/{}/graph` | ✓ | ✓ | giả | ✓ | A |
| POST | `/projects/{}/graph/validate` | ✓ |  | thiếu |  | A |
| POST | `/projects/{}/intel/analyze/{}` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/intel/blueprints` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/intel/blueprints/{}` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/intel/channels` | ✓ |  | thiếu | ✓ | D |
| POST | `/projects/{}/intel/channels` | ✓ |  | thiếu | ✓ | D |
| POST | `/projects/{}/intel/channels/user-data` | ✓ |  | thiếu |  | D |
| GET | `/projects/{}/intel/channels/{}` | ✓ |  | thiếu | ✓ | D |
| POST | `/projects/{}/intel/chat` | ✓ |  | thiếu |  | D |
| POST | `/projects/{}/intel/compliance/check` | ✓ |  | thiếu |  | D |
| POST | `/projects/{}/intel/generate-script` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/intel/ideas` | ✓ |  | thiếu |  | D |
| POST | `/projects/{}/intel/ideas/generate` | ✓ |  | thiếu | ✓ | D |
| PATCH | `/projects/{}/intel/ideas/{}` | ✓ |  | thiếu | ✓ | D |
| POST | `/projects/{}/intel/ideas/{}/promote` | ✓ |  | thiếu | ✓ | D |
| POST | `/projects/{}/intel/ideas/{}/schedule` | ✓ |  | thiếu | ✓ | D |
| * | `/projects/{}/intel/ideas{}` |  |  | thiếu | ✓ | D |
| POST | `/projects/{}/intel/nodes/execute` | ✓ |  | thiếu |  | D |
| GET | `/projects/{}/intel/patterns` | ✓ |  | thiếu |  | D |
| * | `/projects/{}/intel/patterns{}` |  |  | thiếu | ✓ | D |
| GET | `/projects/{}/intel/strategy` | ✓ |  | thiếu | ✓ | D |
| POST | `/projects/{}/intel/strategy` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/library` |  | ✓ | thật |  | D |
| POST | `/projects/{}/library/upload` | ✓ |  | thiếu |  | D |
| GET | `/projects/{}/outputs` | ✓ | ✓ | giả | ✓ | A |
| GET | `/projects/{}/products` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/products` | ✓ |  | thiếu | ✓ | D |
| POST | `/projects/{}/products/collect-url` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/products/import-url` | ✓ |  | thiếu | ✓ | D |
| DELETE | `/projects/{}/products/{}` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/products/{}` | ✓ |  | thiếu | ✓ | D |
| PUT | `/projects/{}/products/{}` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/products/{}/koc-styles` | ✓ |  | thiếu |  | D |
| POST | `/projects/{}/products/{}/koc-styles/analyze-url` | ✓ |  | thiếu |  | D |
| POST | `/projects/{}/products/{}/koc-styles/collect` | ✓ |  | thiếu |  | D |
| GET | `/projects/{}/products/{}/koc-styles/{}` | ✓ |  | thiếu |  | D |
| GET | `/projects/{}/publishing` | ✓ | ✓ | thật | ✓ | D |
| POST | `/projects/{}/publishing` | ✓ |  | thiếu | ✓ | D |
| POST | `/projects/{}/publishing/schedule` |  | ✓ | thật | ✓ | D |
| POST | `/projects/{}/resolve-preview` | ✓ |  | thiếu |  | D |
| POST | `/projects/{}/run` | ✓ | ✓ | giả | ✓ | A |
| POST | `/projects/{}/run-episodes` | ✓ |  | thiếu |  | A |
| POST | `/projects/{}/run-episodes/cancel` | ✓ |  | thiếu |  | A |
| POST | `/projects/{}/run/cancel` | ✓ | ✓ | giả | ✓ | A |
| GET | `/projects/{}/runs` | ✓ | ✓ | giả | ✓ | A |
| GET | `/projects/{}/runs/active` | ✓ | ✓ | giả | ✓ | A |
| DELETE | `/projects/{}/runs/{}` | ✓ |  | thiếu | ✓ | A |
| GET | `/projects/{}/runs/{}` | ✓ |  | thiếu | ✓ | A |
| GET | `/projects/{}/script-data/{}` |  | ✓ | thật |  | D |
| GET | `/projects/{}/social/accounts` | ✓ | ✓ | thật | ✓ | C |
| POST | `/projects/{}/social/accounts` | ✓ |  | thiếu | ✓ | C |
| GET | `/projects/{}/social/accounts/stats` | ✓ |  | thiếu |  | C |
| DELETE | `/projects/{}/social/accounts/{}` | ✓ |  | thiếu | ✓ | C |
| GET | `/projects/{}/social/accounts/{}` | ✓ |  | thiếu | ✓ | C |
| PUT | `/projects/{}/social/accounts/{}` | ✓ |  | thiếu | ✓ | C |
| GET | `/projects/{}/social/accounts/{}/connect` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/accounts/{}/refresh` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/analytics/apply-golden-windows` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/analytics/collect` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/analytics/creatives` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/analytics/generate-insights` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/analytics/heatmap` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/analytics/import-csv` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/analytics/learnings` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/analytics/manual-metrics` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/analytics/outliers` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/analytics/overview` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/audit` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/automation/rules` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/automation/rules` | ✓ |  | thiếu |  | C |
| DELETE | `/projects/{}/social/automation/rules/{}` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/automation/rules/{}/toggle` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/automation/run` | ✓ |  | thiếu |  | A |
| GET | `/projects/{}/social/calendar` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/caption-variants` | ✓ |  | thiếu |  | B |
| POST | `/projects/{}/social/captions/generate` | ✓ |  | thiếu |  | B |
| POST | `/projects/{}/social/ideas/{}/schedule` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/library` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/migrate` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/notes` | ✓ |  | thiếu |  | C |
| PUT | `/projects/{}/social/notes` | ✓ |  | thiếu |  | C |
| DELETE | `/projects/{}/social/notifications` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/notifications` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/notifications/{}/read` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/overview` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/packages/{}/{}` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/packages/{}/{}.zip` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/packages/{}/{}/generate` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/plans/commit` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/plans/preview` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/posts` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/posts` | ✓ | ✓ | thật |  | C |
| POST | `/projects/{}/social/posts/draft` | ✓ |  | thiếu |  | C |
| DELETE | `/projects/{}/social/posts/{}` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/posts/{}` | ✓ |  | thiếu |  | C |
| PATCH | `/projects/{}/social/posts/{}` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/posts/{}/approve` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/posts/{}/cancel` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/posts/{}/duplicate` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/posts/{}/publish-now` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/posts/{}/reject` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/posts/{}/schedule` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/posts/{}/submit` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/posts/{}/targets/{}/mark-published` | ✓ |  | thiếu |  | C |
| PUT | `/projects/{}/social/posts/{}/targets/{}/time` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/posts/{}/targets/{}/validate` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/queue` | ✓ |  | thiếu | ✓ | C |
| GET | `/projects/{}/social/scheduler/status` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/slots` | ✓ |  | thiếu |  | C |
| PUT | `/projects/{}/social/slots` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/stats` |  | ✓ | thật |  | C |
| POST | `/projects/{}/social/suggest-time` | ✓ |  | thiếu |  | C |
| POST | `/projects/{}/social/targets/bulk` | ✓ |  | thiếu |  | C |
| GET | `/projects/{}/social/templates` | ✓ | ✓ | thật | ✓ | C |
| POST | `/projects/{}/social/templates` | ✓ |  | thiếu | ✓ | C |
| DELETE | `/projects/{}/social/templates/{}` | ✓ |  | thiếu | ✓ | C |
| PUT | `/projects/{}/social/templates/{}` | ✓ |  | thiếu | ✓ | C |
| GET | `/projects/{}/social/videos` | ✓ |  | thiếu |  | B |
| GET | `/projects/{}/sources` | ✓ |  | thiếu | ✓ | D |
| POST | `/projects/{}/sources` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/sources/jobs/{}` | ✓ |  | thiếu |  | D |
| DELETE | `/projects/{}/sources/{}` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/sources/{}` | ✓ |  | thiếu | ✓ | D |
| PATCH | `/projects/{}/sources/{}` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/sources/{}/file/{}` | ✓ |  | thiếu | ✓ | D |
| POST | `/projects/{}/sources/{}/localize` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/sources/{}/localize/{}` | ✓ |  | thiếu |  | D |
| POST | `/projects/{}/sources/{}/scenes` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/sources/{}/script` | ✓ |  | thiếu | ✓ | D |
| PUT | `/projects/{}/sources/{}/script` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/sources/{}/transcript` | ✓ |  | thiếu | ✓ | D |
| PUT | `/projects/{}/sources/{}/transcript` | ✓ |  | thiếu | ✓ | D |
| GET | `/projects/{}/spend` | ✓ |  | thiếu |  | A |
| POST | `/projects/{}/video/episodes/{}/build` | ✓ |  | thiếu | ✓ | B |
| POST | `/projects/{}/video/episodes/{}/plan` | ✓ |  | thiếu | ✓ | B |
| GET | `/projects/{}/video/episodes/{}/status` | ✓ |  | thiếu | ✓ | B |
| POST | `/projects/{}/video/render` | ✓ |  | thiếu | ✓ | B |
| GET | `/projects/{}/video/render/{}` | ✓ |  | thiếu | ✓ | B |
| POST | `/projects/{}/video/render/{}/cancel` | ✓ |  | thiếu | ✓ | B |
| GET | `/projects/{}/video/render/{}/events` | ✓ |  | thiếu | ✓ | A |
| GET | `/projects/{}/video/renders` | ✓ |  | thiếu |  | B |
| * | `/projects/{}/video/renders{}` |  |  | thiếu | ✓ | B |
| POST | `/projects/{}/voice/clone` | ✓ | ✓ | thật | ✓ | B |
| POST | `/projects/{}/voice/preview` | ✓ |  | thiếu |  | B |
| POST | `/projects/{}/voice/say` | ✓ | ✓ | thật | ✓ | B |
| * | `/projects/{}/{}` |  |  | thiếu | ✓ | D |
| GET | `/providers` | ✓ | ✓ | giả | ✓ | E |
| GET | `/settings/api-keys` | ✓ |  | thiếu | ✓ | E |
| POST | `/settings/api-keys` | ✓ |  | thiếu | ✓ | E |
| POST | `/settings/api-keys/verify` | ✓ |  | thiếu | ✓ | E |
| GET | `/social/_ping` | ✓ |  | thiếu |  | C |
| GET | `/social/accounts/stats` | ✓ |  | thiếu |  | C |
| POST | `/social/captions/generate` | ✓ |  | thiếu |  | B |
| GET | `/social/library` | ✓ |  | thiếu |  | C |
| GET | `/social/oauth/callback/{}` | ✓ |  | thiếu |  | C |
| GET | `/social/overview` | ✓ |  | thiếu |  | C |
| POST | `/social/plans/commit` | ✓ |  | thiếu |  | C |
| POST | `/social/plans/preview` | ✓ |  | thiếu |  | C |
| GET | `/social/queue` | ✓ |  | thiếu |  | C |
| POST | `/social/targets/bulk` | ✓ |  | thiếu |  | C |
| GET | `/social/templates` | ✓ |  | thiếu |  | C |
| POST | `/social/templates` | ✓ |  | thiếu |  | C |
| DELETE | `/social/templates/{}` | ✓ |  | thiếu |  | C |
| PUT | `/social/templates/{}` | ✓ |  | thiếu |  | C |
| GET | `/social/videos` | ✓ |  | thiếu |  | B |
| GET | `/sources/_ping` | ✓ |  | thiếu |  | E |
| GET | `/video/_ping` | ✓ |  | thiếu |  | B |
| POST | `/video/craft/hooks` | ✓ |  | thiếu | ✓ | B |
| POST | `/video/craft/lint` | ✓ |  | thiếu | ✓ | B |
| GET | `/video/platforms` | ✓ |  | thiếu |  | B |
| POST | `/video/render` |  | ✓ | thật |  | B |
| GET | `/video/status/{}` |  | ✓ | thật |  | B |
| GET | `/video/templates` | ✓ | ✓ | thật | ✓ | B |
| GET | `/video/templates/{}` | ✓ |  | thiếu | ✓ | B |
| POST | `/video/templates/{}/preview` | ✓ |  | thiếu | ✓ | B |
| GET | `/video/templates/{}/preview.jpg` | ✓ |  | thiếu | ✓ | B |
| GET | `/video/templates/{}/preview.mp4` | ✓ |  | thiếu | ✓ | B |
| POST | `/video/templates/{}/validate` | ✓ |  | thiếu | ✓ | B |
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

Python 32 · Rust node_types.json 0 · executor thật đăng ký: 0

| type | Python | Rust /node-types | Executor |
|---|---|---|---|
| bg_remove | ✓ | thiếu | Fallback |
| blueprint_analyze | ✓ | thiếu | Fallback |
| captions | ✓ | thiếu | Fallback |
| cast | ✓ | thiếu | Fallback |
| channel_insight | ✓ | thiếu | Fallback |
| character | ✓ | thiếu | Fallback |
| character_image | ✓ | thiếu | Fallback |
| depth_layers | ✓ | thiếu | Fallback |
| export | ✓ | thiếu | Fallback |
| handdraw | ✓ | thiếu | Fallback |
| hyperframes_render | ✓ | thiếu | Fallback |
| image | ✓ | thiếu | Fallback |
| image_edit | ✓ | thiếu | Fallback |
| image_gen | ✓ | thiếu | Fallback |
| localize | ✓ | thiếu | Fallback |
| platform_export | ✓ | thiếu | Fallback |
| policy | ✓ | thiếu | Fallback |
| product | ✓ | thiếu | Fallback |
| quality_check | ✓ | thiếu | Fallback |
| scene | ✓ | thiếu | Fallback |
| scene_cut | ✓ | thiếu | Fallback |
| script | ✓ | thiếu | Fallback |
| script_from_blueprint | ✓ | thiếu | Fallback |
| source_video | ✓ | thiếu | Fallback |
| template_render | ✓ | thiếu | Fallback |
| text | ✓ | thiếu | Fallback |
| transition | ✓ | thiếu | Fallback |
| video_gen | ✓ | thiếu | Fallback |
| vision_analyze | ✓ | thiếu | Fallback |
| voice_clone | ✓ | thiếu | Fallback |
| voice_tts | ✓ | thiếu | Fallback |
| word_captions | ✓ | thiếu | Fallback |
