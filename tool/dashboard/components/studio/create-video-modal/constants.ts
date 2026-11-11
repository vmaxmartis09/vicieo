import type { ProductFormInfo } from "./types";

export const CATEGORY_CHIPS = [
  { id: "all", label: "Tất cả mẫu" },
  { id: "sales", label: "Bán hàng TikTok" },
  { id: "review", label: "KOC Review" },
  { id: "animation", label: "Hoạt hình 3D" },
  { id: "education", label: "Khám phá / Fact" },
  { id: "affiliate", label: "Affiliate So sánh" },
  { id: "story", label: "Truyện kể" },
  { id: "reup", label: "Bản địa hoá" },
] as const;

export const INITIAL_PRODUCT_INFO: ProductFormInfo = {
  title: "",
  name: "",
  price: "",
  usp: "",
  image: "",
};
