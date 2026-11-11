import type { PublishableVideo, SocialAccount } from "@/lib/api/social";
import type { CompatibilityMatrix } from "../types";

export function computeCompatibilityMatrix(
  selectedVideos: PublishableVideo[],
  selectedAccounts: SocialAccount[]
): CompatibilityMatrix {
  const matrix: CompatibilityMatrix = {};
  for (const v of selectedVideos) {
    matrix[v.id] = {};
    for (const a of selectedAccounts) {
      if (a.status === "expired") {
        matrix[v.id][a.id] = {
          status: "error",
          message: `Tài khoản ${a.display_name} đã hết hạn OAuth`,
        };
        continue;
      }
      if (a.platform === "tiktok" && v.duration_s > 600) {
        matrix[v.id][a.id] = {
          status: "error",
          message: "Video dài hơn 600s của TikTok",
        };
        continue;
      }
      if (
        a.platform === "tiktok" &&
        v.aspect_ratio === "16:9" &&
        !v.variants["tiktok"] &&
        !v.variants["shorts"]
      ) {
        matrix[v.id][a.id] = {
          status: "warn",
          message: "Video 16:9 ngang; TikTok khuyên dùng dọc 9:16",
        };
        continue;
      }
      matrix[v.id][a.id] = { status: "ok", message: "Tương thích hoàn hảo" };
    }
  }
  return matrix;
}
