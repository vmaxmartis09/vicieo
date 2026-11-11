import "@testing-library/jest-dom/vitest";
import React from "react";
import { describe, it, expect, beforeAll, afterEach, afterAll, beforeEach, vi } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import viMessages from "@/messages/vi.json";
import { CORE_URL } from "@/lib/repositories/core";
import { useStudioStore } from "@/store";
import { usePreview } from "@/lib/preview";
import type { Character } from "@/lib/repositories";
import { NodeBody } from "@/components/canvas/NodeBody";
import {
  clearCastCache,
  extractCastView,
  getCastCacheSize,
  getCachedCast,
  MAX_CAST_CACHE_ENTRIES,
  resolveCastMembers,
  setCachedCast,
} from "@/components/canvas/node-body/utils/castExtractor";

const sampleCharacter: Character = {
  id: "soc_nho",
  projectId: "soc-nho-tv",
  name: "Sóc Nhỏ (Bi)",
  role: "main",
  heightCm: 100,
  scaleRatio: 1,
  voiceId: "soc-nho",
  promptLock: "Bi 3D squirrel",
  referenceImages: ["/demo/bi.png"],
  memory: { personality: "Hiếu động", catchphrases: [], habits: [], pastEventsSummary: [] },
};

const server = setupServer(
  http.get(`${CORE_URL}/data/projects/:pid/policy`, () =>
    HttpResponse.json({
      name: "Chính sách Thiếu nhi An toàn",
      mode: "strict",
      scopes: { dialogue: true, caption: true, script: true, prompt: true },
      allowlist: [],
      rules: [
        { id: "r1", category: "safety", label: "Cấm bạo lực", severity: "block", enabled: true, terms: ["đánh", "máu"] },
      ],
    })
  ),
  http.get(`${CORE_URL}/projects/:pid/outputs`, () =>
    HttpResponse.json([
      {
        id: "ep1_out",
        slug: "ep01-rang-sau",
        title: "Tập 1: Chiếc Răng Sâu",
        duration_s: 45,
        engine: "hyperframes",
        files: { video: "output/ep01/final.mp4", thumb: "output/ep01/thumb.jpg" },
        episodeVerify: { pass1to5: true, verdict: "ĐẠT" },
      },
    ])
  ),
  http.get(`${CORE_URL}/data/projects/:pid/script-data/:ep`, () =>
    HttpResponse.json({
      facets: [],
      shotFacets: [],
      script: {
        cast: [{ id: "soc_nho", name: "Sóc Nhỏ (Bi)" }],
        unmatched: ["Bác sĩ Cú"],
      },
      shots: [
        { id: "s1", scene: 1, sceneTitle: "Mở đầu", speaker: "Bi", dialogue: "Chào các bạn!", characterId: "soc_nho" },
      ],
    })
  ),
  http.get(`${CORE_URL}/projects/:pid/products`, () =>
    HttpResponse.json([
      { id: "prod_1", name: "Bàn chải Sóc Nhỏ", cutout_images: ["cut1.png"], review_status: "chua_lam" },
    ])
  ),
  http.get(`${CORE_URL}/data/projects/:pid/library`, () =>
    HttpResponse.json({ images: [], videos: [], audios: [], episodes: [], scripts: [] })
  ),
  http.get(`${CORE_URL}/voices`, () => HttpResponse.json([{ id: "soc-nho", name: "Sóc Nhỏ" }])),
  http.get(`${CORE_URL}/models`, () => HttpResponse.json([])),
  http.get(`${CORE_URL}/projects/:pid/custom-voice-vault`, () => HttpResponse.json([]))
);

function renderNodeBody(
  nodeType: string,
  overrides: {
    nodeId?: string;
    data?: Record<string, any>;
    connectedInputs?: string[];
    compact?: boolean;
    expanded?: boolean;
    character?: Character;
  } = {}
) {
  return render(
    <NextIntlClientProvider locale="vi" messages={viMessages}>
      <NodeBody
        nodeType={nodeType}
        nodeId={overrides.nodeId || `n_${nodeType}`}
        data={{
          nodeType: nodeType as any,
          title: nodeType,
          status: "idle",
          params: {},
          generations: [],
          ...(overrides.data || {}),
        }}
        character={overrides.character}
        connectedInputs={overrides.connectedInputs || []}
        compact={overrides.compact}
        expanded={overrides.expanded}
      />
    </NextIntlClientProvider>
  );
}

beforeAll(() => server.listen());

beforeEach(() => {
  clearCastCache();
  usePreview.getState().close();
  useStudioStore.setState({
    activeProjectId: "soc-nho-tv",
    characters: [sampleCharacter],
    nodeCatalog: [
      {
        typeName: "character",
        displayName: "Nhân vật",
        capabilities: ["speak"],
      } as any,
    ],
    nodes: [
      {
        id: "n_script",
        position: { x: 0, y: 0 },
        data: {
          nodeType: "script",
          title: "Kịch bản",
          params: { mode: "episode", episodeId: "ep01" },
        },
      } as any,
    ],
    edges: [
      { id: "e1", source: "n_script", target: "n_cast", sourceHandle: "lines", targetHandle: "lines" } as any,
    ],
    updateNodeParams: vi.fn(),
  } as any);
});

afterEach(() => {
  server.resetHandlers();
  cleanup();
});

afterAll(() => server.close());

describe("NodeBody characterization tests (all node types)", () => {
  it("renders null cleanly for pure source types", () => {
    for (const srcType of ["text", "input", "video", "audio"]) {
      const { container } = renderNodeBody(srcType);
      expect(container.firstChild).toBeNull();
      cleanup();
    }
  });

  it("renders every specialized nodeType without errors", async () => {
    const cases: Array<{ type: string; data?: any; connectedInputs?: string[]; expanded?: boolean; assertText?: RegExp }> = [
      { type: "script", data: { params: { lines: ["Cảnh 1: Công viên"] } }, assertText: /Cảnh 1: Công viên/ },
      { type: "voice_tts", data: { params: { text: "Xin chào", voiceId: "soc-nho" } }, assertText: /Xin chào/ },
      { type: "image_gen", data: { params: { model: "flux", aspectRatio: "9:16", prompt: "Sóc chạy" } }, assertText: /Sóc chạy/ },
      { type: "image_edit", data: { previewUrl: "/demo/edited.png", params: {} } },
      { type: "video_gen", data: { params: { model: "seedance-2.0", prompt: "Bay lượn" } } },
      { type: "cast", nodeId: "n_cast" } as any,
      { type: "product", data: { params: { productId: "prod_1" } } },
      { type: "captions", data: { params: { lines: ["Phụ đề 1"] } }, assertText: /Phụ đề 1/ },
      { type: "transition", data: { params: { style: "fade", durationSec: 0.8 } }, assertText: /Hiệu ứng/ },
      { type: "hyperframes_render", connectedInputs: ["video", "audio"], assertText: /captions/ },
      { type: "export", data: { params: { outputFileName: "tap_01.mp4", platforms: ["tiktok"] } }, assertText: /tap_01\.mp4/ },
      { type: "policy", expanded: true, data: { params: { extraTerms: ["từ cấm"] } }, assertText: /Từ khoá bổ sung/ },
      { type: "quality_check", assertText: /Chưa kiểm tra/ },
      { type: "vision_analyze", assertText: /Chưa phân tích/ },
      { type: "bg_remove", assertText: /Ảnh tách nền sẽ hiện ở đây/ },
      { type: "depth_layers", assertText: /Lớp chiều sâu sẽ hiện ở đây/ },
      { type: "handdraw", assertText: /Video vẽ tay sẽ hiện ở đây/ },
      { type: "scene_cut", assertText: /Chưa tách cảnh video/ },
      { type: "voice_clone", data: { params: { name: "bi-clone", dialect: "Nam" } }, assertText: /bi-clone/ },
      { type: "word_captions", assertText: /Chưa có phụ đề từng từ/ },
      { type: "template_render", assertText: /Video dựng mẫu sẽ hiện ở đây/ },
      { type: "platform_export", assertText: /Video xuất đa nền tảng sẽ hiện ở đây/ },
      { type: "custom_plugin_node", data: { params: { foo: "bar" } } },
    ];

    for (const c of cases) {
      const { container } = renderNodeBody(c.type, {
        nodeId: (c as any).nodeId,
        data: c.data,
        connectedInputs: c.connectedInputs,
        expanded: c.expanded,
      });
      if (c.assertText) {
        expect(screen.getAllByText(c.assertText).length).toBeGreaterThanOrEqual(1);
      } else {
        expect(container.firstChild).not.toBeNull();
      }
      cleanup();
    }

    // Character node with speak capability
    renderNodeBody("character", { character: sampleCharacter });
    expect(screen.getByText("Sóc Nhỏ (Bi)")).toBeInTheDocument();
  });

  it("loads PolicyBadge, CastPreview, transition edits, and export preview action", async () => {
    const user = userEvent.setup();
    const updateSpy = useStudioStore.getState().updateNodeParams;

    // 1. PolicyBadge fetches policy summary
    renderNodeBody("policy");
    expect(await screen.findByText("Chính sách Thiếu nhi An toàn")).toBeInTheDocument();
    expect(screen.getByText("1 quy tắc · 2 từ")).toBeInTheDocument();
    cleanup();

    // 2. CastPreview fetches script-data & renders matched cast + unmatched warning
    renderNodeBody("cast", { nodeId: "n_cast" });
    expect(await screen.findByText("Sóc Nhỏ (Bi)")).toBeInTheDocument();
    expect(screen.getByText("1 câu")).toBeInTheDocument();
    expect(screen.getByText("Chưa ghép được: Bác sĩ Cú")).toBeInTheDocument();
    cleanup();

    // 3. Transition select triggers updateNodeParams
    renderNodeBody("transition", { nodeId: "n_trans", data: { params: { style: "fade", durationSec: 0.5 } } });
    await user.selectOptions(screen.getByRole("combobox"), "wipe_left");
    expect(updateSpy).toHaveBeenCalledWith("n_trans", { style: "wipe_left" });
    cleanup();

    // 4. Export node preview button opens Studio preview pane
    renderNodeBody("export", {
      nodeId: "n_exp",
      data: { previewType: "video", previewUrl: "/demo/out.mp4", params: { outputFileName: "tap1.mp4" } },
    });
    await user.click(screen.getByTestId("export-open-preview"));
    const item = usePreview.getState().item;
    expect(item?.kind).toBe("video");
    if (item?.kind === "video") {
      expect(item.title).toBe("tap1.mp4");
    }
  });
});

describe("castExtractor pure helpers & bounded LRU cache (K.4)", () => {
  it("extracts cast counts and resolves include/exclude deterministically", () => {
    const view = extractCastView({
      script: { cast: [{ id: "bi", name: "Bi" }, { id: "koala", name: "Koala" }], unmatched: ["Cú"] },
      shots: [
        { characterId: "bi", dialogue: "Chào!" },
        { characterId: "bi", dialogue: "Đi thôi!" },
        { characterId: "koala", dialogue: "" },
      ],
    });
    expect(view.lineCount).toEqual({ bi: 2 });
    expect(view.unmatched).toEqual(["Cú"]);
    expect(resolveCastMembers(view.cast, ["meo", "bi"], ["koala"])).toEqual([
      { id: "bi", name: "Bi" },
      { id: "meo", name: "meo" },
    ]);
  });

  it("evicts least recently used entry when exceeding MAX_CAST_CACHE_ENTRIES", () => {
    for (let i = 0; i < MAX_CAST_CACHE_ENTRIES; i++) {
      setCachedCast(`p:ep_${i}`, { cast: [{ id: `c${i}`, name: `C${i}` }], lineCount: {}, unmatched: [] });
    }
    expect(getCastCacheSize()).toBe(MAX_CAST_CACHE_ENTRIES);

    // Promote ep_0 to MRU so ep_1 becomes LRU
    expect(getCachedCast("p:ep_0")).toBeDefined();

    // Inserting one more evicts ep_1, keeping ep_0
    setCachedCast("p:ep_overflow", { cast: [], lineCount: {}, unmatched: [] });
    expect(getCastCacheSize()).toBe(MAX_CAST_CACHE_ENTRIES);
    expect(getCachedCast("p:ep_1")).toBeUndefined();
    expect(getCachedCast("p:ep_0")).toBeDefined();
    expect(getCachedCast("p:ep_overflow")).toBeDefined();
  });
});
