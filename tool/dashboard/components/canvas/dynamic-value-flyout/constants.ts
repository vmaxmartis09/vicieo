import type { TemplateVariableItem } from "./types";

export const TEMPLATE_VARIABLES: TemplateVariableItem[] = [
  { label: "Prompt lock nhân vật", tag: "{character.promptLock}", desc: "Mô tả cố định diện mạo nhân vật" },
  { label: "Tên nhân vật", tag: "{character.name}", desc: "Tên đại diện KOC / diễn viên" },
  { label: "Lời thoại kịch bản", tag: "{script.dialogue}", desc: "Lời thoại phân cảnh hiện tại" },
  { label: "Mô tả phân cảnh", tag: "{script.prompt}", desc: "Mô tả hình ảnh/camera phân cảnh" },
  { label: "Tên sản phẩm", tag: "{product.name}", desc: "Tên sản phẩm trong dự án" },
  { label: "Mô tả sản phẩm", tag: "{product.description}", desc: "Đặc tính chi tiết sản phẩm" },
];
