export const MAX_SELECTED_ACCOUNTS = 10;

export const WIZARD_STEPS = [
  { step: 1, title: "1. Chọn video" },
  { step: 2, title: "2. Chọn kênh & Tương thích" },
  { step: 3, title: "3. Nội dung & Xem trước" },
  { step: 4, title: "4. Lập lịch tự động" },
  { step: 5, title: "5. Duyệt & Xác nhận" },
] as const;

export const DEFAULT_VIDEO_CAPTION_CONFIG = {
  mainCaption: "",
  tone: "friendly",
  goal: "story",
  includeAffiliate: false,
  affiliateLink: "",
  varyPerAccount: false,
  platformData: {},
} as const;
