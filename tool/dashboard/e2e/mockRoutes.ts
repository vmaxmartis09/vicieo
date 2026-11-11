import type { Page } from "@playwright/test";
import { FALLBACK_NODE_TYPES } from "../lib/core/nodeDefinitions";

const PROJECT_ID = "e2e-fixture";

export const FIXTURE_GRAPH = {
  id: `graph_${PROJECT_ID}`,
  projectId: PROJECT_ID,
  name: "E2E 5-Node Characterization Fixture",
  updatedAt: "2026-11-09T00:00:00.000Z",
  nodes: [
    {
      id: "node_script",
      type: "script",
      position: { x: 80, y: 120 },
      params: { title: "Tập 1: Kịch bản chuẩn", prompt: "Sóc Nhỏ đi siêu thị", _collapsed: false },
      inputs: {},
      ui: { label: "Kịch bản AI" },
      state: "idle",
      isStale: false,
      generations: [],
    },
    {
      id: "node_image",
      type: "image_gen",
      position: { x: 460, y: 100 },
      params: { prompt: "Sóc Nhỏ 3D Pixar", model: "flux-schnell", aspectRatio: "9:16", _collapsed: true },
      inputs: {},
      ui: { label: "Sinh ảnh nhân vật" },
      state: "idle",
      isStale: false,
      generations: [],
    },
    {
      id: "node_tts",
      type: "voice_tts",
      position: { x: 460, y: 340 },
      params: { voiceId: "soc-nho", _collapsed: true },
      inputs: {},
      ui: { label: "Lồng tiếng Sóc Nhỏ" },
      state: "idle",
      isStale: false,
      generations: [],
    },
    {
      id: "node_video",
      type: "video_gen",
      position: { x: 860, y: 100 },
      params: { model: "kling-v3-std", durationSec: 5, aspectRatio: "9:16", _collapsed: true },
      inputs: {},
      ui: { label: "Tạo video chuyển động" },
      state: "idle",
      isStale: false,
      generations: [],
    },
    {
      id: "node_export",
      type: "export",
      position: { x: 1240, y: 200 },
      params: { format: "mp4", resolution: "1080p", _collapsed: true },
      inputs: {},
      ui: { label: "Xuất bản video" },
      state: "idle",
      isStale: false,
      generations: [],
    },
  ],
  edges: [
    { id: "e_script_image", sourceNodeId: "node_script", sourcePort: "scenes", targetNodeId: "node_image", targetPort: "prompt" },
    { id: "e_script_tts", sourceNodeId: "node_script", sourcePort: "lines", targetNodeId: "node_tts", targetPort: "dialogue" },
    { id: "e_image_video", sourceNodeId: "node_image", sourcePort: "image", targetNodeId: "node_video", targetPort: "image" },
    { id: "e_video_export", sourceNodeId: "node_video", sourcePort: "video", targetNodeId: "node_export", targetPort: "video" },
    { id: "e_tts_export", sourceNodeId: "node_tts", sourcePort: "audio", targetNodeId: "node_export", targetPort: "audio" },
  ],
};

const FIXTURE_TEMPLATES = [
  {
    id: "tpl_sales_e2e",
    name: "Review Sản Phẩm Sóc Nhỏ",
    version: 1,
    category: "sales",
    platforms: ["tiktok", "shorts"],
    default_platform: "tiktok",
    duration: { mode: "range", min: 15, max: 45 },
    slots: [{ key: "title", type: "text", label: "Tiêu đề ngắn", required: true, default: "Món ngon hôm nay" }],
    voice: { required: true, default_voice_id: "soc-nho" },
    captions: { style: "tiktok", position: "bottom" },
    bgm: { mood: "upbeat", duck_db: -12 },
    scenes: [{ role: "hook", dur: "3s" }],
    responsive: {},
  },
];

const TRANSPARENT_PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAC0lEQVR4nGNgAAIAAAUAAXpeqz8AAAAASUVORK5CYII=",
  "base64"
);

export async function installCoreMocks(page: Page): Promise<void> {
  await page.route(/^http:\/\/(127\.0\.0\.1|localhost):(8000|8103)\/.*/, async (route) => {
    const req = route.request();
    const url = new URL(req.url());
    const p = url.pathname;
    const method = req.method();

    const json = (data: unknown) =>
      route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(data) });

    if (p.endsWith("/preview.jpg")) {
      return route.fulfill({ status: 200, contentType: "image/png", body: TRANSPARENT_PNG });
    }
    if (p.endsWith("/preview.mp4")) {
      return route.fulfill({ status: 204, body: "" });
    }
    if (p.endsWith("/events")) {
      return route.fulfill({ status: 200, contentType: "text/event-stream", body: ": connected\n\n" });
    }

    if (p === "/health") {
      return json({ status: "healthy", version: "e2e-mock", repo_root: "/mock" });
    }
    if (p === "/node-types") {
      return json(FALLBACK_NODE_TYPES);
    }
    if (p === "/custom-node-types") {
      return json([]);
    }
    if (p === "/video/templates") {
      return json(FIXTURE_TEMPLATES);
    }
    if (p === "/video/platforms") {
      return json([
        {
          id: "tiktok",
          name: "TikTok",
          width: 1080,
          height: 1920,
          aspect_ratio: "9:16",
          fps: 30,
          safe_zone: { top: 180, bottom: 320, left: 40, right: 40 },
          loudness_lufs: -14,
          audio_bitrate: "192k",
          video_bitrate: "6M",
          codec: "h264",
          notes: "",
        },
      ]);
    }
    if (p === "/voices") {
      return json([{ id: "soc-nho", name: "Sóc Nhỏ (Bi)", display_name: "Sóc Nhỏ", status: "ready" }]);
    }
    if (p === "/projects" && method === "GET") {
      return json({ active_project: PROJECT_ID });
    }
    if (p === "/data/projects") {
      return json([{ id: PROJECT_ID, name: "E2E Fixture Project", aspectRatio: "9:16" }]);
    }
    if (p.endsWith("/channels") || p.endsWith("/characters") || p.endsWith("/scheduled_posts") || p.endsWith("/runs") || p.endsWith("/outputs")) {
      return json([]);
    }
    if (p.endsWith("/runs/active")) {
      return json(null);
    }
    if (p.endsWith("/library")) {
      return json({ episodes: [{ id: "ep_01_fixture", title: "Tập 1: Kịch bản chuẩn" }], scripts: [] });
    }
    if (p.endsWith("/graph") && method === "GET") {
      return json(FIXTURE_GRAPH);
    }
    if (p.endsWith("/estimate")) {
      return json({ totalEstimatedCostUsd: 0.12 });
    }
    if (p.endsWith("/social/overview")) {
      return json({
        summary: {
          upcoming_24h_count: 0,
          overdue_count: 0,
          failed_count: 0,
          pending_approval_count: 0,
        },
        drafts_count: 0,
        token_expiring_accounts: [],
        empty_slots_7d: [],
        accounts_count: 1,
        connected_accounts_count: 1,
        videos_count: 1,
        scheduled_count: 0,
        published_count: 0,
        failed_count: 0,
        recent_posts: [],
        upcoming_targets: [],
      });
    }
    if (p.endsWith("/social/accounts")) {
      return json([
        {
          id: "acc_e2e_tt",
          platform: "tiktok",
          handle: "@socnhotv",
          display_name: "Sóc Nhỏ TikTok",
          auth: { mode: "oauth" },
          status: "connected",
          default_hashtags: ["#socnho"],
          default_caption_footer: "",
        },
      ]);
    }
    if (p.endsWith("/social/videos")) {
      return json([
        {
          id: "vid_e2e_1",
          title: "Video Fixture 9:16",
          aspect_ratio: "9:16",
          duration_s: 32,
          size_mb: 12.4,
          posted_count: 0,
          thumb: "",
          variants: { tiktok: "/mock/vid_1.mp4" },
        },
      ]);
    }
    if (p.endsWith("/social/queue")) {
      return json({
        summary: {
          accounts_count: 1,
          max_accounts: 5,
          draft_count: 0,
          pending_approval_count: 0,
          overdue_count: 0,
          upcoming_count: 0,
          published_count: 0,
          failed_count: 0,
        },
        items: [],
      });
    }
    if (p.includes("/social/calendar")) {
      return json({ items: [], notes: [] });
    }
    if (p.endsWith("/social/analytics/learnings")) {
      return json({ learnings_md: "", recommendations: [] });
    }
    if (p.endsWith("/social/analytics/overview")) {
      return json({ kpis: {}, views_by_day: {}, outliers: { top_outliers: [], underperforming: [] } });
    }
    return json({ ok: true });
  });
}
