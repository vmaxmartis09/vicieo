"use client";

import { useState, useCallback, type Dispatch, type SetStateAction } from "react";
import type { Platform, TemplateMeta } from "@/lib/api/video";
import type { StudioStore } from "@/store";
import { dispatchBatchVideoCreation, requestCostEstimate } from "../api";
import { parseBatchTopics, type ProductFormInfo } from "../types";
import { useScriptAndLint } from "./useScriptAndLint";

interface WorkflowParams {
  activeProjectId: string;
  nodeCatalog: StudioStore["nodeCatalog"];
  isMockMode: boolean;
  runPolicy: StudioStore["runPolicy"];
  createEpisode: StudioStore["createEpisode"];
  addJob: StudioStore["addJob"];
  updateJob: StudioStore["updateJob"];
  selectedTemplate: TemplateMeta | null;
  topic: string;
  scriptText: string;
  setScriptText: (val: string) => void;
  slotValues: Record<string, any>;
  setSlotValues: Dispatch<SetStateAction<Record<string, any>>>;
  productInfo: ProductFormInfo;
  selectedCharacterId: string;
  selectedVoiceId: string;
  isBatchMode: boolean;
  batchTopicsText: string;
  platforms: Platform[];
  enableHookABC: boolean;
  onClose: () => void;
}

export function useVideoCreationWorkflow(params: WorkflowParams) {
  const [isEstimating, setIsEstimating] = useState(false);
  const [estimatedCost, setEstimatedCost] = useState<number>(0.0);
  const [isCreating, setIsCreating] = useState(false);
  const [createProgressText, setCreateProgressText] = useState("");

  const scriptAndLint = useScriptAndLint(params);

  const estimateCost = useCallback(async () => {
    if (!params.selectedTemplate) return;
    setIsEstimating(true);
    try {
      const topicCount = params.isBatchMode ? Math.max(1, parseBatchTopics(params.batchTopicsText).length) : 1;
      const total = await requestCostEstimate({
        activeProjectId: params.activeProjectId,
        selectedTemplate: params.selectedTemplate,
        topic: params.topic,
        scriptText: params.scriptText,
        selectedCharacterId: params.selectedCharacterId,
        selectedVoiceId: params.selectedVoiceId,
        platforms: params.platforms,
        slotValues: params.slotValues,
        productInfo: params.productInfo,
        isMockMode: params.isMockMode,
        runPolicy: params.runPolicy,
        topicCount,
      });
      setEstimatedCost(total);
    } catch {
      setEstimatedCost(0.0);
    } finally {
      setIsEstimating(false);
    }
  }, [params]);

  const handleCreateVideos = async () => {
    if (!params.selectedTemplate) return;
    setIsCreating(true);
    const topicsList = params.isBatchMode
      ? parseBatchTopics(params.batchTopicsText)
      : [params.topic.trim() || params.selectedTemplate.name];
    if (topicsList.length === 0) topicsList.push(params.selectedTemplate.name);

    try {
      await dispatchBatchVideoCreation({
        topicsList,
        activeProjectId: params.activeProjectId,
        selectedTemplate: params.selectedTemplate,
        isBatchMode: params.isBatchMode,
        scriptText: params.scriptText,
        selectedCharacterId: params.selectedCharacterId,
        selectedVoiceId: params.selectedVoiceId,
        platforms: params.platforms,
        slotValues: params.slotValues,
        productInfo: params.productInfo,
        nodeCatalog: params.nodeCatalog,
        isMockMode: params.isMockMode,
        runPolicy: params.runPolicy,
        createEpisode: params.createEpisode,
        addJob: params.addJob,
        updateJob: params.updateJob,
        onProgress: setCreateProgressText,
      });
      params.onClose();
    } catch (err) {
      console.error("Create video batch failed:", err);
    } finally {
      setIsCreating(false);
      setCreateProgressText("");
    }
  };

  return {
    ...scriptAndLint,
    isEstimating,
    estimatedCost,
    isCreating,
    createProgressText,
    estimateCost,
    handleCreateVideos,
  };
}
