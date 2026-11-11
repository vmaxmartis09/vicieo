import type { PendingChatConfirmation } from "@/store/slices/chatSlice";

export function buildRunConfirmation(
  activeEpisodeTitle: string | undefined,
  estCost: number,
  isMockMode: boolean,
  promptText: string
): PendingChatConfirmation {
  return {
    id: `conf_${Date.now().toString(36)}`,
    kind: "run",
    titleVi: `Chạy flow “${activeEpisodeTitle || "hiện tại"}”`,
    titleEn: `Run flow “${activeEpisodeTitle || "current"}”`,
    detailVi: `Sẽ tạo 1 job. Chi phí ước tính: $${estCost.toFixed(2)} (${isMockMode ? "chạy thử $0" : "API thật"}).`,
    detailEn: `Creates 1 job. Estimated cost: $${estCost.toFixed(2)} (${isMockMode ? "mock $0" : "real API"}).`,
    estimatedCostUsd: estCost,
    mode: "all",
    prompt: promptText,
  };
}

export function buildPendingConfirmationFromMeta(
  meta: { kind?: string; platform?: string; time?: string; episodeTitle?: string; mode?: string },
  promptText: string,
  estCost: number,
  isMockMode: boolean
): PendingChatConfirmation {
  const isPublish = meta.kind === "publish";
  const platformLabel = (meta.platform || "youtube").toUpperCase();
  return {
    id: `conf_${Date.now().toString(36)}`,
    kind: isPublish ? "publish" : "run",
    titleVi: isPublish
      ? `Xác nhận lên lịch đăng lên ${platformLabel}`
      : "Xác nhận chạy đồ thị từ trợ lý Chat",
    titleEn: isPublish
      ? `Confirm scheduling on ${platformLabel}`
      : "Confirm running graph from Chat assistant",
    detailVi: isPublish
      ? `Bài đăng: "${meta.episodeTitle || "Tập 1"}" lúc ${meta.time || "19:00"} trên nền tảng ${platformLabel}.`
      : `Chế độ chạy: ${meta.mode || "all"}. Chi phí ước tính: $${estCost.toFixed(2)} USD (${isMockMode ? "chạy thử $0" : "API thật"}).`,
    detailEn: isPublish
      ? `Post: "${meta.episodeTitle || "Episode 1"}" at ${meta.time || "19:00"} on ${platformLabel}.`
      : `Run mode: ${meta.mode || "all"}. Estimated cost: $${estCost.toFixed(2)} USD.`,
    estimatedCostUsd: estCost,
    platform: meta.platform,
    time: meta.time,
    episodeTitle: meta.episodeTitle,
    mode: meta.mode,
    prompt: promptText,
  };
}

export function buildScheduledPostFromConfirmation(activeProjectId: string, conf: PendingChatConfirmation) {
  return {
    id: `sched_${Date.now().toString(36)}`,
    projectId: activeProjectId,
    episodeId: "ep_01_rang_sau",
    episodeTitle: conf.episodeTitle || "Tập 1: Sóc Nhỏ và Chiếc Răng Sâu",
    channelId: `ch_${conf.platform || "youtube"}`,
    channelName: `${(conf.platform || "youtube").toUpperCase()} Sóc Nhỏ TV`,
    platform: (conf.platform as any) || "youtube",
    scheduledAt: `Hàng ngày · ${conf.time || "19:00"}`,
    status: "pending_approval" as const,
    manualApprovalRequired: true,
  };
}

export function buildJobFromConfirmation(opts: {
  activeProjectId: string;
  chatModel: string;
  estimatedCostUsd?: number;
  isMockMode: boolean;
  episodeLabel: string;
}) {
  return {
    id: `job_${Date.now().toString(36)}`,
    projectId: opts.activeProjectId,
    type: "render" as const,
    model: opts.chatModel,
    status: "completed" as const,
    progress: 100,
    estimatedCostUsd: opts.estimatedCostUsd || 0,
    actualCostUsd: opts.isMockMode ? 0 : opts.estimatedCostUsd || 0,
    logs: [`Flow: ${opts.episodeLabel}`, "Từ trợ lý Chat"],
    createdAt: new Date().toISOString(),
    completedAt: new Date().toISOString(),
  };
}
