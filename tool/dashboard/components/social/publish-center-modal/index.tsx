"use client";

import React, { useState } from "react";
import { cn } from "@/lib/utils";
import { CalendarView } from "../CalendarView";
import { ChevronLeft, ChevronRight } from "../icons";
import { WIZARD_STEPS } from "./constants";
import { usePublishPlanSelection } from "./hooks/usePublishPlanSelection";
import { usePublishPlanWorkflow } from "./hooks/usePublishPlanWorkflow";
import { ModalErrorBoundary } from "./components/ModalErrorBoundary";
import { ModalHeader } from "./components/ModalHeader";
import { Step1VideoSelector } from "./components/Step1VideoSelector";
import { Step2AccountMatrix } from "./components/Step2AccountMatrix";
import { Step3CaptionPreview } from "./components/Step3CaptionPreview";
import { Step4SchedulePlanner } from "./components/Step4SchedulePlanner";
import { Step5ReviewCommit } from "./components/Step5ReviewCommit";
import type { PublishCenterModalProps, WizardMainTab } from "./types";

export type { PublishCenterModalProps } from "./types";

function PublishCenterModalInner({
  projectId,
  isOpen,
  initialTab = "plan",
  initialVideoIds,
  onClose,
}: PublishCenterModalProps) {
  const [activeMainTab, setActiveMainTab] = useState<WizardMainTab>(initialTab);
  const [currentStep, setCurrentStep] = useState<number>(1);

  const selection = usePublishPlanSelection({
    projectId,
    isOpen,
    initialVideoIds,
    onJumpToStep: setCurrentStep,
  });

  const workflow = usePublishPlanWorkflow({
    projectId,
    currentStep,
    selectedVideoIds: selection.selectedVideoIds,
    selectedVideos: selection.selectedVideos,
    selectedAccountIds: selection.selectedAccountIds,
    selectedAccounts: selection.selectedAccounts,
  });

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-3 sm:p-6 animate-in fade-in-50">
      <div className="relative w-full max-w-6xl h-[92vh] flex flex-col rounded-xl border border-border bg-card shadow-2xl overflow-clip text-foreground">
        <ModalHeader
          accountsCount={selection.accounts.length}
          activeMainTab={activeMainTab}
          onChangeTab={setActiveMainTab}
          onClose={onClose}
        />

        {/* Modal Main Body */}
        <div className="flex-1 overflow-hidden flex flex-col">
          {activeMainTab === "calendar" ? (
            <div className="flex-1 p-4 overflow-y-auto scroll-quiet">
              <CalendarView projectId={projectId} />
            </div>
          ) : (
            <div className="flex-1 flex flex-col overflow-hidden">
              {/* Stepper Header */}
              <div className="flex items-center justify-between border-b border-border bg-background px-6 py-3 text-xs">
                {WIZARD_STEPS.map((s) => {
                  const isCurrent = currentStep === s.step;
                  const isDone = currentStep > s.step;
                  return (
                    <button
                      key={s.step}
                      type="button"
                      onClick={() => setCurrentStep(s.step)}
                      className={cn(
                        "flex items-center gap-2 px-3 py-1 rounded-lg font-semibold transition-all",
                        isCurrent
                          ? "bg-primary/10 text-primary border border-primary/20 shadow-2xs"
                          : isDone
                          ? "text-semantic-success hover:bg-emerald-500/10"
                          : "text-muted-foreground hover:text-foreground"
                      )}
                    >
                      <span
                        className={cn(
                          "flex h-5 w-5 items-center justify-center rounded-full text-caption font-bold",
                          isCurrent
                            ? "bg-primary text-primary-foreground"
                            : isDone
                            ? "bg-emerald-500 text-white"
                            : "bg-muted text-muted-foreground"
                        )}
                      >
                        {isDone ? "✓" : s.step}
                      </span>
                      <span className="hidden md:inline">{s.title}</span>
                    </button>
                  );
                })}
              </div>

              {/* Step Content Container */}
              <div className="flex-1 overflow-y-auto p-4 scroll-quiet">
                {currentStep === 1 && (
                  <Step1VideoSelector
                    videoSearch={selection.videoSearch}
                    onChangeSearch={selection.setVideoSearch}
                    filterUnpostedOnly={selection.filterUnpostedOnly}
                    onToggleFilterUnpostedOnly={selection.setFilterUnpostedOnly}
                    loadingVideos={selection.loadingVideos}
                    filteredVideos={selection.filteredVideos}
                    selectedVideoIds={selection.selectedVideoIds}
                    onChangeSelectedVideoIds={selection.setSelectedVideoIds}
                  />
                )}

                {currentStep === 2 && (
                  <Step2AccountMatrix
                    accounts={selection.accounts}
                    selectedAccounts={selection.selectedAccounts}
                    selectedAccountIds={selection.selectedAccountIds}
                    onChangeSelectedAccountIds={selection.setSelectedAccountIds}
                    allGroups={selection.allGroups}
                    selectedVideos={selection.selectedVideos}
                    compatibilityMatrix={selection.compatibilityMatrix}
                    ignoreErrors={selection.ignoreErrors}
                    onToggleIgnoreErrors={selection.setIgnoreErrors}
                  />
                )}

                {currentStep === 3 && (
                  <Step3CaptionPreview
                    selectedVideos={selection.selectedVideos}
                    selectedAccounts={selection.selectedAccounts}
                    activeVideoIndex={workflow.activeVideoIndex}
                    onChangeActiveVideoIndex={workflow.setActiveVideoIndex}
                    videoCaptions={workflow.videoCaptions}
                    onChangeVideoCaptions={workflow.setVideoCaptions}
                    isGeneratingCaptions={workflow.isGeneratingCaptions}
                    onGenerateCaptions={workflow.handleGenerateCaptions}
                  />
                )}

                {currentStep === 4 && (
                  <Step4SchedulePlanner
                    scheduleStrategy={workflow.scheduleStrategy}
                    onChangeStrategy={workflow.setScheduleStrategy}
                    scheduleStartDate={workflow.scheduleStartDate}
                    onChangeStartDate={workflow.setScheduleStartDate}
                    onReloadPreview={workflow.handleLoadPreview}
                    isLoadingPreview={workflow.isLoadingPreview}
                    previewTargets={workflow.previewTargets}
                    onUpdateTargetSchedule={workflow.setPreviewTargets}
                  />
                )}

                {currentStep === 5 && (
                  <Step5ReviewCommit
                    commitResult={workflow.commitResult}
                    selectedVideosCount={selection.selectedVideos.length}
                    selectedAccountsCount={selection.selectedAccounts.length}
                    previewTargetsCount={workflow.previewTargets.length}
                    scheduleStrategy={workflow.scheduleStrategy}
                    approvalRequired={workflow.approvalRequired}
                    onToggleApprovalRequired={workflow.setApprovalRequired}
                    isSubmittingPlan={workflow.isSubmittingPlan}
                    onCommitPlan={workflow.handleCommitPlan}
                    onOpenCalendarTab={() => setActiveMainTab("calendar")}
                    onClose={onClose}
                  />
                )}
              </div>

              {/* Wizard Bottom Navigation Bar */}
              <div className="flex items-center justify-between border-t border-border bg-background px-6 py-3 shrink-0 text-xs">
                <button
                  type="button"
                  disabled={currentStep === 1}
                  onClick={() => setCurrentStep(currentStep - 1)}
                  className="flex items-center gap-1 rounded-lg border border-border bg-background px-3 py-1 font-semibold text-foreground hover:bg-accent transition-colors disabled:opacity-40"
                >
                  <ChevronLeft className="h-4 w-4" />
                  <span>Quay lại</span>
                </button>

                <div className="text-muted-foreground font-mono">
                  Bước {currentStep} / 5
                </div>

                {currentStep < 5 ? (
                  <button
                    type="button"
                    disabled={
                      selection.selectedVideoIds.length === 0 ||
                      (currentStep >= 2 &&
                        selection.selectedAccountIds.length === 0)
                    }
                    onClick={() => setCurrentStep(currentStep + 1)}
                    className="flex items-center gap-1 rounded-lg bg-primary px-4 py-1 font-semibold text-primary-foreground hover:bg-primary/90 transition-colors shadow-2xs disabled:opacity-40"
                  >
                    <span>Tiếp tục</span>
                    <ChevronRight className="h-4 w-4" />
                  </button>
                ) : (
                  <span />
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export function PublishCenterModal(props: PublishCenterModalProps) {
  return (
    <ModalErrorBoundary onClose={props.onClose}>
      <PublishCenterModalInner {...props} />
    </ModalErrorBoundary>
  );
}
