import { create } from "zustand";
import { persist } from "zustand/middleware";
import { createUISlice, type UISlice } from "./slices/uiSlice";
import {
  createWorkspaceSlice,
  type WorkspaceSlice,
} from "./slices/workspaceSlice";
import { createCanvasSlice, type CanvasSlice } from "./slices/canvasSlice";
import { createJobsSlice, type JobsSlice } from "./slices/jobsSlice";
import {
  createRightPanelSlice,
  type RightPanelSlice,
} from "./slices/rightPanelSlice";
import { createChatSlice, type ChatSlice } from "./slices/chatSlice";
import {
  createUserSettingsSlice,
  type UserSettingsSlice,
} from "./slices/userSettingsSlice";

export type StudioStore = UISlice &
  WorkspaceSlice &
  CanvasSlice &
  JobsSlice &
  RightPanelSlice &
  ChatSlice &
  UserSettingsSlice;

export const useStudioStore = create<StudioStore>()(
  persist(
    (...a) => ({
      ...createUISlice(...a),
      ...createWorkspaceSlice(...a),
      ...createCanvasSlice(...a),
      ...createJobsSlice(...a),
      ...createRightPanelSlice(...a),
      ...createChatSlice(...a),
      ...createUserSettingsSlice(...a),
    }),
    {
      name: "ai_video_studio_settings_v2",
      partialize: (state) => ({
        theme: state.theme,
        locale: state.locale,
        isMockMode: state.isMockMode,
        runPolicy: state.runPolicy,
        spendingLimitUsd: state.spendingLimitUsd,
        perRunSpendingLimitUsd: state.perRunSpendingLimitUsd,
        dailySpendingLimitUsd: state.dailySpendingLimitUsd,
        userProfile: state.userProfile,
        storageBackend: state.storageBackend,
        apiKeyStatus: state.apiKeyStatus,
        activeRightTab: state.activeRightTab,
        rightPanelWidth: state.rightPanelWidth,
        activeProjectId: state.activeProjectId,
        aspectRatio: state.aspectRatio,
        episodes: state.episodes,
        activeEpisodeId: state.activeEpisodeId,
        nodes: state.nodes,
        edges: state.edges,
        graphsByEpisode: state.graphsByEpisode,
        projectWorkspaces: state.projectWorkspaces,
        saveState: state.saveState,
      }),
      // Panel nổi luôn khởi động ở trạng thái đóng, kể cả khi bản lưu cũ đang mở
      merge: (persisted, current) => {
        const p = (persisted || {}) as Partial<StudioStore>;
        const normalizeNodes = (list?: any[]) =>
          Array.isArray(list)
            ? list.map((n) => {
                const st = n?.data?.status;
                const nextStatus =
                  st === "completed"
                    ? "succeeded"
                    : st === "running" || st === "queued"
                    ? "idle"
                    : st || "idle";
                return n?.data
                  ? { ...n, data: { ...n.data, status: nextStatus } }
                  : n;
              })
            : [];
        const merged: StudioStore = {
          ...current,
          ...p,
          perRunSpendingLimitUsd:
            typeof p.perRunSpendingLimitUsd === "number" && p.perRunSpendingLimitUsd >= 10.0
              ? p.perRunSpendingLimitUsd
              : 10.0,
          dailySpendingLimitUsd:
            typeof p.dailySpendingLimitUsd === "number" && p.dailySpendingLimitUsd >= 50.0
              ? p.dailySpendingLimitUsd
              : 50.0,
          spendingLimitUsd:
            typeof p.spendingLimitUsd === "number" && p.spendingLimitUsd >= 50.0
              ? p.spendingLimitUsd
              : 50.0,
          activeRightTab:
            p.activeRightTab === "chat"
              ? "project"
              : p.activeRightTab ?? current.activeRightTab,
          nodes: normalizeNodes(p.nodes ?? current.nodes),
          rightPanelCollapsed: true,
          loadedGraphs: {},
          isRunningGraph: false,
        };
        const pid = merged.activeProjectId || "soc-nho-tv";
        const ws = merged.projectWorkspaces?.[pid];
        const epsMatchProject =
          Array.isArray(merged.episodes) &&
          merged.episodes.length > 0 &&
          merged.episodes.every((ep) => !ep.projectId || ep.projectId === pid);
        if (!epsMatchProject) {
          if (ws && Array.isArray(ws.episodes) && ws.episodes.length > 0) {
            merged.episodes = ws.episodes;
            merged.activeEpisodeId = ws.activeEpisodeId || ws.episodes[0].id;
            merged.graphsByEpisode = ws.graphsByEpisode || {};
            merged.nodes = normalizeNodes(ws.nodes);
            merged.edges = ws.edges || [];
          } else {
            const epId = pid === "soc-nho-tv" ? "ep_01_rang_sau" : `ep_${pid}_01`;
            const epTitle =
              pid === "soc-nho-tv" ? "Tập 1: Núi lửa trong bụng Bi" : "Tập 1";
            merged.episodes = [
              {
                id: epId,
                projectId: pid,
                episodeNumber: 1,
                title: epTitle,
                status: "draft",
                nodeCount: merged.nodes?.length || 0,
                updatedAt: new Date().toISOString(),
              },
            ];
            merged.activeEpisodeId = epId;
            merged.graphsByEpisode = {
              [epId]: { nodes: merged.nodes || [], edges: merged.edges || [] },
            };
          }
        }
        return merged;
      },
    }
  )
);
