import type { RunPolicy } from "../core-contracts";

export function estimateNodeCostUsd(
  typeName: string,
  params: Record<string, any>,
  isMockMode: boolean,
  runPolicy?: RunPolicy
): number {
  if (isMockMode) return 0.0;
  if (runPolicy) {
    if (typeName === "image_gen" || typeName === "image_edit") {
      if (runPolicy.image === "mock") return 0.0;
    }
    if (typeName === "video_gen") {
      if (runPolicy.video === "off" || runPolicy.video === "mock") return 0.0;
    }
    if (typeName === "tts" || typeName === "voice_clone") {
      if (runPolicy.voice === "local" || runPolicy.voice === "mock") return 0.0;
    }
  }
  const model = String(params.model || "").toLowerCase();
  // Nguồn free / local theo hợp đồng C1/C3
  if (
    model.includes("pollinations") ||
    model.includes("cloudflare") ||
    model.includes("mflux") ||
    model.includes("local") ||
    model === "mock-image" ||
    model === "mock-video"
  ) {
    return 0.0;
  }
  if (typeName === "image_gen" || typeName === "image_edit") {
    const n = Number(params.n || 1);
    const m = String(params.model || "gemini-2.5-flash-image");
    const rate = m.includes("flux-schnell")
      ? 0.003
      : m.includes("flux-pro")
        ? 0.04
        : m.includes("kontext")
          ? 0.04
          : 0.03;
    return Number((rate * n).toFixed(4));
  }
  if (typeName === "video_gen") {
    const dur = Number(params.durationSec || 8);
    const m = String(params.model || "google-veo-2");
    const rate = m.includes("ltx")
      ? 0.04
      : m.includes("kling")
        ? 0.084
        : m.includes("veo-3")
          ? 0.4
          : 0.15;
    return Number((dur * rate).toFixed(4));
  }
  if (typeName === "quality_check" || typeName === "vision_analyze") {
    return 0.005;
  }
  return 0.0;
}
