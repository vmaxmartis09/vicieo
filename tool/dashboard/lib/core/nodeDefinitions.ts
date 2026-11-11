import type { NodeDefinition, PortType } from "../core-contracts";

export function isPortCompatible(
  sourceType: PortType,
  targetType: PortType
): boolean {
  if (sourceType === targetType) return true;
  if (sourceType === "image" && targetType === "image[]") return true;
  // Lời thoại có cấu trúc nối được vào cổng văn bản (Core ghép thành "Tên: câu")
  if (sourceType === "dialogue" && targetType === "text") return true;
  return false;
}

/**
 * Danh mục 13 loại node chuẩn từ Core (dùng làm bản dự phòng khi Core chưa bật).
 * Khi Core chạy, giao diện luôn lấy trực tiếp từ GET /node-types.
 */
export const FALLBACK_NODE_TYPES: NodeDefinition[] = [
  {
    typeName: "text",
    displayName: "Văn bản / Prompt",
    category: "Input",
    description: "Nhập văn bản tự do, ý tưởng hoặc prompt tùy chỉnh",
    inputPorts: [],
    outputPorts: [
      { name: "text", portType: "text", description: "Văn bản đầu ra", required: true },
    ],
    defaultParams: { content: "" },
    mockable: true,
  },
  {
    typeName: "image",
    displayName: "Tải ảnh / Thư viện",
    category: "Input",
    description: "Tải ảnh lên hoặc chọn ảnh từ thư viện tài nguyên",
    inputPorts: [],
    outputPorts: [
      { name: "image", portType: "image", description: "Ảnh đã chọn", required: true },
    ],
    defaultParams: { uri: "/demo/scenes/canh_01.jpg" },
    mockable: true,
  },
  {
    typeName: "character",
    displayName: "Nhân vật (Character Bible)",
    category: "Identity",
    description: "Chọn nhân vật của dự án (DNA ảnh tham chiếu, prompt lock, voiceId, bộ nhớ)",
    inputPorts: [],
    outputPorts: [
      { name: "character", portType: "character", description: "Dữ liệu hồ sơ nhân vật", required: true },
      { name: "reference_images", portType: "image[]", description: "Danh sách ảnh tham chiếu DNA", required: true },
    ],
    defaultParams: { characterId: "soc_nho" },
    mockable: true,
  },
  {
    typeName: "product",
    displayName: "Sản phẩm",
    category: "Identity",
    description: "Chọn sản phẩm của dự án: ảnh đã tách nền và hồ sơ sản phẩm để thử đồ và dựng video review",
    inputPorts: [],
    outputPorts: [
      { name: "product", portType: "product", description: "Hồ sơ sản phẩm", required: true },
      { name: "image", portType: "image", description: "Ảnh sản phẩm đã tách nền (ảnh được chọn)", required: true },
      { name: "images", portType: "image[]", description: "Các ảnh sản phẩm đã tách nền", required: true },
    ],
    defaultParams: { productId: "", variant: "" },
    mockable: true,
  },
  {
    typeName: "script",
    displayName: "Kịch bản phân cảnh",
    category: "Structure",
    description: "Biến kịch bản thành danh sách cảnh có prompt ảnh, chuyển động và lời thoại",
    inputPorts: [
      { name: "text", portType: "text", description: "Ý tưởng hoặc cốt truyện", required: true },
    ],
    outputPorts: [
      { name: "scenes", portType: "scenes", description: "Danh sách phân cảnh cấu trúc", required: true },
      { name: "prompt", portType: "text", description: "Prompt tạo ảnh/video theo cảnh", required: true },
      { name: "dialogue", portType: "text", description: "Lời thoại theo cảnh", required: true },
      { name: "lines", portType: "dialogue", description: "Lời thoại có người nói: từng câu kèm nhân vật, shot, mốc thời gian", required: true },
    ],
    defaultParams: { targetSceneCount: 6, targetDurationSec: 45 },
    mockable: true,
  },
  {
    typeName: "vision_analyze",
    displayName: "Phân tích Video / Ảnh",
    category: "Intelligence",
    description: "Phân tích cấu trúc video, góc quay, nhịp cảnh và lời thoại qua model nhìn",
    inputPorts: [
      { name: "video", portType: "video", description: "Video nguồn để phân tích", required: false },
      { name: "image", portType: "image", description: "Ảnh nguồn để phân tích", required: false },
    ],
    outputPorts: [
      { name: "script", portType: "script", description: "Kịch bản cấu trúc trích xuất", required: true },
      { name: "scenes", portType: "scenes", description: "Danh sách phân cảnh trích xuất", required: true },
    ],
    defaultParams: { goal: "extract_scenes_and_dialogue" },
    mockable: true,
  },
  {
    typeName: "quality_check",
    displayName: "Kiểm tra chất lượng (Gate)",
    category: "Quality",
    description: "Chấm điểm ảnh trước khi tốn tiền tạo video",
    inputPorts: [
      { name: "media", portType: "image", description: "Ảnh hoặc video cần thẩm định", required: false },
      { name: "criteria", portType: "text", description: "Tiêu chí thẩm định", required: false },
      { name: "text", portType: "text", description: "Lời nói / kịch bản / prompt cần kiểm chính sách", required: false },
      { name: "captions", portType: "captions", description: "Caption cần kiểm chính sách", required: false },
      { name: "scenes", portType: "scenes", description: "Danh sách cảnh cần kiểm", required: false },
      { name: "policy", portType: "policy", description: "Chính sách riêng (mặc định: chính sách của dự án)", required: false },
    ],
    outputPorts: [
      { name: "media", portType: "image", description: "Media đã qua kiểm duyệt", required: true },
      { name: "pass_fail", portType: "text", description: "Kết quả đạt/không đạt + điểm + vi phạm chính sách", required: true },
      { name: "violations", portType: "text", description: "Danh sách từ ngữ vi phạm chính sách", required: false },
    ],
    defaultParams: {
      thresholdScore: 0.85,
      maxRetries: 2,
      policyAutoScan: true,
      criteria: "Nhân vật Sóc Nhỏ 100cm, áo phông vàng, quần short đen, đuôi xù lớn, không chữ lạ",
    },
    mockable: true,
  },
  {
    typeName: "policy",
    displayName: "Chính sách nội dung",
    category: "Quality",
    description: "Chính sách của dự án (từ cấm, mức vi phạm) — nối vào node Kiểm tra chất lượng hoặc để mặc định theo dự án",
    inputPorts: [],
    outputPorts: [{ name: "policy", portType: "policy", description: "Chính sách nội dung đang áp dụng", required: true }],
    defaultParams: { extraTerms: [], extraSeverity: "block" },
    mockable: true,
  },
  {
    typeName: "image_gen",
    displayName: "Tạo ảnh AI",
    category: "Generation",
    description: "Tạo ảnh từ prompt mô tả kết hợp các ảnh tham chiếu nhận diện nhân vật",
    inputPorts: [
      { name: "prompt", portType: "text", description: "Prompt mô tả cảnh", required: true },
      { name: "references", portType: "image[]", description: "Ảnh tham chiếu DNA", required: false },
      { name: "scenes", portType: "scenes", description: "Lặp theo cảnh: nối danh sách cảnh để tạo mỗi cảnh 1 ảnh", required: false },
    ],
    outputPorts: [
      { name: "images", portType: "image[]", description: "Danh sách ảnh sinh ra", required: true },
      { name: "image", portType: "image", description: "Ảnh đại diện / khung đầu tiên sinh ra", required: true },
      { name: "scenes", portType: "scenes", description: "Danh sách cảnh đã gắn ảnh (khi lặp theo cảnh)", required: false },
    ],
    defaultParams: {
      model: "gemini-2.5-flash-image",
      aspectRatio: "9:16",
      resolution: "1080p",
      n: 1,
      seed: null,
    },
    mockable: true,
  },
  {
    typeName: "image_edit",
    displayName: "Sửa ảnh / Tách trang phục",
    category: "Generation",
    description: "Chỉnh sửa chi tiết hoặc thay đổi trang phục trên ảnh gốc",
    inputPorts: [
      { name: "image", portType: "image", description: "Ảnh gốc cần sửa", required: true },
      { name: "references", portType: "image[]", description: "Ảnh tham chiếu mới", required: false },
      { name: "prompt", portType: "text", description: "Mô tả chỉnh sửa", required: false },
    ],
    outputPorts: [
      { name: "image", portType: "image", description: "Ảnh kết quả sau sửa", required: true },
    ],
    defaultParams: { model: "gemini-2.5-flash-image", aspectRatio: "9:16" },
    mockable: true,
  },
  {
    typeName: "video_gen",
    displayName: "Tạo video chuyển động",
    category: "Generation",
    description: "Biến ảnh khung đầu thành video chuyển động 3D qua model Veo / LTX",
    inputPorts: [
      { name: "first_frame", portType: "image", description: "Ảnh khung đầu (First frame)", required: false },
      { name: "image", portType: "image", description: "Ảnh khung đầu (alias)", required: false },
      { name: "prompt", portType: "text", description: "Prompt chuyển động camera", required: false },
      { name: "references", portType: "image[]", description: "Ảnh tham chiếu phụ", required: false },
    ],
    outputPorts: [
      { name: "video", portType: "video", description: "Clip video sinh ra", required: true },
    ],
    defaultParams: {
      model: "google-veo-2",
      durationSec: 8,
      aspectRatio: "9:16",
      hasAudio: false,
    },
    mockable: true,
  },
  {
    typeName: "voice_tts",
    displayName: "Văn bản thành giọng nói",
    category: "Audio",
    description: "Đọc lời thoại kịch bản bằng giọng Sóc Nhỏ (soc-nho) chuẩn ngữ điệu cảm xúc",
    inputPorts: [
      { name: "text", portType: "text", description: "Lời thoại cần đọc", required: true },
      { name: "character", portType: "character", description: "Hồ sơ nhân vật chứa voiceId", required: false },
    ],
    outputPorts: [
      { name: "audio", portType: "audio", description: "Tệp âm thanh MP3 đã master -14 LUFS", required: true },
    ],
    defaultParams: {
      voiceId: "soc-nho",
      tempo: 1.1,
      mastering: true,
    },
    mockable: true,
  },
  {
    typeName: "captions",
    displayName: "Phụ đề Karaoke (Whisper)",
    category: "Audio",
    description: "Trích xuất phụ đề từng từ khớp mốc thời gian (Word-level timestamps)",
    inputPorts: [
      { name: "audio", portType: "audio", description: "Tệp âm thanh thoại", required: true },
      { name: "text", portType: "text", description: "Text đối chiếu chuẩn", required: false },
    ],
    outputPorts: [
      { name: "captions", portType: "captions", description: "Dữ liệu phụ đề mốc thời gian", required: true },
    ],
    defaultParams: { language: "vi", style: "kinetic_karaoke" },
    mockable: true,
  },
  {
    typeName: "hyperframes_render",
    displayName: "Dựng phim HyperFrames",
    category: "Composition",
    description: "Ghép các phân cảnh, chạy phụ đề karaoke, chèn BGM ducking chuẩn Safe Zone 9:16",
    inputPorts: [
      { name: "scenes", portType: "scenes", description: "Cấu trúc các cảnh", required: false },
      { name: "video", portType: "video", description: "Video clip đầu vào", required: false },
      { name: "audio", portType: "audio", description: "Thoại chính voiceover", required: false },
      { name: "captions", portType: "captions", description: "Phụ đề karaoke", required: false },
      { name: "bgm", portType: "audio", description: "Nhạc nền BGM", required: false },
    ],
    outputPorts: [
      { name: "video", portType: "video", description: "Video thành phẩm MP4", required: true },
    ],
    defaultParams: {
      aspectRatio: "9:16",
      template: "default_cartoon",
      safeZonePaddingTop: 180,
      safeZonePaddingBottom: 320,
      duckingLevel: 0.25,
    },
    mockable: true,
  },
  {
    typeName: "export",
    displayName: "Xuất & Phân phối",
    category: "Output",
    description: "Xuất video hoàn chỉnh vào thư mục output/ và tạo manifest đóng gói",
    inputPorts: [
      { name: "video", portType: "video", description: "Video cần xuất bản", required: true },
    ],
    outputPorts: [
      { name: "manifest", portType: "composition", description: "Manifest thành phẩm", required: true },
    ],
    defaultParams: {
      platforms: ["tiktok", "youtube"],
      outputFileName: "soc_nho_tap_01.mp4",
    },
    mockable: true,
  },
  {
    typeName: "template_render",
    displayName: "Dựng video theo Template",
    category: "Composition",
    description: "Đổ dữ liệu slot vào HyperFrames Template và render chuẩn đa nền tảng",
    inputPorts: [
      { name: "text", portType: "text", description: "Văn bản / Hook đầu vào", required: false },
      { name: "script", portType: "script", description: "Kịch bản phân cảnh", required: false },
      { name: "images", portType: "image[]", description: "Danh sách ảnh B-roll", required: false },
      { name: "image", portType: "image", description: "Ảnh đơn sản phẩm", required: false },
      { name: "video", portType: "video", description: "Clip video nguồn", required: false },
      { name: "audio", portType: "audio", description: "Thoại chính voiceover", required: false },
      { name: "reference_images", portType: "image[]", description: "Ảnh nhân vật KOC", required: false },
      { name: "product", portType: "product", description: "Hồ sơ sản phẩm", required: false },
    ],
    outputPorts: [
      { name: "video", portType: "video", description: "Video thành phẩm MP4", required: true },
    ],
    defaultParams: {
      template_id: "koc-review-v1",
      platforms: ["tiktok"],
      slot_mapping: {},
    },
    mockable: true,
  },
  {
    typeName: "platform_export",
    displayName: "Xuất đa nền tảng",
    category: "Output",
    description: "Tự động biến đổi kích thước video sang các tỷ lệ đa nền tảng",
    inputPorts: [
      { name: "video", portType: "video", description: "Video nguồn cần chuyển đổi", required: true },
    ],
    outputPorts: [
      { name: "video", portType: "video", description: "Video sau chuyển đổi", required: true },
    ],
    defaultParams: {
      platforms: ["tiktok"],
    },
    mockable: true,
  },
  {
    typeName: "character_image",
    displayName: "Nhân vật từ ảnh",
    category: "Identity",
    description: "Dùng một ảnh bất kỳ làm nhân vật tham chiếu, không cần tạo hồ sơ đầy đủ",
    inputPorts: [],
    outputPorts: [
      { name: "image", portType: "image", description: "Ảnh nhân vật", required: true },
      { name: "reference_images", portType: "image[]", description: "Ảnh tham chiếu nhân vật", required: true },
    ],
    defaultParams: { uri: "", name: "" },
    mockable: true,
  },
  {
    typeName: "scene",
    displayName: "Cảnh",
    category: "Structure",
    description: "Ghép một ảnh và lời thoại thành một cảnh để dựng phim",
    inputPorts: [
      { name: "image", portType: "image", description: "Ảnh của cảnh", required: true },
      { name: "text", portType: "text", description: "Lời thoại hoặc mô tả cảnh", required: false },
    ],
    outputPorts: [
      { name: "scenes", portType: "scenes", description: "Cảnh đã ghép", required: true },
      { name: "image", portType: "image", description: "Ảnh của cảnh", required: true },
    ],
    defaultParams: { title: "Cảnh mới", durationSec: 6 },
    mockable: true,
  },
  {
    typeName: "transition",
    displayName: "Chuyển cảnh",
    category: "Structure",
    description: "Đặt kiểu chuyển cảnh (mờ dần, trượt, cắt) giữa các cảnh",
    inputPorts: [
      { name: "scenes", portType: "scenes", description: "Các cảnh cần chuyển tiếp", required: true },
    ],
    outputPorts: [
      { name: "scenes", portType: "scenes", description: "Các cảnh kèm chuyển cảnh", required: true },
    ],
    defaultParams: { style: "fade", durationSec: 0.5 },
    mockable: true,
  },
  {
    typeName: "cast",
    displayName: "Dàn diễn viên",
    category: "Identity",
    description: "Nhận diện danh sách nhân vật cần cho tập phim từ kịch bản hoặc chọn thủ công",
    inputPorts: [
      { name: "lines", portType: "text", description: "Danh sách lời thoại", required: false },
      { name: "scenes", portType: "scenes", description: "Danh sách phân cảnh", required: false },
    ],
    outputPorts: [
      { name: "characters", portType: "character", description: "Thông tin nhân vật", required: true },
      { name: "reference_images", portType: "image[]", description: "Ảnh mẫu DNA nhân vật", required: true },
      { name: "voice", portType: "text", description: "Mã giọng đã ghép", required: true },
    ],
    defaultParams: { include: ["soc_nho"] },
    mockable: true,
  },
  {
    typeName: "bg_remove",
    displayName: "Tách nền ảnh (Toolbox)",
    category: "Structure",
    description: "Tách nền ảnh bằng mô hình u2net cục bộ không tốn phí",
    inputPorts: [
      { name: "image", portType: "image", description: "Ảnh đầu vào", required: true },
    ],
    outputPorts: [
      { name: "image", portType: "image", description: "Ảnh đã tách nền", required: true },
      { name: "mask", portType: "image", description: "Mặt nạ nhị phân", required: true },
    ],
    defaultParams: { model: "u2net", alpha_matting: false },
    mockable: true,
  },
  {
    typeName: "word_captions",
    displayName: "Phụ đề từng từ (Toolbox)",
    category: "Audio",
    description: "Nhận diện và dóng hàng thời gian từng từ cho audio qua Whisper",
    inputPorts: [
      { name: "audio", portType: "audio", description: "File âm thanh", required: true },
    ],
    outputPorts: [
      { name: "captions", portType: "captions", description: "File phụ đề SRT", required: true },
      { name: "words", portType: "text", description: "Dữ liệu thời gian từng từ JSON", required: true },
    ],
    defaultParams: { model_size: "small", language: "vi" },
    mockable: true,
  },
  {
    typeName: "scene_cut",
    displayName: "Tách cảnh video (Toolbox)",
    category: "Structure",
    description: "Phát hiện điểm chuyển cảnh và chia nhỏ video",
    inputPorts: [
      { name: "video", portType: "video", description: "File video", required: true },
    ],
    outputPorts: [
      { name: "scenes", portType: "scenes", description: "Danh sách các cảnh", required: true },
      { name: "report", portType: "text", description: "Báo cáo phân cảnh", required: true },
    ],
    defaultParams: { threshold: 3.0, min_scene_s: 1.0, export_clips: false, aspect: "keep", max_scenes: 200 },
    mockable: true,
  },
  {
    typeName: "depth_layers",
    displayName: "Tách lớp chiều sâu (Toolbox)",
    category: "Structure",
    description: "Phân tách ảnh thành tiền cảnh, trung cảnh, hậu cảnh và bản đồ độ sâu",
    inputPorts: [
      { name: "image", portType: "image", description: "Ảnh đầu vào", required: true },
    ],
    outputPorts: [
      { name: "bg", portType: "image", description: "Lớp hậu cảnh", required: true },
      { name: "mid", portType: "image", description: "Lớp trung cảnh", required: true },
      { name: "fg", portType: "image", description: "Lớp tiền cảnh", required: true },
      { name: "depth", portType: "image", description: "Bản đồ độ sâu", required: true },
    ],
    defaultParams: { fg_pct: 70, mid_pct: 35, feather: 3, max_side: 1280 },
    mockable: true,
  },
  {
    typeName: "handdraw",
    displayName: "Vẽ tay hoạt hình (Toolbox)",
    category: "Generation",
    description: "Tạo video giả lập bàn tay vẽ nét đen trên nền trắng",
    inputPorts: [
      { name: "image", portType: "image", description: "Ảnh phác thảo nét đen nền trắng", required: true },
    ],
    outputPorts: [
      { name: "video", portType: "video", description: "Video hoạt hoạ vẽ tay", required: true },
    ],
    defaultParams: { duration_sec: 5.0, fps: 24, width: 540, height: 960, show_hand: true },
    mockable: true,
  },
  {
    typeName: "voice_clone",
    displayName: "Nhân bản giọng (Toolbox)",
    category: "Audio",
    description: "Nhân bản giọng nói mẫu 3-5s vào kho giọng cục bộ",
    inputPorts: [
      { name: "audio", portType: "audio", description: "File mẫu giọng 3.5–6s", required: true },
    ],
    outputPorts: [
      { name: "voice_id", portType: "text", description: "Mã định danh giọng", required: true },
      { name: "preview", portType: "audio", description: "File nghe thử", required: true },
      { name: "report", portType: "text", description: "Báo cáo clone", required: true },
    ],
    defaultParams: { name: "", display_name: "", consent_note: "", dialect: "south", duration: 4.5, verify: true },
    mockable: true,
  },
  {
    typeName: "source_video",
    displayName: "Nguồn Video (Douyin/YT/Local)",
    category: "Input",
    description: "Trích xuất video, âm thanh, metadata từ liên kết hoặc file tải lên",
    inputPorts: [
      { name: "url", portType: "text", description: "URL video", required: false },
    ],
    outputPorts: [
      { name: "video", portType: "video", description: "Video nguồn", required: true },
      { name: "audio", portType: "audio", description: "Audio bóc tách", required: true },
      { name: "metadata", portType: "text", description: "Metadata video", required: true },
    ],
    defaultParams: { url: "", rights: "reference_only" },
    mockable: true,
  },
  {
    typeName: "localize",
    displayName: "Bản địa hoá (Douyin/TikTok -> VI)",
    category: "Intelligence",
    description: "Bản địa hoá video: nhận diện lời thoại, dịch, viết lại và lồng tiếng",
    inputPorts: [
      { name: "video", portType: "video", description: "Video nguồn", required: true },
      { name: "audio", portType: "audio", description: "Audio gốc", required: true },
    ],
    outputPorts: [
      { name: "script", portType: "script", description: "Kịch bản tiếng Việt", required: true },
      { name: "audio", portType: "audio", description: "Audio tiếng Việt", required: true },
      { name: "captions", portType: "captions", description: "Phụ đề tiếng Việt", required: true },
    ],
    defaultParams: { mode: "R", style: "faithful", voice_id: "soc-nho", auto_draft: true },
    mockable: true,
  },
  {
    typeName: "blueprint_analyze",
    displayName: "Phân tích Blueprint Video",
    category: "Intelligence",
    description: "Bóc tách cấu trúc video mẫu thành Hook, Problem, Demo, Proof, CTA",
    inputPorts: [
      { name: "video", portType: "video", description: "Video mẫu", required: true },
    ],
    outputPorts: [
      { name: "blueprint", portType: "text", description: "Dữ liệu Blueprint JSON", required: true },
    ],
    defaultParams: {},
    mockable: true,
  },
  {
    typeName: "script_from_blueprint",
    displayName: "Sinh kịch bản KOC/Affiliate",
    category: "Intelligence",
    description: "Áp dụng cấu trúc Blueprint cho sản phẩm và nhân vật",
    inputPorts: [
      { name: "blueprint", portType: "text", description: "Dữ liệu Blueprint mẫu", required: true },
      { name: "character", portType: "character", description: "Nhân vật KOC", required: false },
    ],
    outputPorts: [
      { name: "script", portType: "script", description: "Kịch bản hoàn chỉnh", required: true },
      { name: "render_request", portType: "text", description: "Cấu hình RenderRequest", required: true },
    ],
    defaultParams: { generator_type: "koc_review", voice_id: "soc-nho" },
    mockable: true,
  },
  {
    typeName: "channel_insight",
    displayName: "Phân tích Kênh Đối thủ",
    category: "Intelligence",
    description: "Bóc tách thống kê, pillars nội dung và công thức viral từ kênh",
    inputPorts: [
      { name: "channel_url", portType: "text", description: "URL kênh đối thủ", required: true },
    ],
    outputPorts: [
      { name: "blueprint", portType: "text", description: "Dữ liệu ChannelBlueprint", required: true },
      { name: "pillars", portType: "text", description: "Danh sách pillars nội dung", required: true },
      { name: "summary", portType: "text", description: "Tóm tắt phân tích", required: true },
    ],
    defaultParams: { max_videos: 50, channel_url: "" },
    mockable: true,
  }
];
