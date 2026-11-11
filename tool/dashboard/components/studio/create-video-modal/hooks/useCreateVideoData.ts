"use client";

import { useState, useEffect, useCallback, useMemo } from "react";
import { videoApi, type TemplateMeta, type Platform, type PlatformPreset } from "@/lib/api/video";
import { getSavedSocialAccounts } from "@/lib/socialConfig";

export function useCreateVideoData(
  isOpen: boolean,
  onApplyTemplateDefaults: (tpl: TemplateMeta) => void
) {
  const [templates, setTemplates] = useState<TemplateMeta[]>([]);
  const [selectedCategory, setSelectedCategory] = useState("all");
  const [selectedTemplate, setSelectedTemplate] = useState<TemplateMeta | null>(null);
  const [platformPresets, setPlatformPresets] = useState<PlatformPreset[]>([]);
  const [connectedMap, setConnectedMap] = useState<Record<string, boolean>>({});

  const refreshConnections = useCallback(() => {
    const accs = getSavedSocialAccounts();
    setConnectedMap({
      tiktok: Boolean(accs.tiktok?.connected),
      youtube: Boolean(accs.youtube?.connected),
      facebook: Boolean(accs.facebook?.connected),
      instagram: Boolean(accs.instagram?.connected),
      shorts: Boolean(accs.youtube?.connected),
      reels: Boolean(accs.instagram?.connected || accs.facebook?.connected),
    });
  }, []);

  useEffect(() => {
    refreshConnections();
    const handleUpdate = () => refreshConnections();
    window.addEventListener("social-accounts-updated", handleUpdate);
    return () => window.removeEventListener("social-accounts-updated", handleUpdate);
  }, [refreshConnections]);

  useEffect(() => {
    if (!isOpen) return;
    async function loadData() {
      try {
        const [tpls, plats] = await Promise.all([
          videoApi.fetchTemplates().catch(() => []),
          videoApi.fetchPlatforms().catch(() => []),
        ]);
        setTemplates(tpls);
        setPlatformPresets(plats);
        if (tpls.length > 0) {
          setSelectedTemplate((prev) => prev ?? tpls[0]);
        }
      } catch (err) {
        console.error("Failed to load templates:", err);
      }
    }
    loadData();
  }, [isOpen]);

  useEffect(() => {
    if (selectedTemplate) {
      onApplyTemplateDefaults(selectedTemplate);
    }
  }, [selectedTemplate, onApplyTemplateDefaults]);

  const filteredTemplates = useMemo(
    () =>
      templates.filter((t) => {
        if (selectedCategory === "all") return true;
        return t.category === selectedCategory;
      }),
    [templates, selectedCategory]
  );

  return {
    templates,
    filteredTemplates,
    selectedCategory,
    setSelectedCategory,
    selectedTemplate,
    setSelectedTemplate,
    platformPresets,
    connectedMap,
    refreshConnections,
  };
}
