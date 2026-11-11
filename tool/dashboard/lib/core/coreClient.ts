import type {
  NodeDefinition,
  Graph,
  GraphValidationResult,
  GraphRunSummary,
  RunMode,
  RunPolicy,
} from "../core-contracts";
import { CORE_URL } from "../repositories/core";
import { FALLBACK_NODE_TYPES } from "./nodeDefinitions";
import { validateGraphLocally } from "./graphValidator";
import { normalizeGraphForCore } from "./graphTransformers";
import { estimateNodeCostUsd } from "./costEstimator";
import { parseStudioCommandClientSide } from "./commandParser";

export { CORE_URL };

export async function checkCoreHealth(): Promise<{
  online: boolean;
  version: string;
  repoRoot?: string;
  outdated?: boolean;
}> {
  try {
    const res = await fetch(`${CORE_URL}/health`, { cache: "no-store" });
    if (!res.ok) return { online: false, version: "–" };
    const data = await res.json();
    let outdated = false;
    try {
      const tmplRes = await fetch(`${CORE_URL}/video/templates`, { cache: "no-store" });
      if (!tmplRes.ok) outdated = true;
    } catch {
      outdated = true;
    }
    return {
      online: data.status === "healthy",
      version: data.version || "0.1.0",
      repoRoot: data.repo_root,
      outdated,
    };
  } catch {
    return { online: false, version: "–" };
  }
}

export async function fetchNodeTypes(): Promise<{
  definitions: NodeDefinition[];
  fromCore: boolean;
}> {
  try {
    const res = await fetch(`${CORE_URL}/node-types`, { cache: "no-store" });
    if (!res.ok) throw new Error("Failed");
    const defs = (await res.json()) as NodeDefinition[];
    if (Array.isArray(defs) && defs.length > 0) {
      return { definitions: defs, fromCore: true };
    }
  } catch {
    // Fallback to FALLBACK_NODE_TYPES
  }
  return { definitions: FALLBACK_NODE_TYPES, fromCore: false };
}

export async function fetchCustomNodeTypes(projectId: string): Promise<NodeDefinition[]> {
  try {
    const res = await fetch(`${CORE_URL}/custom-node-types?projectId=${encodeURIComponent(projectId)}`, { cache: "no-store" });
    if (!res.ok) return [];
    const list = (await res.json()) as NodeDefinition[];
    return Array.isArray(list) ? list : [];
  } catch {
    return [];
  }
}

export async function saveCustomNodeType(
  def: NodeDefinition,
  scope: "project" | "shared",
  projectId: string
): Promise<void> {
  const res = await fetch(
    `${CORE_URL}/custom-node-types?scope=${scope}&projectId=${encodeURIComponent(projectId)}`,
    { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(def) }
  );
  if (!res.ok) throw new Error((await res.text()) || "Lưu loại node thất bại");
}

export async function deleteCustomNodeType(typeName: string, scope: "project" | "shared", projectId: string): Promise<void> {
  await fetch(
    `${CORE_URL}/custom-node-types/${encodeURIComponent(typeName)}?scope=${scope}&projectId=${encodeURIComponent(projectId)}`,
    { method: "DELETE" }
  );
}

export async function validateGraphWithCore(
  projectId: string,
  graph: Graph,
  nodeCatalog: NodeDefinition[]
): Promise<GraphValidationResult> {
  try {
    const res = await fetch(`${CORE_URL}/graph/validate`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(graph),
    });
    if (res.ok) {
      return (await res.json()) as GraphValidationResult;
    }
  } catch {
    // Local validation fallback when Core is offline
  }
  return validateGraphLocally(graph, nodeCatalog);
}

const graphBaseline: Record<string, string> = {};

/** Ghi nhanh đồ thị xuống Core (dùng khi chuyển dự án hoặc trước khi reload trang, có keepalive). */
export async function flushGraphToCore(
  projectId: string,
  graph: Graph
): Promise<boolean> {
  try {
    const base = graphBaseline[projectId];
    const url = base
      ? `${CORE_URL}/projects/${projectId}/graph?baseUpdatedAt=${encodeURIComponent(base)}`
      : `${CORE_URL}/projects/${projectId}/graph`;
    const res = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(normalizeGraphForCore(graph)),
      keepalive: true,
    });
    if (res.status === 409) {
      if (typeof window !== "undefined") {
        window.dispatchEvent(new CustomEvent("studio:graph-conflict", { detail: { projectId } }));
      }
      return false;
    }
    if (res.ok) {
      const j = await res.json().catch(() => null);
      if (j?.updatedAt) graphBaseline[projectId] = j.updatedAt;
    }
    return res.ok;
  } catch {
    return false;
  }
}

export async function saveGraphToCore(
  projectId: string,
  graph: Graph
): Promise<boolean> {
  try {
    const base = graphBaseline[projectId];
    if (base) {
      const cur = await fetch(`${CORE_URL}/projects/${projectId}/graph`, { cache: "no-store" })
        .then((r) => (r.ok ? r.json() : null))
        .catch(() => null);
      if (cur?.updatedAt && cur.updatedAt !== base) {
        // Đồ thị đã bị thay ở nơi khác (tab khác, công cụ): KHÔNG lưu đè, nạp lại bản mới
        if (typeof window !== "undefined") {
          window.dispatchEvent(new CustomEvent("studio:graph-conflict", { detail: { projectId } }));
        }
        return false;
      }
    }
    const url = base
      ? `${CORE_URL}/projects/${projectId}/graph?baseUpdatedAt=${encodeURIComponent(base)}`
      : `${CORE_URL}/projects/${projectId}/graph`;
    const res = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(normalizeGraphForCore(graph)),
      keepalive: true,
    });
    if (res.status === 409) {
      if (typeof window !== "undefined") {
        window.dispatchEvent(new CustomEvent("studio:graph-conflict", { detail: { projectId } }));
      }
      return false;
    }
    if (res.ok) {
      const j = await res.json().catch(() => null);
      if (j?.updatedAt) graphBaseline[projectId] = j.updatedAt;
    }
    return res.ok;
  } catch {
    return false;
  }
}

export interface CoreCostEstimateResponse {
  projectId: string;
  mode: RunMode;
  mockMode: boolean;
  nodesToExecute: Array<{
    nodeId: string;
    nodeType: string;
    model: string;
    provider: string;
    estimatedCostUsd: number;
  }>;
  skippedNodes: string[];
  totalEstimatedCostUsd: number;
  requiresConfirmation: boolean;
  missingApiKeys: string[];
  spendStatus: Record<string, any>;
}

export async function estimateGraphOnCore(
  projectId: string,
  payload: {
    graph: Graph;
    mode: RunMode;
    targetNodeId?: string;
    spendingLimitUsd?: number;
    mockMode: boolean;
    runPolicy?: RunPolicy;
  }
): Promise<CoreCostEstimateResponse | null> {
  try {
    const res = await fetch(`${CORE_URL}/projects/${projectId}/estimate`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (!res.ok) return null;
    return (await res.json()) as CoreCostEstimateResponse;
  } catch {
    return null;
  }
}

export async function runGraphOnCore(
  projectId: string,
  payload: {
    graph: Graph;
    mode: RunMode;
    targetNodeId?: string;
    spendingLimitUsd?: number;
    mockMode: boolean;
    runPolicy?: RunPolicy;
    confirmPaidRun?: boolean;
  }
): Promise<GraphRunSummary | null> {
  const res = await fetch(`${CORE_URL}/projects/${projectId}/run`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      ...payload,
      confirmPaidRun: payload.confirmPaidRun ?? true,
    }),
  });
  if (!res.ok) {
    const detail = await res.text();
    throw new Error(detail);
  }
  return (await res.json()) as GraphRunSummary;
}

export async function cancelRunOnCore(projectId: string): Promise<void> {
  try {
    await fetch(`${CORE_URL}/projects/${projectId}/run/cancel`, {
      method: "POST",
    });
  } catch {
    // ignore if offline
  }
}

export async function deleteRunOnCore(projectId: string, runId: string): Promise<boolean> {
  try {
    const res = await fetch(`${CORE_URL}/projects/${projectId}/runs/${encodeURIComponent(runId)}`, { method: "DELETE" });
    return res.ok;
  } catch {
    return false;
  }
}

export async function fetchActiveRun(projectId: string): Promise<any | null> {
  try {
    const res = await fetch(`${CORE_URL}/projects/${projectId}/runs/active`, { cache: "no-store" });
    if (res.status === 204 || !res.ok) return null;
    return await res.json();
  } catch {
    return null;
  }
}

export async function fetchProjectRuns(projectId: string, limit: number = 20): Promise<any[]> {
  try {
    const res = await fetch(`${CORE_URL}/projects/${projectId}/runs?limit=${limit}`, { cache: "no-store" });
    if (!res.ok) return [];
    return await res.json();
  } catch {
    return [];
  }
}

export async function fetchProjectOutputs(projectId: string): Promise<any[]> {
  try {
    const res = await fetch(`${CORE_URL}/projects/${projectId}/outputs`, { cache: "no-store" });
    if (!res.ok) return [];
    return await res.json();
  } catch {
    return [];
  }
}

export async function sendChatToCore(
  projectId: string,
  prompt: string,
  context?: Record<string, unknown>
): Promise<{
  reply: string;
  mode: string;
  actions: any[];
  requiresConfirmation?: boolean;
  confirmationMeta?: any;
}> {
  try {
    const res = await fetch(`${CORE_URL}/projects/${projectId}/chat`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ prompt, context }),
    });
    if (res.ok) {
      return await res.json();
    }
  } catch {
    // Fallback to client-side rule parser if Core is unreachable
  }
  return parseStudioCommandClientSide(prompt);
}

export async function fetchGraphFromCore(projectId: string): Promise<Graph | null> {
  try {
    const res = await fetch(`${CORE_URL}/projects/${projectId}/graph`, { cache: "no-store" });
    if (!res.ok) return null;
    const g = (await res.json()) as Graph;
    if ((g as any)?.updatedAt) graphBaseline[projectId] = (g as any).updatedAt;
    return g;
  } catch {
    return null;
  }
}

/** Đọc id dự án đang mở (active_project) từ data/workspace.json của Core. */
export async function fetchActiveProjectFromCore(): Promise<string | null> {
  try {
    const res = await fetch(`${CORE_URL}/projects`, { cache: "no-store" });
    if (!res.ok) return null;
    const data = await res.json();
    const pid = data?.active_project || data?.activeProject;
    return typeof pid === "string" && pid.trim() ? pid.trim() : null;
  } catch {
    return null;
  }
}

/** Lưu id dự án đang mở (active_project) xuống data/workspace.json của Core. */
export async function saveActiveProjectToCore(projectId: string): Promise<boolean> {
  if (!projectId) return false;
  try {
    const res = await fetch(`${CORE_URL}/projects/active`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ projectId }),
      keepalive: true,
    });
    return res.ok;
  } catch {
    return false;
  }
}
