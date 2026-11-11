"use client";

import { useState, type Dispatch, type SetStateAction } from "react";
import type { Platform, TemplateMeta } from "@/lib/api/video";
import type { StudioStore } from "@/store";
import { requestCraftLint, requestScriptAndHooks } from "../api";
import type { HookVariantItem, LintReport } from "../types";

export function useScriptAndLint(params: {
  activeProjectId: string;
  isMockMode: boolean;
  runPolicy: StudioStore["runPolicy"];
  selectedTemplate: TemplateMeta | null;
  topic: string;
  scriptText: string;
  setScriptText: (val: string) => void;
  slotValues: Record<string, any>;
  setSlotValues: Dispatch<SetStateAction<Record<string, any>>>;
  selectedCharacterId: string;
  platforms: Platform[];
  enableHookABC: boolean;
}) {
  const [isGeneratingScript, setIsGeneratingScript] = useState(false);
  const [hookVariants, setHookVariants] = useState<HookVariantItem[]>([]);
  const [selectedHookVariantId, setSelectedHookVariantId] = useState<string>("A");
  const [lintReport, setLintReport] = useState<LintReport | null>(null);
  const [isCheckingLint, setIsCheckingLint] = useState(false);

  const handleGenerateScript = async () => {
    const currentTopic = params.topic.trim();
    if (!currentTopic) return;
    setIsGeneratingScript(true);
    try {
      const res = await requestScriptAndHooks({
        activeProjectId: params.activeProjectId,
        prompt: currentTopic,
        selectedCharacterId: params.selectedCharacterId,
        enableHookABC: params.enableHookABC,
        isMockMode: params.isMockMode,
        runPolicy: params.runPolicy,
      });
      if (res) {
        if (res.hookData) {
          setHookVariants(res.hookData.variants || []);
          setSelectedHookVariantId(res.hookData.best_variant_id || "A");
        }
        params.setScriptText(res.markdown);
        const updatedSlots = { ...params.slotValues };
        params.selectedTemplate?.slots.forEach((s) => {
          if (s.key.includes("title") || s.key.includes("question") || s.key === "item_a") {
            updatedSlots[s.key] = currentTopic;
          }
        });
        params.setSlotValues(updatedSlots);
      }
    } catch (err) {
      console.error("Generate script failed:", err);
    } finally {
      setIsGeneratingScript(false);
    }
  };

  const handleSelectHookVariant = (variant: HookVariantItem) => {
    setSelectedHookVariantId(variant.variant_id);
    if (!params.scriptText) {
      params.setScriptText(variant.text);
      return;
    }
    const lines = params.scriptText.split("\n");
    lines[0] = variant.text;
    params.setScriptText(lines.join("\n"));
  };

  const handleRunLintCheck = async () => {
    setIsCheckingLint(true);
    try {
      const report = await requestCraftLint({
        script: params.scriptText || params.topic || "Video mới",
        platform: params.platforms[0] || "tiktok",
      });
      if (report) setLintReport(report);
    } catch (e) {
      console.error("Lint check failed:", e);
    } finally {
      setIsCheckingLint(false);
    }
  };

  return {
    isGeneratingScript,
    hookVariants,
    selectedHookVariantId,
    lintReport,
    isCheckingLint,
    handleGenerateScript,
    handleSelectHookVariant,
    handleRunLintCheck,
  };
}
