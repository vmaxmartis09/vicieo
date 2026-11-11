import "@testing-library/jest-dom/vitest";
import React from "react";
import { describe, it, expect, beforeAll, afterEach, afterAll, beforeEach, vi } from "vitest";
import { render, screen, waitFor, cleanup, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
  usePathname: () => "/vi/studio",
  useSearchParams: () => new URLSearchParams(""),
}));

import { SocialHub } from "@/components/social/hub/SocialHub";
import { useSocialStore } from "@/store/modules/social";
import { useStudioStore } from "@/store";
import { SOCIAL_CORE_URL } from "@/lib/api/social";

const server = setupServer(
  http.all(`${SOCIAL_CORE_URL}/*`, ({ request }) => {
    const url = new URL(request.url);
    if (url.pathname.endsWith("/social/overview")) {
      return HttpResponse.json({
        accounts_count: 1,
        connected_accounts_count: 1,
        videos_count: 1,
        scheduled_count: 0,
        published_count: 0,
        failed_count: 0,
        recent_posts: [],
        upcoming_targets: [],
      });
    }
    if (url.pathname.endsWith("/social/queue")) {
      return HttpResponse.json({
        summary: {
          accounts_count: 1,
          max_accounts: 5,
          draft_count: 0,
          pending_approval_count: 0,
          overdue_count: 0,
          upcoming_count: 0,
          published_count: 0,
          failed_count: 0,
        },
        items: [],
      });
    }
    if (url.pathname.endsWith("/social/videos")) {
      return HttpResponse.json([
        {
          id: "vid_hub_1",
          title: "Video Hub Test",
          aspect_ratio: "9:16",
          duration_s: 20,
          size_mb: 5.2,
          posted_count: 0,
          thumb: "",
          variants: {},
        },
      ]);
    }
    if (url.pathname.endsWith("/social/accounts")) {
      return HttpResponse.json([
        {
          id: "acc_hub_1",
          platform: "tiktok",
          handle: "@socnho",
          display_name: "Sóc Nhỏ Hub",
          auth: { mode: "oauth" },
          status: "connected",
          default_hashtags: [],
          default_caption_footer: "",
        },
      ]);
    }
    return HttpResponse.json([]);
  })
);

beforeAll(() => server.listen());
beforeEach(() => {
  useStudioStore.setState({ activeProjectId: "soc-nho-tv" });
  useSocialStore.setState({
    isHubOpen: false,
    hubView: "overview",
    hubSelectedEntity: null,
    hubPreselectedVideoIds: [],
    isPublishCenterOpen: false,
    accounts: [],
    queueData: null,
  });
});
afterEach(() => {
  cleanup();
  server.resetHandlers();
});
afterAll(() => server.close());

describe("SocialHub characterization tests", () => {
  it("renders nothing when closed, opens with navigation views, and lazy-opens PublishCenterModal on plan action", async () => {
    const user = userEvent.setup();
    const { container } = render(<SocialHub />);
    expect(container.firstChild).toBeNull();

    act(() => {
      useSocialStore.getState().openHub({ view: "overview" });
    });

    await waitFor(() => {
      expect(screen.getByRole("dialog", { name: "Trung tâm Quản lý Social" })).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: "Tài khoản" }));
    expect(useSocialStore.getState().hubView).toBe("accounts");

    await user.click(screen.getByRole("button", { name: /Lên kế hoạch đăng/i }));

    await waitFor(
      () => {
        expect(useSocialStore.getState().isPublishCenterOpen).toBe(true);
        expect(screen.getByText(/Video Hub Test/i)).toBeInTheDocument();
      },
      { timeout: 5000 }
    );

    await user.keyboard("{Escape}");
    await waitFor(() => {
      expect(useSocialStore.getState().isPublishCenterOpen).toBe(false);
    });
    expect(useSocialStore.getState().isHubOpen).toBe(true);

    await user.keyboard("{Escape}");
    await waitFor(() => {
      expect(useSocialStore.getState().isHubOpen).toBe(false);
    });
  });
});
