# SophisticatedChatInput (`components/studio/chat/sophisticated-input/`)

## Vai trò
Thanh soạn thảo hợp nhất ở cuối Studio Canvas với autocomplete (`/` slash, `@` nhân vật & ảnh canvas), 4 popover cài đặt (`+`, `Skill`, `Model/Ratio`, `Mode`), dải đính kèm, và nhập liệu giọng nói.

## Cấu trúc
- `index.tsx` (~190 dòng): shell chính, ghép các popover, dải đính kèm và toolbar.
- `types.ts`, `constants.ts`: định nghĩa props, danh sách model, craft presets, tỉ lệ khung hình.
- `hooks/useComposerMenus.ts`: trạng thái 4 popover, auto-resize textarea, click-outside, upload ảnh.
- `hooks/useVoiceDictation.ts`: hook Web Speech API fallback an toàn sang `showAlert`.
- `components/`: `AutocompletePopovers`, `PlusActionPopover`, `SkillCraftPopover`, `ModelSettingsPopover`, `ModeSwitchPopover`, `AttachmentsBar`, `ComposerToolbar`.

## Tương thích
Re-export qua `components/studio/chat/SophisticatedChatInput.tsx` để bảo toàn import cũ.
