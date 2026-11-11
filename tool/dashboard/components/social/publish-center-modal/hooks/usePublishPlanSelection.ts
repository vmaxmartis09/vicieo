import { useEffect, useMemo, useState } from "react";
import { socialApi, type PublishableVideo } from "@/lib/api/social";
import { useSocialStore } from "@/store/modules/social";
import { computeCompatibilityMatrix } from "../utils/compatibilityMatrix";

export function usePublishPlanSelection(params: {
  projectId: string;
  isOpen: boolean;
  initialVideoIds?: string[];
  onJumpToStep: (step: number) => void;
}) {
  const { projectId, isOpen, initialVideoIds, onJumpToStep } = params;
  const accounts = useSocialStore((s) => s.accounts);
  const loadAccounts = useSocialStore((s) => s.loadAccounts);

  const [videos, setVideos] = useState<PublishableVideo[]>([]);
  const [loadingVideos, setLoadingVideos] = useState<boolean>(false);
  const [videoSearch, setVideoSearch] = useState("");
  const [filterUnpostedOnly, setFilterUnpostedOnly] = useState(false);
  const [selectedVideoIds, setSelectedVideoIds] = useState<string[]>([]);
  const [selectedAccountIds, setSelectedAccountIds] = useState<string[]>([]);
  const [ignoreErrors, setIgnoreErrors] = useState<boolean>(false);

  useEffect(() => {
    if (isOpen && initialVideoIds && initialVideoIds.length > 0) {
      setSelectedVideoIds(initialVideoIds);
      onJumpToStep(2);
    }
  }, [isOpen, initialVideoIds, onJumpToStep]);

  useEffect(() => {
    if (isOpen && projectId) {
      setLoadingVideos(true);
      socialApi
        .getVideos(projectId)
        .then((vids) => {
          setVideos(vids);
          if (vids.length > 0 && selectedVideoIds.length === 0) {
            setSelectedVideoIds([vids[0].id]);
          }
        })
        .catch(() => {})
        .finally(() => setLoadingVideos(false));

      if (accounts.length === 0) {
        loadAccounts(projectId);
      }
    }
  }, [isOpen, projectId, loadAccounts]);

  useEffect(() => {
    if (accounts.length > 0 && selectedAccountIds.length === 0) {
      setSelectedAccountIds(accounts.map((a) => a.id));
    }
  }, [accounts, selectedAccountIds.length]);

  const allGroups = useMemo(() => {
    const set = new Set<string>();
    accounts.forEach((a) => {
      (a.groups || []).forEach((g) => set.add(g));
    });
    return Array.from(set);
  }, [accounts]);

  const filteredVideos = useMemo(() => {
    return videos.filter((v) => {
      const matchSearch =
        !videoSearch || v.title.toLowerCase().includes(videoSearch.toLowerCase());
      const matchUnposted = !filterUnpostedOnly || v.posted_count === 0;
      return matchSearch && matchUnposted;
    });
  }, [videos, videoSearch, filterUnpostedOnly]);

  const selectedVideos = useMemo(
    () => videos.filter((v) => selectedVideoIds.includes(v.id)),
    [videos, selectedVideoIds]
  );

  const selectedAccounts = useMemo(
    () => accounts.filter((a) => selectedAccountIds.includes(a.id)),
    [accounts, selectedAccountIds]
  );

  const compatibilityMatrix = useMemo(
    () => computeCompatibilityMatrix(selectedVideos, selectedAccounts),
    [selectedVideos, selectedAccounts]
  );

  return {
    accounts,
    loadingVideos,
    videoSearch,
    setVideoSearch,
    filterUnpostedOnly,
    setFilterUnpostedOnly,
    selectedVideoIds,
    setSelectedVideoIds,
    selectedAccountIds,
    setSelectedAccountIds,
    ignoreErrors,
    setIgnoreErrors,
    allGroups,
    filteredVideos,
    selectedVideos,
    selectedAccounts,
    compatibilityMatrix,
  };
}
