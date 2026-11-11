import { describe, it, expect } from "vitest";
import type { Edge } from "@xyflow/react";
import { wouldCreateCycle } from "@/lib/graph/wouldCreateCycle";

describe("wouldCreateCycle graph algorithm", () => {
  it("returns true for self-loop (connecting a node to itself)", () => {
    expect(wouldCreateCycle([], "nodeA", "nodeA")).toBe(true);
  });

  it("returns false for a simple linear non-cyclic connection", () => {
    const edges: Edge[] = [
      { id: "e1", source: "nodeA", target: "nodeB" },
    ];
    // Connecting B -> C does not create cycle
    expect(wouldCreateCycle(edges, "nodeB", "nodeC")).toBe(false);
  });

  it("detects direct 2-node cycle (A -> B, then connecting B -> A)", () => {
    const edges: Edge[] = [
      { id: "e1", source: "nodeA", target: "nodeB" },
    ];
    // Attempting to connect B -> A
    expect(wouldCreateCycle(edges, "nodeB", "nodeA")).toBe(true);
  });

  it("detects indirect 3-node cycle (A -> B -> C, then connecting C -> A)", () => {
    const edges: Edge[] = [
      { id: "e1", source: "nodeA", target: "nodeB" },
      { id: "e2", source: "nodeB", target: "nodeC" },
    ];
    // Attempting to connect C -> A
    expect(wouldCreateCycle(edges, "nodeC", "nodeA")).toBe(true);
  });

  it("returns false when connecting into a parallel branch without cycles", () => {
    const edges: Edge[] = [
      { id: "e1", source: "nodeA", target: "nodeB" },
      { id: "e2", source: "nodeA", target: "nodeC" },
    ];
    // Connecting B -> C (DAG Diamond pattern) does not create cycle
    expect(wouldCreateCycle(edges, "nodeB", "nodeC")).toBe(false);
  });
});
