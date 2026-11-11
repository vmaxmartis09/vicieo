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

export const createHistoryExecutionSubSlice = (
  set: CanvasSet,
  get: CanvasGet
): Pick<
  CanvasSlice,
  | "setIsMockMode"
  | "setRunPolicy"
  | "setRunPreset"
  | "pushUndoSnapshot"
  | "undoCanvas"
  | "redoCanvas"
  | "loadCanvasTemplate"
  | "setSaveState"
  | "setIsRunningGraph"
  | "setValidationResult"
> => ({
  setIsMockMode: (isMockMode) =>
    set(() => {
      const newPolicy = isMockMode ? RUN_PRESETS.quick : RUN_PRESETS.animatic;
      return { isMockMode, runPolicy: newPolicy };
    }),
  setRunPolicy: (policyPatch) =>
    set((state) => {
      const newPolicy = { ...state.runPolicy, ...policyPatch };
      const isAllMock =
        newPolicy.image === "mock" &&
        newPolicy.video === "mock" &&
        newPolicy.text === "mock" &&
        newPolicy.voice === "mock";
      return { runPolicy: newPolicy, isMockMode: isAllMock };
    }),
  setRunPreset: (preset) =>
    set(() => {
      if (preset === "custom") return {};
      const newPolicy = RUN_PRESETS[preset];
      const isAllMock = preset === "quick";
      return { runPolicy: newPolicy, isMockMode: isAllMock };
    }),
  pushUndoSnapshot: () =>
    set((state) => ({
      undoStack: [
        ...state.undoStack.slice(-24),
        { nodes: state.nodes, edges: state.edges },
      ],
      redoStack: [],
    })),
  undoCanvas: () =>
    set((state) => {
      if (state.undoStack.length === 0) return state;
      const previous = state.undoStack[state.undoStack.length - 1];
      const nextUndo = state.undoStack.slice(0, -1);
      const nextGraphsByEpisode = {
        ...state.graphsByEpisode,
        [state.activeEpisodeId]: previous,
      };
      return {
        nodes: previous.nodes,
        edges: previous.edges,
        undoStack: nextUndo,
        redoStack: [
          ...state.redoStack,
          { nodes: state.nodes, edges: state.edges },
        ],
        saveState: "unsaved",
        graphsByEpisode: nextGraphsByEpisode,
        projectWorkspaces: syncProjectWorkspace(state as any, {
          graphsByEpisode: nextGraphsByEpisode,
          nodes: previous.nodes,
          edges: previous.edges,
        }),
      };
    }),
  redoCanvas: () =>
    set((state) => {
      if (state.redoStack.length === 0) return state;
      const next = state.redoStack[state.redoStack.length - 1];
      const nextRedo = state.redoStack.slice(0, -1);
      const nextGraphsByEpisode = {
        ...state.graphsByEpisode,
        [state.activeEpisodeId]: next,
      };
      return {
        nodes: next.nodes,
        edges: next.edges,
        undoStack: [
          ...state.undoStack,
          { nodes: state.nodes, edges: state.edges },
        ],
        redoStack: nextRedo,
        saveState: "unsaved",
        graphsByEpisode: nextGraphsByEpisode,
        projectWorkspaces: syncProjectWorkspace(state as any, {
          graphsByEpisode: nextGraphsByEpisode,
          nodes: next.nodes,
          edges: next.edges,
        }),
      };
    }),
  loadCanvasTemplate: (template) =>
    set((state) => {
      const nextGraph =
        template === "soc_nho_story"
          ? genericizeTemplate(buildSocNhoStoryTemplate(state.nodeCatalog), (state as any).activeProjectId)
          : {
              nodes: [
                {
                  id: "node_text_1",
                  type: "studioCard",
                  position: { x: 120, y: 140 },
                  data: {
                    nodeType: "text",
                    status: "idle" as NodeStatus,
                    params: {
                      content: "Nhập ý tưởng kịch bản cho tập mới...",
                    },
                    generations: [],
                    nextStepHint: "Bấm dấu + ở cổng ra để thêm bước tiếp theo",
                  },
                },
              ],
              edges: [],
            };
      const nextGraphsByEpisode = {
        ...state.graphsByEpisode,
        [state.activeEpisodeId]: nextGraph,
      };
      const nextEpisodes = state.episodes.map((ep) =>
        ep.id === state.activeEpisodeId
          ? { ...ep, nodeCount: nextGraph.nodes.length }
          : ep
      );
      return {
        nodes: nextGraph.nodes,
        edges: nextGraph.edges,
        selectedNodeId: nextGraph.nodes[0]?.id || null,
        undoStack: [
          ...state.undoStack.slice(-24),
          { nodes: state.nodes, edges: state.edges },
        ],
        redoStack: [],
        saveState: "unsaved",
        graphsByEpisode: nextGraphsByEpisode,
        episodes: nextEpisodes,
        projectWorkspaces: syncProjectWorkspace(state as any, {
          episodes: nextEpisodes,
          graphsByEpisode: nextGraphsByEpisode,
          nodes: nextGraph.nodes,
          edges: nextGraph.edges,
        }),
      };
    }),
  setSaveState: (saveState) => set({ saveState }),
  setIsRunningGraph: (isRunningGraph) => set({ isRunningGraph }),
  setValidationResult: (validationResult) => set({ validationResult }),
});
