# character-profile-detail

Module chi tiết hồ sơ nhân vật (5 mục: Hồ sơ, Ảnh DNA, Video, Giọng nói, Dùng ở đâu).

## Cấu trúc

- `index.tsx`: Điều phối header (`ProfileHeaderBar`) và 5 section độc lập.
- `utils/characterAssets.ts`: Hàm thuần xử lý danh sách ảnh mẫu (`resolveReferenceLibrary`, `appendUploadedReferenceAsset`, `markPrimaryReferenceAsset`, `removeReferenceAsset`, `findReferencingNodes`).
- `hooks/useCharacterVoiceTest.ts`: Gọi `/api/voice/preview` và giữ trạng thái đọc thử giọng OmniVoice.
- `components/`: `ProfileHeaderBar`, `SectionProfileForm`, `SectionImagesGallery`, `SectionVideosClips`, `SectionVoiceCloning`, `SectionUsageNodes`.

## Lợi ích cô lập render

- `useStudioStore` (`nodes`, `setSelectedNodeId`, `flashNode`) được thu gọn vào `SectionUsageNodes` nên thao tác kéo/chạy node trên canvas không làm render lại các tab Hồ sơ / Ảnh / Giọng.
- State `lightboxImage` và `cloneModalOpen` được khoanh vùng đúng section dùng nó.

## Ai gọi

- Re-export qua `components/studio/tabs/CharacterProfileDetail.tsx` để giữ nguyên import tại `ProjectTab.tsx`.
