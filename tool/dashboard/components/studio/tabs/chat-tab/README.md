# ChatTab (`components/studio/tabs/chat-tab/`)

## Vai trò
Thanh dock trợ lý AI điều khiển Studio (tạo/nối node trên Canvas, chạy flow, quản lý nhân vật, gọi local agent & Core Chat API).

## Luồng dữ liệu & Cấu trúc
- `index.tsx` (~205 dòng): `ChatTab` bọc `ChatTabInner` trong `ChatErrorBoundary`.
- `hooks/`:
  - `useChatStoreBindings.ts`: narrow `useShallow` selector cho store.
  - `useChatComposerState.ts`: state thanh soạn thảo, collapse/Escape, đính kèm.
  - `useChatCanvasActions.ts`: ghi Canvas, Undo snapshot, xác nhận chạy/lên lịch.
  - `useChatContextActions.ts`: slash command (`/grid`, `/video`, `/image`, `/run`, `/clear`, `/blank`, `/char`) và ngữ cảnh nhân vật/dự án.
  - `useChatSend.ts`: luồng gửi tin (`runContextAction` -> `runLocalAgent` -> `sendChatToCore`).
- `utils/`: `chatGraphBuilders.ts`, `chatMediaUtils.ts`, `chatConfirmation.ts` (logic thuần không phụ thuộc React).
- `components/`: `ChatErrorBoundary`, `SelectedImageToolbar`, `ChatCollapsedCapsule`, `ChatExpandedDock`, `QuickPromptsStrip` (tái sử dụng `components/studio/chat/*`).

## Tương thích
Re-export qua `components/studio/tabs/ChatTab.tsx` cho `app/[locale]/layout.tsx`.
