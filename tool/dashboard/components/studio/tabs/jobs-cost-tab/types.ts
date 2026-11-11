export type FilterTab = "all" | "running" | "completed" | "failed";

export interface JobNodeChip {
  id: string;
  type: string;
  state: string;
  progressPct?: number;
}

export interface CombinedJobItem {
  id: string;
  model?: string;
  status: "queued" | "running" | "completed" | "failed" | "cancelled";
  stage?: string;
  progress: number;
  actualCostUsd: number;
  outputs: Record<string, string>;
  createdAt: string;
  elapsedSec?: number;
  nodes?: JobNodeChip[];
  error?: string | null;
}
