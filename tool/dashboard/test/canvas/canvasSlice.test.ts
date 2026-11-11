import { describe, it, expect, beforeEach } from "vitest";
import { useStudioStore } from "@/store";
import { sample5Nodes, sampleEdges } from "../_shared/testData";

describe("canvasSlice characterization tests", () => {
  beforeEach(() => {
    useStudioStore.getState().setNodesAndEdges(sample5Nodes, sampleEdges, false);
    useStudioStore.setState({ undoStack: [], redoStack: [], selectedNodeId: null });
  });

  it("updates selectedNodeId via setSelectedNodeId", () => {
    useStudioStore.getState().setSelectedNodeId("node_script");
    expect(useStudioStore.getState().selectedNodeId).toBe("node_script");

    useStudioStore.getState().setSelectedNodeId(null);
    expect(useStudioStore.getState().selectedNodeId).toBeNull();
  });

  it("sets nodes and edges and tracks undo/redo history", () => {
    const initialCount = useStudioStore.getState().nodes.length;
    expect(initialCount).toBe(5);

    useStudioStore.getState().setNodesAndEdges([], [], true);
    expect(useStudioStore.getState().nodes).toHaveLength(0);
    expect(useStudioStore.getState().undoStack).toHaveLength(1);

    useStudioStore.getState().undoCanvas();
    expect(useStudioStore.getState().nodes).toHaveLength(5);
    expect(useStudioStore.getState().redoStack).toHaveLength(1);

    useStudioStore.getState().redoCanvas();
    expect(useStudioStore.getState().nodes).toHaveLength(0);
  });

  it("updates node params via updateNodeParams", () => {
    useStudioStore.getState().updateNodeParams("node_script", { prompt: "Updated Prompt" });
    const node = useStudioStore.getState().nodes.find((n) => n.id === "node_script");
    expect(node?.data.params.prompt).toBe("Updated Prompt");
  });

  it("updates node execution state via updateNodeExecutionState", () => {
    useStudioStore.getState().updateNodeExecutionState("node_image", {
      status: "running",
      progressPct: 45,
    });
    const node = useStudioStore.getState().nodes.find((n) => n.id === "node_image");
    expect(node?.data.status).toBe("running");
    expect(node?.data.progressPct).toBe(45);
  });

  it("collapses and expands nodes via setAllNodesCollapsed and toggleAllNodesCollapsed", () => {
    useStudioStore.getState().setAllNodesCollapsed(true);
    const nodesAfterCollapse = useStudioStore.getState().nodes;
    expect(nodesAfterCollapse.some((n) => n.data.params._collapsed === true)).toBe(true);

    useStudioStore.getState().toggleAllNodesCollapsed();
    const nodesAfterToggle = useStudioStore.getState().nodes;
    expect(nodesAfterToggle.every((n) => n.data.params._collapsed === false)).toBe(true);
  });

  it("loads blank and soc_nho_story templates", () => {
    useStudioStore.getState().loadCanvasTemplate("blank");
    expect(useStudioStore.getState().nodes).toHaveLength(1);
    expect(useStudioStore.getState().nodes[0].id).toBe("node_text_1");
    expect(useStudioStore.getState().edges).toHaveLength(0);

    useStudioStore.getState().loadCanvasTemplate("soc_nho_story");
    expect(useStudioStore.getState().nodes.length).toBeGreaterThan(1);
    expect(useStudioStore.getState().edges.length).toBeGreaterThan(0);
  });

  it("adds and removes a node via setNodesAndEdges and restores cleanly with undo/redo", () => {
    const newNode = {
      id: "node_extra_gate",
      type: "studioCard",
      position: { x: 800, y: 120 },
      data: {
        nodeType: "quality_check",
        status: "idle" as const,
        params: {},
        generations: [],
      },
    };
    const prev = useStudioStore.getState();
    useStudioStore.getState().setNodesAndEdges([...prev.nodes, newNode], prev.edges, true);
    expect(useStudioStore.getState().nodes).toHaveLength(6);

    const added = useStudioStore.getState();
    useStudioStore.getState().setNodesAndEdges(
      added.nodes.filter((n) => n.id !== "node_extra_gate"),
      added.edges,
      true
    );
    expect(useStudioStore.getState().nodes).toHaveLength(5);

    useStudioStore.getState().undoCanvas();
    expect(useStudioStore.getState().nodes.some((n) => n.id === "node_extra_gate")).toBe(true);
  });

  it("marks succeeded downstream nodes stale on _bypass change but NOT on _collapsed UI change", () => {
    useStudioStore.getState().updateNodeExecutionState("node_image", { status: "succeeded" });
    useStudioStore.getState().updateNodeParams("node_script", { _collapsed: true });
    expect(useStudioStore.getState().nodes.find((n) => n.id === "node_image")?.data.isStale).toBeFalsy();

    useStudioStore.getState().updateNodeParams("node_script", { _bypass: true });
    const updatedSource = useStudioStore.getState().nodes.find((n) => n.id === "node_script");
    const downstream = useStudioStore.getState().nodes.find((n) => n.id === "node_image");
    expect(updatedSource?.data.params._bypass).toBe(true);
    expect(downstream?.data.isStale).toBe(true);
  });
});
