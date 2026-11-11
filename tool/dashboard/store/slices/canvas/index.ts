import type { StateCreator } from "zustand";
import { RUN_PRESETS } from "@/lib/core-contracts";
import { FALLBACK_NODE_TYPES } from "@/lib/core-api";
import type { CanvasSlice } from "./types";
import { DEFAULT_EPISODES } from "./templates/socNhoStoryTemplate";
import { createProjectGraphSubSlice } from "./subSlices/projectGraphSlice";
import { createNodesEdgesSubSlice } from "./subSlices/nodesEdgesSlice";
import { createCastWorkflowSubSlice } from "./subSlices/castWorkflowSlice";
import { createHistoryExecutionSubSlice } from "./subSlices/historyExecutionSlice";

export * from "./types";
export * from "./templates/socNhoStoryTemplate";
export * from "./utils";

export const createCanvasSlice: StateCreator<
  CanvasSlice,
  [],
  [],
  CanvasSlice
> = (set, get) => ({
  selectedNodeId: null,
  aspectRatio: "9:16",
  zoomLevel: 1.0,
  isMockMode: false,
  runPolicy: RUN_PRESETS.animatic,
  episodes: DEFAULT_EPISODES,
  activeEpisodeId: "ep_01_rang_sau",
  nodeCatalog: FALLBACK_NODE_TYPES,
  nodeCatalogFromCore: false,
  nodes: [],
  edges: [],
  graphsByEpisode: {
    ep_01_rang_sau: { nodes: [], edges: [] },
  },
  undoStack: [],
  redoStack: [],
  saveState: "saved",
  isRunningGraph: false,
  hasActiveRun: false,
  setHasActiveRun: (hasActiveRun) => set({ hasActiveRun }),
  validationResult: null,
  projectWorkspaces: {},
  loadedGraphs: {},
  ...createProjectGraphSubSlice(set, get),
  ...createNodesEdgesSubSlice(set, get),
  ...createCastWorkflowSubSlice(set, get),
  ...createHistoryExecutionSubSlice(set, get),
});
