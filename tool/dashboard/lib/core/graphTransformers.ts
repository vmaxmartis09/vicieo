import type { Graph } from "../core-contracts";

export function normalizeGraphForCore(graph: Graph): Graph {
  return {
    ...graph,
    nodes: graph.nodes.map((n: any) => ({
      ...n,
      ...(n.state === "succeeded" ? { state: "completed" } : {}),
      ...(Array.isArray(n.generations)
        ? {
            generations: n.generations.map((g: any) => ({
              ...g,
              nodeId: g.nodeId ?? n.id,
              artifactId: g.artifactId ?? g.id,
            })),
          }
        : {}),
    })),
  } as Graph;
}

/** Chuyển danh sách nodes/edges trên canvas thành cấu trúc Graph chuẩn của Core (kèm ui metadata). */
export function toCoreGraph(
  projectId: string,
  nodes: any[],
  edges: any[]
): Graph {
  const byId = new Map(nodes.map((n: any) => [n.id, n]));
  const real = nodes.filter((n: any) => n.type !== "templateGroup");
  return {
    id: `graph_${projectId}`,
    projectId,
    name: `Graph ${projectId}`,
    nodes: real.map((n: any) => {
      const d = n.data || {};
      const ui: Record<string, any> = {};
      if (d.previewUrl) ui.previewUrl = d.previewUrl;
      if (d.previewType) ui.previewType = d.previewType;
      if (d.summaryText) ui.summaryText = d.summaryText;
      if (d.inputSummary) ui.inputSummary = d.inputSummary;
      if (d.outputSummary) ui.outputSummary = d.outputSummary;
      if (d.errorMessage) ui.errorMessage = d.errorMessage;
      if (d.lastLog) ui.lastLog = d.lastLog;
      if (d.actualCostUsd !== undefined) ui.actualCostUsd = d.actualCostUsd;
      if (d.label) ui.label = d.label;
      return {
        id: n.id,
        type: d.nodeType,
        position:
          n.parentId && byId.get(n.parentId)
            ? {
                x: n.position.x + byId.get(n.parentId)!.position.x,
                y: n.position.y + byId.get(n.parentId)!.position.y,
              }
            : n.position,
        params: d.params || {},
        inputs: (d.inputs as any) || {},
        ui,
        state: (d.status || "idle") as any,
        isStale: Boolean(d.isStale),
        pinnedGenerationId: d.pinnedGenerationId ?? null,
        generations: (d.generations as any) || [],
      };
    }),
    edges: edges.map((e: any) => ({
      id: e.id,
      sourceNodeId: e.source,
      sourcePort: e.sourceHandle || "text",
      targetNodeId: e.target,
      targetPort: e.targetHandle || "text",
    })),
    updatedAt: new Date().toISOString(),
  };
}

/** Mốc `updatedAt` của đồ thị lần nạp/lưu gần nhất theo dự án — để không lưu đè bản mới hơn do nơi khác ghi. */
