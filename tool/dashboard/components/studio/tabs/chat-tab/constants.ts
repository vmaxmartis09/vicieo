export interface ChatSlashCommand {
  cmd: string;
  hint: string;
}

export function buildSlashCommands(vi: boolean): ChatSlashCommand[] {
  return [
    { cmd: "/video", hint: vi ? "Tạo Video từ ảnh tham chiếu / ý tưởng (xuất Node Video)" : "Generate Video from reference image / idea" },
    { cmd: "/image", hint: vi ? "Tạo Node Ảnh AI trên Canvas từ mô tả / ảnh gốc" : "Generate AI Image node on Canvas" },
    { cmd: "/grid", hint: vi ? "Tạo Node List Ảnh / Multi-Angle 4 góc máy trên Canvas" : "Create Multi-Angle Image List node on Canvas" },
    { cmd: "/flow", hint: vi ? "Tạo flow mới từ mẫu theo ý tưởng" : "New flow from template + idea" },
    { cmd: "/blank", hint: vi ? "Tạo flow trắng tinh" : "Create an empty flow" },
    { cmd: "/char", hint: vi ? "Tạo nhân vật mới từ mô tả (khóa tỉ lệ)" : "Create a character from description" },
    { cmd: "/run", hint: vi ? "Chạy flow hiện tại (tạo job)" : "Run current flow (creates a job)" },
    { cmd: "/clear", hint: vi ? "Xóa hội thoại" : "Clear chat" },
  ];
}
