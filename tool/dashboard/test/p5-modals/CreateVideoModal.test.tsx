import "@testing-library/jest-dom/vitest";
import React from "react";
import { describe, it, expect, beforeAll, afterEach, afterAll, beforeEach, vi } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import { CreateVideoModal } from "@/components/studio/CreateVideoModal";
import { ModalErrorBoundary } from "@/components/studio/create-video-modal/components/ModalErrorBoundary";
import { useStudioStore } from "@/store";
import { saveSocialAccountConnection } from "@/lib/socialConfig";

const mockTemplates = [
  {
    id: "tpl_sales_1",
    name: "Review Sản Phẩm Sóc Nhỏ",
    version: 1,
    category: "sales",
    platforms: ["tiktok", "shorts"],
    default_platform: "tiktok",
    duration: { mode: "range", min: 15, max: 45 },
    slots: [
      { key: "title", type: "text", label: "Tiêu đề ngắn", required: true, default: "Món ngon hôm nay" },
    ],
    voice: { required: true, default_voice_id: "soc-nho" },
    captions: { style: "tiktok", position: "bottom" },
    bgm: { mood: "upbeat", duck_db: -12 },
    scenes: [{ role: "hook", dur: "3s" }],
    responsive: {},
  },
  {
    id: "tpl_story_2",
    name: "Truyện Kể Rừng Xanh",
    version: 1,
    category: "story",
    platforms: ["youtube"],
    default_platform: "youtube",
    duration: { mode: "range", min: 30, max: 60 },
    slots: [],
    voice: { required: true, default_voice_id: "soc-nho" },
    captions: { style: "cinema", position: "bottom" },
    bgm: { mood: "calm", duck_db: -10 },
    scenes: [{ role: "intro", dur: "5s" }],
    responsive: {},
  },
];

let validateCalled = false;
let runCalled = false;

const server = setupServer(
  http.get("*/video/templates", () => HttpResponse.json(mockTemplates)),
  http.get("*/video/platforms", () =>
    HttpResponse.json([
      {
        id: "tiktok",
        name: "TikTok",
        width: 1080,
        height: 1920,
        aspect_ratio: "9:16",
        fps: 30,
        safe_zone: { top: 180, bottom: 320, left: 40, right: 40 },
        loudness_lufs: -14,
        audio_bitrate: "192k",
        video_bitrate: "6M",
        codec: "h264",
        notes: "",
      },
    ])
  ),
  http.get("*/voices", () =>
    HttpResponse.json([
      { id: "soc-nho", name: "Sóc Nhỏ (Bi)", display_name: "Sóc Nhỏ", status: "ready", sampleUrl: "/voices/soc-nho/sample" },
    ])
  ),
  http.post("*/projects/*/ai/script", () =>
    HttpResponse.json({ markdown: "Mở đầu gốc\nThân bài chi tiết" })
  ),
  http.post("*/video/craft/hooks", () =>
    HttpResponse.json({
      best_variant_id: "A",
      selected_hook: "Khoan đã! Bí mật của rừng xanh",
      variants: [
        { variant_id: "A", text: "Khoan đã! Bí mật của rừng xanh", score: { total_score: 92 } },
        { variant_id: "B", text: "3 giây kiểm chứng ngay!", score: { total_score: 88 } },
      ],
    })
  ),
  http.post("*/projects/*/estimate", () =>
    HttpResponse.json({ totalEstimatedCostUsd: 0.12 })
  ),
  http.post("*/video/craft/lint", () =>
    HttpResponse.json({ passed: true, error_count: 0, warning_count: 0, total_checks: 6, issues: [] })
  ),
  http.post("*/graph/validate", () => {
    validateCalled = true;
    return HttpResponse.json({ isValid: true, errors: [], warnings: [] });
  }),
  http.post("*/projects/*/run", () => {
    runCalled = true;
    return HttpResponse.json({ isSuccess: true, totalCostUsd: 0.0 });
  })
);

beforeAll(() => {
  Object.defineProperty(HTMLMediaElement.prototype, "play", {
    configurable: true,
    value: vi.fn().mockResolvedValue(undefined),
  });
  Object.defineProperty(HTMLMediaElement.prototype, "pause", {
    configurable: true,
    value: vi.fn(),
  });
  server.listen();
});
beforeEach(() => {
  validateCalled = false;
  runCalled = false;
  localStorage.clear();
  saveSocialAccountConnection("tiktok", "@socnhotv", { client_key: "abc" });
  useStudioStore.setState({
    activeProjectId: "soc-nho-tv",
    isMockMode: true,
    jobs: [],
    episodes: [],
    episodesLoadedProjectId: "soc-nho-tv",
  } as any);
});
afterEach(() => {
  server.resetHandlers();
  cleanup();
  vi.restoreAllMocks();
});
afterAll(() => server.close());

describe("CreateVideoModal characterization tests", () => {
  it("completes all 3 wizard steps, retains form state across step switches, and triggers creation", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();

    render(<CreateVideoModal isOpen={true} onClose={onClose} />);

    // Step 1: loads templates & filters by category
    expect(await screen.findByText("Review Sản Phẩm Sóc Nhỏ")).toBeInTheDocument();
    expect(screen.getByText("Truyện Kể Rừng Xanh")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Truyện kể" }));
    expect(screen.queryByText("Review Sản Phẩm Sóc Nhỏ")).not.toBeInTheDocument();
    expect(screen.getByText("Truyện Kể Rừng Xanh")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Tất cả mẫu" }));
    await user.click(screen.getByText("Review Sản Phẩm Sóc Nhỏ"));

    // Proceed to Step 2
    await user.click(screen.getByRole("button", { name: /Tiếp tục/i }));
    expect(screen.getByText("(Bước 2/3)")).toBeInTheDocument();

    // Enter topic & generate script + Hook variants
    const topicInput = screen.getByPlaceholderText(/Nhập ý tưởng/i);
    await user.type(topicInput, "Hạt dẻ thần kỳ");
    await user.click(screen.getByRole("button", { name: /Tự viết kịch bản/i }));

    expect(await screen.findByText(/Phương án B/i)).toBeInTheDocument();
    await user.click(screen.getByText(/Phương án B/i));

    // Confirm script updates line 0
    const scriptArea = document.querySelectorAll("textarea")[0] as HTMLTextAreaElement;
    expect(scriptArea.value).toContain("3 giây kiểm chứng ngay!");

    // Proceed to Step 3
    await user.click(screen.getByRole("button", { name: /Tiếp tục/i }));
    expect(screen.getByText("(Bước 3/3)")).toBeInTheDocument();
    expect(await screen.findByText("$0.12 USD")).toBeInTheDocument();

    // Run QA lint check
    await user.click(screen.getByRole("button", { name: /Kiểm tra ngay/i }));
    expect(await screen.findByText(/Đạt toàn bộ tiêu chuẩn Creative QA/i)).toBeInTheDocument();

    // Navigate back to Step 2 -> verify form values are preserved
    await user.click(screen.getByRole("button", { name: "Quay lại" }));
    expect(screen.getByText("(Bước 2/3)")).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/Nhập ý tưởng/i)).toHaveValue("Hạt dẻ thần kỳ");

    // Forward back to Step 3 and submit creation
    await user.click(screen.getByRole("button", { name: /Tiếp tục/i }));
    await user.click(screen.getByRole("button", { name: /Tạo video ngay/i }));

    await waitFor(() => {
      expect(validateCalled).toBe(true);
      expect(runCalled).toBe(true);
      expect(onClose).toHaveBeenCalledTimes(1);
    });
  });

  it("supports batch mode topic counting and retains product inputs", async () => {
    const user = userEvent.setup();
    render(<CreateVideoModal isOpen={true} onClose={vi.fn()} />);

    expect(await screen.findByText("Review Sản Phẩm Sóc Nhỏ")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Tiếp tục/i }));

    // Fill product inputs (available on sales category)
    const prodInput = screen.getByPlaceholderText("Tên sản phẩm...");
    await user.type(prodInput, "Kem chống nắng Bi");
    expect(prodInput).toHaveValue("Kem chống nắng Bi");

    // Toggle batch mode
    const checkboxes = screen.getAllByRole("checkbox");
    await user.click(checkboxes[0]);

    const batchTextarea = screen.getByPlaceholderText(/Chủ đề 1:/i);
    await user.type(batchTextarea, "Tập 1\nTập 2");
    expect(screen.getByText("2")).toBeInTheDocument();

    // Step 3 button reflects batch count
    await user.click(screen.getByRole("button", { name: /Tiếp tục/i }));
    expect(screen.getByRole("button", { name: /Tạo 2 video hàng loạt/i })).toBeInTheDocument();
  });

  it("isolates modal crash with ModalErrorBoundary and recovers", async () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const user = userEvent.setup();
    let explode = true;
    function Bomb() {
      if (explode) throw new Error("Modal exploded");
      return <div>Recovered modal</div>;
    }
    const onClose = vi.fn();
    render(
      <ModalErrorBoundary onClose={onClose}>
        <Bomb />
      </ModalErrorBoundary>
    );
    expect(screen.getByText("Không thể hiển thị Tạo video")).toBeInTheDocument();
    expect(screen.getByText("Modal exploded")).toBeInTheDocument();

    explode = false;
    await user.click(screen.getByRole("button", { name: "Thử lại" }));
    expect(screen.getByText("Recovered modal")).toBeInTheDocument();
    consoleSpy.mockRestore();
  });
});
