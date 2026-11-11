"use client";

import { useState, useEffect, useCallback } from "react";
import { useShallow } from "zustand/react/shallow";
import { useStudioStore, type StudioStore } from "@/store";
import type { Platform, TemplateMeta } from "@/lib/api/video";
import type { SocialPlatformId } from "@/lib/socialConfig";
import { INITIAL_PRODUCT_INFO } from "../constants";
import { type ProductFormInfo } from "../types";
import { useCreateVideoData } from "./useCreateVideoData";
import { useVideoCreationWorkflow } from "./useVideoCreationWorkflow";

export function useVideoCreationForm(isOpen: boolean, onClose: () => void) {
  const {
    activeProjectId,
    nodeCatalog,
    isMockMode,
    runPolicy,
    createEpisode,
    addJob,
    updateJob,
  } = useStudioStore(
    useShallow((s: StudioStore) => ({
      activeProjectId: s.activeProjectId,
      nodeCatalog: s.nodeCatalog,
      isMockMode: s.isMockMode,
      runPolicy: s.runPolicy,
      createEpisode: s.createEpisode,
      addJob: s.addJob,
      updateJob: s.updateJob,
    }))
  );

  const [step, setStep] = useState<1 | 2 | 3>(1);
  const [topic, setTopic] = useState("");
  const [scriptText, setScriptText] = useState("");
  const [slotValues, setSlotValues] = useState<Record<string, any>>({});
  const [productInfo, setProductInfo] = useState<ProductFormInfo>(INITIAL_PRODUCT_INFO);
  const [selectedCharacterId, setSelectedCharacterId] = useState("soc_nho");
  const [selectedVoiceId, setSelectedVoiceId] = useState("soc-nho");
  const [isBatchMode, setIsBatchMode] = useState(false);
  const [batchTopicsText, setBatchTopicsText] = useState("");
  const [platforms, setPlatforms] = useState<Platform[]>(["tiktok"]);
  const [connectModalPlatform, setConnectModalPlatform] = useState<SocialPlatformId | null>(null);
  const [qualityLevel, setQualityLevel] = useState<"standard" | "high">("standard");
  const [enableHookABC, setEnableHookABC] = useState(true);
  const [enable3LayerDepth, setEnable3LayerDepth] = useState(true);
  const [enableAutoTransitions, setEnableAutoTransitions] = useState(true);
  const [showAdvancedCraft, setShowAdvancedCraft] = useState(false);

  const handleApplyTemplateDefaults = useCallback((tpl: TemplateMeta) => {
    setPlatforms([tpl.default_platform || "tiktok"]);
    setSelectedVoiceId(tpl.voice?.default_voice_id || "soc-nho");
    const initialSlots: Record<string, any> = {};
    tpl.slots.forEach((s) => {
      if (s.default !== undefined && s.default !== null) {
        initialSlots[s.key] = s.default;
      }
    });
    setSlotValues(initialSlots);
  }, []);

  const data = useCreateVideoData(isOpen, handleApplyTemplateDefaults);

  const workflow = useVideoCreationWorkflow({
    activeProjectId,
    nodeCatalog,
    isMockMode,
    runPolicy,
    createEpisode,
    addJob,
    updateJob,
    selectedTemplate: data.selectedTemplate,
    topic,
    scriptText,
    setScriptText,
    slotValues,
    setSlotValues,
    productInfo,
    selectedCharacterId,
    selectedVoiceId,
    isBatchMode,
    batchTopicsText,
    platforms,
    enableHookABC,
    onClose,
  });

  useEffect(() => {
    if (step === 3 && data.selectedTemplate) {
      workflow.estimateCost();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [step, data.selectedTemplate, isBatchMode, batchTopicsText]);

  return {
    activeProjectId,
    isMockMode,
    step,
    setStep,
    ...data,
    topic,
    setTopic,
    scriptText,
    setScriptText,
    slotValues,
    setSlotValues,
    productInfo,
    setProductInfo,
    selectedCharacterId,
    setSelectedCharacterId,
    selectedVoiceId,
    setSelectedVoiceId,
    isBatchMode,
    setIsBatchMode,
    batchTopicsText,
    setBatchTopicsText,
    platforms,
    setPlatforms,
    connectModalPlatform,
    setConnectModalPlatform,
    qualityLevel,
    setQualityLevel,
    enableHookABC,
    setEnableHookABC,
    enable3LayerDepth,
    setEnable3LayerDepth,
    enableAutoTransitions,
    setEnableAutoTransitions,
    showAdvancedCraft,
    setShowAdvancedCraft,
    ...workflow,
  };
}
