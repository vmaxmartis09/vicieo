"use client";

import { useEffect, useState } from "react";
import {
  fetchProjectScriptEpisodes,
  fetchScriptData,
  type ScriptEpisode,
} from "@/lib/api/nodes";
import type { FlyoutTabId, ScriptDataState } from "../types";

export function useFlyoutScriptData(activeTab: FlyoutTabId, activeProjectId?: string | null) {
  const [episodes, setEpisodes] = useState<ScriptEpisode[] | null>(null);
  const [scriptData, setScriptData] = useState<ScriptDataState | null>(null);
  const [loadingScript, setLoadingScript] = useState(false);

  useEffect(() => {
    if (activeTab === "script" && episodes === null && activeProjectId) {
      setLoadingScript(true);
      fetchProjectScriptEpisodes(activeProjectId)
        .then((eps) => {
          setEpisodes(eps);
          if (eps[0]) {
            return fetchScriptData(activeProjectId, eps[0].id);
          }
          return null;
        })
        .then((sd) => {
          if (sd) setScriptData(sd);
        })
        .finally(() => setLoadingScript(false));
    }
  }, [activeTab, episodes, activeProjectId]);

  return { scriptData, loadingScript };
}
