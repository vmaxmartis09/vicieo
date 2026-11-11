"use client";

import React, { useState, useEffect, useCallback } from "react";
import { useShallow } from "zustand/react/shallow";
import { useStudioStore } from "@/store";
import { useSocialStore, type HubView } from "@/store/modules/social";
import { socialApi } from "@/lib/api/social";
import { usePreview } from "@/lib/preview";
import { getSavedSocialAccounts, type SocialPlatformId } from "@/lib/socialConfig";
import { SocialConnectDrawer } from "@/components/social/SocialConnectDrawer";
import type { LearningsState, PublishTabBucket } from "../types";
import { mapConnectedPlatformFlags } from "../utils/publishQueueMappers";

export function usePublishTabData() {
  const activeProjectId = useStudioStore((s) => s.activeProjectId);
  const {
    accounts,
    queueData,
    loadAccounts,
    loadQueue,
    openHub,
    setQueueBucket,
    isPublishCenterOpen,
    openPublishCenter,
    closePublishCenter,
  } = useSocialStore(
    useShallow((s) => ({
      accounts: s.accounts,
      queueData: s.queueData,
      loadAccounts: s.loadAccounts,
      loadQueue: s.loadQueue,
      openHub: s.openHub,
      setQueueBucket: s.setQueueBucket,
      isPublishCenterOpen: s.isPublishCenterOpen,
      openPublishCenter: s.openPublishCenter,
      closePublishCenter: s.closePublishCenter,
    }))
  );

  const [connectedMap, setConnectedMap] = useState<Record<string, boolean>>({});
  const [learningsData, setLearningsData] = useState<LearningsState | null>(null);
  const [loadingLearnings, setLoadingLearnings] = useState(false);

  const refreshConnectedMap = useCallback(() => {
    setConnectedMap(mapConnectedPlatformFlags(getSavedSocialAccounts()));
  }, []);

  const openConnectDrawer = useCallback(
    (pId: SocialPlatformId) => {
      usePreview.getState().open({
        kind: "custom",
        title: `Kết nối tài khoản ${pId.toUpperCase()}`,
        render: (close) => (
          <SocialConnectDrawer
            platform={pId}
            onClose={close}
            onConnected={() => {
              refreshConnectedMap();
              close();
            }}
          />
        ),
      });
    },
    [refreshConnectedMap]
  );

  useEffect(() => {
    refreshConnectedMap();
    const handleUpdate = () => refreshConnectedMap();
    window.addEventListener("social-accounts-updated", handleUpdate);
    return () => window.removeEventListener("social-accounts-updated", handleUpdate);
  }, [refreshConnectedMap]);

  const fetchLearnings = useCallback(async (pid: string) => {
    setLoadingLearnings(true);
    try {
      const res = await socialApi.getAnalyticsLearnings(pid);
      setLearningsData({
        recommendations: res.recommendations,
        learnings_md: res.learnings_md,
      });

      const ov = await socialApi.getAnalyticsOverview(pid, "30d");
      setLearningsData((prev) => ({
        ...prev,
        kpis: ov.kpis,
      }));
    } catch {
      // Ignore if no analytics yet
    } finally {
      setLoadingLearnings(false);
    }
  }, []);

  useEffect(() => {
    if (activeProjectId) {
      loadAccounts(activeProjectId);
      loadQueue(activeProjectId, "upcoming");
      fetchLearnings(activeProjectId);
    }
  }, [activeProjectId, loadAccounts, loadQueue, fetchLearnings]);

  const handleOpenHubView = useCallback(
    (view: HubView, bucket?: PublishTabBucket) => {
      if (bucket) {
        setQueueBucket(bucket);
      }
      openHub({ view });
    },
    [openHub, setQueueBucket]
  );

  const handleMarkPublished = useCallback(
    async (postId: string, targetId: string, url: string) => {
      if (!activeProjectId) return;
      await socialApi.markPublished(activeProjectId, postId, targetId, url);
      loadQueue(activeProjectId, "upcoming");
      fetchLearnings(activeProjectId);
    },
    [activeProjectId, loadQueue, fetchLearnings]
  );

  const handleRefreshAll = useCallback(() => {
    if (activeProjectId) {
      loadAccounts(activeProjectId);
      loadQueue(activeProjectId, "upcoming");
      fetchLearnings(activeProjectId);
    }
  }, [activeProjectId, loadAccounts, loadQueue, fetchLearnings]);

  return {
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
  };
}
