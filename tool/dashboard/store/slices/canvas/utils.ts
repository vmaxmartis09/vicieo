import type { Edge } from "@xyflow/react";
import type { CanvasSlice, ProjectWorkspace } from "./types";

export function collectDownstreamIds(
  startNodeId: string,
  edges: Edge[],
  /** Chỉ đi theo dây xuất phát từ các cổng này của node đầu (null = mọi cổng) */
  startPorts: Set<string> | null = null
): Set<string> {
  const visited = new Set<string>();
  const queue = [startNodeId];
  while (queue.length > 0) {
    const curr = queue.shift()!;
    for (const e of edges) {
      if (
        curr === startNodeId &&
        startPorts &&
        !startPorts.has(e.sourceHandle || "")
      ) {
        continue;
      }
      if (e.source === curr && !visited.has(e.target)) {
        visited.add(e.target);
        queue.push(e.target);
      }
    }
  }
  return visited;
}

export function syncProjectWorkspace(
  state: CanvasSlice & { activeProjectId?: string },
  patch: Partial<ProjectWorkspace>
): Record<string, ProjectWorkspace> {
  const pid = state.activeProjectId || "soc-nho-tv";
  const prev = state.projectWorkspaces[pid] || {
    episodes: state.episodes,
    activeEpisodeId: state.activeEpisodeId,
    graphsByEpisode: state.graphsByEpisode,
    nodes: state.nodes,
    edges: state.edges,
  };
  return {
    ...state.projectWorkspaces,
    [pid]: {
      ...prev,
      ...patch,
    },
  };
}

export const inFlightGraphLoads = new Map<string, Promise<void>>();
