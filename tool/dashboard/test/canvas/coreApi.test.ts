import { describe, it, expect } from "vitest";
import {
  isPortCompatible,
  estimateNodeCostUsd,
  validateGraphLocally,
  toCoreGraph,
  normalizeGraphForCore,
  parseStudioCommandClientSide,
  FALLBACK_NODE_TYPES,
} from "@/lib/core-api";

describe("core-api characterization tests", () => {
  describe("isPortCompatible", () => {
    it("returns true when source and target types match", () => {
      expect(isPortCompatible("text", "text")).toBe(true);
      expect(isPortCompatible("image", "image")).toBe(true);
      expect(isPortCompatible("video", "video")).toBe(true);
    });

    it("returns true for special compatible types", () => {
      expect(isPortCompatible("image", "image[]")).toBe(true);
      expect(isPortCompatible("dialogue", "text")).toBe(true);
    });

    it("returns false for incompatible types", () => {
      expect(isPortCompatible("text", "video")).toBe(false);
      expect(isPortCompatible("audio", "image")).toBe(false);
    });
  });

  describe("estimateNodeCostUsd", () => {
    it("estimates cost for image generation nodes", () => {
      const cost = estimateNodeCostUsd("image_gen", { model: "flux-schnell" }, false);
      expect(cost).toBeGreaterThan(0);
    });

    it("returns 0 in mock mode", () => {
      const cost = estimateNodeCostUsd("image_gen", { model: "flux-schnell" }, true);
      expect(cost).toBe(0);
    });

    it("estimates cost for video generation nodes based on durationSec", () => {
      const costShort = estimateNodeCostUsd("video_gen", { model: "kling-v1.6", durationSec: 5 }, false);
      const costLong = estimateNodeCostUsd("video_gen", { model: "kling-v1.6", durationSec: 10 }, false);
      expect(costLong).toBeGreaterThan(costShort);
    });
  });

  describe("validateGraphLocally", () => {
    it("validates an empty graph without errors", () => {
      const result = validateGraphLocally(
        { id: "test_graph", projectId: "test_proj", nodes: [], edges: [] },
        FALLBACK_NODE_TYPES
      );
      expect(result.isValid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it("detects edges pointing to non-existent nodes", () => {
      const invalidGraph = {
        id: "test_graph",
        projectId: "test_proj",
        nodes: [
          {
            id: "n1",
            type: "text",
            params: {},
            position: { x: 0, y: 0 },
            isStale: false,
          },
        ],
        edges: [
          {
            id: "e1",
            sourceNodeId: "n1",
            targetNodeId: "non_existent_node",
            sourcePort: "text",
            targetPort: "prompt",
          },
        ],
      };
      const result = validateGraphLocally(invalidGraph, FALLBACK_NODE_TYPES);
      expect(result.isValid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });
  });

  describe("toCoreGraph & normalizeGraphForCore", () => {
    it("filters templateGroup nodes, offsets child positions by parentId, and maps edges", () => {
      const nodes = [
        {
          id: "grp_1",
          type: "templateGroup",
          position: { x: 100, y: 200 },
          data: {},
        },
        {
          id: "n_child",
          type: "studioCard",
          parentId: "grp_1",
          position: { x: 25, y: 40 },
          data: {
            nodeType: "text",
            status: "succeeded",
            previewUrl: "/media/preview.png",
            params: { content: "Xin chao" },
            generations: [{ id: "gen_1" }],
          },
        },
        {
          id: "n_target",
          type: "studioCard",
          position: { x: 400, y: 200 },
          data: { nodeType: "script", params: {} },
        },
      ];
      const edges = [
        {
          id: "e_1",
          source: "n_child",
          sourceHandle: "text",
          target: "n_target",
          targetHandle: "prompt",
        },
      ];

      const graph = toCoreGraph("soc-nho-tv", nodes, edges);
      expect(graph.id).toBe("graph_soc-nho-tv");
      expect(graph.nodes).toHaveLength(2);
      expect(graph.nodes.some((n) => n.id === "grp_1")).toBe(false);
      expect(graph.nodes[0].position).toEqual({ x: 125, y: 240 });
      expect((graph.nodes[0] as any).ui.previewUrl).toBe("/media/preview.png");
      expect(graph.edges[0]).toEqual({
        id: "e_1",
        sourceNodeId: "n_child",
        sourcePort: "text",
        targetNodeId: "n_target",
        targetPort: "prompt",
      });

      const normalized = normalizeGraphForCore(graph);
      expect(normalized.nodes[0].state).toBe("completed");
      expect((normalized.nodes[0] as any).generations[0]).toMatchObject({
        id: "gen_1",
        nodeId: "n_child",
        artifactId: "gen_1",
      });
    });
  });

  describe("parseStudioCommandClientSide", () => {
    it("parses quality gate, aspect ratio, schedule confirmation, character, and run commands", () => {
      const gate = parseStudioCommandClientSide("thêm bước kiểm tra chất lượng trước khi tạo video");
      expect(gate.actions[0]).toMatchObject({ type: "add_node", nodeType: "quality_check" });
      expect(gate.requiresConfirmation).toBe(false);

      const ratio = parseStudioCommandClientSide("đổi tỉ lệ khung hình sang 16:9");
      expect(ratio.actions[0]).toMatchObject({ type: "set_aspect_ratio", aspectRatio: "16:9" });

      const openTab = parseStudioCommandClientSide("mở lịch đăng cho mình");
      expect(openTab.actions[0]).toMatchObject({ type: "open_tab", tab: "publish" });

      const schedule = parseStudioCommandClientSide("lên lịch đăng lên tiktok lúc 20:00");
      expect(schedule.requiresConfirmation).toBe(true);
      expect(schedule.confirmationMeta).toMatchObject({ kind: "publish", platform: "tiktok", time: "20:00" });

      const character = parseStudioCommandClientSide("xem nhân vật Koala");
      expect(character.actions[0]).toMatchObject({ type: "open_character", characterId: "koala" });

      const run = parseStudioCommandClientSide("chạy từ cảnh 2");
      expect(run.requiresConfirmation).toBe(true);
      expect(run.confirmationMeta).toMatchObject({ kind: "run", mode: "downstream" });
    });
  });
});
