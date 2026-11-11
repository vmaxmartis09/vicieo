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

export const createNodesEdgesSubSlice = (
  set: CanvasSet,
  get: CanvasGet
): Pick<
  CanvasSlice,
  | "setSelectedNodeId"
  | "setAspectRatio"
  | "setZoomLevel"
  | "setNodesAndEdges"
  | "setAllNodesCollapsed"
  | "toggleAllNodesCollapsed"
  | "updateNodeParams"
  | "updateNodeExecutionState"
  | "pinNodeGeneration"
  | "markDownstreamStale"
> => ({
  setSelectedNodeId: (id) =>
    set((state) => {
      const nextNodes = state.nodes.map((n) => {
        const shouldBeSelected = Boolean(id && n.id === id);
        const nextZ = shouldBeSelected ? 1000 : 0;
        return n.selected !== shouldBeSelected || (n.zIndex ?? 0) !== nextZ
          ? { ...n, selected: shouldBeSelected, zIndex: nextZ }
          : n;
      });
      return {
        selectedNodeId: id,
        nodes: nextNodes,
      };
    }),
  setAspectRatio: (aspectRatio) =>
    set((state) => ({
      aspectRatio,
      saveState: "unsaved",
      nodes: state.nodes.map((n) =>
        n.data.params && "aspectRatio" in n.data.params
          ? {
              ...n,
              data: {
                ...n.data,
                params: { ...n.data.params, aspectRatio },
              },
            }
          : n
      ),
    })),
  setZoomLevel: (zoomLevel) => set({ zoomLevel }),
  setNodesAndEdges: (nodes, edges, recordHistory = true) =>
    set((state) => {
      const nextUndo = recordHistory
        ? [...state.undoStack.slice(-24), { nodes: state.nodes, edges: state.edges }]
        : state.undoStack;
      const nextGraphsByEpisode = {
        ...state.graphsByEpisode,
        [state.activeEpisodeId]: { nodes, edges },
      };
      const nextEpisodes = state.episodes.map((ep) =>
        ep.id === state.activeEpisodeId
          ? { ...ep, nodeCount: nodes.length, updatedAt: new Date().toISOString() }
          : ep
      );
      return {
        nodes,
        edges,
        undoStack: nextUndo,
        redoStack: recordHistory ? [] : state.redoStack,
        saveState: "unsaved",
        graphsByEpisode: nextGraphsByEpisode,
        episodes: nextEpisodes,
        projectWorkspaces: syncProjectWorkspace(state as any, {
          episodes: nextEpisodes,
          graphsByEpisode: nextGraphsByEpisode,
          nodes,
          edges,
        }),
      };
    }),
  setAllNodesCollapsed: (collapsed: boolean) =>
    set((state) => {
      const nextNodes = state.nodes.map((n) => {
        const cfg = getNodeCollapseConfig(n.data.nodeType);
        if (cfg.alwaysOpen) return n;
        return {
          ...n,
          data: {
            ...n.data,
            params: {
              ...n.data.params,
              _collapsed: collapsed,
            },
          },
        };
      });
      const nextGraphsByEpisode = {
        ...state.graphsByEpisode,
        [state.activeEpisodeId]: { nodes: nextNodes, edges: state.edges },
      };
      return {
        nodes: nextNodes,
        saveState: "unsaved",
        graphsByEpisode: nextGraphsByEpisode,
        projectWorkspaces: syncProjectWorkspace(state as any, {
          graphsByEpisode: nextGraphsByEpisode,
          nodes: nextNodes,
        }),
      };
    }),
  toggleAllNodesCollapsed: () => {
    const s = get() as any;
    const collapsable = (s.nodes || []).filter(
      (n: any) => !getNodeCollapseConfig(n.data.nodeType).alwaysOpen
    );
    const areAllCollapsed =
      collapsable.length > 0 &&
      collapsable.every((n: any) => n.data.params?._collapsed === true);
    s.setAllNodesCollapsed(!areAllCollapsed);
  },
  updateNodeParams: (nodeId, patch) =>
    set((state) => {
      // Lỗi thời theo từng cổng ra: sửa tham số của 1 chức năng (vd. "Nói") chỉ làm lỗi thời
      // các node nối từ cổng ra của chức năng đó; sửa tham số khác -> mọi node phía sau như cũ.
      const editedNode = state.nodes.find((n) => n.id === nodeId);
      const editedDef = editedNode
        ? state.nodeCatalog.find((d) => d.typeName === editedNode.data.nodeType)
        : undefined;
      const affectedPorts = outputPortsAffectedByPatch(editedDef, patch);
      // Thay đổi UI như thu gọn / mở rộng không làm hỏng kết quả chạy của các node phía sau
      const isUiOnlyPatch =
        Object.keys(patch).length > 0 &&
        Object.keys(patch).every((k) => k === "_collapsed");
      const downstreamIds = isUiOnlyPatch
        ? new Set<string>()
        : collectDownstreamIds(nodeId, state.edges, affectedPorts);
      const nextNodes = state.nodes.map((n) => {
        if (n.id === nodeId) {
          return {
            ...n,
            data: {
              ...n.data,
              params: { ...n.data.params, ...patch },
            },
          };
        }
        if (downstreamIds.has(n.id) && n.data.status === "succeeded") {
          return {
            ...n,
            data: {
              ...n.data,
              isStale: true,
            },
          };
        }
        return n;
      });
      const nextGraphsByEpisode = {
        ...state.graphsByEpisode,
        [state.activeEpisodeId]: { nodes: nextNodes, edges: state.edges },
      };
      return {
        nodes: nextNodes,
        saveState: "unsaved",
        graphsByEpisode: nextGraphsByEpisode,
        projectWorkspaces: syncProjectWorkspace(state as any, {
          graphsByEpisode: nextGraphsByEpisode,
          nodes: nextNodes,
        }),
      };
    }),
  updateNodeExecutionState: (nodeId, patch) =>
    set((state) => {
      const nextNodes = state.nodes.map((n) =>
        n.id === nodeId ? { ...n, data: { ...n.data, ...patch } } : n
      );
      const nextGraphsByEpisode = {
        ...state.graphsByEpisode,
        [state.activeEpisodeId]: { nodes: nextNodes, edges: state.edges },
      };
      return {
        nodes: nextNodes,
        graphsByEpisode: nextGraphsByEpisode,
        projectWorkspaces: syncProjectWorkspace(state as any, {
          graphsByEpisode: nextGraphsByEpisode,
          nodes: nextNodes,
        }),
      };
    }),
  pinNodeGeneration: (nodeId, generationId) =>
    set((state) => {
      const nextNodes = state.nodes.map((n) => {
        if (n.id !== nodeId) return n;
        const gen = n.data.generations.find((g) => g.id === generationId);
        return {
          ...n,
          data: {
            ...n.data,
            pinnedGenerationId: generationId,
            previewUrl: gen?.previewUrl || n.data.previewUrl,
            previewType: gen?.previewType || n.data.previewType,
            summaryText: gen?.summaryText || n.data.summaryText,
          },
        };
      });
      const nextGraphsByEpisode = {
        ...state.graphsByEpisode,
        [state.activeEpisodeId]: { nodes: nextNodes, edges: state.edges },
      };
      return {
        nodes: nextNodes,
        saveState: "unsaved",
        graphsByEpisode: nextGraphsByEpisode,
        projectWorkspaces: syncProjectWorkspace(state as any, {
          graphsByEpisode: nextGraphsByEpisode,
          nodes: nextNodes,
        }),
      };
    }),
  markDownstreamStale: (sourceNodeId) =>
    set((state) => {
      const downstreamIds = collectDownstreamIds(sourceNodeId, state.edges);
      const nextNodes = state.nodes.map((n) =>
        downstreamIds.has(n.id)
          ? { ...n, data: { ...n.data, isStale: true } }
          : n
      );
      const nextGraphsByEpisode = {
        ...state.graphsByEpisode,
        [state.activeEpisodeId]: { nodes: nextNodes, edges: state.edges },
      };
      return {
        nodes: nextNodes,
        graphsByEpisode: nextGraphsByEpisode,
        projectWorkspaces: syncProjectWorkspace(state as any, {
          graphsByEpisode: nextGraphsByEpisode,
          nodes: nextNodes,
        }),
      };
    }),
});
