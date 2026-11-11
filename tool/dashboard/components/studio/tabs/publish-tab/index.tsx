"use client";

import React, { useMemo, useState } from "react";
import dynamic from "next/dynamic";
import { Plus, Sparkles } from "@/components/ui/icons";
import { ManualPackageModal } from "@/components/social/ManualPackageModal";
import { ManualMetricsModal } from "@/components/social/ManualMetricsModal";
import type { PackagePostSelection } from "./types";
import {
  computeSummaryCounts,
  filterUpcomingQueueItems,
  mapPublishedTargets,
} from "./utils/publishQueueMappers";
import { usePublishTabData } from "./hooks/usePublishTabData";
import { PublishOverviewCard } from "./components/PublishOverviewCard";
import { UpcomingPostsSection } from "./components/UpcomingPostsSection";
import { LearningsAnalyticsCard } from "./components/LearningsAnalyticsCard";
import { ConnectedChannelsSection } from "./components/ConnectedChannelsSection";

const PublishCenterModal = dynamic(
  () => import("@/components/social/PublishCenterModal").then((m) => m.PublishCenterModal),
  { ssr: false }
);

export function PublishTab() {
  const {
    activeProjectId,
    accounts,
    queueData,
    connectedMap,
    learningsData,
    loadingLearnings,
    isPublishCenterOpen,
    openPublishCenter,
    closePublishCenter,
    openConnectDrawer,
    fetchLearnings,
    handleOpenHubView,
    handleMarkPublished,
    handleRefreshAll,
  } = usePublishTabData();

  const [selectedPackagePost, setSelectedPackagePost] = useState<PackagePostSelection | null>(
    null
  );
  const [metricsModalOpen, setMetricsModalOpen] = useState(false);

  const counts = useMemo(() => computeSummaryCounts(queueData), [queueData]);
  const upcomingPosts = useMemo(() => filterUpcomingQueueItems(queueData), [queueData]);
  const publishedTargets = useMemo(() => mapPublishedTargets(queueData), [queueData]);

  return (
    <div className="flex h-full flex-col p-4 space-y-4 text-footnote select-none overflow-y-auto scroll-quiet">
      <PublishOverviewCard
        accountsCount={accounts.length}
        loadingLearnings={loadingLearnings}
        counts={counts}
        onRefreshAll={handleRefreshAll}
        onOpenHubView={handleOpenHubView}
      />

      {/* Quick Actions */}
      <div className="grid grid-cols-2 gap-2">
        <button
          type="button"
          onClick={() => openPublishCenter("plan")}
          className="h-9 px-3 rounded-ctl border border-border bg-card hover:bg-muted text-foreground font-medium text-footnote flex items-center justify-center gap-2 transition-colors shadow-2xs"
        >
          <Plus className="h-3.5 w-3.5 text-primary" />
          <span>Lên kế hoạch đăng</span>
        </button>

        <button
          type="button"
          onClick={() => setMetricsModalOpen(true)}
          className="h-9 px-3 rounded-ctl border border-border bg-card hover:bg-muted text-foreground font-medium text-footnote flex items-center justify-center gap-2 transition-colors shadow-2xs"
        >
          <Sparkles className="h-3.5 w-3.5 text-amber-400" />
          <span>Nhập số liệu & Gợi ý</span>
        </button>
      </div>

      <UpcomingPostsSection
        upcomingPosts={upcomingPosts}
        accounts={accounts}
        onOpenHubView={handleOpenHubView}
        onOpenPublishCenter={openPublishCenter}
        onSelectPackagePost={setSelectedPackagePost}
      />

      <LearningsAnalyticsCard
        learningsData={learningsData}
        onRefreshLearnings={() => activeProjectId && fetchLearnings(activeProjectId)}
        onOpenMetricsModal={() => setMetricsModalOpen(true)}
      />

      <ConnectedChannelsSection
        accounts={accounts}
        connectedMap={connectedMap}
        onOpenConnectDrawer={openConnectDrawer}
        onOpenHubView={handleOpenHubView}
      />

      {selectedPackagePost && (
        <ManualPackageModal
          isOpen={Boolean(selectedPackagePost)}
          onClose={() => setSelectedPackagePost(null)}
          projectId={activeProjectId || "soc-nho-tv"}
          post={selectedPackagePost}
          onMarkPublished={handleMarkPublished}
        />
      )}

      {metricsModalOpen && (
        <ManualMetricsModal
          isOpen={metricsModalOpen}
          onClose={() => setMetricsModalOpen(false)}
          projectId={activeProjectId || "soc-nho-tv"}
          targets={publishedTargets}
          onSuccess={() => {
            if (activeProjectId) {
              fetchLearnings(activeProjectId);
            }
          }}
        />
      )}

      {isPublishCenterOpen && (
        <PublishCenterModal
          projectId={activeProjectId || "soc-nho-tv"}
          isOpen={isPublishCenterOpen}
          onClose={closePublishCenter}
        />
      )}
    </div>
  );
}
