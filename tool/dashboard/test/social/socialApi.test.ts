import { describe, it, expect, beforeAll, afterEach, afterAll } from "vitest";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import { socialApi, SOCIAL_CORE_URL } from "@/lib/api/social";

interface RecordedRequest {
  method: string;
  path: string;
  query: string;
  body: unknown;
}

const requests: RecordedRequest[] = [];

const server = setupServer(
  http.all(`${SOCIAL_CORE_URL}/*`, async ({ request }) => {
    const url = new URL(request.url);
    let body: unknown = null;
    if (request.method !== "GET" && request.method !== "DELETE") {
      try {
        body = await request.json();
      } catch {
        body = null;
      }
    }
    requests.push({
      method: request.method,
      path: url.pathname,
      query: url.search,
      body,
    });

    if (url.pathname.endsWith("/social/accounts")) {
      return HttpResponse.json([
        {
          id: "acc_1",
          platform: "tiktok",
          handle: "@socnho",
          display_name: "Sóc Nhỏ TV",
          auth: { mode: "oauth" },
          status: "connected",
          default_hashtags: ["#socnho"],
          default_caption_footer: "",
        },
      ]);
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
    return HttpResponse.json({ ok: true, items: [] });
  })
);

beforeAll(() => server.listen());
afterEach(() => {
  requests.length = 0;
  server.resetHandlers();
});
afterAll(() => server.close());

describe("socialApi characterization tests", () => {
  it("fetches social accounts for project", async () => {
    const accounts = await socialApi.listAccounts("soc-nho-tv");
    expect(accounts).toHaveLength(1);
    expect(accounts[0].handle).toBe("@socnho");
    expect(accounts[0].platform).toBe("tiktok");
  });

  it("fetches queue items for project", async () => {
    const queue = await socialApi.getQueue("soc-nho-tv");
    expect(queue.summary.accounts_count).toBe(1);
    expect(queue.items).toHaveLength(0);
  });

  it("sends expected URLs, HTTP methods, queries, and bodies across all 8 social endpoint modules", async () => {
    await socialApi.createAccount("p1", { platform: "youtube", handle: "@bi" });
    await socialApi.deleteAccount("p1", "acc_2");

    await socialApi.getQueue("p1", "all", { platform: "tiktok" });
    await socialApi.previewPlan("p1", {
      video_ids: ["v1"],
      account_ids: ["acc_1"],
      start_at: "2026-11-10T10:00:00Z",
      spacing_hours: 4,
    } as any);
    await socialApi.commitPlan("p1", [{ video_id: "v1" } as any], false);

    await socialApi.createDraftPost("p1", { title: "Ep 1", platforms: ["tiktok"] });
    await socialApi.updateTargetTime("p1", "post_1", "tgt_1", "2026-11-10T19:00:00Z");
    await socialApi.approvePost("p1", "post_1", "ok");

    await socialApi.getCalendar("p1", { view: "week" });
    await socialApi.saveSlots("p1", []);

    await socialApi.publishNow("p1", "post_1");
    await socialApi.generatePackage("p1", "post_1", "tiktok");
    expect(socialApi.getPackageZipUrl("p1", "post_1", "tiktok")).toBe(
      `${SOCIAL_CORE_URL}/projects/p1/social/packages/post_1/tiktok.zip`
    );

    await socialApi.getAnalyticsOverview("p1", "7d", "tiktok", "acc_1");
    await socialApi.importCsvMetrics("p1", "views\n100", "tiktok");

    await socialApi.saveAutomationRule("p1", { name: "Golden hour" } as any);
    await socialApi.clearNotifications("p1");

    await socialApi.fetchLibrary("p1", { filter: "ready", q: "navie" });
    await socialApi.createTemplate("p1", { name: "Review Hook" } as any);

    expect(requests.map((r) => `${r.method} ${r.path}${r.query}`)).toEqual([
      "POST /projects/p1/social/accounts",
      "DELETE /projects/p1/social/accounts/acc_2",
      "GET /projects/p1/social/queue?platform=tiktok",
      "POST /projects/p1/social/plans/preview",
      "POST /projects/p1/social/plans/commit",
      "POST /projects/p1/social/posts/draft",
      "PUT /projects/p1/social/posts/post_1/targets/tgt_1/time",
      "POST /projects/p1/social/posts/post_1/approve",
      "GET /projects/p1/social/calendar?view=week",
      "PUT /projects/p1/social/slots",
      "POST /projects/p1/social/posts/post_1/publish-now",
      "POST /projects/p1/social/packages/post_1/tiktok/generate",
      "GET /projects/p1/social/analytics/overview?window=7d&platform=tiktok&account_id=acc_1",
      "POST /projects/p1/social/analytics/import-csv",
      "POST /projects/p1/social/automation/rules",
      "DELETE /projects/p1/social/notifications",
      "GET /projects/p1/social/library?filter=ready&q=navie",
      "POST /projects/p1/social/templates",
    ]);

    expect(requests[4].body).toEqual({
      targets: [{ video_id: "v1" }],
      approval_required: false,
    });
    expect(requests[6].body).toEqual({
      time: "2026-11-10T19:00:00Z",
      actor: "user",
    });
  });
});
