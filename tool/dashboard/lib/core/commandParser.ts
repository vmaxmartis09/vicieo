export function parseStudioCommandClientSide(prompt: string): {
  reply: string;
  mode: string;
  actions: any[];
  requiresConfirmation?: boolean;
  confirmationMeta?: any;
} {
  const p = prompt.toLowerCase().trim();
  const actions: any[] = [];
  let reply =
    "[Chế độ chạy thử — Trợ lý quy tắc] Đã ghi nhận lệnh. Bạn có thể gõ: 'thêm bước kiểm tra trước khi tạo video', 'đổi tỉ lệ sang 16:9', 'chạy từ cảnh 3', 'lên lịch đăng tập 1 lúc 19:00 lên YouTube', 'mở lịch đăng', hoặc 'xem nhân vật Koala'.";
  let requiresConfirmation = false;
  let confirmationMeta: any = null;

  if (p.includes("kiểm tra") || p.includes("quality") || p.includes("gate")) {
    actions.push({
      type: "add_node",
      nodeType: "quality_check",
      afterNodeType: "image_gen",
      beforeNodeType: "video_gen",
      summaryVi: "Đã thêm node Kiểm tra chất lượng (Gate) và nối với Tạo ảnh AI",
      summaryEn: "Added Quality Check (Gate) node and wired to AI Image Gen",
    });
    reply =
      "[Chế độ chạy thử] Đã thêm node Kiểm tra chất lượng (Gate) và nối giữa Tạo ảnh AI và Tạo video chuyển động.";
  } else if (p.includes("16:9") || p.includes("9:16") || p.includes("1:1") || p.includes("4:5")) {
    const ratio = p.includes("16:9") ? "16:9" : p.includes("1:1") ? "1:1" : p.includes("4:5") ? "4:5" : "9:16";
    actions.push({
      type: "set_aspect_ratio",
      aspectRatio: ratio,
      summaryVi: `Đã đổi tỉ lệ khung hình sang ${ratio}`,
      summaryEn: `Changed aspect ratio to ${ratio}`,
    });
    reply = `[Chế độ chạy thử] Đã chuyển tỉ lệ khung hình sang ${ratio}.`;
  } else if (p.includes("mở lịch") || p.includes("open schedule")) {
    actions.push({
      type: "open_tab",
      tab: "publish",
      summaryVi: "Đã mở tab Lịch đăng & Kênh",
      summaryEn: "Opened Schedule & Channels tab",
    });
    reply = "[Chế độ chạy thử] Đã mở tab Lịch đăng & Kênh ở khung bên phải.";
  } else if (p.includes("lên lịch") || p.includes("schedule") || p.includes("youtube") || p.includes("tiktok")) {
    const platform = p.includes("tiktok") ? "tiktok" : "youtube";
    const time = p.includes("20:00") ? "20:00" : "19:00";
    requiresConfirmation = true;
    confirmationMeta = {
      kind: "publish",
      platform,
      time,
      episodeTitle: "Tập 1: Sóc Nhỏ và Chiếc Răng Sâu",
    };
    actions.push({
      type: "schedule_post",
      platform,
      time,
      episodeTitle: "Tập 1: Sóc Nhỏ và Chiếc Răng Sâu",
      summaryVi: `Chuẩn bị lên lịch đăng Tập 1 lúc ${time} lên ${platform.toUpperCase()} (chờ xác nhận)`,
      summaryEn: `Prepared schedule for Episode 1 at ${time} on ${platform.toUpperCase()} (awaiting confirmation)`,
    });
    reply = `[Chế độ chạy thử] Lệnh lên lịch đăng bài lên ${platform.toUpperCase()} lúc ${time} cần bạn xác nhận trước khi thực hiện.`;
  } else if (p.includes("nhân vật") || p.includes("koala") || p.includes("sóc nhỏ") || p.includes("character")) {
    const characterId = p.includes("koala") ? "koala" : "soc_nho";
    actions.push({
      type: "open_character",
      tab: "project",
      characterId,
      summaryVi: `Đã mở hồ sơ nhân vật ${characterId === "koala" ? "Bác sĩ Koala" : "Sóc Nhỏ (Bi)"}`,
      summaryEn: `Opened character profile for ${characterId === "koala" ? "Dr. Koala" : "Little Squirrel (Bi)"}`,
    });
    reply = `[Chế độ chạy thử] Đã mở tab Dự án và hiển thị hồ sơ nhân vật ${characterId === "koala" ? "Bác sĩ Koala" : "Sóc Nhỏ (Bi)"}.`;
  } else if (p.includes("chạy") || p.includes("run")) {
    requiresConfirmation = true;
    confirmationMeta = {
      kind: "run",
      mode: p.includes("từ") || p.includes("from") ? "downstream" : "all",
      estimatedCostUsd: 0.0,
    };
    actions.push({
      type: "run_graph",
      mode: p.includes("từ") || p.includes("from") ? "downstream" : "all",
      summaryVi: "Yêu cầu chạy đồ thị (chờ xác nhận chi phí)",
      summaryEn: "Requested graph run (awaiting cost confirmation)",
    });
    reply = "[Chế độ chạy thử] Vui lòng xác nhận chi phí trước khi chạy đồ thị.";
  } else {
    actions.push({
      type: "add_node",
      nodeType: "quality_check",
      summaryVi: "Đã thêm node Kiểm tra chất lượng (Gate) vào đồ thị",
      summaryEn: "Added Quality Check (Gate) node to graph",
    });
  }

  return {
    reply,
    mode: "rule_based_mock",
    actions,
    requiresConfirmation,
    confirmationMeta,
  };
}

/** Đọc đồ thị đã lưu của dự án từ Core; null nếu Core không phản hồi. */
