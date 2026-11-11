import type { StateCreator } from "zustand";
import type { Node, Edge } from "@xyflow/react";
import type {
  NodeDefinition,
  NodeStatus,
  GraphValidationResult,
  RunPolicy,
  RunPresetMode,
} from "@/lib/core-contracts";
import { RUN_PRESETS } from "@/lib/core-contracts";
import type { Episode } from "@/lib/repositories/types";
import {
  FALLBACK_NODE_TYPES,
  fetchGraphFromCore,
  fetchCustomNodeTypes,
  toCoreGraph,
  flushGraphToCore,
  saveActiveProjectToCore,
} from "@/lib/core-api";
import { outputPortsAffectedByPatch } from "@/lib/node-ports";
import { autoLayout, resolveOverlaps } from "@/lib/graph-layout";
import type { CanvasSlice, StudioNodeData } from "../types";
import { getNodeCollapseConfig, isNodeDefaultOpen } from "../types";
import { buildSocNhoStoryTemplate, genericizeTemplate, DEFAULT_EPISODES } from "../templates/socNhoStoryTemplate";
import { collectDownstreamIds, syncProjectWorkspace, inFlightGraphLoads } from "../utils";

type CanvasSet = Parameters<StateCreator<CanvasSlice, [], [], CanvasSlice>>[0];
type CanvasGet = Parameters<StateCreator<CanvasSlice, [], [], CanvasSlice>>[1];

export const createCastWorkflowSubSlice = (
  set: CanvasSet,
  get: CanvasGet
): Pick<
  CanvasSlice,
  "convertCharactersToCast" | "ensureCastForEpisode" | "expandCastToCharacters"
> => ({
  convertCharactersToCast: () => {
    const state = get();
    const chars = state.nodes.filter((n) => n.data.nodeType === "character");
    if (chars.length === 0 || state.nodes.some((n) => n.data.nodeType === "cast")) return false;
    const charIds = new Set(chars.map((n) => n.id));
    const script = state.nodes.find((n) => n.data.nodeType === "script");
    if (!script) return false;
    const castDef = state.nodeCatalog.find((d) => d.typeName === "cast");
    const castId = `node_cast_${Date.now().toString(36)}`;
    const first = chars[0];
    const castNode: Node<StudioNodeData> = {
      id: castId,
      type: "studioCard",
      position: { x: first.position.x, y: first.position.y },
      selected: true,
      data: {
        nodeType: "cast",
        status: "idle",
        isStale: true,
        params: { ...(castDef?.defaultParams || { speak: { enabled: true }, include: [], exclude: [], onMissingVoice: "stop" }) },
        generations: [],
      },
    };
    const mk = (source: string, sh: string, target: string, th: string): Edge => ({
      id: `edge_${Math.random().toString(36).slice(2, 9)}`,
      source,
      sourceHandle: sh,
      target,
      targetHandle: th,
      type: "default",
    });
    const kept = state.edges.filter((e) => !charIds.has(e.source) && !charIds.has(e.target));
    const rewired: Edge[] = [];
    const seen = new Set<string>();
    for (const e of state.edges) {
      if (!charIds.has(e.source) || charIds.has(e.target)) continue;
      // ảnh DNA và giọng của các nhân vật cũ -> nối từ Dàn diễn viên tới cùng đích
      if (e.sourceHandle === "reference_images" || e.sourceHandle === "voice") {
        const k = `${e.sourceHandle}>${e.target}:${e.targetHandle}`;
        if (!seen.has(k)) {
          seen.add(k);
          rewired.push(mk(castId, e.sourceHandle, e.target, e.targetHandle || e.sourceHandle));
        }
      }
    }
    const nextEdges = [...kept, mk(script.id, "lines", castId, "lines"), mk(script.id, "scenes", castId, "scenes"), ...rewired];
    const nextNodes = [...state.nodes.filter((n) => !charIds.has(n.id)).map((n) => ({ ...n, selected: false })), castNode];
    get().setNodesAndEdges(nextNodes, nextEdges, true);
    return true;
  },
  ensureCastForEpisode: (scriptNodeId) => {
    const state = get();
    if (state.nodes.some((n) => n.data.nodeType === "cast")) return false;
    if (state.nodes.some((n) => n.data.nodeType === "character")) return get().convertCharactersToCast();
    const script = state.nodes.find((n) => n.id === scriptNodeId);
    if (!script) return false;
    const castDef = state.nodeCatalog.find((d) => d.typeName === "cast");
    if (!castDef) return false;
    const castId = `node_cast_${Date.now().toString(36)}`;
    const castNode: Node<StudioNodeData> = {
      id: castId,
      type: "studioCard",
      position: { x: script.position.x + 380, y: script.position.y },
      selected: false,
      data: { nodeType: "cast", status: "idle", isStale: true, params: { ...castDef.defaultParams }, generations: [] },
    };
    const mk = (source: string, sh: string, target: string, th: string): Edge => ({
      id: `edge_${Math.random().toString(36).slice(2, 9)}`,
      source,
      sourceHandle: sh,
      target,
      targetHandle: th,
      type: "default",
    });
    const edges: Edge[] = [mk(scriptNodeId, "lines", castId, "lines"), mk(scriptNodeId, "scenes", castId, "scenes")];
    const img = state.nodes.find((n) => n.data.nodeType === "image_gen");
    const render = state.nodes.find((n) => n.data.nodeType === "hyperframes_render");
    if (img) edges.push(mk(castId, "reference_images", img.id, "references"));
    if (render) edges.push(mk(castId, "voice", render.id, "voice_clips"));
    get().setNodesAndEdges([...state.nodes, castNode], [...state.edges, ...edges], true);
    return true;
  },
  expandCastToCharacters: (castNodeId, characterIds) => {
    const state = get();
    const cast = state.nodes.find((n) => n.id === castNodeId);
    if (!cast || characterIds.length === 0) return false;
    const script = state.edges.find((e) => e.target === castNodeId && e.sourceHandle === "lines");
    const stamp = Date.now().toString(36);
    const newNodes: Node<StudioNodeData>[] = characterIds.map((cid, i) => ({
      id: `node_character_${stamp}_${i}`,
      type: "studioCard",
      position: { x: cast.position.x, y: cast.position.y + i * 760 },
      selected: false,
      data: {
        nodeType: "character",
        status: "idle",
        isStale: true,
        params: { characterId: cid, speak: { enabled: true } },
        generations: [],
      },
    }));
    const mk = (source: string, sh: string, target: string, th: string): Edge => ({
      id: `edge_${Math.random().toString(36).slice(2, 9)}`,
      source,
      sourceHandle: sh,
      target,
      targetHandle: th,
      type: "default",
    });
    const kept = state.edges.filter((e) => e.source !== castNodeId && e.target !== castNodeId);
    const out: Edge[] = [];
    for (const nn of newNodes) {
      if (script) out.push(mk(script.source, "lines", nn.id, "lines"));
      for (const e of state.edges) {
        if (e.source !== castNodeId) continue;
        if (e.sourceHandle === "reference_images" || e.sourceHandle === "voice") {
          out.push(mk(nn.id, e.sourceHandle, e.target, e.targetHandle || e.sourceHandle));
        }
      }
    }
    get().setNodesAndEdges(
      [...state.nodes.filter((n) => n.id !== castNodeId).map((n) => ({ ...n, selected: false })), ...newNodes],
      [...kept, ...out],
      true
    );
    return true;
  },
});
