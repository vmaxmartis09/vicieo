import type { Edge } from "@xyflow/react";

/**
 * Checks whether connecting sourceId to targetId would introduce a directed cycle in the graph.
 */
export function wouldCreateCycle(
  edges: Edge[],
  sourceId: string,
  targetId: string
): boolean {
  if (sourceId === targetId) return true;
  const visited = new Set<string>();
  const stack = [targetId];
  while (stack.length > 0) {
    const curr = stack.pop()!;
    if (curr === sourceId) return true;
    if (visited.has(curr)) continue;
    visited.add(curr);
    for (const e of edges) {
      if (e.source === curr) {
        stack.push(e.target);
      }
    }
  }
  return false;
}
