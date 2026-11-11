import "@testing-library/jest-dom/vitest";
import React from "react";
import { describe, it, expect, beforeAll, afterEach, afterAll, beforeEach } from "vitest";
import { render, screen, waitFor, cleanup, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import { SOCIAL_CORE_URL } from "@/lib/api/social";
import { useStudioStore } from "@/store";
import { useSocialStore } from "@/store/modules/social";
import { usePreview } from "@/lib/preview";
import { saveSocialAccountConnection } from "@/lib/socialConfig";
import { PublishTab } from "@/components/studio/tabs/PublishTab";
import {
  computeSummaryCounts,
  filterUpcomingQueueItems,
  mapConnectedPlatformFlags,
} from "@/components/studio/tabs/publish-tab/utils/publishQueueMappers";

const mockAccounts = [
  {
    id: "acc_tt_1",
    platform: "tiktok",
    handle: "@socnhotv",
    display_name: "Sóc Nhỏ Official",
    auth: { mode: "oauth" },
    status: "connected",
    default_hashtags: ["#socnhotv"],
    default_caption_footer: "",
    groups: [],
  },
];

const mockQueue = {
  summary: {
    upcoming_count: 2,
    draft_count: 1,
    pending_approval_count: 1,
    overdue_count: 1,
    failed_count: 2,
    published_count: 4,
  },
  items: [
    {
      target_id: "tgt_sched_1",
      post_id: "post_1",
      account_id: "acc_tt_1",
      platform: "tiktok",
      status: "scheduled",
      video_title: "Tập 1: Chiếc Răng Sâu",
      title: "Tập 1",
      scheduled_at: "2026-11-10T19:00:00+07:00",
      thumbnail: "",
      caption: "Hôm nay Bi đi khám răng!",
    },
    {
      target_id: "tgt_pub_skip",
      post_id: "post_2",
      account_id: "acc_tt_1",
      platform: "tiktok",
      status: "published",
      video_title: "Video Đã Đăng (Không Hiện Ở Sắp Đăng)",
      title: "Đã đăng",
      thumbnail: "",
    },
  ],
};

const server = setupServer(
  http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/accounts`, () =>
    HttpResponse.json(mockAccounts)
  ),
  http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/queue`, () =>
    HttpResponse.json(mockQueue)
  ),
  http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/analytics/learnings`, () =>
    HttpResponse.json({
      learnings_md: "### Đúc kết tuần\n- Hook câu hỏi giúp tăng retention +14%.",
      recommendations: {
        next_episode_suggestions: {
          hook_recommendation: "Mở đầu 1.5s bằng tình huống bất ngờ",
          recommended_shot_pacing_s: 2.1,
          target_duration_s: 45,
          character_ratio: "9:16",
        },
      },
    })
  ),
  http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/analytics/overview`, () =>
    HttpResponse.json({
      kpis: {
        total_views: 12500,
        avg_retention_pct: 72,
        engagement_rate: 8.4,
      },
      views_by_day: {},
      outliers: { top_outliers: [], underperforming: [] },
    })
  ),
  http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/packages/:postId/:platform`, () =>
    HttpResponse.json({
      post_id: "post_1",
      platform: "tiktok",
      package_dir: "output/pkg",
      zip_path: "/tmp/pkg.zip",
      zip_filename: "post_1_tiktok.zip",
      has_real_video: true,
      caption: "Hôm nay Bi đi khám răng!",
      hashtags: "#socnhotv #hoathinh",
      checklist: "- Kiểm tra âm lượng -14 LUFS",
    })
  ),
  http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/videos`, () => HttpResponse.json([]))
);

beforeAll(() => server.listen());

beforeEach(() => {
  localStorage.clear();
  usePreview.getState().close();
  useStudioStore.setState({ activeProjectId: "soc-nho-tv" });
  useSocialStore.setState({
    accounts: [],
    queueData: null,
    queueBucket: "upcoming",
    isHubOpen: false,
    hubView: "overview",
    isPublishCenterOpen: false,
    publishCenterTab: "plan",
  });
});

afterEach(() => {
  server.resetHandlers();
  cleanup();
  localStorage.clear();
});

afterAll(() => server.close());

describe("PublishTab characterization tests", () => {
  it("loads queue summary, upcoming posts, learnings KPIs, and navigates hub buckets + details", async () => {
    const user = userEvent.setup();
    render(<PublishTab />);

    // Queue summary pills + banner
    expect(await screen.findByText("1/10 tài khoản kết nối")).toBeInTheDocument();
    expect(screen.getAllByText("Tập 1: Chiếc Răng Sâu").length).toBeGreaterThanOrEqual(1);
    expect(
      screen.queryByText("Video Đã Đăng (Không Hiện Ở Sắp Đăng)")
    ).not.toBeInTheDocument();

    // Summary counters including overdue + failed = 3
    expect(screen.getByRole("button", { name: /Sắp tới/ })).toHaveTextContent("2");
    expect(screen.getByRole("button", { name: /Quá hạn\/Lỗi/ })).toHaveTextContent("3");

    // Click "Bản nháp" pill sets bucket & opens Hub
    await user.click(screen.getByRole("button", { name: /Bản nháp/ }));
    expect(useSocialStore.getState().queueBucket).toBe("draft");
    expect(useSocialStore.getState().hubView).toBe("queue");
    expect(useSocialStore.getState().isHubOpen).toBe(true);

    // Learnings KPIs & Next Episode recommendation
    expect(await screen.findByText("72%")).toBeInTheDocument();
    expect(screen.getByText("8.4%")).toBeInTheDocument();
    expect(
      screen.getByText("Mở đầu 1.5s bằng tình huống bất ngờ")
    ).toBeInTheDocument();

    // Toggle full learnings markdown
    await user.click(screen.getByRole("button", { name: "Xem đúc kết chi tiết" }));
    expect(screen.getByText(/Hook câu hỏi giúp tăng retention/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thu gọn" }));
    expect(screen.queryByText(/Hook câu hỏi giúp tăng retention/)).not.toBeInTheDocument();
  });

  it("handles brand connection drawer, social-accounts-updated event, and quick modals", async () => {
    const user = userEvent.setup();
    render(<PublishTab />);

    await screen.findByText("1/10 tài khoản kết nối");

    // Sync via localStorage + custom event
    act(() => {
      saveSocialAccountConnection("tiktok", "@socnhotv_official", {
        creator_handle: "@socnhotv_official",
      });
    });

    const tiktokBtn = screen.getByRole("button", { name: /tiktok/i });
    await waitFor(() => {
      expect(tiktokBtn.querySelector('[title="Đã kết nối"]')).toBeInTheDocument();
    });

    // Clicking brand button opens custom SocialConnectDrawer in preview pane
    await user.click(tiktokBtn);
    const previewItem = usePreview.getState().item;
    expect(previewItem?.kind).toBe("custom");
    if (previewItem?.kind === "custom") {
      expect(previewItem.title).toBe("Kết nối tài khoản TIKTOK");
    }

    // Clicking "Gói đăng (.zip)" opens ManualPackageModal & loads package
    await user.click(screen.getByRole("button", { name: /Gói đăng \(\.zip\)/i }));
    expect(await screen.findByText("- Kiểm tra âm lượng -14 LUFS")).toBeInTheDocument();

    // Clicking "Lên kế hoạch đăng" opens PublishCenterModal
    await user.click(screen.getByRole("button", { name: "Lên kế hoạch đăng" }));
    expect(useSocialStore.getState().isPublishCenterOpen).toBe(true);
    expect(useSocialStore.getState().publishCenterTab).toBe("plan");
  });

  it("validates pure publish queue & connected platform mappers", () => {
    expect(
      computeSummaryCounts({
        summary: {
          upcoming_count: 3,
          draft_count: 1,
          pending_approval_count: 2,
          overdue_count: 4,
          failed_count: 1,
        },
        items: [],
      } as any)
    ).toEqual({
      upcomingCount: 3,
      draftCount: 1,
      pendingCount: 2,
      overdueOrFailedCount: 5,
    });

    expect(
      filterUpcomingQueueItems(mockQueue as any).map((i) => i.target_id)
    ).toEqual(["tgt_sched_1"]);

    expect(
      mapConnectedPlatformFlags({
        tiktok: { platform: "tiktok", connected: true },
        youtube: { platform: "youtube", connected: false },
      })
    ).toEqual({ tiktok: true, youtube: false, facebook: false, instagram: false });
  });
});
