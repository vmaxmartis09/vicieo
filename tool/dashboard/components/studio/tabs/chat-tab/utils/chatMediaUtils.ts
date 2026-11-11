import type { Node } from "@xyflow/react";
import type { StudioNodeData } from "@/store/slices/canvasSlice";
import type { Character } from "@/lib/repositories/types";
import type { CanvasImageItem } from "@/components/studio/chat/SophisticatedChatInput";

export function extractCanvasImages(
  nodes: Node<StudioNodeData>[],
  characters: Character[]
): CanvasImageItem[] {
  const list: CanvasImageItem[] = [];
  for (const n of nodes) {
    if (n.data.nodeType === "image") {
      const urls = Array.isArray(n.data.params?.urls)
        ? (n.data.params.urls as string[]).filter((u): u is string => typeof u === "string" && !!u)
        : [];
      const singleUrl = (n.data.params?.url as string) || urls[0];
      if (singleUrl) {
        list.push({
          id: n.id,
          nodeId: n.id,
          name: (n.data.params?._customName as string) || (urls.length > 1 ? `List Ảnh (${urls.length})` : `Ảnh #${n.id.slice(-4)}`),
          url: singleUrl,
          kind: urls.length > 1 ? "image_list" : "image",
        });
      }
    } else if (n.data.nodeType === "image_gen") {
      const genUrl = (n.data.previewUrl as string) || n.data.generations?.[0]?.previewUrl || "";
      if (genUrl) {
        list.push({
          id: n.id,
          nodeId: n.id,
          name: (n.data.params?._customName as string) || `AI Image #${n.id.slice(-4)}`,
          url: genUrl,
          kind: "image",
        });
      }
    }
  }
  for (const ch of characters) {
    const refUrl = ch.referenceImages?.[0];
    if (refUrl) {
      list.push({
        id: `char_${ch.id}`,
        nodeId: ch.id,
        name: ch.name,
        url: refUrl,
        kind: "character",
      });
    }
  }
  return list;
}

export function buildQuickPrompts(opts: {
  nodesLength: number;
  activeCharName?: string;
  vi: boolean;
  addGateLabel: string;
  ratio169Label: string;
}): string[] {
  const { nodesLength, activeCharName, vi, addGateLabel, ratio169Label } = opts;
  const out: string[] = [];
  if (nodesLength === 0) out.push(vi ? "/flow Sóc Nhỏ đi tìm kho báu" : "/flow Squirrel treasure hunt");
  else out.push("/run");
  out.push(
    vi ? "/video Chuyển động camera mượt từ ảnh" : "/video Smooth camera motion from image",
    vi ? "/grid Storyboard 4 góc máy nhân vật" : "/grid 4-angle character storyboard"
  );
  if (activeCharName) out.push(vi ? `Làm ${activeCharName} hiền hơn` : `Make ${activeCharName} gentler`);
  else out.push(vi ? "/char bác sĩ cú mèo 140cm, hiền" : "/char owl doctor 140cm, gentle");
  out.push(addGateLabel, ratio169Label);
  return out;
}
