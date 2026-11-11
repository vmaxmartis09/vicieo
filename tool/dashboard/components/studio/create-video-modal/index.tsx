"use client";

import React from "react";
import { motion } from "framer-motion";
import { ChevronRight, Loader2, Sparkles, X } from "@/components/ui/icons";
import type { Platform } from "@/lib/api/video";
import { SocialConnectModal } from "@/components/canvas/SocialConnectModal";
import { cn } from "@/lib/utils";
import { ModalErrorBoundary } from "./components/ModalErrorBoundary";
import { useVideoCreationForm } from "./hooks/useVideoCreationForm";
import { Step1TemplatePicker } from "./steps/Step1TemplatePicker";
import { Step2ContentCraft } from "./steps/Step2ContentCraft";
import { Step3PublishEstimate } from "./steps/Step3PublishEstimate";
import { parseBatchTopics, type CreateVideoModalProps } from "./types";

function CreateVideoModalContent({ isOpen, onClose }: CreateVideoModalProps) {
  const form = useVideoCreationForm(isOpen, onClose);

  if (!isOpen) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      className="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-4 bg-background/80 backdrop-blur-sm"
    >
      <motion.div
        initial={{ opacity: 0, scale: 0.96, y: 8 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.96 }}
        className="relative flex flex-col w-full max-w-4xl max-h-[90vh] rounded-sheet border border-border bg-card shadow-2xl overflow-clip"
      >
        <div className="flex items-center justify-between border-b border-border px-5 py-3 shrink-0 bg-accent/30">
          <div className="flex items-center gap-2">
            <div className="flex h-8 w-8 items-center justify-center rounded-ctl bg-primary/10 text-primary">
              <Sparkles className="h-4 w-4" />
            </div>
            <div>
              <h2 className="text-callout font-semibold text-foreground flex items-center gap-2">
                Tạo video tự động trong Studio
                <span className="text-caption font-normal text-muted-foreground">
                  (Bước {form.step}/3)
                </span>
              </h2>
              <p className="text-footnote text-muted-foreground">
                {form.step === 1 && "Chọn mẫu kịch bản & visual định sẵn"}
                {form.step === 2 && "Tự sinh lời thoại hoặc tuỳ biến nội dung"}
                {form.step === 3 && "Xem trước chuẩn nền tảng & tạo đồ thị"}
              </p>
            </div>
          </div>

          <div className="hidden sm:flex items-center gap-2 text-footnote">
            <button
              onClick={() => form.setStep(1)}
              className={cn(
                "px-3 py-1 rounded-ctl transition-colors",
                form.step === 1 ? "bg-primary text-primary-foreground font-medium" : "text-muted-foreground hover:bg-accent"
              )}
            >
              1. Mẫu
            </button>
            <ChevronRight className="h-3.5 w-3.5 text-muted-foreground" />
            <button
              onClick={() => form.selectedTemplate && form.setStep(2)}
              disabled={!form.selectedTemplate}
              className={cn(
                "px-3 py-1 rounded-ctl transition-colors",
                form.step === 2 ? "bg-primary text-primary-foreground font-medium" : "text-muted-foreground hover:bg-accent disabled:opacity-40"
              )}
            >
              2. Nội dung
            </button>
            <ChevronRight className="h-3.5 w-3.5 text-muted-foreground" />
            <button
              onClick={() => form.selectedTemplate && form.setStep(3)}
              disabled={!form.selectedTemplate}
              className={cn(
                "px-3 py-1 rounded-ctl transition-colors",
                form.step === 3 ? "bg-primary text-primary-foreground font-medium" : "text-muted-foreground hover:bg-accent disabled:opacity-40"
              )}
            >
              3. Xuất
            </button>
          </div>

          <button
            onClick={onClose}
            className="flex h-8 w-8 items-center justify-center rounded-ctl text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-5 scroll-quiet">
          {form.step === 1 && (
            <Step1TemplatePicker
              selectedCategory={form.selectedCategory}
              onSelectCategory={form.setSelectedCategory}
              filteredTemplates={form.filteredTemplates}
              selectedTemplate={form.selectedTemplate}
              onSelectTemplate={form.setSelectedTemplate}
            />
          )}

          {form.step === 2 && form.selectedTemplate && (
            <Step2ContentCraft
              selectedTemplate={form.selectedTemplate}
              activeProjectId={form.activeProjectId}
              isBatchMode={form.isBatchMode}
              setIsBatchMode={form.setIsBatchMode}
              batchTopicsText={form.batchTopicsText}
              setBatchTopicsText={form.setBatchTopicsText}
              topic={form.topic}
              setTopic={form.setTopic}
              isGeneratingScript={form.isGeneratingScript}
              onGenerateScript={form.handleGenerateScript}
              scriptText={form.scriptText}
              setScriptText={form.setScriptText}
              slotValues={form.slotValues}
              setSlotValues={form.setSlotValues}
              productInfo={form.productInfo}
              setProductInfo={form.setProductInfo}
              selectedCharacterId={form.selectedCharacterId}
              setSelectedCharacterId={form.setSelectedCharacterId}
              selectedVoiceId={form.selectedVoiceId}
              setSelectedVoiceId={form.setSelectedVoiceId}
              enableHookABC={form.enableHookABC}
              setEnableHookABC={form.setEnableHookABC}
              hookVariants={form.hookVariants}
              selectedHookVariantId={form.selectedHookVariantId}
              onSelectHookVariant={form.handleSelectHookVariant}
              showAdvancedCraft={form.showAdvancedCraft}
              setShowAdvancedCraft={form.setShowAdvancedCraft}
              enable3LayerDepth={form.enable3LayerDepth}
              setEnable3LayerDepth={form.setEnable3LayerDepth}
              enableAutoTransitions={form.enableAutoTransitions}
              setEnableAutoTransitions={form.setEnableAutoTransitions}
            />
          )}

          {form.step === 3 && form.selectedTemplate && (
            <Step3PublishEstimate
              selectedTemplate={form.selectedTemplate}
              platforms={form.platforms}
              setPlatforms={form.setPlatforms}
              connectedMap={form.connectedMap}
              onOpenConnectModal={form.setConnectModalPlatform}
              qualityLevel={form.qualityLevel}
              setQualityLevel={form.setQualityLevel}
              isCheckingLint={form.isCheckingLint}
              onRunLintCheck={form.handleRunLintCheck}
              lintReport={form.lintReport}
              isMockMode={form.isMockMode}
              isEstimating={form.isEstimating}
              estimatedCost={form.estimatedCost}
            />
          )}
        </div>

        <div className="flex items-center justify-between border-t border-border px-5 py-3 shrink-0 bg-accent/20">
          <div>
            {form.step > 1 && (
              <button
                type="button"
                onClick={() => form.setStep((s) => (s - 1) as 1 | 2)}
                disabled={form.isCreating}
                className="h-9 rounded-ctl border border-border px-3 text-footnote font-semibold text-foreground hover:bg-accent disabled:opacity-40 transition-colors"
              >
                Quay lại
              </button>
            )}
          </div>

          <div className="flex items-center gap-2">
            {form.step < 3 ? (
              <button
                type="button"
                onClick={() => form.setStep((s) => (s + 1) as 2 | 3)}
                disabled={!form.selectedTemplate}
                className="flex h-9 items-center gap-2 rounded-ctl bg-primary px-4 text-footnote font-semibold text-primary-foreground hover:bg-primary/90 disabled:opacity-40 transition-colors shadow-2xs"
              >
                <span>Tiếp tục</span>
                <ChevronRight className="h-3.5 w-3.5" />
              </button>
            ) : (
              <button
                type="button"
                onClick={form.handleCreateVideos}
                disabled={form.isCreating}
                className="flex h-9 items-center gap-2 rounded-ctl bg-primary px-5 text-footnote font-semibold text-primary-foreground hover:bg-primary/90 disabled:opacity-40 transition-colors shadow"
              >
                {form.isCreating ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin" />
                    <span>{form.createProgressText || "Đang khởi tạo đồ thị..."}</span>
                  </>
                ) : (
                  <>
                    <Sparkles className="h-4 w-4" />
                    <span>
                      {form.isBatchMode
                        ? `Tạo ${Math.max(1, parseBatchTopics(form.batchTopicsText).length)} video hàng loạt`
                        : "Tạo video ngay"}
                    </span>
                  </>
                )}
              </button>
            )}
          </div>
        </div>
      </motion.div>

      <SocialConnectModal
        platform={form.connectModalPlatform}
        isOpen={Boolean(form.connectModalPlatform)}
        onClose={() => form.setConnectModalPlatform(null)}
        onConnected={(connectedPlat) => {
          form.refreshConnections();
          if (!form.platforms.includes(connectedPlat as Platform)) {
            form.setPlatforms([...form.platforms, connectedPlat as Platform]);
          }
        }}
      />
    </div>
  );
}

export function CreateVideoModal(props: CreateVideoModalProps) {
  return (
    <ModalErrorBoundary onClose={props.onClose}>
      <CreateVideoModalContent {...props} />
    </ModalErrorBoundary>
  );
}
