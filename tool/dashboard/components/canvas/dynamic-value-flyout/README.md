# `dynamic-value-flyout/`

Popover chọn nguồn giá trị động (`{node.port}`, `{Kịch bản.cảnh_N}`, `{Nhân vật}`, `{model.key}`, `{biến}`) dùng trong `DynamicValueSelector`.

## Cấu trúc
- `index.tsx`: Shell container kết nối store hẹp, thanh tìm kiếm, thanh 5 tab và điều phối kết nối node/chọn chuỗi động. Nhập danh mục model trực tiếp từ `@/lib/data/extendedModelsCatalog` để không kéo theo `AIModelsLibraryModal`.
- `types.ts` & `constants.ts`: Định nghĩa props/state và danh sách `TEMPLATE_VARIABLES`.
- `utils/flyoutFilters.ts`: Hàm thuần không phụ thuộc React (`buildCandidateNodes`, `filterCandidateNodes`, `filterCharactersList`, `filterExtendedModels`).
- `hooks/usePopoverDismiss.ts` & `hooks/useFlyoutScriptData.ts`: Tách lắng nghe phím Escape/click ngoài và lazy-load kịch bản theo tập.
- `tabs/`: 5 component tab (`NodesTab`, `ScriptShotsTab`, `CharactersTab`, `ModelsTab`, `VariablesTab`).

## Tương thích ngược
- Re-export qua `components/canvas/DynamicValueFlyout.tsx` (1 dòng barrel) cho `DynamicValueSelector.tsx`.
