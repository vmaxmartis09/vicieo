import type { Node, Edge } from "@xyflow/react";
import type { NodeDefinition, NodeStatus, PortType } from "@/lib/core-contracts";
import type { Episode } from "@/lib/repositories/types";
import { FALLBACK_NODE_TYPES } from "@/lib/core-api";
import { autoLayout } from "@/lib/graph-layout";
import { PORT_TYPE_COLORS, type StudioNodeData } from "../types";

export function genericizeTemplate(
  tpl: { nodes: Node<StudioNodeData>[]; edges: Edge[] },
  projectId: string
): { nodes: Node<StudioNodeData>[]; edges: Edge[] } {
  if (projectId === "soc-nho-tv") return tpl;
  return {
    edges: tpl.edges,
    nodes: tpl.nodes.map((n) => {
      const params = { ...(n.data.params || {}) };
      if (n.data.nodeType === "text") params.content = "";
      if (n.data.nodeType === "character") params.characterId = "";
      if (n.data.nodeType === "export") params.outputFileName = `${projectId}_tap_01.mp4`;
      return { ...n, data: { ...n.data, params } };
    }),
  };
}

export function buildSocNhoStoryTemplate(
  catalog: NodeDefinition[] = FALLBACK_NODE_TYPES
): { nodes: Node<StudioNodeData>[]; edges: Edge[] } {
  const getDef = (t: string) =>
    catalog.find((d) => d.typeName === t) ||
    FALLBACK_NODE_TYPES.find((d) => d.typeName === t)!;

  const nodes: Node<StudioNodeData>[] = [
    {
      id: "node_text",
      type: "studioCard",
      position: { x: 20, y: 20 },
      data: {
        nodeType: "text",
        status: "idle",
        params: {
          ...getDef("text")?.defaultParams,
          content:
            "Tập 1: Sóc Nhỏ (Bi, 100cm) mê ăn kẹo dẻo hạt dẻ trước khi đi ngủ, bị đau răng sâu và được Bác sĩ Koala (150cm) khám răng, hướng dẫn đánh răng đúng cách.",
        },
        generations: [],
        nextStepHint: "Bước 1: Kiểm tra cốt truyện hoặc bấm Chạy cả chuỗi",
      },
    },
    {
      id: "node_script",
      type: "studioCard",
      position: { x: 380, y: 20 },
      data: {
        nodeType: "script",
        status: "idle",
        params: {
          ...getDef("script")?.defaultParams,
          targetSceneCount: 6,
          targetDurationSec: 45,
        },
        generations: [],
        nextStepHint: "Bước 2: Tách cốt truyện thành 6 cảnh có lời thoại",
      },
    },
    {
      id: "node_character",
      type: "studioCard",
      position: { x: 20, y: 640 },
      data: {
        nodeType: "character",
        status: "idle",
        params: {
          ...getDef("character")?.defaultParams,
          characterId: "soc_nho",
        },
        generations: [],
        nextStepHint: "Hồ sơ DNA Sóc Nhỏ (100cm) & giọng soc-nho",
      },
    },
    {
      id: "node_image_gen",
      type: "studioCard",
      position: { x: 740, y: 20 },
      data: {
        nodeType: "image_gen",
        status: "idle",
        params: {
          ...getDef("image_gen")?.defaultParams,
          model: "gemini-2.5-flash-image",
          aspectRatio: "9:16",
          n: 1,
        },
        generations: [],
        nextStepHint: "Bước 3: Sinh ảnh 3D Pixar theo DNA nhân vật",
      },
    },
    {
      id: "node_quality",
      type: "studioCard",
      position: { x: 380, y: 640 },
      data: {
        nodeType: "quality_check",
        status: "idle",
        params: {
          ...getDef("quality_check")?.defaultParams,
          thresholdScore: 0.85,
        },
        generations: [],
        nextStepHint: "Bước 4: Chấm điểm ảnh trước khi tạo video",
      },
    },
    {
      id: "node_video_gen",
      type: "studioCard",
      position: { x: 740, y: 640 },
      data: {
        nodeType: "video_gen",
        status: "idle",
        params: {
          ...getDef("video_gen")?.defaultParams,
          model: "google-veo-2",
          durationSec: 8,
          aspectRatio: "9:16",
        },
        generations: [],
        nextStepHint: "Bước 5: Tạo chuyển động video từ ảnh đạt chuẩn",
      },
    },
    {
      id: "node_voice",
      type: "studioCard",
      position: { x: 380, y: 1260 },
      data: {
        nodeType: "voice_tts",
        status: "idle",
        params: {
          ...getDef("voice_tts")?.defaultParams,
          voiceId: "soc-nho",
          tempo: 1.1,
        },
        generations: [],
        nextStepHint: "Bước 6: Đọc lời thoại bằng giọng Sóc Nhỏ (-14 LUFS)",
      },
    },
    {
      id: "node_render",
      type: "studioCard",
      position: { x: 740, y: 1260 },
      data: {
        nodeType: "hyperframes_render",
        status: "idle",
        params: {
          ...getDef("hyperframes_render")?.defaultParams,
          aspectRatio: "9:16",
          template: "default_cartoon",
        },
        generations: [],
        nextStepHint: "Bước 7: Ghép cảnh, phụ đề & nhạc nền HyperFrames",
      },
    },
    {
      id: "node_export",
      type: "studioCard",
      position: { x: 1100, y: 1260 },
      data: {
        nodeType: "export",
        status: "idle",
        params: {
          ...getDef("export")?.defaultParams,
          platforms: ["tiktok", "youtube"],
          outputFileName: "soc_nho_tap_01.mp4",
        },
        generations: [],
        nextStepHint: "Bước 8: Đóng gói MP4 & sẵn sàng lên lịch đăng",
      },
    },
  ];

  const makeEdge = (
    id: string,
    source: string,
    sourceHandle: string,
    target: string,
    targetHandle: string,
    portType: PortType
  ): Edge => ({
    id,
    source,
    sourceHandle,
    target,
    targetHandle,
    type: "default",
    style: {
      stroke: PORT_TYPE_COLORS[portType] || "#737373",
      strokeWidth: 1.75,
    },
    data: { portType },
  });

  const edges: Edge[] = [
    makeEdge("e_text_script", "node_text", "text", "node_script", "text", "text"),
    makeEdge(
      "e_script_img_prompt",
      "node_script",
      "prompt",
      "node_image_gen",
      "prompt",
      "text"
    ),
    makeEdge(
      "e_char_img_refs",
      "node_character",
      "reference_images",
      "node_image_gen",
      "references",
      "image[]"
    ),
    makeEdge(
      "e_img_quality",
      "node_image_gen",
      "image",
      "node_quality",
      "media",
      "image"
    ),
    makeEdge(
      "e_quality_vid",
      "node_quality",
      "media",
      "node_video_gen",
      "first_frame",
      "image"
    ),
    makeEdge(
      "e_script_vid_prompt",
      "node_script",
      "prompt",
      "node_video_gen",
      "prompt",
      "text"
    ),
    makeEdge(
      "e_script_voice",
      "node_script",
      "dialogue",
      "node_voice",
      "text",
      "text"
    ),
    makeEdge(
      "e_char_voice",
      "node_character",
      "character",
      "node_voice",
      "character",
      "character"
    ),
    makeEdge(
      "e_script_render_scenes",
      "node_script",
      "scenes",
      "node_render",
      "scenes",
      "scenes"
    ),
    makeEdge(
      "e_vid_render",
      "node_video_gen",
      "video",
      "node_render",
      "video",
      "video"
    ),
    makeEdge(
      "e_voice_render",
      "node_voice",
      "audio",
      "node_render",
      "audio",
      "audio"
    ),
    makeEdge(
      "e_render_export",
      "node_render",
      "video",
      "node_export",
      "video",
      "video"
    ),
  ];

  // Mặc định xếp gọn trái → phải theo thứ tự chạy, giảm dây chéo
  return { nodes: autoLayout(nodes, edges), edges };
}

export const DEFAULT_EPISODES: Episode[] = [
  {
    id: "ep_01_rang_sau",
    projectId: "soc-nho-tv",
    episodeNumber: 1,
    title: "Tập 1: Núi lửa trong bụng Bi",
    status: "draft",
    nodeCount: 0,
    updatedAt: "2026-11-05T16:00:00Z",
  },
];
