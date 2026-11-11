import "@testing-library/jest-dom/vitest";
import React from "react";
import { describe, it, expect, beforeAll, afterEach, afterAll, beforeEach, vi } from "vitest";
import { render, screen, waitFor, cleanup, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import viMessages from "@/messages/vi.json";
import { CORE_URL } from "@/lib/repositories/core";
import { useStudioStore } from "@/store";
import { useSocialStore } from "@/store/modules/social";
import { usePreview } from "@/lib/preview";
import { useConfirmStore } from "@/lib/confirm";
import { JobsCostTab } from "@/components/studio/tabs/JobsCostTab";
import {
  computeDailySpendPct,
  computeJobCounts,
  extractJobMediaUrl,
  filterJobsByTab,
  mergeAndSortJobs,
} from "@/components/studio/tabs/jobs-cost-tab/utils/mergeJobsList";

let runsData: any[] = [];
let rendersData: any[] = [];
let outputsData: any[] = [];
let cancelledRunCount = 0;
let cancelledRenderIds: string[] = [];
let deletedRunIds: string[] = [];

const server = setupServer(
  http.get(`${CORE_URL}/projects/:pid/runs`, () => HttpResponse.json(runsData)),
  http.get(`${CORE_URL}/projects/:pid/video/renders`, () => HttpResponse.json(rendersData)),
  http.get(`${CORE_URL}/projects/:pid/outputs`, () => HttpResponse.json(outputsData)),
  http.post(`${CORE_URL}/projects/:pid/run/cancel`, () => {
    cancelledRunCount += 1;
    runsData = runsData.map((r) =>
      r.runId === "run_live1234" ? { ...r, status: "cancelled" } : r
    );
    return HttpResponse.json({ ok: true });
  }),
  http.post(`${CORE_URL}/projects/:pid/video/render/:jobId/cancel`, ({ params }) => {
    cancelledRenderIds.push(String(params.jobId));
    return HttpResponse.json({ job_id: params.jobId, status: "failed" });
  }),
  http.delete(`${CORE_URL}/projects/:pid/runs/:runId`, ({ params }) => {
    deletedRunIds.push(String(params.runId));
    runsData = runsData.filter((r) => (r.runId || r.id) !== String(params.runId));
    return HttpResponse.json({ ok: true });
  })
);

function renderJobsTab() {
  return render(
    <NextIntlClientProvider locale="vi" messages={viMessages}>
      <JobsCostTab />
    </NextIntlClientProvider>
  );
}

beforeAll(() => server.listen());

beforeEach(() => {
  localStorage.clear();
  useConfirmStore.setState({ current: null });
  usePreview.getState().close();
  useSocialStore.getState().closeComposer();

  cancelledRunCount = 0;
  cancelledRenderIds = [];
  deletedRunIds = [];

  outputsData = [
    {
      id: "out_ep1",
      slug: "ep_01_rang_sau",
      title: "Tập 1: Chiếc Răng Sâu",
      duration_s: 45,
      size_mb: 12.4,
      engine: "hyperframes",
      files: { video: "output/ep_01/final_45s.mp4", thumb: "output/ep_01/thumb.jpg" },
      episodeVerify: { pass1to5: true, verdict: "Đạt chuẩn 1-5" },
    },
  ];

  runsData = [
    {
      runId: "run_live1234",
      mode: "full",
      status: "running",
      stage: "Đang sinh cảnh 2",
      progressPct: 45,
      actualCostUsd: 0.12,
      createdAt: "2026-11-09T10:15:00Z",
      nodes: [
        { id: "node_script_1", type: "script", state: "completed" },
        { id: "node_img_1", type: "image_gen", state: "running" },
      ],
    },
    {
      runId: "run_done5678",
      mode: "animatic",
      status: "completed",
      progressPct: 100,
      actualCostUsd: 0.05,
      createdAt: "2026-11-09T09:00:00Z",
      output: { files: { video: "output/ep_01/run_done.mp4" } },
    },
  ];

  rendersData = [
    {
      job_id: "rend_fail99",
      project_id: "soc-nho-tv",
      status: "failed",
      outputs: {},
      error: "Thiếu âm thanh lời thoại",
      created_at: "2026-11-09T08:00:00Z",
    },
  ];

  useStudioStore.setState({
    activeProjectId: "soc-nho-tv",
    totalSpentUsd: 3.75,
    perRunSpendingLimitUsd: 5,
    dailySpendingLimitUsd: 15,
    selectedNodeId: null,
    activeRightTab: "jobs",
    jobs: [
      {
        id: "job_store_1",
        type: "video_gen",
        model: "seedance-2.0",
        status: "completed",
        progress: 100,
        actualCostUsd: 0.2,
        outputUrl: "output/store_clip.mp4",
        createdAt: "2026-11-09T07:00:00Z",
      } as any,
    ],
  });
});

afterEach(() => {
  server.resetHandlers();
  cleanup();
  localStorage.clear();
  useConfirmStore.setState({ current: null });
});

afterAll(() => server.close());

describe("JobsCostTab characterization tests", () => {
  it("renders budget card, ready outputs with preview/publish actions, and combined job filters", async () => {
    const user = userEvent.setup();
    renderJobsTab();

    // Budget ceiling card
    expect(screen.getByText("Chi phí so với trần ngân sách")).toBeInTheDocument();
    expect(screen.getByText("Trần: $15.00 USD")).toBeInTheDocument();
    expect(screen.getByText("Đã dùng hôm nay: $3.75")).toBeInTheDocument();
    expect(screen.getByText("25%")).toBeInTheDocument();

    // Ready output section from MSW
    expect(await screen.findByText("Video Thành phẩm (1)")).toBeInTheDocument();
    expect(screen.getByText("Tập 1: Chiếc Răng Sâu")).toBeInTheDocument();
    expect(screen.getByText("✓ verify 1–5 ĐẠT")).toBeInTheDocument();
    expect(screen.getByText("HF")).toBeInTheDocument();

    // Play ready output in Studio preview
    await user.click(screen.getByTestId("output-play"));
    const preview = usePreview.getState().item;
    expect(preview?.kind).toBe("video");
    if (preview?.kind === "video") {
      expect(preview.items[0]?.url).toContain("/projects/soc-nho-tv/output/ep_01/final_45s.mp4");
    }

    // Filter chips combine Core runs (2) + Legacy render (1) + Store job (1) = 4
    expect(screen.getByRole("button", { name: "Tất cả (4)" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Chạy (1)" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Xong (2)" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Lỗi/Huỷ (1)" })).toBeInTheDocument();

    // Filter to failed shows error message and hides running stage
    await user.click(screen.getByRole("button", { name: "Lỗi/Huỷ (1)" }));
    expect(screen.getByText("Thiếu âm thanh lời thoại")).toBeInTheDocument();
    expect(screen.queryByText("#run_live12")).not.toBeInTheDocument();

    // Filter back to Running and focus node on canvas
    await user.click(screen.getByRole("button", { name: "Chạy (1)" }));
    await user.click(screen.getByRole("button", { name: "node_img_1" }));
    expect(useStudioStore.getState().selectedNodeId).toBe("node_img_1");
    expect(useStudioStore.getState().activeRightTab).toBe("node");

    // Click "Đăng" on ready output opens Social composer & switches tab to publish
    await user.click(screen.getByRole("button", { name: "Đăng" }));
    expect(useSocialStore.getState().isComposerOpen).toBe(true);
    expect(useSocialStore.getState().editingPost?.id).toBe("post_from_ep_01_rang_sau");
    expect(useStudioStore.getState().activeRightTab).toBe("publish");
  });

  it("cancels running run, deletes failed job with confirmation + localStorage dismiss, schedules done job, and clears finished", async () => {
    const user = userEvent.setup();
    renderJobsTab();

    expect(await screen.findByText("#run_live12")).toBeInTheDocument();

    // Cancel running core run
    await user.click(screen.getByRole("button", { name: "Huỷ" }));
    await waitFor(() => {
      expect(cancelledRunCount).toBe(1);
    });

    // Delete failed render job via confirmation dialog -> persists dismissed in localStorage
    const failedMsg = screen.getByText("Thiếu âm thanh lời thoại");
    const failedCard = failedMsg.closest(".rounded-card");
    expect(failedCard).not.toBeNull();
    const deleteButtons = screen.getAllByRole("button", { name: "Xoá job" });
    const targetDeleteBtn = deleteButtons.find((btn) => failedCard?.contains(btn));
    expect(targetDeleteBtn).toBeDefined();
    await user.click(targetDeleteBtn!);
    await waitFor(() => {
      expect(useConfirmStore.getState().current?.title).toBe("Xoá job?");
    });
    act(() => {
      useConfirmStore.getState().current?.resolve(true);
    });

    await waitFor(() => {
      expect(screen.queryByText("Thiếu âm thanh lời thoại")).not.toBeInTheDocument();
    });
    const storedDismissed = JSON.parse(
      localStorage.getItem("socnhotv:dismissedJobs:soc-nho-tv") || "[]"
    );
    expect(storedDismissed).toContain("rend_fail99");

    // Schedule done job via "Đăng / Lên lịch"
    const scheduleButtons = screen.getAllByRole("button", { name: "Đăng / Lên lịch" });
    await user.click(scheduleButtons[0]);
    expect(useSocialStore.getState().isComposerOpen).toBe(true);
    expect(useSocialStore.getState().editingPost?.id).toBe("post_from_run_done5678");
    expect(useStudioStore.getState().activeRightTab).toBe("publish");

    // Clear all finished jobs
    await user.click(screen.getByRole("button", { name: "Xoá job đã kết thúc" }));
    await waitFor(() => {
      expect(useConfirmStore.getState().current?.title).toContain("job đã kết thúc?");
    });
    act(() => {
      useConfirmStore.getState().current?.resolve(true);
    });
    await waitFor(() => {
      expect(deletedRunIds).toContain("run_done5678");
      expect(useStudioStore.getState().jobs).toHaveLength(0);
    });
  });

  it("validates pure job merging, filtering, media priority, and budget cap helpers", () => {
    const merged = mergeAndSortJobs(
      [{ runId: "r1", mode: "full", status: "canceled", progress: 0.5, createdAt: "2026-11-09T12:00:00Z" }],
      [
        { job_id: "r1", project_id: "p", status: "done", outputs: {}, created_at: "2026-11-09T11:00:00Z" },
        { job_id: "rend_2", project_id: "p", status: "rendering", outputs: { youtube: "yt.mp4" }, created_at: "2026-11-09T13:00:00Z" },
      ],
      [{ id: "s1", type: "script", status: "queued", progress: 0, createdAt: "2026-11-09T10:00:00Z" }],
      ["s1"]
    );
    // Dedupes r1 (keeps Core Run), dismisses s1, sorts newest first (rend_2 before r1)
    expect(merged.map((m) => m.id)).toEqual(["rend_2", "r1"]);
    expect(merged[1]?.status).toBe("cancelled");
    expect(computeJobCounts(merged)).toEqual({ all: 2, running: 0, completed: 0, failed: 1 });
    expect(filterJobsByTab(merged, "failed").map((m) => m.id)).toEqual(["r1"]);
    expect(extractJobMediaUrl({ youtube: "yt.mp4", tiktok: "tt.mp4" })).toBe("tt.mp4");
    expect(computeDailySpendPct(10, 0)).toBe(100);
  });
});
