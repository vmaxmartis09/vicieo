# `node-body/`

Module điều phối và hiển thị nội dung phần thân node (`NodeBody`) trên Studio Canvas.

## Cấu trúc
- `index.tsx`: `NodeBody` (memoized) nối store hẹp (`updateNodeParams`, `nodeCatalog`), chuẩn bị `lines` và `hasSpeakCapability`.
- `NodeBodyDispatcher.tsx`: Phân nhánh theo `nodeType` (`SOURCE_TYPES` -> `null`, `cast`/`product`/`character`, hoặc 4 nhóm renderer).
- `types.ts`: Kiểu `NodeBodyProps` và `NodeRendererContextProps`.
- `components/`: 6 helper UI trích từ file gốc (`Empty`, `ImagePreview`, `VoiceHoverSpeaker`, `PolicyBadge`, `EpisodeOutputPreview`, `ShotPackLink`).
- `renderers/`:
  - `ScriptAndVoiceRenderer.tsx`: `script`, `voice_tts`, `captions`.
  - `MediaGenRenderer.tsx`: `image_gen`, `image_edit`, `video_gen`, `transition`.
  - `ExportAndPolicyRenderer.tsx`: `hyperframes_render`, `export`, `policy`, `quality_check`, `vision_analyze`.
  - `ToolboxAndGenericRenderer.tsx`: các node công cụ hậu kỳ, `template_render`, `platform_export` và nhánh `default`.
- `utils/castExtractor.ts`: Hàm thuần bóc xuất diễn viên từ `script-data` và **bounded LRU cache** (`MAX_CAST_CACHE_ENTRIES = 32`, quyết định K.4) dùng cho `CastPreview.tsx`.

## Người gọi & Tương thích ngược
- Re-export qua `components/canvas/NodeBody.tsx` cho `StudioNode.tsx`.
- `CastPreview.tsx` dùng lại cache & mapper từ `node-body/utils/castExtractor`.

## Bẫy đã biết
- Giữ nguyên `data-testid="episode-output"`, `data-testid="shot-pack-download"`, `data-testid="export-open-preview"` đã có sẵn ở mã gốc.
- Giữ chuẩn Morphicons (`@/components/ui/icons`) và class `nodrag` trên điều khiển tương tác.
