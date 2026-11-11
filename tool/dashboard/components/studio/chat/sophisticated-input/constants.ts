import type { ChatModel } from "@/store/slices/chatSlice";
import type { ComposerActionMode, ComposerAspectRatio } from "./types";

export const MODEL_OPTIONS: { id: ChatModel; name: string; tag: string; desc: string }[] = [
  {
    id: "gemini-3.8-flash",
    name: "Gemini 3.8 Flash",
    tag: "High · 0.1x",
    desc: "Mô hình thế hệ mới: suy luận cực nhanh, thông minh và tối ưu chi phí",
  },
  {
    id: "gemini-2.5-pro",
    name: "Gemini 2.5 Pro",
    tag: "SOTA · 2x",
    desc: "Lập luận sâu, phân tích kịch bản phức tạp và sinh cấu trúc flow chuẩn xác",
  },
  {
    id: "gemini-2.5-flash",
    name: "Gemini 2.5 Flash",
    tag: "Fast · 0.5x",
    desc: "Cân bằng tốc độ cao và phản hồi tức thời cho các lệnh thông thường",
  },
  {
    id: "auto",
    name: "Auto Route",
    tag: "Smart",
    desc: "Tự động phân bổ mô hình phù hợp dựa trên độ phức tạp của yêu cầu",
  },
  {
    id: "rules",
    name: "Quy tắc Cục bộ",
    tag: "$0",
    desc: "Chạy thử không tốn API, điều khiển đồ thị và nhân vật theo mẫu",
  },
];

export const CRAFT_PRESETS: { id: string; labelVi: string; labelEn: string; group: string }[] = [
  { id: "3D Pixar Pixar-style", labelVi: "Phong cách: 3D Hoạt hình", labelEn: "Style: 3D Animation", group: "style" },
  { id: "Cinematic Realism", labelVi: "Phong cách: Điện ảnh thật", labelEn: "Style: Cinematic", group: "style" },
  { id: "Close-up 50mm f/1.4", labelVi: "Camera: Cận cảnh 50mm", labelEn: "Camera: Close-up 50mm", group: "camera" },
  { id: "Dynamic Tracking Shot", labelVi: "Chuyển động: Dolly bám theo", labelEn: "Motion: Dolly Tracking", group: "motion" },
  { id: "Golden Hour Rim Light", labelVi: "Ánh sáng: Viền nắng ấm", labelEn: "Light: Golden Rim", group: "light" },
  { id: "Low-angle Hero Shot", labelVi: "Góc máy: Góc thấp Hero", labelEn: "Angle: Low-angle", group: "angle" },
];

export const ASPECT_RATIO_OPTIONS: readonly ComposerAspectRatio[] = ["9:16", "16:9", "1:1", "4:5"] as const;

export const OUTPUT_MODE_OPTIONS: readonly {
  mode: ComposerActionMode;
  labelVi: string;
  labelEn: string;
}[] = [
  { mode: "chat", labelVi: "Trợ lý Chat / Flow", labelEn: "Chat / Flow" },
  { mode: "image", labelVi: "Tạo Node Ảnh", labelEn: "Single Image Node" },
  { mode: "image_list", labelVi: "Node List Ảnh (4x)", labelEn: "Image List Node (4x)" },
  { mode: "video", labelVi: "Ảnh → Video (9:16)", labelEn: "Image → Video" },
] as const;

export function getModePlaceholder(actionMode: ComposerActionMode, isVi: boolean): string {
  if (actionMode === "video") {
    return isVi
      ? "Mô tả chuyển động video từ ảnh tham chiếu (hoặc gõ @ chọn ảnh trên Canvas)..."
      : "Describe video motion from reference images (or @ to pick Canvas image)...";
  }
  if (actionMode === "image") {
    return isVi
      ? "Mô tả ảnh cần tạo hoặc chỉnh sửa từ ảnh tham chiếu..."
      : "Describe the image to generate or edit from reference...";
  }
  if (actionMode === "image_list") {
    return isVi
      ? "Mô tả bộ ảnh / Multi-Angle Storyboard (tạo Node List Ảnh trên Canvas)..."
      : "Describe image set / Multi-Angle Storyboard (creates Image List node)...";
  }
  return isVi
    ? "Mô tả ý tưởng, dùng @ để gắn ảnh/nhân vật, / để gọi kỹ năng..."
    : "Describe your idea. Use @ to reference assets, / for skills...";
}
