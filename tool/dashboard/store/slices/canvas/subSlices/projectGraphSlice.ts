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
import type { CanvasSlice, StudioNodeData, ProjectWorkspace } from "../types";
import { getNodeCollapseConfig, isNodeDefaultOpen } from "../types";
import { buildSocNhoStoryTemplate, genericizeTemplate, DEFAULT_EPISODES } from "../templates/socNhoStoryTemplate";
import { collectDownstreamIds, syncProjectWorkspace, inFlightGraphLoads } from "../utils";

type CanvasSet = Parameters<StateCreator<CanvasSlice, [], [], CanvasSlice>>[0];
type CanvasGet = Parameters<StateCreator<CanvasSlice, [], [], CanvasSlice>>[1];

export const createProjectGraphSubSlice = (
  set: CanvasSet,
  get: CanvasGet
): Pick<
  CanvasSlice,
  | "loadProjectGraph"
  | "switchProject"
  | "setEpisodes"
  | "selectEpisode"
  | "createEpisode"
  | "refreshCustomNodeTypes"
  | "setNodeCatalog"
> => ({
  loadProjectGraph: async (projectId) => {
    const existingPromise = inFlightGraphLoads.get(projectId);
    if (existingPromise) return existingPromise;

    const task = (async () => {
      try {
        const s0 = get() as any;
        const epsBelongToProject =
          Array.isArray(s0.episodes) &&
          s0.episodes.length > 0 &&
          s0.episodes.every((ep: Episode) => !ep.projectId || ep.projectId === projectId);
        // Nếu người dùng vừa chỉnh sửa trước khi reload và chưa kịp lưu xuống Core -> đẩy bản mới nhất xuống Core trước
        if (
          s0.activeProjectId === projectId &&
          epsBelongToProject &&
          s0.saveState === "unsaved" &&
          Array.isArray(s0.nodes) &&
          s0.nodes.length > 0
        ) {
          await flushGraphToCore(
            projectId,
            toCoreGraph(projectId, s0.nodes, s0.edges || [])
          );
        }

        const g = await fetchGraphFromCore(projectId);
        if (!g) return; // Core không phản hồi: không đánh dấu đã nạp, tránh lưu đè
        const s = get() as any;
        if (s.activeProjectId !== projectId) return; // đã chuyển dự án khác trong lúc chờ
        if (g.nodes.length > 0) {
          const existingById = new Map<string, Node<StudioNodeData>>(
            (s.nodes || []).map((en: Node<StudioNodeData>) => [en.id, en])
          );
          const rawNodes: Node<StudioNodeData>[] = g.nodes.map((n: any) => {
            const prev = existingById.get(n.id)?.data;
            const ui = n.ui || {};
            const rawState = n.state || n.status || "idle";
            const normalizedStatus = (
              rawState === "completed" ? "succeeded" : rawState
            ) as NodeStatus;
            const rawImg = n.params?.image || n.params?.variant;
            const fallbackImgUrl = typeof rawImg === "string" && rawImg.trim()
              ? (rawImg.startsWith("http") ? rawImg : rawImg.startsWith("/") ? `${CORE_URL}${rawImg}` : `${CORE_URL}/data/projects/${projectId}/files/${rawImg}`)
              : undefined;
            const rawVid = n.params?.video;
            const fallbackVidUrl = typeof rawVid === "string" && rawVid.trim()
              ? (rawVid.startsWith("http") ? rawVid : rawVid.startsWith("/") ? `${CORE_URL}${rawVid}` : `${CORE_URL}/data/projects/${projectId}/files/${rawVid}`)
              : undefined;
            const resolvedPreviewUrl = ui.previewUrl ?? fallbackVidUrl ?? fallbackImgUrl ?? prev?.previewUrl;
            const resolvedPreviewType = ui.previewType ?? (fallbackVidUrl ? "video" : fallbackImgUrl ? "image" : prev?.previewType);

            return {
              id: n.id,
              type: "studioCard",
              position: n.position ?? { x: 0, y: 0 },
              data: {
                ...prev,
                nodeType: n.type,
                label: ui.label ?? prev?.label,
                status: normalizedStatus,
                isStale: Boolean(n.isStale),
                params: n.params || {},
                inputs: n.inputs || prev?.inputs || {},
                pinnedGenerationId: n.pinnedGenerationId ?? null,
                generations: n.generations || prev?.generations || [],
                previewUrl: resolvedPreviewUrl,
                previewType: resolvedPreviewType,
                summaryText: ui.summaryText ?? prev?.summaryText,
                inputSummary: ui.inputSummary ?? prev?.inputSummary,
                outputSummary: ui.outputSummary ?? prev?.outputSummary,
                errorMessage: ui.errorMessage ?? prev?.errorMessage,
                lastLog: ui.lastLog ?? prev?.lastLog,
                actualCostUsd: ui.actualCostUsd ?? prev?.actualCostUsd,
              },
            };
          });
          const edges: Edge[] = g.edges.map((e: any) => ({
            id: e.id,
            source: e.sourceNodeId,
            sourceHandle: e.sourcePort,
            target: e.targetNodeId,
            targetHandle: e.targetPort,
            type: "default",
          }));
          // Chỉ tự động xếp vị trí nếu các node chưa có tọa độ lưu sẵn (tất cả ở 0,0)
          const needsAutoLayout =
            rawNodes.length > 1 &&
            rawNodes.every(
              (n) => !n.position || (n.position.x === 0 && n.position.y === 0)
            );
          const nodes = resolveOverlaps(needsAutoLayout ? autoLayout(rawNodes, edges) : rawNodes);
          const nextGraphsByEpisode = {
            ...s.graphsByEpisode,
            [s.activeEpisodeId]: { nodes, edges },
          };
          const nextEpisodes = (s.episodes || []).map((ep: Episode) =>
            ep.id === s.activeEpisodeId ? { ...ep, nodeCount: nodes.length } : ep
          );
          set({
            nodes,
            edges,
            episodes: nextEpisodes,
            graphsByEpisode: nextGraphsByEpisode,
            projectWorkspaces: {
              ...s.projectWorkspaces,
              [projectId]: {
                episodes: nextEpisodes,
                activeEpisodeId: s.activeEpisodeId,
                graphsByEpisode: nextGraphsByEpisode,
                nodes,
                edges,
              },
            },
            loadedGraphs: { ...s.loadedGraphs, [projectId]: true },
            saveState: "saved",
          } as any);
        } else {
          const nextGraphsByEpisode = {
            ...s.graphsByEpisode,
            [s.activeEpisodeId]: { nodes: [], edges: [] },
          };
          const nextEpisodes = (s.episodes || []).map((ep: Episode) =>
            ep.id === s.activeEpisodeId ? { ...ep, nodeCount: 0 } : ep
          );
          set({
            nodes: [],
            edges: [],
            episodes: nextEpisodes,
            graphsByEpisode: nextGraphsByEpisode,
            projectWorkspaces: {
              ...s.projectWorkspaces,
              [projectId]: {
                episodes: nextEpisodes,
                activeEpisodeId: s.activeEpisodeId,
                graphsByEpisode: nextGraphsByEpisode,
                nodes: [],
                edges: [],
              },
            },
            loadedGraphs: { ...s.loadedGraphs, [projectId]: true },
            saveState: "saved",
          } as any);
        }
      } finally {
        inFlightGraphLoads.delete(projectId);
      }
    })();

    inFlightGraphLoads.set(projectId, task);
    return task;
  },
  switchProject: (projectId) => {
    const s = get() as any;
    if (!projectId) return;
    if (s.activeProjectId === projectId) {
      void saveActiveProjectToCore(projectId);
      if (!s.loadedGraphs[projectId]) {
        void get().loadProjectGraph(projectId);
      }
      return;
    }
    const prevProjectId = s.activeProjectId;
    if (prevProjectId && s.loadedGraphs[prevProjectId] && s.saveState === "unsaved") {
      void flushGraphToCore(
        prevProjectId,
        toCoreGraph(prevProjectId, s.nodes, s.edges)
      );
    }
    const current: ProjectWorkspace = {
      episodes: s.episodes,
      activeEpisodeId: s.activeEpisodeId,
      graphsByEpisode: {
        ...s.graphsByEpisode,
        [s.activeEpisodeId]: { nodes: s.nodes, edges: s.edges },
      },
      nodes: s.nodes,
      edges: s.edges,
    };
    const saved = { ...s.projectWorkspaces, [prevProjectId]: current };
    let next: ProjectWorkspace | undefined = saved[projectId];
    if (!next) {
      // Dự án chưa mở lần nào: một tập trống, canvas trống để chọn mẫu
      const epId = projectId === "soc-nho-tv" ? "ep_01_rang_sau" : `ep_${projectId}_01`;
      const epTitle = projectId === "soc-nho-tv" ? "Tập 1: Núi lửa trong bụng Bi" : "Tập 1";
      next = {
        episodes: [
          {
            id: epId,
            projectId,
            episodeNumber: 1,
            title: epTitle,
            status: "draft",
            nodeCount: 0,
            updatedAt: new Date().toISOString(),
          },
        ],
        activeEpisodeId: epId,
        graphsByEpisode: { [epId]: { nodes: [], edges: [] } },
        nodes: [],
        edges: [],
      };
    }
    set({
      activeProjectId: projectId,
      projectWorkspaces: { ...saved, [projectId]: next },
      episodes: next.episodes,
      activeEpisodeId: next.activeEpisodeId,
      graphsByEpisode: next.graphsByEpisode,
      nodes: next.nodes,
      edges: next.edges,
      selectedNodeId: null,
      undoStack: [],
      redoStack: [],
      saveState: "saved",
    } as any);
    void saveActiveProjectToCore(projectId);
    void get().loadProjectGraph(projectId);
  },
  setEpisodes: (episodes) =>
    set((state) => ({
      episodes,
      projectWorkspaces: syncProjectWorkspace(state as any, { episodes }),
    })),
  selectEpisode: (episodeId) =>
    set((state) => {
      const savedCurrent = {
        ...state.graphsByEpisode,
        [state.activeEpisodeId]: { nodes: state.nodes, edges: state.edges },
      };
      const nextGraph = savedCurrent[episodeId] || { nodes: [], edges: [] };
      return {
        activeEpisodeId: episodeId,
        graphsByEpisode: savedCurrent,
        nodes: nextGraph.nodes,
        edges: nextGraph.edges,
        selectedNodeId: nextGraph.nodes[0]?.id || null,
        undoStack: [],
        redoStack: [],
        saveState: "unsaved",
        projectWorkspaces: syncProjectWorkspace(state as any, {
          activeEpisodeId: episodeId,
          graphsByEpisode: savedCurrent,
          nodes: nextGraph.nodes,
          edges: nextGraph.edges,
        }),
      };
    }),
  createEpisode: (title, opts) => {
    const state = get();
    const nextNum = state.episodes.length + 1;
    const tpl = opts?.customGraph
      ? opts.customGraph
      : opts?.blank
      ? { nodes: [] as Node<StudioNodeData>[], edges: [] as Edge[] }
      : genericizeTemplate(
          buildSocNhoStoryTemplate(state.nodeCatalog),
          (state as any).activeProjectId || "soc-nho-tv"
        );
    const newEp: Episode = {
      id: `ep_0${nextNum}_${Date.now().toString(36)}`,
      projectId: (state as any).activeProjectId || "soc-nho-tv",
      episodeNumber: nextNum,
      title: title.trim() || `Tập ${nextNum}: Câu chuyện mới`,
      status: "draft",
      nodeCount: tpl.nodes.length,
      updatedAt: new Date().toISOString(),
    };
    const shouldSelect = opts?.autoSelect !== false;
    const nextEpisodes = [...state.episodes, newEp];
    const nextGraphsByEpisode = {
      ...state.graphsByEpisode,
      ...(shouldSelect
        ? { [state.activeEpisodeId]: { nodes: state.nodes, edges: state.edges } }
        : {}),
      [newEp.id]: tpl,
    };
    const nextActiveEpId = shouldSelect ? newEp.id : state.activeEpisodeId;
    const nextNodes = shouldSelect ? tpl.nodes : state.nodes;
    const nextEdges = shouldSelect ? tpl.edges : state.edges;
    set({
      episodes: nextEpisodes,
      ...(shouldSelect
        ? {
            activeEpisodeId: newEp.id,
            nodes: tpl.nodes,
            edges: tpl.edges,
            selectedNodeId: tpl.nodes[0]?.id || null,
            undoStack: [],
            redoStack: [],
            saveState: "unsaved",
          }
        : {}),
      graphsByEpisode: nextGraphsByEpisode,
      projectWorkspaces: syncProjectWorkspace(state as any, {
        episodes: nextEpisodes,
        activeEpisodeId: nextActiveEpId,
        graphsByEpisode: nextGraphsByEpisode,
        nodes: nextNodes,
        edges: nextEdges,
      }),
    });
    return newEp;
  },
  refreshCustomNodeTypes: async (projectId) => {
    const customs = await fetchCustomNodeTypes(projectId);
    const s = get() as any;
    const base = (s.nodeCatalog as NodeDefinition[]).filter((d) => !d.baseType);
    set({ nodeCatalog: [...base, ...customs] } as any);
  },
  setNodeCatalog: (nodeCatalog, nodeCatalogFromCore) =>
    set({ nodeCatalog, nodeCatalogFromCore }),
});
