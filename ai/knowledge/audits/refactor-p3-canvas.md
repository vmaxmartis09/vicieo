# BÁO CÁO TRẠNG THÁI GÓI 3: STUDIO CANVAS (`P3-CANVAS`)
> **Tài liệu kiểm định:** `ai/knowledge/audits/refactor-p3-canvas.md`  
> **Gói thực hiện:** Gói 3 (`components/canvas/StudioCanvas.tsx`)  
> **Thời gian:** 2026-11-09  
> **Trạng thái:** TẠM HOÃN AN TOÀN DO FILE ĐANG CÓ THAY ĐỔI SONG SONG CHƯA COMMIT (QUY TẮC H.2.2)

---

## 1. Lý do tạm hoãn (Không vi phạm an toàn nguồn code)

Tại thời điểm triển khai, `tool/dashboard/components/canvas/StudioCanvas.tsx` (2.131 LOC) đang có chỉnh sửa cục bộ chưa commit (`M tool/dashboard/components/canvas/StudioCanvas.tsx`, timestamp `2026-11-09 22:29`) đi cùng các tệp canvas khác (`StudioNode.tsx`, `NodeInputs.tsx`, `VideoGenParams.tsx`, `PortValueInput.tsx`, `GenericParams.tsx`, `Reveal.tsx`, `graph-layout.ts`).  
Tuân thủ **Mục H.2.2** của `ai/plans/refactor-components.md`: *"nếu file trong gói có thay đổi của người khác thì dừng hoặc chỉ làm file sạch"*, Gói 3 KHÔNG chạm vào `StudioCanvas.tsx` để tránh xung đột hợp nhất hoặc làm mất thay đổi UX đang chạy dở.

---

## 2. Khảo sát chỉ đọc sẵn sàng cho lượt triển khai ngay khi file sạch

Đã xác minh vị trí chính xác trên `StudioCanvas.tsx` (không chỉnh sửa file):
- Dòng `115`: `export { toCoreGraph }` (đã trỏ từ `@/lib/core-api`, phải giữ re-export ở `studio-canvas/index.tsx`).
- Dòng `117`: `function wouldCreateCycle(...)` (bản trùng cục bộ; thay bằng `import { wouldCreateCycle } from "@/lib/graph/wouldCreateCycle"` đã có sẵn từ Gói 1 ở bước cuối).
- Dòng `236`: `useShallow((s: StudioStore) => ...)` đang bốc khối state lớn vào component vỏ chính.
- Dòng `287`: `const [activeDragPort, setActiveDragPort] = useState<ActiveDragPort | null>(...)` gây re-render tầng cao khi kéo nối dây; cần cô lập vào `components/LinkGhostLayer.tsx` và context kéo riêng.

### Danh sách việc cần chạy lần lượt ngay khi `git status` trên `StudioCanvas.tsx` sạch
1. Viết `tool/dashboard/test/p3-canvas/StudioCanvas.test.tsx` (Test First — render 5 node, chọn node cập nhật `selectedNodeId`, phím `Delete`, `Cmd+Z`, nối cổng + chặn vòng, chạy mock) và xác nhận XANH trên mã chưa tách.
2. Chạy đo gốc React `<Profiler>` cho 3 thao tác (Mục E.3: gõ prompt, kéo node, kéo dây nối) trước khi tách.
3. Tách vào `components/canvas/studio-canvas/`: `utils/graphAlgorithms.ts` (chứa `resolveOverlaps`), 4 hooks (`useCanvasShortcuts`, `useCanvasConnections`, `useCanvasAutoSave`, `useCanvasExecution`), các UI chrome (`CanvasOverlayToolbar`, `CanvasBottomDock`, `CanvasModalsHost` giữ `next/dynamic` cho `CreateVideoModal`, `LinkGhostLayer`), `StudioCanvasViewport`, `StudioCanvasRoot` (bọc `CanvasErrorBoundary`), và `index.tsx` giữ `export { StudioCanvas, toCoreGraph }`.
4. Thu hẹp selector và đo lại `<Profiler>` sau tách; chạy đủ Checklist D.2.

---

## 3. Điều kiện môi trường bổ sung cần lưu ý (Mục K.2, K.5, K.6)

- **Playwright (`K.2`):** ĐÃ TRIỂN KHAI & ĐO XÁC THỰC trong `tool/dashboard/e2e/` (fallback Chrome `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`, `npm run test:e2e:visual`, sai số thực đo `0% – 0.0008% <= 0.2%`; xem chi tiết tại `ai/knowledge/audits/refactor-p0-test-infra.md`). Lưu ý: nếu `dtthoai` đẩy thay đổi UI canvas trước khi Gói 3 chạy, cần chạy `UPDATE_BASELINES=1 npm run test:e2e:visual` để chốt lại ảnh mốc sạch trước bước tách đầu tiên.
- **Bundle Analyzer (`K.5`):** Hiện chưa cài `@next/bundle-analyzer`; đã trích xuất bảng 16 chunks (tổng 352,4 kB gzip) của `/[locale]/studio` trực tiếp từ `app-build-manifest.json` trong `ai/knowledge/audits/refactor-p6-quickwins.md`.
- **Chiến dịch `data-testid` (`K.6`):** Cần thực hiện trong một commit riêng SAU khi các tệp canvas bị khóa (`StudioCanvas.tsx`, `StudioNode.tsx`, `NodeInputs.tsx`) được tách hoàn tất.
