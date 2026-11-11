import { describe, it, expect, beforeAll, afterEach, afterAll, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import { SOCIAL_CORE_URL } from "@/lib/api/social";
import { useSocialStore } from "@/store/modules/social";
import {
  applyTargetTimeToCalendar,
  applyTargetTimeToPosts,
  countUnreadNotifications,
  findTargetScheduledTime,
} from "@/store/modules/social/utils/socialHelpers";

let failTimeUpdate = false;
const updatedTimes: Array<{ postId: string; targetId: string; time: string }> = [];

const server = setupServer(
  http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/accounts`, () =>
    HttpResponse.json([
      {
        id: "acc_1",
        platform: "tiktok",
        handle: "@socnho",
        display_name: "Sóc Nhỏ",
        auth: { mode: "oauth" },
        status: "connected",
        default_hashtags: [],
        default_caption_footer: "",
      },
    ])
  ),
  http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/posts`, () =>
    HttpResponse.json([
      {
        id: "post_1",
        title: "Tập 1",
        targets: [{ id: "tgt_1", account_id: "acc_1", platform: "tiktok", scheduled_at: "2026-11-10T09:00:00Z", status: "scheduled" }],
      },
    ])
  ),
  http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/calendar`, () =>
    HttpResponse.json({
      items: [
        {
          target_id: "tgt_1",
          post_id: "post_1",
          scheduled_at: "2026-11-10T09:00:00Z",
          platform: "tiktok",
          status: "scheduled",
        },
      ],
    })
  ),
  http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/notifications`, () =>
    HttpResponse.json([
      { id: "n1", title: "Xong", read: false },
      { id: "n2", title: "Cũ", read: true },
    ])
  ),
  http.post(`${SOCIAL_CORE_URL}/projects/:pid/social/notifications/:nid/read`, () =>
    HttpResponse.json({ status: "ok", success: true })
  ),
  http.delete(`${SOCIAL_CORE_URL}/projects/:pid/social/notifications`, () =>
    HttpResponse.json({ status: "ok" })
  ),
  http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/queue`, () =>
    HttpResponse.json({
      summary: {
        accounts_count: 1,
        max_accounts: 5,
        draft_count: 1,
        pending_approval_count: 0,
        overdue_count: 0,
        upcoming_count: 2,
        published_count: 3,
        failed_count: 0,
      },
      items: [],
    })
  ),
  http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/overview`, () =>
    HttpResponse.json({ counts: { upcoming: 2 } })
  ),
  http.post(`${SOCIAL_CORE_URL}/projects/:pid/social/targets/bulk`, async ({ request }) => {
    const body = (await request.json()) as any;
    return HttpResponse.json({ results: (body.ids || []).map((id: string) => ({ id, success: true })) });
  }),
  http.post(`${SOCIAL_CORE_URL}/projects/:pid/social/analytics/apply-golden-windows`, () =>
    HttpResponse.json({ status: "ok", updated_accounts: ["acc_1"], windows: [] })
  ),
  http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/automation/rules`, () =>
    HttpResponse.json([{ id: "rule_1", name: "Auto retry", enabled: true }])
  ),
  http.post(`${SOCIAL_CORE_URL}/projects/:pid/social/automation/rules/:rid/toggle`, () =>
    HttpResponse.json({ id: "rule_1", enabled: false })
  ),
  http.put(`${SOCIAL_CORE_URL}/projects/:pid/social/posts/:postId/targets/:targetId/time`, async ({ params, request }) => {
    const body = (await request.json()) as { time: string };
    if (failTimeUpdate) {
      failTimeUpdate = false;
      return HttpResponse.json({ detail: "fail" }, { status: 500 });
    }
    updatedTimes.push({ postId: String(params.postId), targetId: String(params.targetId), time: body.time });
    return HttpResponse.json({ id: params.postId, targets: [] });
  })
);

beforeAll(() => server.listen());

beforeEach(() => {
  failTimeUpdate = false;
  updatedTimes.length = 0;
  useSocialStore.setState({
    accounts: [],
    posts: [],
    calendarData: null,
    queueData: null,
    queueBucket: "upcoming",
    loading: false,
    error: null,
    isHubOpen: false,
    hubView: "overview",
    hubSelectedEntity: null,
    hubPreselectedVideoIds: [],
    overviewData: null,
    libraryVideos: [],
    templates: [],
    accountsStats: [],
    analyticsOverview: null,
    heatmapData: null,
    creativesData: null,
    learningsData: null,
    automationRules: [],
    notifications: [],
    unreadNotifCount: 0,
    activeTab: "calendar",
    calendarView: "month",
    filters: { platform: "all", status: "all", accountId: "all", search: "" },
    isComposerOpen: false,
    editingPost: null,
    composerInitialDate: null,
    composerInitialAccount: null,
    isPublishCenterOpen: false,
    publishCenterTab: "plan",
    isAutomationModalOpen: false,
    isNotifDrawerOpen: false,
    isManualMetricsModalOpen: false,
    isImportCsvModalOpen: false,
    lastDragAction: null,
  });
});

afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe("useSocialStore characterization tests", () => {
  it("manages synchronous UI, composer, publish center, and Social Hub state", () => {
    const st = useSocialStore.getState();
    st.setActiveTab("approvals");
    st.setFilters({ platform: "tiktok", search: "răng" });
    st.setQueueBucket("draft");
    st.openComposer(null, "2026-11-12", "acc_1");
    expect(useSocialStore.getState().activeTab).toBe("approvals");
    expect(useSocialStore.getState().filters).toEqual({
      platform: "tiktok",
      status: "all",
      accountId: "all",
      search: "răng",
    });
    expect(useSocialStore.getState().isComposerOpen).toBe(true);
    expect(useSocialStore.getState().composerInitialDate).toBe("2026-11-12");

    st.closeComposer();
    expect(useSocialStore.getState().isComposerOpen).toBe(false);
    expect(useSocialStore.getState().composerInitialAccount).toBeNull();

    st.openPublishCenter("calendar");
    expect(useSocialStore.getState().isPublishCenterOpen).toBe(true);
    expect(useSocialStore.getState().publishCenterTab).toBe("calendar");
    st.closePublishCenter();
    expect(useSocialStore.getState().isPublishCenterOpen).toBe(false);

    st.openHub({ view: "queue", videoIds: ["v1"] });
    expect(useSocialStore.getState().isHubOpen).toBe(true);
    expect(useSocialStore.getState().hubView).toBe("queue");
    expect(useSocialStore.getState().hubPreselectedVideoIds).toEqual(["v1"]);
    st.closeHub();
    expect(useSocialStore.getState().isHubOpen).toBe(false);
  });

  it("loads aggregated data with unread count, bulkAction refresh, and error recording", async () => {
    await useSocialStore.getState().loadAll("soc-nho-tv");
    const afterLoad = useSocialStore.getState();
    expect(afterLoad.accounts).toHaveLength(1);
    expect(afterLoad.posts).toHaveLength(1);
    expect(afterLoad.unreadNotifCount).toBe(1);
    expect(afterLoad.loading).toBe(false);

    const bulkRes = await useSocialStore.getState().bulkAction("soc-nho-tv", ["tgt_1"], "approve");
    expect(bulkRes.results).toEqual([{ id: "tgt_1", success: true }]);
    expect(useSocialStore.getState().overviewData).toEqual({ counts: { upcoming: 2 } });

    await useSocialStore.getState().toggleAutomationRule("soc-nho-tv", "rule_1");
    expect(useSocialStore.getState().automationRules).toHaveLength(1);

    await useSocialStore.getState().clearNotifications("soc-nho-tv");
    expect(useSocialStore.getState().notifications).toEqual([]);
    expect(useSocialStore.getState().unreadNotifCount).toBe(0);

    server.use(
      http.get(`${SOCIAL_CORE_URL}/projects/:pid/social/accounts`, () =>
        HttpResponse.json({ detail: "unavailable" }, { status: 500 })
      )
    );
    await useSocialStore.getState().loadAccounts("soc-nho-tv");
    expect(useSocialStore.getState().error).toBeTruthy();
  });

  it("optimistically moves target time, supports undoLastDrag, and rolls back on failure", async () => {
    await useSocialStore.getState().loadAll("soc-nho-tv");

    await useSocialStore.getState().moveTargetTime("soc-nho-tv", "post_1", "tgt_1", "2026-11-11T12:00:00Z");
    expect(updatedTimes).toEqual([{ postId: "post_1", targetId: "tgt_1", time: "2026-11-11T12:00:00Z" }]);
    expect(useSocialStore.getState().calendarData?.items[0].scheduled_at).toBe("2026-11-11T12:00:00Z");
    expect(useSocialStore.getState().posts[0].targets[0].scheduled_at).toBe("2026-11-11T12:00:00Z");
    expect(useSocialStore.getState().lastDragAction).toEqual({
      postId: "post_1",
      targetId: "tgt_1",
      previousTime: "2026-11-10T09:00:00Z",
    });

    await useSocialStore.getState().undoLastDrag("soc-nho-tv");
    expect(updatedTimes[1]).toEqual({ postId: "post_1", targetId: "tgt_1", time: "2026-11-10T09:00:00Z" });
    expect(useSocialStore.getState().lastDragAction).toBeNull();

    // Error triggers rollback via undoLastDrag (transient failure on first PUT, succeeds on rollback PUT)
    failTimeUpdate = true;
    await expect(
      useSocialStore.getState().moveTargetTime("soc-nho-tv", "post_1", "tgt_1", "2026-11-15T08:00:00Z")
    ).rejects.toThrow();
    await new Promise((r) => setTimeout(r, 20));
    expect(useSocialStore.getState().lastDragAction).toBeNull();
  });
});

describe("socialHelpers pure functions", () => {
  it("computes unread notifications and updates target schedule immutably", () => {
    expect(
      countUnreadNotifications([
        { id: "1", read: false } as any,
        { id: "2", read: true } as any,
        { id: "3", read: false } as any,
      ])
    ).toBe(2);

    const cal = { items: [{ target_id: "t1", scheduled_at: "2026-11-10T09:00:00Z" }] } as any;
    expect(findTargetScheduledTime(cal, "t1")).toBe("2026-11-10T09:00:00Z");
    expect(findTargetScheduledTime(cal, "missing")).toBeNull();

    const nextCal = applyTargetTimeToCalendar(cal, "t1", "2026-11-11T10:00:00Z");
    expect(nextCal.items[0].scheduled_at).toBe("2026-11-11T10:00:00Z");
    expect(cal.items[0].scheduled_at).toBe("2026-11-10T09:00:00Z");

    const posts = [{ id: "p1", targets: [{ id: "t1", scheduled_at: "old" }] }] as any[];
    const nextPosts = applyTargetTimeToPosts(posts, "p1", "t1", "new");
    expect(nextPosts[0].targets[0].scheduled_at).toBe("new");
    expect(posts[0].targets[0].scheduled_at).toBe("old");
  });
});
