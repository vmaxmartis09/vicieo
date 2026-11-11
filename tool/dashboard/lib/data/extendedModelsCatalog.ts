export interface ExtendedModelItem {
  key: string;
  name: string;
  brand: string;
  provider: string;
  category: "image" | "video" | "voice" | "text";
  tier: "paid" | "free" | "test";
  costStr: string;
  costNote: string;
  badge: string;
  specs: {
    resolutions: string[];
    ratios: string[];
    maxDuration?: string;
    hasAudio?: boolean;
    hasRefImage?: boolean;
  };
  description: string;
  strengths: string[];
  recommendedUse: string;
}

export const EXTENDED_MODELS_CATALOG: ExtendedModelItem[] = [
  // --- VIDEO MODELS ---
  {
    key: "kling-v3-std",
    name: "Kling v3 Standard",
    brand: "Kuaishou AI",
    provider: "Kuaishou",
    category: "video",
    tier: "paid",
    costStr: "$0.08/s",
    costNote: "$2.40 / clip 30s",
    badge: "Phổ biến",
    specs: {
      resolutions: ["720p", "1080p"],
      ratios: ["9:16", "16:9", "1:1"],
      maxDuration: "60s",
      hasAudio: false,
      hasRefImage: true,
    },
    description: "Mô hình sinh video AI chuẩn điện ảnh từ Kuaishou, chuyển động mượt mà, tái hiện dáng người và trang phục chân thực.",
    strengths: ["Chuyển động người tự nhiên", "Giữ chi tiết trang phục KOC cực tốt", "Camera pan/zoom mượt"],
    recommendedUse: "Video review sản phẩm KOC, video thời trang TikTok/Reels, clip người nói chuyện.",
  },
  {
    key: "kling-v3-pro",
    name: "Kling v3 Pro",
    brand: "Kuaishou AI",
    provider: "Kuaishou",
    category: "video",
    tier: "paid",
    costStr: "$0.14/s",
    costNote: "$4.20 / clip 30s",
    badge: "Chất lượng cao",
    specs: {
      resolutions: ["1080p", "4K"],
      ratios: ["9:16", "16:9", "1:1"],
      maxDuration: "60s",
      hasAudio: false,
      hasRefImage: true,
    },
    description: "Phiên bản Pro độ phân giải cao nhất của Kling, kết cấu da mặt, tóc và ánh sáng studio chân thực đỉnh cao.",
    strengths: ["Độ phân giải 1080p & 4K sắc nét", "Ánh sáng studio chân thực", "Hạn chế tối đa biến dạng chi tiết"],
    recommendedUse: "Quảng cáo TVC thương hiệu, video cận cảnh gương mặt nhân vật, video sản phẩm cao cấp.",
  },
  {
    key: "veo-2",
    name: "Veo 2.0 Video",
    brand: "Google DeepMind",
    provider: "Google",
    category: "video",
    tier: "paid",
    costStr: "$0.15/s",
    costNote: "$4.50 / clip 30s",
    badge: "SOTA Google",
    specs: {
      resolutions: ["1080p", "4K"],
      ratios: ["9:16", "16:9", "1:1", "4:5"],
      maxDuration: "60s",
      hasAudio: true,
      hasRefImage: true,
    },
    description: "Siêu mô hình tạo video mới nhất từ Google DeepMind, hiểu sâu ngữ cảnh vật lý 3D, hỗ trợ âm thanh đồng bộ.",
    strengths: ["Hiểu vật lý không gian 3D", "Tương tác nhân vật và đồ vật hoàn hảo", "Hỗ trợ âm thanh AI đồng bộ"],
    recommendedUse: "Video kịch bản phức tạp, cảnh hành động, phim ngắn đa góc máy.",
  },
  {
    key: "ltx-video",
    name: "LTX Video Ultra Fast",
    brand: "Lightricks AI",
    provider: "Lightricks",
    category: "video",
    tier: "paid",
    costStr: "$0.05/s",
    costNote: "$1.50 / clip 30s",
    badge: "Siêu nhanh",
    specs: {
      resolutions: ["720p", "1080p"],
      ratios: ["9:16", "16:9"],
      maxDuration: "30s",
      hasAudio: false,
      hasRefImage: true,
    },
    description: "Mô hình video thời gian thực siêu tốc độ từ Lightricks, sinh 1 clip 5s chỉ trong 4-8 giây.",
    strengths: ["Tốc độ render nhanh nhất thị trường", "Chi phí tiết kiệm", "Thích hợp xem trước preview"],
    recommendedUse: "Test phân cảnh nhanh, tạo hàng loạt biến thể video quảng cáo ngắn.",
  },
  {
    key: "wan-2-1",
    name: "Wan 2.1 Video",
    brand: "Alibaba Wanx",
    provider: "Alibaba",
    category: "video",
    tier: "paid",
    costStr: "$0.05/s",
    costNote: "$1.50 / clip 30s",
    badge: "Tiết kiệm",
    specs: {
      resolutions: ["720p", "1080p"],
      ratios: ["9:16", "16:9", "1:1"],
      maxDuration: "30s",
      hasAudio: false,
      hasRefImage: true,
    },
    description: "Mô hình mã nguồn mở hiệu năng cao từ Alibaba, chuyên tạo chuyển động từ ảnh chụp tĩnh sản phẩm thời trang.",
    strengths: ["Rất ổn định với ảnh sản phẩm phẳng", "Chi phí thấp", "Màu sắc sống động"],
    recommendedUse: "Video giới thiệu sản phẩm thương mại điện tử (E-commerce), TikTok Shop.",
  },

  // --- IMAGE MODELS ---
  {
    key: "pollinations-flux",
    name: "Pollinations FLUX.1",
    brand: "Black Forest Labs",
    provider: "Pollinations",
    category: "image",
    tier: "paid",
    costStr: "$0.003/ảnh",
    costNote: "Thương mại chính thức",
    badge: "Chất lượng ảnh SOTA",
    specs: {
      resolutions: ["1080p", "2K"],
      ratios: ["9:16", "16:9", "1:1", "4:5"],
      hasRefImage: true,
    },
    description: "Mô hình sinh ảnh hàng đầu thế giới của Black Forest Labs (các cựu kỹ sư Stable Diffusion).",
    strengths: ["Hiểu prompt tiếng Anh và mô tả chi tiết chuẩn xác", "Vẽ ngón tay và mắt hoàn hảo", "Kết cấu da siêu thực"],
    recommendedUse: "Ảnh nhân vật KOC, ảnh mẫu thời trang lookbook, ảnh bìa video sắc nét.",
  },
  {
    key: "pollinations-sana",
    name: "Pollinations Sana Fast",
    brand: "NVIDIA Sana Lab",
    provider: "Pollinations",
    category: "image",
    tier: "paid",
    costStr: "$0.002/ảnh",
    costNote: "Tốc độ cao",
    badge: "Siêu nhanh",
    specs: {
      resolutions: ["1080p"],
      ratios: ["9:16", "16:9", "1:1"],
      hasRefImage: true,
    },
    description: "Mô hình sinh ảnh thế hệ mới áp dụng kiến trúc Linear Attention, sinh ảnh Full HD trong 1-2 giây.",
    strengths: ["Tốc độ sinh ảnh 1-2 giây", "Tiêu thụ tài nguyên thấp", "Màu sắc tươi tắn chuẩn quảng cáo"],
    recommendedUse: "Tạo ảnh khung đầu (first frame) cho video chuyển động, ảnh background.",
  },
  {
    key: "midjourney-v6",
    name: "Midjourney v6.1",
    brand: "Midjourney Inc",
    provider: "Midjourney",
    category: "image",
    tier: "paid",
    costStr: "$0.04/ảnh",
    costNote: "Thương mại",
    badge: "Phong cách nghệ thuật",
    specs: {
      resolutions: ["1080p", "2K"],
      ratios: ["9:16", "16:9", "1:1", "4:5"],
      hasRefImage: true,
    },
    description: "Tiêu chuẩn vàng trong ngành hình ảnh nghệ thuật, ánh sáng cinematic và thẩm mỹ quảng cáo.",
    strengths: ["Thẩm mỹ thị giác đỉnh cao", "Ánh sáng nghệ thuật và chiều sâu trường ảnh", "Phong cách tạp chí thời trang"],
    recommendedUse: "Poster, ảnh chiến dịch thương hiệu, concept visual.",
  },
  {
    key: "recraft-v3",
    name: "Recraft Vector & SOTA",
    brand: "Recraft AI",
    provider: "Recraft",
    category: "image",
    tier: "paid",
    costStr: "$0.03/ảnh",
    costNote: "Thương mại",
    badge: "Vector & Thiết kế",
    specs: {
      resolutions: ["1080p", "4K"],
      ratios: ["9:16", "16:9", "1:1", "4:5"],
      hasRefImage: true,
    },
    description: "Mô hình chuyên biệt cho thiết kế đồ họa, typography và ảnh sản phẩm thương mại không méo chữ.",
    strengths: ["Hiển thị chữ/text tiếng Anh cực nét trên ảnh", "Kiểm soát màu sắc theo bảng màu thương hiệu", "Xuất vector SVG"],
    recommendedUse: "Banner sản phẩm kèm chữ, thumbnail YouTube/TikTok, logo nhận diện.",
  },

  // --- VOICE & AUDIO MODELS ---
  {
    key: "omni-voice-lyly",
    name: "OmniVoice LyLy KOC",
    brand: "OmniVoice Studio",
    provider: "OmniVoice",
    category: "voice",
    tier: "paid",
    costStr: "$0.001/từ",
    costNote: "~$0.02 / câu thoại",
    badge: "Giọng KOC Việt Nam",
    specs: {
      resolutions: ["48kHz Hi-Fi"],
      ratios: [],
      hasAudio: true,
    },
    description: "Giọng đọc truyền cảm chuyên cho KOC review sản phẩm bán hàng, ngữ điệu tự nhiên, nhấn nhá thu hút.",
    strengths: ["Ngữ điệu KOC bán hàng sống động", "Tiếng Việt chuẩn giọng miền Bắc / Nam", "Hỗ trợ chèn cảm xúc vui tươi"],
    recommendedUse: "Lời thoại review sản phẩm, video bán hàng TikTok Shop, podcast ngắn.",
  },
  {
    key: "kokoro-tts-v1",
    name: "Kokoro TTS Fast",
    brand: "Hexgrad",
    provider: "Hexgrad",
    category: "voice",
    tier: "paid",
    costStr: "$0.0005/từ",
    costNote: "Siêu tiết kiệm",
    badge: "Tiết kiệm",
    specs: {
      resolutions: ["24kHz"],
      ratios: [],
      hasAudio: true,
    },
    description: "Mô hình chuyển văn bản thành giọng nói siêu nhẹ, độ trễ cực thấp dưới 150ms.",
    strengths: ["Độ trễ thấp gần như tức thì", "Chi phí siêu rẻ", "Đa ngôn ngữ"],
    recommendedUse: "Thuyết minh hướng dẫn, lồng tiếng phụ trợ.",
  },

  // --- TEXT & LLM MODELS ---
  {
    key: "gemini-2-5-flash",
    name: "Gemini 2.5 Flash",
    brand: "Google DeepMind",
    provider: "Google",
    category: "text",
    tier: "paid",
    costStr: "$0.0001/1k tokens",
    costNote: "Siêu tốc độ",
    badge: "Mặc định Studio",
    specs: {
      resolutions: ["1M context window"],
      ratios: [],
    },
    description: "Bộ não AI xử lý kịch bản phân cảnh, tối ưu prompt video và phân tích hình ảnh đa phương thức từ Google.",
    strengths: ["Hiểu ngữ cảnh kịch bản 100 phân cảnh", "Tạo prompt video tương thích hoàn hảo Veo/Kling", "Xử lý cực nhanh"],
    recommendedUse: "Sinh kịch bản tự động, chuyển kịch bản thô thành danh sách prompt phân cảnh chi tiết.",
  },
  {
    key: "claude-3-7-sonnet",
    name: "Claude 3.7 Sonnet",
    brand: "Anthropic",
    provider: "Anthropic",
    category: "text",
    tier: "paid",
    costStr: "$0.003/1k tokens",
    costNote: "Tư duy sâu sắc",
    badge: "Viết kịch bản đỉnh cao",
    specs: {
      resolutions: ["200k context window"],
      ratios: [],
    },
    description: "Mô hình ngôn ngữ tư duy logic và sáng tạo văn học đỉnh cao từ Anthropic, tạo câu chuyện lôi cuốn giữ chân người xem.",
    strengths: ["Văn phong tiếng Việt tự nhiên sâu sắc", "Hook kịch bản 3 giây đầu giữ chân người xem cực tốt", "Lập dàn ý chặt chẽ"],
    recommendedUse: "Kịch bản phim ngắn, video kể chuyện viral, thông điệp thương hiệu tinh tế.",
  },
];