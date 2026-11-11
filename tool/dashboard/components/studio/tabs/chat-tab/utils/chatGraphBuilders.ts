import type { Edge, Node } from "@xyflow/react";
import { FALLBACK_NODE_TYPES } from "@/lib/core-api";
import { PORT_TYPE_COLORS, type StudioNodeData } from "@/store/slices/canvasSlice";
import type { ChatAttachment } from "@/components/studio/chat/SophisticatedChatInput";
import type { DirectMediaMode } from "../types";

export function buildDirectMediaGraphUpdate(opts: {
  mode: DirectMediaMode;
  promptBody: string;
  currentNodes: Node<StudioNodeData>[];
  currentEdges: Edge[];
  craftTags: string[];
  attachments: ChatAttachment[];
  aspectRatio: "9:16" | "16:9" | "1:1" | "4:5";
  chatModel: string;
  vi: boolean;
}) {
  const { mode, promptBody, currentNodes, currentEdges, craftTags, attachments, aspectRatio, chatModel, vi } = opts;
  const fullPrompt = [promptBody, ...craftTags].filter(Boolean).join(", ") || "Cinematic scene";
  const lastNode = currentNodes[currentNodes.length - 1];
  const baseX = lastNode ? lastNode.position.x + 280 : 320;
  const baseY = lastNode ? lastNode.position.y : 220;

  const nextNodes: Node<StudioNodeData>[] = currentNodes.map((n) => ({ ...n, selected: false }));
  const nextEdges: Edge[] = [...currentEdges];
  let sourceImageNodeId: string | undefined = attachments.find((a) => a.nodeId)?.nodeId;

  const uploadedUrls = attachments.filter((a) => a.source === "upload").map((a) => a.url);
  if (uploadedUrls.length > 0) {
    const imgNodeId = `node_image_${Date.now().toString(36).slice(-4)}`;
    nextNodes.push({
      id: imgNodeId,
      type: "studioCard",
      position: { x: baseX, y: baseY },
      selected: mode === "image" && !promptBody,
      data: {
        nodeType: "image",
        status: "succeeded",
        params: {
          _customName:
            uploadedUrls.length > 1
              ? vi
                ? `List Ảnh tải lên (${uploadedUrls.length})`
                : `Uploaded Image List (${uploadedUrls.length})`
              : attachments[0]?.name || (vi ? "Ảnh tham chiếu" : "Reference Image"),
          url: uploadedUrls[0],
          urls: uploadedUrls,
          aspectRatio,
        },
        generations: [],
        nextStepHint: vi ? "Ảnh tham chiếu từ thanh Composer" : "Reference image from Composer",
      },
    });
    sourceImageNodeId = imgNodeId;
  }

  const posX = baseX + (uploadedUrls.length > 0 ? 280 : 0);
  const ts = Date.now().toString(36).slice(-4);
  let targetNodeId = "";
  let reply = "";
  let summaryVi = "";
  let summaryEn = "";

  if (mode === "image_list") {
    targetNodeId = `node_imglist_${ts}`;
    const seedBase = encodeURIComponent(fullPrompt.slice(0, 32) || "storyboard");
    const baseUrls =
      attachments.length >= 2
        ? attachments.map((a) => a.url)
        : [
            attachments[0]?.url || `https://picsum.photos/seed/${seedBase}_1/540/960`,
            `https://picsum.photos/seed/${seedBase}_2/540/960`,
            `https://picsum.photos/seed/${seedBase}_3/540/960`,
            `https://picsum.photos/seed/${seedBase}_4/540/960`,
          ];
    nextNodes.push({
      id: targetNodeId,
      type: "studioCard",
      position: { x: posX, y: baseY },
      selected: true,
      data: {
        nodeType: "image",
        status: "succeeded",
        params: {
          _customName: vi ? "Storyboard 4 Góc Máy (Multi-Angle)" : "4-Angle Storyboard",
          urls: baseUrls,
          url: baseUrls[0],
          aspectRatio,
          storyboardGrid: true,
        },
        generations: [],
        nextStepHint: vi ? "Tạo từ lệnh /grid trên Chat" : "Created from /grid command",
      },
    });
    reply = vi
      ? `Đã tạo Node List Ảnh Multi-Angle với 4 góc máy dựa trên ý tưởng: "${fullPrompt}".`
      : `Created Multi-Angle Image List node with 4 frames based on: "${fullPrompt}".`;
    summaryVi = "Đã tạo Node List Ảnh trên Canvas";
    summaryEn = "Created Image List Node on Canvas";
  } else if (mode === "video") {
    targetNodeId = `node_video_${ts}`;
    nextNodes.push({
      id: targetNodeId,
      type: "studioCard",
      position: { x: posX, y: baseY },
      selected: true,
      data: {
        nodeType: "video_gen",
        status: "idle",
        params: {
          prompt: fullPrompt,
          model: chatModel.includes("kling") || chatModel.includes("veo") ? chatModel : "kling-v3-std",
          aspectRatio,
          duration: 5,
          cameraMovement: craftTags.find((t) => t.startsWith("Cam:"))?.replace("Cam: ", "") || "Pan smooth",
          hasReferenceImage: Boolean(sourceImageNodeId),
        },
        generations: [],
        nextStepHint: vi ? "Node Video sinh từ ảnh tham chiếu" : "Video node generated from reference",
      },
    });
    reply = vi
      ? `Đã tạo Node Video AI (#${targetNodeId.slice(-4)}) nối dây từ ảnh tham chiếu với prompt: "${fullPrompt}". Bấm "Chạy" để kết xuất.`
      : `Created Video node (#${targetNodeId.slice(-4)}) linked to reference image. Prompt: "${fullPrompt}".`;
    summaryVi = "Đã tạo Node Video nối dây trên Canvas";
    summaryEn = "Created linked Video Node on Canvas";
  } else {
    targetNodeId = `node_imggen_${ts}`;
    nextNodes.push({
      id: targetNodeId,
      type: "studioCard",
      position: { x: posX, y: baseY },
      selected: true,
      data: {
        nodeType: "image_gen",
        status: "idle",
        params: { prompt: fullPrompt, model: "fal-flux-1.1-pro", aspectRatio },
        generations: [],
        nextStepHint: vi ? "Node Tạo Ảnh sinh từ Composer" : "Image Gen node from Composer",
      },
    });
    reply = vi
      ? `Đã tạo Node Tạo Ảnh AI (#${targetNodeId.slice(-4)}) với prompt: "${fullPrompt}".`
      : `Created Image Gen node (#${targetNodeId.slice(-4)}) with prompt: "${fullPrompt}".`;
    summaryVi = "Đã tạo Node Ảnh trên Canvas";
    summaryEn = "Created Image Node on Canvas";
  }

  if (sourceImageNodeId) {
    nextEdges.push({
      id: `edge_${sourceImageNodeId}_${targetNodeId}`,
      source: sourceImageNodeId,
      target: targetNodeId,
      sourceHandle: "image",
      targetHandle: "image",
    });
  }

  return { nextNodes, nextEdges, targetNodeId, reply, summaryVi, summaryEn };
}

export function buildAddNodeGraphUpdate(
  currentNodes: Node<StudioNodeData>[],
  currentEdges: Edge[],
  action: { nodeType?: string; afterNodeType?: string },
  vi: boolean
) {
  const nodeType = action.nodeType || "quality_check";
  const def = FALLBACK_NODE_TYPES.find((d) => d.typeName === nodeType) || FALLBACK_NODE_TYPES[0];
  const newId = `node_${nodeType}_${Date.now().toString(36).slice(-3)}`;

  const afterNode =
    currentNodes.find((n) => n.data.nodeType === (action.afterNodeType || "image_gen")) ||
    currentNodes[currentNodes.length - 1];
  const pos = afterNode
    ? { x: afterNode.position.x + 170, y: afterNode.position.y + 120 }
    : { x: 600, y: 200 };

  const newNode: Node<StudioNodeData> = {
    id: newId,
    type: "studioCard",
    position: pos,
    selected: true,
    data: {
      nodeType: def.typeName,
      status: "idle" as const,
      params: { ...(def.defaultParams || {}) },
      generations: [],
      nextStepHint: vi ? "Được thêm từ trợ lý Chat" : "Added from Chat assistant",
    },
  };

  const nextEdges = [...currentEdges];
  if (afterNode && def.inputPorts.length > 0) {
    nextEdges.push({
      id: `e_chat_${Date.now().toString(36)}`,
      source: afterNode.id,
      sourceHandle: afterNode.data.nodeType === "image_gen" ? "image" : "text",
      target: newId,
      targetHandle: def.inputPorts[0].name,
      type: "default",
      style: {
        stroke: PORT_TYPE_COLORS[def.inputPorts[0].portType] || "#1b6c80",
        strokeWidth: 1.75,
      },
    });
  }

  return {
    nextNodes: [...currentNodes.map((n) => ({ ...n, selected: false })), newNode],
    nextEdges,
    targetNodeId: newId,
  };
}
