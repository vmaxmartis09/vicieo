"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { videoApi, type RenderJob } from "@/lib/api/video";
import { fetchProjectRuns, fetchProjectOutputs } from "@/lib/core-api";

export function useJobsPolling(activeProjectId: string, hasActiveStoreJobs: boolean) {
  const [remoteJobs, setRemoteJobs] = useState<RenderJob[]>([]);
  const [coreRuns, setCoreRuns] = useState<any[]>([]);
  const [outputsList, setOutputsList] = useState<any[]>([]);

  const dismissKey = `socnhotv:dismissedJobs:${activeProjectId}`;
  const [dismissed, setDismissed] = useState<string[]>([]);

  const hasActiveRef = useRef(hasActiveStoreJobs);
  hasActiveRef.current = hasActiveStoreJobs;

  useEffect(() => {
    try {
      setDismissed(JSON.parse(localStorage.getItem(dismissKey) || "[]"));
    } catch {
      setDismissed([]);
    }
  }, [dismissKey]);

  const rememberDismissed = useCallback(
    (ids: string[]) =>
      setDismissed((prev) => {
        const next = Array.from(new Set([...prev, ...ids])).slice(-500);
        try {
          localStorage.setItem(dismissKey, JSON.stringify(next));
        } catch {
          /* bỏ qua */
        }
        return next;
      }),
    [dismissKey]
  );

  const fetchJobs = useCallback(async () => {
    try {
      const [list, runs, outs] = await Promise.all([
        videoApi.listRenderJobs(activeProjectId).catch(() => []),
        fetchProjectRuns(activeProjectId).catch(() => []),
        fetchProjectOutputs(activeProjectId).catch(() => []),
      ]);
      setRemoteJobs(list);
      setCoreRuns(runs);
      setOutputsList(outs);
    } catch {
      // Keep store jobs as fallback
    }
  }, [activeProjectId]);

  useEffect(() => {
    if (!activeProjectId) return;
    fetchJobs();

    let timeoutId: NodeJS.Timeout | null = null;
    let cancelled = false;
    let delay = 3000;

    const tick = async () => {
      if (cancelled) return;
      if (typeof document !== "undefined" && document.hidden) {
        timeoutId = setTimeout(tick, 5000);
        return;
      }
      await fetchJobs();
      if (cancelled) return;

      delay = hasActiveRef.current ? 3000 : Math.min(delay * 1.5, 10000);
      timeoutId = setTimeout(tick, delay);
    };

    timeoutId = setTimeout(tick, delay);

    const onVisibilityChange = () => {
      if (!document.hidden && !cancelled) {
        delay = 3000;
        fetchJobs();
      }
    };
    document.addEventListener("visibilitychange", onVisibilityChange);

    return () => {
      cancelled = true;
      if (timeoutId) clearTimeout(timeoutId);
      document.removeEventListener("visibilitychange", onVisibilityChange);
    };
  }, [activeProjectId, fetchJobs]);

  return {
    remoteJobs,
    coreRuns,
    outputsList,
    dismissed,
    rememberDismissed,
    fetchJobs,
  };
}
