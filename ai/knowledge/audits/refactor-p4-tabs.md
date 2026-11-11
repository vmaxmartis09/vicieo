# BÁO CÁO HOÀN THÀNH GÓI 4: TAB STUDIO & CHAT (`P4-TABS`)
> **Tài liệu kiểm định:** `ai/knowledge/audits/refactor-p4-tabs.md`  
> **Gói thực hiện:** Gói 4 (`CharacterProfileDetail`, `SophisticatedChatInput`, `ChatTab`, `JobsCostTab`, `PublishTab`; giữ nguyên `ProjectTab`; hoãn `ModelPicker`)  
> **Thời gian:** 2026-11-09  
> **Trạng thái:** HOÀN THÀNH 5/5 TỆP SẠCH + HOÃN AN TOÀN `ModelPicker.tsx` DO ĐANG CÓ SỬA ĐỔI SONG SONG

---

## 1. Tổng hợp trước / sau từng tệp

| Tệp gốc | Trước (LOC) | Barrel mới (LOC) | Thư mục tính năng & Tệp lớn nhất | Commit |
| :--- | :---: | :---: | :--- | :---: |
| `components/studio/tabs/CharacterProfileDetail.tsx` | 795 | 4 | `character-profile-detail/` — `ProfileHeaderBar` + 5 section (max 186 LOC) + `useCharacterVoiceTest` (48 LOC) + pure `characterAssets.ts` (83 LOC) | `9d322b164` |
| `components/studio/chat/SophisticatedChatInput.tsx` | 948 | 6 | `chat/sophisticated-input/` — 7 popover/bar/toolbar (max 193 LOC ở `index.tsx`) + `useComposerMenus` (93 LOC) + `useVoiceDictation` (54 LOC) + `constants.ts` (78 LOC) | `3fdba4d42` |
| `components/studio/tabs/ChatTab.tsx` | 1.207 | 1 | `chat-tab/` — `index.tsx` (222 LOC, bọc `ChatErrorBoundary`) + 5 component con (max 185 LOC) + 5 hooks (max 148 LOC) + 3 file pure `.ts` (max 214 LOC) | `8a571603e` |
| `components/studio/tabs/JobsCostTab.tsx` | 706 | 1 | `jobs-cost-tab/` — `index.tsx` (140 LOC) + 4 component con (max 194 LOC ở `JobQueueCard`) + 2 hooks (max 101 LOC) + pure `mergeJobsList.ts` (120 LOC) | `3843a0e09` |
| `components/studio/tabs/PublishTab.tsx` | 605 | 1 | `publish-tab/` — `index.tsx` (137 LOC) + 4 section (max 134 LOC) + `usePublishTabData` (148 LOC) + pure `publishQueueMappers.ts` (62 LOC) | `556cbd5e0` |

### Quy chuẩn kiến trúc đã đạt
- Toàn bộ component hiển thị đều **≤ 222 dòng** (ngưỡng ≤ 250).
- Toàn bộ custom hook đều **≤ 148 dòng** (ngưỡng ≤ 150).
- Toàn bộ logic thuần không phụ thuộc React (`characterAssets.ts`, `chatMediaUtils.ts`, `chatConfirmation.ts`, `chatGraphBuilders.ts`, `mergeJobsList.ts`, `publishQueueMappers.ts`) được tách riêng thành `.ts` và có unit test trực tiếp.
- Mọi thư mục mới đều có `README.md` mô tả vai trò, luồng dữ liệu, người gọi và bẫy đã biết.
- Giữ nguyên 100% đường dẫn import cũ nhờ re-export tại barrel gốc (`CharacterProfileDetail.tsx`, `SophisticatedChatInput.tsx`, `ChatTab.tsx`, `JobsCostTab.tsx`, `PublishTab.tsx`).
- Cách ly lỗi ở luồng chat bằng `ChatErrorBoundary` kèm nút `Thử lại`.

---

## 2. Kết quả khảo sát cấu trúc & Quyết định phạm vi (Mục C.8 & Gói 4)

1. **`CharacterProfileDetail.tsx` (795 LOC — bắt buộc khảo sát trước khi tách):**
   - Trước khi tách, file trộn lẫn 4 nhóm trách nhiệm: header điều hướng, 5 section tab con (`Hồ sơ`, `Ảnh`, `Video`, `Giọng nói`, `Dùng ở đâu`), luồng tổng hợp thử giọng qua OmniVoice (`/api/voice/preview`), và hàm thuần quản lý thư viện ảnh DNA (`resolveReferenceLibrary`, `markPrimaryReferenceAsset`).
   - Sau khảo sát đã tách gọn thành 6 component riêng biệt, 1 hook `useCharacterVoiceTest` (48 LOC), và 1 module pure `utils/characterAssets.ts` (83 LOC), với selector hẹp `useStudioStore` tại `SectionUsageNodes.tsx`.
2. **`ProjectTab.tsx` (425 LOC):** Đã đọc kiểm tra trực tiếp — cấu trúc sạch, giữ nguyên không tách thêm đúng theo Mục C.8 của kế hoạch.
3. **`ModelPicker.tsx` (585 LOC — HOÃN theo quy tắc điều phối H.2.2):**
   - Lệnh `git status -s` xác nhận `tool/dashboard/components/studio/ModelPicker.tsx` đang có chỉnh sửa chưa commit thuộc nhánh công việc song song `ai/plans/dynamic-data-nodes.md`.
   - Tuân thủ tuyệt đối nguyên tắc không ghi đè file người khác đang sửa dở: Gói 4 giữ nguyên không chạm vào `ModelPicker.tsx`. Sau khi nhánh `dynamic-data-nodes` hoàn tất và commit sạch, có thể tách `ModelPicker.tsx` ở bước kế tiếp theo đúng quy trình Test-First.

---

## 3. Bộ test đặc tả hành vi đã bổ sung (Test First — Xanh trước khi tách)

Tổng **18 tests mới** trên **5 test files** đều được chạy **XANH trên mã gốc chưa tách** trước khi chỉnh sửa từng tệp:

1. `test/p4-tabs/CharacterProfileDetail.test.tsx` (2 tests):
   - Kiểm tra form hồ sơ, tự cập nhật `scaleRatio` khi đổi chiều cao, chuyển đủ 5 section và nghe thử OmniVoice.
   - Kiểm thử pure helpers `resolveReferenceLibrary`, `appendUploadedReferenceAsset`.
2. `test/p4-tabs/SophisticatedChatInput.test.tsx` (5 tests):
   - Autocomplete `/`, `@` (nhân vật & ảnh canvas), 4 popover `+` / `Skill` / `Model·Ratio` / `Mode`, fallback & Web Speech API cho thu âm giọng, dải `AttachmentsBar` và submit payload.
3. `test/p4-tabs/ChatTab.test.tsx` (5 tests):
   - Mở từ capsule thu gọn, gửi tin qua `/chat`, tạo node actionCard + `Hoàn tác` snapshot, dialog xác nhận `/run`, thanh `SelectedImageToolbar` ("Tạo Video"), pure helpers `extractCanvasImages`/`buildDirectMediaGraphUpdate`, và cô lập lỗi trong `ChatErrorBoundary`.
4. `test/p4-tabs/JobsCostTab.test.tsx` (3 tests):
   - Thẻ trần ngân sách, danh sách thành phẩm (`outputsList`) + nút Xem mở `usePreview` / nút Đăng mở `useSocialStore.openComposer` & chuyển tab `publish`, lọc trạng thái 3 nguồn job (`run_*`, legacy renders, store jobs), huỷ run, xoá job qua `confirm()` + lưu `localStorage`, xoá hết job đã kết thúc, và kiểm thử pure helpers `mergeAndSortJobs` / `filterJobsByTab` / `extractJobMediaUrl`.
5. `test/p4-tabs/PublishTab.test.tsx` (3 tests):
   - Đồng bộ `/social/accounts`, `/social/queue`, `/social/analytics/learnings`, `/social/analytics/overview`; click 4 metric pills mở đúng bucket/view trong `useSocialStore`; bật/tắt `Xem đúc kết chi tiết`; phản hồi sự kiện `social-accounts-updated` + mở `SocialConnectDrawer` qua `usePreview`; mở `ManualPackageModal` và `PublishCenterModal`; kiểm thử pure helpers `computeSummaryCounts`, `filterUpcomingQueueItems`, `mapConnectedPlatformFlags`.

---

## 4. Kết quả kiểm tra xác thực cuối cùng (Checklist D.2)

- `npx tsc --noEmit --incremental false`: **0 lỗi kiểu dữ liệu**.
- `npx vitest run`: **13/13 files, 53/53 tests PASS (100% XANH)** — riêng Gói 4 đóng góp **18/18 tests**.
- `npm run -s check:design`: **0 vi phạm trên 250 tệp được quét**.
- `NEXT_DIST_DIR=.next-build-p4 npx next build`: **Compiled successfully (7.7s)** (`/[locale]/studio` = 96 kB / 361 kB First Load JS); đã xoá `.next-build-p4` và giữ nguyên `tsconfig.json`.

---

## 5. Khuyến nghị bước tiếp theo

- Khi `tool/dashboard/components/studio/ModelPicker.tsx` sạch trên `git status`, tách `ModelPicker.tsx` (hiện 585 LOC) theo cùng quy trình Test-First -> Checklist D.2.
- Gói kế tiếp sạch để thực hiện trước khi các file canvas khác hết khoá: **`components/canvas/NodeBody.tsx` (908 LOC)** (đang sạch trên `git status`, chú ý không sửa sang `GenericParams.tsx`, `NodeInputs.tsx`, `VideoGenParams.tsx` đang có chỉnh sửa song song).
