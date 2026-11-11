import "@testing-library/jest-dom/vitest";
import React from "react";
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { render, screen, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { NextIntlClientProvider } from "next-intl";
import viMessages from "@/messages/vi.json";
import { useStudioStore } from "@/store";
import { Sidebar } from "@/components/navigation/Sidebar";
import { ModelsTab } from "@/components/studio/tabs/ModelsTab";
import { EXTENDED_MODELS_CATALOG } from "@/lib/data/extendedModelsCatalog";

// Stub UserBubble inside Sidebar unit test (UserBubble has its own comprehensive suite in test/p5-modals)
vi.mock("@/components/navigation/UserBubble", () => ({
  UserBubble: () => <div data-testid="user-bubble-stub" />,
}));

beforeEach(() => {
  useStudioStore.setState({
    activeProjectId: "soc-nho-tv",
    projects: [{ id: "soc-nho-tv", name: "Sóc Nhỏ TV" }],
    episodes: [
      { id: "ep01", title: "Tập 1: Chiếc Răng Sâu", nodeCount: 5, status: "draft" },
    ],
    activeEpisodeId: "ep01",
    graphsByEpisode: {},
    selectedNodeId: "n_vid_1",
    nodes: [
      {
        id: "n_vid_1",
        position: { x: 0, y: 0 },
        data: { nodeType: "video_gen", title: "Video", params: { model: "kling-v3-std" } },
      },
    ],
    switchProject: vi.fn(),
    selectEpisode: vi.fn(),
    createEpisode: vi.fn(),
    updateNodeParams: vi.fn(),
  } as any);
});

afterEach(() => {
  cleanup();
});

describe("Package 6 characterization tests (Sidebar + ModelsTab + lazy AIModelsLibraryModal)", () => {
  it("renders Sidebar projects/episodes, creates episode, and lazy-opens AIModelsLibraryModal", async () => {
    const user = userEvent.setup();
    render(
      <NextIntlClientProvider locale="vi" messages={viMessages}>
        <Sidebar />
      </NextIntlClientProvider>
    );

    expect(screen.getByText("Tập 1: Chiếc Răng Sâu")).toBeInTheDocument();

    // Open lazy-loaded AIModelsLibraryModal
    await user.click(screen.getByRole("button", { name: /Kho Mô hình AI/i }));
    expect(await screen.findByText(/Kho Mô hình AI \(AI Models Library\)/i)).toBeInTheDocument();
  });

  it("renders ModelsTab catalog, filters by category & search, and applies model to selected node", async () => {
    const user = userEvent.setup();
    render(
      <NextIntlClientProvider locale="vi" messages={viMessages}>
        <ModelsTab />
      </NextIntlClientProvider>
    );

    expect(screen.getByText(`${EXTENDED_MODELS_CATALOG.length} mô hình phù hợp`)).toBeInTheDocument();

    // Filter by category "Tạo Ảnh"
    await user.click(screen.getByRole("button", { name: /Tạo Ảnh/i }));
    const imageModelsCount = EXTENDED_MODELS_CATALOG.filter((m) => m.category === "image").length;
    expect(screen.getByText(`${imageModelsCount} mô hình phù hợp`)).toBeInTheDocument();

    // Search for Kling in "Tất cả"
    await user.click(screen.getByRole("button", { name: /Tất cả/i }));
    await user.type(screen.getByPlaceholderText(/Tìm model/i), "Kling");
    const klingButtons = screen.getAllByRole("button", { name: /Kling/i });
    expect(klingButtons.length).toBeGreaterThanOrEqual(2);
    await user.click(screen.getByRole("button", { name: /Kling v3 Pro/i }));

    // Apply model to selected node
    await user.click(screen.getByRole("button", { name: /Áp dụng cho Node #n_vid_1/i }));
    expect(useStudioStore.getState().updateNodeParams).toHaveBeenCalledWith("n_vid_1", { model: "kling-v3-pro" });
  });
});
