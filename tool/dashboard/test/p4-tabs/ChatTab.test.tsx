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

const mockPush = vi.fn();
vi.mock("@/i18n/routing", () => ({
  useRouter: () => ({ push: mockPush }),
  usePathname: () => "/studio",
}));

import { ChatTab } from "@/components/studio/tabs/ChatTab";
import { ChatErrorBoundary } from "@/components/studio/tabs/chat-tab/components/ChatErrorBoundary";
import { extractCanvasImages } from "@/components/studio/tabs/chat-tab/utils/chatMediaUtils";
import { buildDirectMediaGraphUpdate } from "@/components/studio/tabs/chat-tab/utils/chatGraphBuilders";

let lastChatBody: { prompt?: string; context?: Record<string, unknown> } | null = null;

const server = setupServer(
  http.post(`${CORE_URL}/projects/:pid/chat`, async ({ request }) => {
    lastChatBody = (await request.json()) as { prompt?: string; context?: Record<string, unknown> };
    return HttpResponse.json({
      reply: "Đã tiếp nhận: thêm node Quality Check trước Video Gen.",
      mode: "provider",
      actions: [
        {
          type: "add_node",
          nodeType: "quality_check",
          afterNodeType: "image_gen",
          summaryVi: "Đã thêm bước Kiểm tra chất lượng",
          summaryEn: "Added Quality Check gate",
        },
      ],
    });
  })
);

function renderChat() {
  return render(
    <NextIntlClientProvider locale="vi" messages={viMessages}>
      <ChatTab />
    </NextIntlClientProvider>
  );
}

beforeAll(() => {
  Element.prototype.scrollIntoView = vi.fn();
  server.listen();
});

beforeEach(() => {
  lastChatBody = null;
  mockPush.mockClear();
  useStudioStore.setState({
    activeProjectId: "soc-nho-tv",
    projects: [
      {
        id: "soc-nho-tv",
        name: "Sóc Nhỏ TV",
        charactersCount: 2,
        episodesCount: 1,
        aspectRatio: "9:16",
        defaultVoiceId: "soc-nho",
        createdAt: "2026-11-01T00:00:00Z",
      },
    ],
    characters: [
      {
        id: "soc_nho",
        projectId: "soc-nho-tv",
        name: "Sóc Nhỏ (Bi)",
        role: "main",
        heightCm: 100,
        scaleRatio: 1,
        voiceId: "soc-nho",
        promptLock: "Bi 3D squirrel",
        referenceImages: ["/demo/bi.png"],
        memory: { personality: "Tò mò", catchphrases: [], habits: [], pastEventsSummary: [] },
      },
    ],
    episodes: [
      {
        id: "ep_01",
        projectId: "soc-nho-tv",
        episodeNumber: 1,
        title: "Tập 1: Chiếc Răng Sâu",
        nodeCount: 1,
        status: "draft",
        updatedAt: "2026-11-01T00:00:00Z",
      },
    ],
    activeEpisodeId: "ep_01",
    nodes: [
      {
        id: "node_img_gen_1",
        type: "studioCard",
        position: { x: 100, y: 100 },
        selected: false,
        data: {
          nodeType: "image_gen",
          status: "succeeded",
          params: { prompt: "Sóc Nhỏ ở công viên", _customName: "Ảnh Sóc Nhỏ" },
          previewUrl: "/demo/scene1.png",
          generations: [],
        },
      },
    ],
    edges: [],
    selectedNodeId: null,
    aspectRatio: "9:16",
    isMockMode: true,
    jobs: [],
    chatContext: {
      focus: "flow",
      model: "gemini-3.8-flash",
      flowMode: "current",
      characterId: null,
      characterMode: "update",
    },
    chatByProject: {
      "soc-nho-tv": [],
    },
    pendingConfirmation: null,
  } as any);
});

afterEach(() => {
  server.resetHandlers();
  cleanup();
  vi.restoreAllMocks();
});

afterAll(() => server.close());

describe("ChatTab characterization tests", () => {
  it("expands from capsule, sends a message via Core API, displays reply, and supports Undo on actionCard", async () => {
    const user = userEvent.setup();
    renderChat();

    // Collapsed capsule -> click to expand
    await user.click(screen.getByText(/Hỏi đáp hoặc gõ \/run, \/flow, \/video/i));
    expect(screen.getByRole("tablist", { name: "Chat context focus" })).toBeInTheDocument();

    const textarea = screen.getByPlaceholderText(/Mô tả ý tưởng/i);
    await user.type(textarea, "Thêm bước kiểm tra chất lượng");
    await user.click(screen.getByRole("button", { name: "Gửi" }));

    await waitFor(() => {
      expect(lastChatBody?.prompt).toBe("Thêm bước kiểm tra chất lượng");
    });

    expect(
      await screen.findByText("Đã tiếp nhận: thêm node Quality Check trước Video Gen.")
    ).toBeInTheDocument();
    expect(screen.getByText("Đã thêm bước Kiểm tra chất lượng")).toBeInTheDocument();
    expect(useStudioStore.getState().nodes.length).toBe(2);

    // Undo restores previous nodes snapshot
    await user.click(screen.getByRole("button", { name: "Hoàn tác" }));
    expect(useStudioStore.getState().nodes.length).toBe(1);
    expect(screen.getByText("Đã hoàn tác")).toBeInTheDocument();
  });

  it("handles /run slash command confirmation dialog and executes flow job", async () => {
    const user = userEvent.setup();
    renderChat();

    await user.click(screen.getByText(/Hỏi đáp hoặc gõ \/run, \/flow, \/video/i));
    const textarea = screen.getByPlaceholderText(/Mô tả ý tưởng/i);
    await user.type(textarea, "/run");
    await user.click(screen.getByRole("button", { name: "Gửi" }));

    const dialog = await screen.findByRole("dialog", {
      name: "Xác nhận thao tác có chi phí / đăng ra ngoài",
    });
    expect(dialog).toHaveTextContent(/Chạy flow “Tập 1: Chiếc Răng Sâu”/);

    await user.click(screen.getByRole("button", { name: "Đồng ý thực hiện" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(useStudioStore.getState().jobs.length).toBe(1);
    expect(useStudioStore.getState().jobs[0]?.status).toBe("completed");
  });

  it("executes /grid directly on Canvas and surfaces Selected Image quick toolbar", async () => {
    const user = userEvent.setup();
    useStudioStore.setState({ selectedNodeId: "node_img_gen_1" });
    renderChat();

    // Selected image toolbar renders above collapsed bar
    expect(screen.getByText("Ảnh Sóc Nhỏ")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Tạo Video" }));

    // Clicking "Tạo Video" expands composer in video mode with canvas image attached
    expect(screen.getByText("Chế độ: Ảnh → Video")).toBeInTheDocument();

    // Submit directly with attachment
    await user.click(screen.getByRole("button", { name: "Gửi" }));
    await waitFor(() => {
      expect(useStudioStore.getState().nodes.some((n) => n.data.nodeType === "video_gen")).toBe(true);
    });
    expect(useStudioStore.getState().edges.length).toBe(1);
  });

  it("validates pure graph helpers extractCanvasImages and buildDirectMediaGraphUpdate", () => {
    const state = useStudioStore.getState();
    const imgs = extractCanvasImages(state.nodes, state.characters);
    expect(imgs.map((x) => x.name)).toEqual(["Ảnh Sóc Nhỏ", "Sóc Nhỏ (Bi)"]);

    const built = buildDirectMediaGraphUpdate({
      mode: "image_list",
      promptBody: "Sóc Nhỏ đi học",
      currentNodes: state.nodes,
      currentEdges: [],
      craftTags: ["Golden Hour"],
      attachments: [],
      aspectRatio: "9:16",
      chatModel: "gemini-3.8-flash",
      vi: true,
    });
    expect(built.nextNodes.length).toBe(2);
    expect(built.nextNodes[1]?.data.params?.storyboardGrid).toBe(true);
    expect(built.nextNodes[1]?.data.params?.urls).toHaveLength(4);
  });

  it("isolates Chat render crashes inside ChatErrorBoundary with retry recovery", async () => {
    const user = userEvent.setup();
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    let shouldThrow = true;

    function Bomb() {
      if (shouldThrow) throw new Error("Chat crash test");
      return <div>Chat recovered</div>;
    }

    render(
      <div>
        <div>Studio Shell Alive</div>
        <ChatErrorBoundary>
          <Bomb />
        </ChatErrorBoundary>
      </div>
    );

    expect(screen.getByText("Studio Shell Alive")).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent(/Chat crash test/);

    shouldThrow = false;
    await user.click(screen.getByRole("button", { name: "Thử lại" }));
    expect(screen.getByText("Chat recovered")).toBeInTheDocument();
    consoleSpy.mockRestore();
  });
});
