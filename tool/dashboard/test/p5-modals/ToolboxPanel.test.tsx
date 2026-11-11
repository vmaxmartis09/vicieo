import "@testing-library/jest-dom/vitest";
import React from "react";
import { describe, it, expect, beforeAll, afterEach, afterAll, vi } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import { ToolboxPanel } from "@/components/toolbox/ToolboxPanel";
import { ToolboxErrorBoundary } from "@/components/toolbox/panel/shared/ToolboxErrorBoundary";

const mockTools = [
  { id: "bg-remove", name: "Tách nền", version: "1.0.0", status: "verified", license: { code: "MIT", commercial_ok: true }, requires_gpu: false, requires_ffmpeg: false },
  { id: "stt-align", name: "Canh từ STT", version: "1.2.0", status: "verified", license: { code: "MIT", commercial_ok: true }, requires_gpu: false, requires_ffmpeg: true },
  { id: "depth-layers", name: "Tách lớp độ sâu", version: "2.0.0", status: "installed", requires_gpu: false, requires_ffmpeg: false },
  { id: "scene-cut", name: "Cắt cảnh video", version: "1.1.0", status: "verified", requires_gpu: false, requires_ffmpeg: true },
  { id: "voice-clone", name: "Clone giọng nói", version: "1.5.0", status: "verified", requires_gpu: false, requires_ffmpeg: true },
  { id: "ffmpeg-bin", name: "FFmpeg Binary", version: "7.0", status: "verified", requires_gpu: false, requires_ffmpeg: false },
];

const mockVaults = {
  cutouts: [
    {
      id: "cut_1",
      source: "soc_nho.png",
      metadata: { width: 512, height: 512 },
      files: { cutout: "/tmp/cut_1.png", mask: "/tmp/cut_1_mask.png" },
    },
  ],
  audios: [],
  voices: [
    {
      id: "soc-nho",
      metadata: { display_name: "Sóc Nhỏ", status: "ready", gender: "child", dialect: "south" },
      files: { samples: ["sample.wav"] },
    },
  ],
};

const mockTargets = {
  projects: [{ id: "soc-nho-tv", name: "Sóc Nhỏ TV" }],
  characters: { "soc-nho-tv": [{ id: "bi", name: "Sóc Nhỏ (Bi)" }] },
  active: "soc-nho-tv",
  sharedDir: "/tmp/shared",
};

const server = setupServer(
  http.get("*/api/toolbox", ({ request }) => {
    const url = new URL(request.url);
    const action = url.searchParams.get("action");
    if (action === "list") return HttpResponse.json({ ok: true, tools: mockTools });
    if (action === "vaults") return HttpResponse.json({ ok: true, vaults: mockVaults });
    if (action === "targets") return HttpResponse.json({ ok: true, ...mockTargets });
    return HttpResponse.json({ ok: false, error: "Unknown action" }, { status: 400 });
  }),
  http.post("*/api/toolbox/upload", () => {
    return HttpResponse.json({ ok: true, path: "/tmp/uploaded.png", name: "uploaded.png" });
  }),
  http.post("*/api/toolbox", async ({ request }) => {
    const body = (await request.json()) as { toolId: string };
    if (body.toolId === "bg-remove") {
      return HttpResponse.json({
        ok: true,
        result: {
          outputs: [{ kind: "cutout", path: "/tmp/cutout.png" }],
          elapsed_s: 1.2,
        },
      });
    }
    if (body.toolId === "voice-clone") {
      return HttpResponse.json({
        ok: true,
        result: {
          outputs: [{ kind: "voice_preview", path: "/tmp/preview.wav" }],
          elapsed_s: 4,
        },
      });
    }
    return HttpResponse.json({ ok: true, result: { outputs: [], elapsed_s: 0.5 } });
  }),
  http.post("*/api/toolbox/save", () => {
    return HttpResponse.json({ ok: true, relative: "library/cutout.png" });
  })
);

beforeAll(() => server.listen());
afterEach(() => {
  server.resetHandlers();
  cleanup();
  vi.restoreAllMocks();
});
afterAll(() => server.close());

describe("ToolboxPanel characterization tests", () => {
  it("loads tools catalog and switches across all 5 workspaces", async () => {
    const user = userEvent.setup();
    render(<ToolboxPanel />);

    // Wait for initial load & default opened bg-remove workspace
    expect(await screen.findByText("Toolbox cục bộ")).toBeInTheDocument();
    expect(await screen.findByLabelText("Nhập ảnh cần tách nền")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Tách nền" })).toBeDisabled();

    // 2. Switch to AudioWorkspace (stt-align)
    await user.click(screen.getByRole("button", { name: /Canh từ STT/i }));
    expect(await screen.findByLabelText("Nhập mẫu giọng / file âm thanh")).toBeInTheDocument();
    expect(screen.getByLabelText("Ngôn ngữ")).toHaveValue("vi");

    // 3. Switch to DepthWorkspace (depth-layers)
    await user.click(screen.getByRole("button", { name: /Tách lớp độ sâu/i }));
    expect(await screen.findByLabelText("Nhập ảnh cần tách lớp")).toBeInTheDocument();

    // 4. Switch to SceneWorkspace (scene-cut)
    await user.click(screen.getByRole("button", { name: /Cắt cảnh video/i }));
    expect(await screen.findByLabelText("Nhập video cần tách cảnh")).toBeInTheDocument();

    // 5. Switch to VoiceCloneWorkspace (voice-clone)
    await user.click(screen.getByRole("button", { name: /Clone giọng nói/i }));
    expect(await screen.findByLabelText("Nhập mẫu giọng cần clone")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Clone giọng" })).toBeDisabled();

    // Extra: Switch to ffmpeg-bin card
    await user.click(screen.getByRole("button", { name: /FFmpeg Binary/i }));
    expect(await screen.findByRole("button", { name: "Kiểm tra / cài" })).toBeInTheDocument();
  });

  it("uploads an image, runs bg-remove, and allows saving output to project", async () => {
    const user = userEvent.setup();
    render(<ToolboxPanel />);

    const dropzone = await screen.findByLabelText("Nhập ảnh cần tách nền");
    const fileInput = dropzone.querySelector('input[type="file"]') as HTMLInputElement;
    expect(fileInput).not.toBeNull();

    const file = new File(["fake-png"], "avatar.png", { type: "image/png" });
    await user.upload(fileInput, file);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Tách nền" })).toBeEnabled();
    });

    await user.click(screen.getByRole("button", { name: "Tách nền" }));

    expect(await screen.findByAltText("Ảnh đã tách nền")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Lưu" })).toBeEnabled();

    await user.click(screen.getByRole("button", { name: "Lưu" }));
    expect(await screen.findByText(/Đã lưu: library\/cutout\.png/i)).toBeInTheDocument();
  });

  it("switches between subtabs (cutouts, audio, voices) and displays vault items", async () => {
    const user = userEvent.setup();
    render(<ToolboxPanel />);

    await screen.findByLabelText("Nhập ảnh cần tách nền");

    await user.click(screen.getByRole("button", { name: /Tách nền\s*\(1\)/i }));
    expect(await screen.findByText(/Nguồn: soc_nho\.png/i)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /Canh từ\s*\(0\)/i }));
    expect(await screen.findByText("Chưa có bản phiên âm")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /Giọng\s*\(1\)/i }));
    expect(await screen.findByText("Sóc Nhỏ")).toBeInTheDocument();
    expect(screen.getByText("sẵn sàng")).toBeInTheDocument();
  });

  it("isolates runtime errors via ToolboxErrorBoundary and supports recovery", async () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const user = userEvent.setup();
    let shouldThrow = true;
    function Bomb() {
      if (shouldThrow) throw new Error("Simulated toolbox crash");
      return <div>Toolbox recovered</div>;
    }
    render(
      <div>
        <div>Outside Studio Shell</div>
        <ToolboxErrorBoundary>
          <Bomb />
        </ToolboxErrorBoundary>
      </div>
    );
    expect(screen.getByText("Outside Studio Shell")).toBeInTheDocument();
    expect(screen.getByText("Không thể hiển thị Toolbox")).toBeInTheDocument();
    expect(screen.getByText("Simulated toolbox crash")).toBeInTheDocument();

    shouldThrow = false;
    await user.click(screen.getByRole("button", { name: "Thử lại" }));
    expect(screen.getByText("Toolbox recovered")).toBeInTheDocument();
    consoleSpy.mockRestore();
  });
});
