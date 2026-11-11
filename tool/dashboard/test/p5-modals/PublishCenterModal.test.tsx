import "@testing-library/jest-dom/vitest";
import React from "react";
import { describe, it, expect, beforeAll, afterEach, afterAll, beforeEach, vi } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import { PublishCenterModal } from "@/components/social/PublishCenterModal";
import { ModalErrorBoundary } from "@/components/social/publish-center-modal/components/ModalErrorBoundary";
import { computeCompatibilityMatrix } from "@/components/social/publish-center-modal/utils/compatibilityMatrix";
import { useSocialStore } from "@/store/modules/social";
import { SOCIAL_CORE_URL } from "@/lib/api/social";

const mockVideos = [
  {
    id: "vid_1",
    title: "Tập 1: Sóc Nhỏ Đi Siêu Thị",
    aspect_ratio: "9:16",
    duration_s: 32,
    size_mb: 12.4,
    posted_count: 0,
    thumb: "",
    variants: { tiktok: "/tmp/vid_1_tt.mp4" },
  },
  {
    id: "vid_2",
    title: "Tập 2: Ngang Youtube",
    aspect_ratio: "16:9",
    duration_s: 45,
    size_mb: 18.0,
    posted_count: 1,
    thumb: "",
    variants: {},
  },
];

const mockAccounts = [
  {
    id: "acc_tt",
    platform: "tiktok",
    handle: "@socnhotv",
    display_name: "Sóc Nhỏ TikTok",
    auth: { mode: "oauth" },
    status: "connected",
    default_hashtags: ["#socnho"],
    default_caption_footer: "",
    groups: ["KOC"],
  },
  {
    id: "acc_yt",
    platform: "youtube",
    handle: "@socnhoyt",
    display_name: "Sóc Nhỏ YouTube",
    auth: { mode: "oauth" },
    status: "connected",
    default_hashtags: ["#shorts"],
    default_caption_footer: "",
    groups: [],
  },
];

let lastPreviewStrategy = "";
let committedPayload: any = null;

const server = setupServer(
  http.get(`${SOCIAL_CORE_URL}/projects/soc-nho-tv/social/videos`, () =>
    HttpResponse.json(mockVideos)
  ),
  http.post(`${SOCIAL_CORE_URL}/projects/soc-nho-tv/social/captions/generate`, () =>
    HttpResponse.json([
      {
        platform: "tiktok",
        title: "AI Title",
        caption: "Caption AI siêu cuốn cho Sóc Nhỏ!",
        hashtags: ["#socnhotv", "#hoathinh"],
      },
    ])
  ),
  http.post(`${SOCIAL_CORE_URL}/projects/soc-nho-tv/social/plans/preview`, async ({ request }) => {
    const body = (await request.json()) as { strategy: string };
    lastPreviewStrategy = body.strategy;
    return HttpResponse.json([
      {
        video_id: "vid_1",
        video_title: "Tập 1: Sóc Nhỏ Đi Siêu Thị",
        account_id: "acc_tt",
        account_name: "Sóc Nhỏ TikTok",
        platform: "tiktok",
        scheduled_at: "2026-11-10T19:00:00+07:00",
        variant_used: "tiktok_9_16",
        issues: [],
      },
    ]);
  }),
  http.post(`${SOCIAL_CORE_URL}/projects/soc-nho-tv/social/plans/commit`, async ({ request }) => {
    committedPayload = await request.json();
    return HttpResponse.json({
      created_posts_count: 1,
      created_targets_count: 1,
      posts: [],
    });
  })
);

beforeAll(() => server.listen());
beforeEach(() => {
  lastPreviewStrategy = "";
  committedPayload = null;
  useSocialStore.setState({
    accounts: mockAccounts as any,
    calendarData: { items: [], notes: [] } as any,
    posts: [],
    loadAccounts: vi.fn().mockResolvedValue(undefined),
    loadAll: vi.fn().mockResolvedValue(undefined),
  } as any);
});
afterEach(() => {
  server.resetHandlers();
  cleanup();
  vi.restoreAllMocks();
});
afterAll(() => server.close());

describe("PublishCenterModal characterization tests", () => {
  it("runs 5-step publish flow: selects videos/accounts, generates caption, schedules, and commits plan", async () => {
    const user = userEvent.setup();
    render(
      <PublishCenterModal projectId="soc-nho-tv" isOpen={true} onClose={vi.fn()} />
    );

    // Step 1: loads videos & supports unposted filter
    expect(await screen.findByText("Tập 1: Sóc Nhỏ Đi Siêu Thị")).toBeInTheDocument();
    expect(screen.getByText("Tập 2: Ngang Youtube")).toBeInTheDocument();

    await user.click(screen.getByLabelText("Chỉ hiện video chưa đăng"));
    expect(screen.queryByText("Tập 2: Ngang Youtube")).not.toBeInTheDocument();

    // Step 2: accounts & compatibility matrix
    await user.click(screen.getByRole("button", { name: /Tiếp tục/i }));
    expect(await screen.findByText(/Ma trận Tương thích/i)).toBeInTheDocument();

    // Deselect YouTube account, keep TikTok
    await user.click(screen.getAllByText("Sóc Nhỏ YouTube")[0]);
    expect(screen.getByText(/Chọn tài khoản \(1\/2\):/i)).toBeInTheDocument();

    // Step 3: content & AI caption generation + PhonePreview
    await user.click(screen.getByRole("button", { name: /Tiếp tục/i }));
    expect(await screen.findByText("Caption chính:")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /Viết caption tự động/i }));
    expect(await screen.findByDisplayValue("Caption AI siêu cuốn cho Sóc Nhỏ!")).toBeInTheDocument();

    // Step 4: auto-schedule preview & switch strategy to "Đăng ngay"
    await user.click(screen.getByRole("button", { name: /Tiếp tục/i }));
    expect(await screen.findByText(/Tổng cộng: 1 mục tiêu xuất bản/i)).toBeInTheDocument();
    expect(lastPreviewStrategy).toBe("auto");

    await user.click(screen.getByRole("button", { name: "Đăng ngay" }));
    await waitFor(() => expect(lastPreviewStrategy).toBe("now"));

    // Step 5: confirm & commit plan
    await user.click(screen.getByRole("button", { name: /Tiếp tục/i }));
    expect(await screen.findByText("Tóm tắt kế hoạch xuất bản:")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /Xác nhận lên lịch tất cả bài viết/i }));
    expect(await screen.findByText("Lên kế hoạch xuất bản thành công!")).toBeInTheDocument();
    expect(committedPayload?.targets).toHaveLength(1);
    expect(committedPayload?.targets[0]?.caption).toBe("Caption AI siêu cuốn cho Sóc Nhỏ!");
  });

  it("switches to Calendar tab cleanly", async () => {
    const user = userEvent.setup();
    render(
      <PublishCenterModal projectId="soc-nho-tv" isOpen={true} onClose={vi.fn()} />
    );

    expect(await screen.findByText("Kế hoạch đăng (5 bước)")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Xem Lịch Đăng" }));
    expect(screen.queryByText(/Bước 1 \/ 5/i)).not.toBeInTheDocument();
  });

  it("computeCompatibilityMatrix flags expired OAuth, oversized TikTok duration, and 16:9 warning", () => {
    const matrix = computeCompatibilityMatrix(
      [
        ...mockVideos,
        {
          id: "vid_long",
          title: "Too long",
          aspect_ratio: "9:16",
          duration_s: 605,
          size_mb: 50,
          posted_count: 0,
          thumb: "",
          variants: {},
        },
      ] as any,
      [
        mockAccounts[0],
        { ...mockAccounts[1], id: "acc_exp", status: "expired" },
      ] as any
    );

    expect(matrix["vid_1"]["acc_tt"].status).toBe("ok");
    expect(matrix["vid_2"]["acc_tt"].status).toBe("warn");
    expect(matrix["vid_long"]["acc_tt"].status).toBe("error");
    expect(matrix["vid_1"]["acc_exp"].status).toBe("error");
  });

  it("isolates modal crash with ModalErrorBoundary and recovers", async () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const user = userEvent.setup();
    let explode = true;
    function Bomb() {
      if (explode) throw new Error("Publish wizard crashed");
      return <div>Recovered wizard</div>;
    }
    const onClose = vi.fn();
    render(
      <ModalErrorBoundary onClose={onClose}>
        <Bomb />
      </ModalErrorBoundary>
    );
    expect(screen.getByText("Không thể hiển thị Trung tâm xuất bản")).toBeInTheDocument();
    expect(screen.getByText("Publish wizard crashed")).toBeInTheDocument();

    explode = false;
    await user.click(screen.getByRole("button", { name: "Thử lại" }));
    expect(screen.getByText("Recovered wizard")).toBeInTheDocument();
    consoleSpy.mockRestore();
  });
});
