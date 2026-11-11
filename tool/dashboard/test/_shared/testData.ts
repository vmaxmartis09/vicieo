import type { Node, Edge } from "@xyflow/react";
import type { StudioNodeData } from "@/store/slices/canvasSlice";

export const sample5Nodes: Node<StudioNodeData>[] = [
  {
    id: "node_script",
    type: "studioCard",
    position: { x: 100, y: 100 },
    data: {
      nodeType: "llm_script",
      label: "Kịch bản AI",
      status: "idle",
      params: { prompt: "Viết kịch bản về Sóc Nhỏ", _collapsed: false },
      generations: [],
    },
  },
  {
    id: "node_image",
    type: "studioCard",
    position: { x: 450, y: 100 },
    data: {
      nodeType: "image_gen",
      label: "Sinh ảnh nhân vật",
      status: "idle",
      params: { prompt: "Sóc Nhỏ 3D render", model: "flux-schnell", _collapsed: false },
      generations: [],
    },
  },
  {
    id: "node_video",
    type: "studioCard",
    position: { x: 800, y: 100 },
    data: {
      nodeType: "video_gen",
      label: "Tạo video",
      status: "idle",
      params: { model: "kling-v1.6", duration: 5, _collapsed: false },
      generations: [],
    },
  },
  {
    id: "node_tts",
    type: "studioCard",
    position: { x: 450, y: 400 },
    data: {
      nodeType: "tts",
      label: "Lồng tiếng",
      status: "idle",
      params: { voiceId: "soc-nho", _collapsed: false },
      generations: [],
    },
  },
  {
    id: "node_export",
    type: "studioCard",
    position: { x: 1150, y: 200 },
    data: {
      nodeType: "video_export",
      label: "Xuất bản video",
      status: "idle",
      params: { format: "mp4", resolution: "1080p", _collapsed: false },
      generations: [],
    },
  },
];

export const sampleEdges: Edge[] = [
  {
    id: "edge_script_to_image",
    source: "node_script",
    target: "node_image",
    sourceHandle: "script_text",
    targetHandle: "prompt",
  },
  {
    id: "edge_image_to_video",
    source: "node_image",
    target: "node_video",
    sourceHandle: "image_url",
    targetHandle: "image",
  },
  {
    id: "edge_script_to_tts",
    source: "node_script",
    target: "node_tts",
    sourceHandle: "script_text",
    targetHandle: "text",
  },
  {
    id: "edge_video_to_export",
    source: "node_video",
    target: "node_export",
    sourceHandle: "video_url",
    targetHandle: "video_stream",
  },
  {
    id: "edge_tts_to_export",
    source: "node_tts",
    target: "node_export",
    sourceHandle: "audio_url",
    targetHandle: "audio_stream",
  },
];
