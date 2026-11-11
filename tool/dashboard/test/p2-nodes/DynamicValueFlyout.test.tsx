import "@testing-library/jest-dom/vitest";
import React from "react";
import { describe, it, expect, beforeAll, afterEach, afterAll, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import { CORE_URL } from "@/lib/repositories/core";
import { useStudioStore } from "@/store";
import { DynamicValueFlyout } from "@/components/canvas/DynamicValueFlyout";
import {
  buildCandidateNodes,
  filterCandidateNodes,
  filterCharactersList,
  filterExtendedModels,
} from "@/components/canvas/dynamic-value-flyout/utils/flyoutFilters";
import { EXTENDED_MODELS_CATALOG } from "@/lib/data/extendedModelsCatalog";

const server = setupServer(
  http.get(`${CORE_URL}/data/projects/:pid/library`, () =>
    HttpResponse.json({
      images: [],
      videos: [],
      audios: [],
      episodes: [{ id: "ep01", title: "Tập 1: Chiếc Răng Sâu" }],
      scripts: [{ episodeId: "ep01" }],
    })
  ),
  http.get(`${CORE_URL}/data/projects/:pid/script-data/:ep`, () =>
    HttpResponse.json({
      facets: [],
      shotFacets: [],
      shots: [
        {
          id: "shot_1",
          sceneTitle: "Rừng ban mai",
          speaker: "Sóc Nhỏ",
          prompt: "Cute 3D squirrel holding toothbrush",
          dialogue: "Chào buổi sáng các bạn!",
        },
      ],
    })
  )
);

function renderFlyout(overrides: Partial<React.ComponentProps<typeof DynamicValueFlyout>> = {}) {
  const triggerBtn = document.createElement("button");
  document.body.appendChild(triggerBtn);
  const triggerRef = { current: triggerBtn };
  const onClose = overrides.onClose || vi.fn();
  const onSelectValue = overrides.onSelectValue || vi.fn();

  const res = render(
    <DynamicValueFlyout
      nodeId="n_target"
      portName="prompt"
      currentValue=""
      topOffset={40}
      triggerRef={triggerRef}
      onClose={onClose}
      onSelectValue={onSelectValue}
      {...overrides}
    />
  );

  return { ...res, onClose, onSelectValue, triggerBtn };
}

beforeAll(() => server.listen());

beforeEach(() => {
  useStudioStore.setState({
    activeProjectId: "soc-nho-tv",
    characters: [
      {
        id: "soc_nho",
        projectId: "soc-nho-tv",
        name: "Sóc Nhỏ (Bi)",
        role: "main",
        heightCm: 100,
        scaleRatio: 1,
        voiceId: "soc-nho",
        promptLock: "Bi 3D squirrel red hoodie",
        referenceImages: ["/demo/bi.png"],
        memory: { personality: "", catchphrases: [], habits: [], pastEventsSummary: [] },
      },
    ],
    nodeCatalog: [
      {
        typeName: "character",
        displayName: "Nhân vật",
        outputPorts: [{ name: "character", portType: "character" }],
      },
      {
        typeName: "script",
        displayName: "Kịch bản",
        outputPorts: [{ name: "lines", portType: "lines" }],
      },
    ] as any,
    nodes: [
      {
        id: "n_char",
        position: { x: 0, y: 0 },
        data: { nodeType: "character", title: "Nhân vật", params: { _customName: "Sóc Nhỏ" } },
      },
      {
        id: "n_target",
        position: { x: 200, y: 0 },
        data: { nodeType: "image_gen", title: "Tạo ảnh", params: {} },
      },
    ] as any,
    edges: [],
    setNodesAndEdges: vi.fn(),
  } as any);
});

afterEach(() => {
  server.resetHandlers();
  cleanup();
  document.body.innerHTML = "";
});

afterAll(() => server.close());

describe("DynamicValueFlyout characterization tests", () => {
  it("lists candidate nodes excluding current node, creates edge and emits ref on click", async () => {
    const user = userEvent.setup();
    const { onSelectValue, onClose } = renderFlyout();

    expect(screen.getByText("Sóc Nhỏ")).toBeInTheDocument();
    expect(screen.queryByText("Tạo ảnh")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "{Sóc Nhỏ.character}" }));

    expect(useStudioStore.getState().setNodesAndEdges).toHaveBeenCalledTimes(1);
    const [, nextEdges, markDirty] = (useStudioStore.getState().setNodesAndEdges as any).mock.calls[0];
    expect(nextEdges).toHaveLength(1);
    expect(nextEdges[0]).toMatchObject({
      source: "n_char",
      sourceHandle: "character",
      target: "n_target",
      targetHandle: "prompt",
    });
    expect(markDirty).toBe(true);
    expect(onSelectValue).toHaveBeenCalledWith("{Sóc Nhỏ.character}");
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("switches through Script, Character, Model, and Variable tabs with filtering and text insertion", async () => {
    const user = userEvent.setup();
    const onInsertText = vi.fn();
    const onSelectValue = vi.fn();
    const { onClose } = renderFlyout({
      currentValue: "Đã có sẵn ",
      onInsertText,
      onSelectValue,
    });

    // Script tab lazy-loads episodes + shots
    await user.click(screen.getByRole("button", { name: "Kịch bản" }));
    expect(await screen.findByText(/Rừng ban mai/)).toBeInTheDocument();
    await user.click(screen.getByText(/Cute 3D squirrel holding toothbrush/).closest("button")!);
    expect(onInsertText).toHaveBeenCalledWith("{Kịch bản.cảnh_1.prompt}");

    // Character tab + search
    await user.click(screen.getByRole("button", { name: "Nhân vật" }));
    await user.click(screen.getByRole("button", { name: "{Sóc Nhỏ (Bi).promptLock}" }));
    expect(onInsertText).toHaveBeenCalledWith("{Sóc Nhỏ (Bi).promptLock}");

    // Model tab + search
    await user.click(screen.getByRole("button", { name: "Model" }));
    const searchInput = screen.getByPlaceholderText(/Tìm node, kịch bản/i);
    await user.type(searchInput, "Kling");
    const modelChips = screen.getAllByText(/\{model\./);
    expect(modelChips.length).toBeGreaterThanOrEqual(1);
    await user.clear(searchInput);

    // Variable tab
    await user.click(screen.getByRole("button", { name: "Biến" }));
    await user.click(screen.getByText("{product.name}").closest("button")!);
    expect(onInsertText).toHaveBeenCalledWith("{product.name}");
    expect(onSelectValue).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });

  it("closes on Escape key and outside pointerdown", () => {
    const { onClose } = renderFlyout();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);

    fireEvent.pointerDown(document.body);
    expect(onClose).toHaveBeenCalledTimes(2);
  });
});

describe("flyoutFilters pure helpers", () => {
  it("builds candidate list excluding self and filters across queries", () => {
    const nodes = [
      { id: "a", data: { nodeType: "character", params: { _customName: "Sóc Nhỏ" } } },
      { id: "b", data: { nodeType: "image_gen", params: { prompt: "Rừng xanh" } } },
    ];
    const catalog = [{ typeName: "character", displayName: "Nhân vật", outputPorts: [{ name: "character" }] }];
    const candidates = buildCandidateNodes(nodes, catalog, "b");
    expect(candidates).toHaveLength(1);
    expect(candidates[0]).toEqual({
      id: "a",
      title: "Sóc Nhỏ",
      typeName: "character",
      summary: undefined,
      outputs: ["character"],
    });
    expect(filterCandidateNodes(candidates, "sóc")).toHaveLength(1);
    expect(filterCandidateNodes(candidates, "không khớp")).toHaveLength(0);
    expect(filterCharactersList([{ id: "1", name: "Bi", promptLock: "squirrel" } as any], "squirrel")).toHaveLength(1);
    expect(filterExtendedModels(EXTENDED_MODELS_CATALOG, "kling").length).toBeGreaterThanOrEqual(1);
  });
});
