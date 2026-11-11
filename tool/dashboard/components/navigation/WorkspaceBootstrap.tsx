"use client";

import { useShallow } from "zustand/react/shallow";
import React, { useEffect, useRef } from "react";
import { useStudioStore, type StudioStore } from "@/store";
import { getRepository } from "@/lib/repositories";
import {
  checkCoreHealth,
  fetchNodeTypes,
  fetchActiveProjectFromCore,
  saveActiveProjectToCore,
} from "@/lib/core-api";

/** Nạp dữ liệu workspace từ Core và gắn phím tắt Ctrl+B; không hiển thị gì. */
export function WorkspaceBootstrap() {
  const {
    activeProjectId,
    setProjects,
    setCharacters,
    setChannels,
    setScheduledPosts,
    toggleRightPanel,
    setNodeCatalog,
    setCoreHealth,
  } = useStudioStore(
    useShallow((s: StudioStore) => ({
      activeProjectId: s.activeProjectId,
      setProjects: s.setProjects,
      setCharacters: s.setCharacters,
      setChannels: s.setChannels,
      setScheduledPosts: s.setScheduledPosts,
      toggleRightPanel: s.toggleRightPanel,
      setNodeCatalog: s.setNodeCatalog,
      setCoreHealth: s.setCoreHealth,
    }))
  );
  const syncedInitialRef = useRef(false);

  useEffect(() => {
    let active = true;
    async function bootstrapWorkspace() {
      const repo = getRepository();
      if (!syncedInitialRef.current) {
        syncedInitialRef.current = true;
        let hasPersistedProject = false;
        try {
          const raw = window.localStorage.getItem("ai_video_studio_settings_v2");
          if (raw) {
            const parsed = JSON.parse(raw);
            hasPersistedProject = Boolean(parsed?.state?.activeProjectId);
          }
        } catch {
          // ignore
        }
        if (!hasPersistedProject) {
          const coreActive = await fetchActiveProjectFromCore();
          if (active && coreActive && coreActive !== activeProjectId) {
            useStudioStore.getState().switchProject(coreActive);
            return;
          }
        } else {
          void saveActiveProjectToCore(activeProjectId);
        }
      }
      const [projs, chars, chans, posts, health, catalog] = await Promise.all([
        repo.listProjects(),
        repo.listCharacters(activeProjectId),
        repo.listChannels(activeProjectId),
        repo.listScheduledPosts(activeProjectId),
        checkCoreHealth(),
        fetchNodeTypes(),
      ]);
      if (!active) return;
      setProjects(projs);
      setCharacters(chars);
      setChannels(chans);
      setScheduledPosts(posts);
      setCoreHealth({ ...health, checkedAt: new Date().toISOString() });
      setNodeCatalog(catalog.definitions, catalog.fromCore);
      if (catalog.fromCore) {
        await useStudioStore.getState().refreshCustomNodeTypes(activeProjectId);
      }
      if (!useStudioStore.getState().loadedGraphs[activeProjectId]) {
        void useStudioStore.getState().loadProjectGraph(activeProjectId);
      }
    }
    bootstrapWorkspace();
    return () => {
      active = false;
    };
  }, [
    activeProjectId,
    setProjects,
    setCharacters,
    setChannels,
    setScheduledPosts,
    setCoreHealth,
    setNodeCatalog,
  ]);

  // Keyboard shortcut Ctrl+B to toggle right panel
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "b") {
        e.preventDefault();
        toggleRightPanel();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [toggleRightPanel]);

  return null;
}
