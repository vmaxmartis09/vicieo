"use client";

import React, { useCallback, useMemo, useRef } from "react";
import { useLocale, useTranslations } from "next-intl";
import { useRouter } from "@/i18n/routing";
import { ChatPendingConfirmModal } from "@/components/studio/chat/ChatPendingConfirmModal";
import { buildSlashCommands } from "./constants";
import { buildQuickPrompts, extractCanvasImages } from "./utils/chatMediaUtils";
import { useChatStoreBindings } from "./hooks/useChatStoreBindings";
import { useChatComposerState } from "./hooks/useChatComposerState";
import { useChatCanvasActions } from "./hooks/useChatCanvasActions";
import { useChatContextActions } from "./hooks/useChatContextActions";
import { useChatSend } from "./hooks/useChatSend";
import { ChatErrorBoundary } from "./components/ChatErrorBoundary";
import { SelectedImageToolbar } from "./components/SelectedImageToolbar";
import { ChatCollapsedCapsule } from "./components/ChatCollapsedCapsule";
import { ChatExpandedDock } from "./components/ChatExpandedDock";

export { ChatErrorBoundary };

function ChatTabInner() {
  const t = useTranslations("chatTab");
  const locale = useLocale() as "vi" | "en";
  const vi = locale === "vi";
  const router = useRouter();
  const s = useChatStoreBindings();

  const dockRef = useRef<HTMLDivElement>(null);
  const endRef = useRef<HTMLDivElement>(null);
  const messages = s.chatByProject[s.activeProjectId] || [];
  const activeChar = s.characters.find((ch) => ch.id === s.chatContext.characterId) || null;
  const activeEpisode = s.episodes.find((e) => e.id === s.activeEpisodeId);

  const c = useChatComposerState(dockRef, endRef, messages.length);
  const canvasImages = useMemo(() => extractCanvasImages(s.nodes, s.characters), [s.nodes, s.characters]);
  const selectedCanvasImage = useMemo(
    () => (s.selectedNodeId ? canvasImages.find((item) => item.nodeId === s.selectedNodeId) || null : null),
    [s.selectedNodeId, canvasImages]
  );
  const slashCommands = useMemo(() => buildSlashCommands(vi), [vi]);
  const quickPrompts = useMemo(
    () =>
      buildQuickPrompts({
        nodesLength: s.nodesLength,
        activeCharName: activeChar?.name,
        vi,
        addGateLabel: t("quickPrompts.addGate"),
        ratio169Label: t("quickPrompts.ratio169"),
      }),
    [s.nodesLength, activeChar?.name, vi, t]
  );

  const say = useCallback(
    (text: string, targetNodeId?: string, snapshotBefore?: any, summaryVi?: string, summaryEn?: string) => {
      const actionCard = targetNodeId || snapshotBefore
        ? {
            id: `act_${Date.now().toString(36)}`,
            summaryVi: summaryVi || "Đã tạo node trực tiếp trên Canvas",
            summaryEn: summaryEn || "Created node on Studio Canvas",
            targetNodeId,
            snapshotBefore,
          }
        : undefined;
      s.addChatMessage(s.activeProjectId, {
        id: `msg_a_${Date.now().toString(36)}${Math.random().toString(36).slice(2, 4)}`,
        role: "assistant",
        text,
        timestamp: new Date().toISOString(),
        mode: "provider",
        actionCard,
      });
    },
    [s.activeProjectId, s.addChatMessage]
  );

  const canvasActions = useChatCanvasActions({
    ...s,
    activeEpisodeTitle: activeEpisode?.title,
    chatModel: s.chatContext.model,
    craftTags: c.craftTags,
    attachments: c.attachments,
    vi,
    say,
  });

  const routerPush = useCallback((path: string) => router.push(path), [router]);
  const runContextAction = useChatContextActions({
    ...s,
    vi,
    actionMode: c.actionMode,
    attachments: c.attachments,
    chatFocus: s.chatContext.focus,
    activeChar,
    projectNew: c.projectNew,
    routerPush,
    setProjectNew: c.setProjectNew,
    setAttachments: c.setAttachments,
    startRunConfirmation: canvasActions.startRunConfirmation,
    executeDirectMediaGeneration: canvasActions.executeDirectMediaGeneration,
  });

  const handleSendCommand = useChatSend({
    ...s,
    ...c,
    vi,
    chatModel: s.chatContext.model,
    chatCharacterId: s.chatContext.characterId,
    chatCharacterMode: s.chatContext.characterMode,
    runContextAction,
    executeStudioAction: canvasActions.executeStudioAction,
    say,
    routerPush,
  });

  // Khi flow đang chạy hoặc đang hiển thị tiến trình xử lý (process bar), ẩn hoàn toàn dock chat
  if (s.isRunningGraph || s.hasActiveRun) {
    return null;
  }

  return (
    <div className="pointer-events-none fixed inset-x-0 bottom-14 md:bottom-3 z-40 flex justify-center px-3">
      <div
        ref={dockRef}
        className={`pointer-events-auto flex w-full flex-col gap-2 text-footnote transition-all duration-200 ${c.expanded ? "max-w-[680px]" : "max-w-[540px]"}`}
      >
        {selectedCanvasImage && (
          <SelectedImageToolbar
            selectedCanvasImage={selectedCanvasImage}
            vi={vi}
            onAddAttachment={c.addAttachmentUnique}
            onSelectModeAndExpand={(mode) => {
              if (mode) c.setActionMode(mode);
              c.setExpanded(true);
              c.setShouldFocusInput(true);
            }}
          />
        )}

        {!c.expanded ? (
          <ChatCollapsedCapsule
            vi={vi}
            input={c.input}
            attachmentsCount={c.attachments.length}
            chatFocus={s.chatContext.focus}
            chatModel={s.chatContext.model}
            activeCharName={activeChar?.name}
            messagesCount={messages.length}
            showHistory={c.showHistory}
            onExpand={() => {
              c.setExpanded(true);
              c.setShouldFocusInput(true);
            }}
            onExpandHistory={() => {
              c.setExpanded(true);
              c.setShowHistory(true);
            }}
          />
        ) : (
          <ChatExpandedDock
            vi={vi}
            chatContext={s.chatContext}
            projects={s.projects}
            activeProjectId={s.activeProjectId}
            characters={s.characters}
            activeChar={activeChar}
            episodes={s.episodes}
            activeEpisodeId={s.activeEpisodeId}
            showHistory={c.showHistory}
            messages={messages}
            sending={c.sending}
            projectNew={c.projectNew}
            quickPrompts={quickPrompts}
            input={c.input}
            isMockMode={s.isMockMode}
            canvasImages={canvasImages}
            attachments={c.attachments}
            craftTags={c.craftTags}
            actionMode={c.actionMode}
            aspectRatio={s.aspectRatio}
            slashCommands={slashCommands}
            shouldFocusInput={c.shouldFocusInput}
            endRef={endRef}
            onSetChatContext={s.setChatContext}
            onSwitchProject={s.switchProject}
            onToggleHistory={() => c.setShowHistory((v) => !v)}
            onCollapse={() => {
              c.setExpanded(false);
              c.setShouldFocusInput(false);
            }}
            onSelectEpisode={s.selectEpisode}
            onStartRun={() => canvasActions.startRunConfirmation("/run")}
            onToggleProjectNew={() => c.setProjectNew((v) => !v)}
            onOpenPublishTab={() => s.setActiveRightTab("publish")}
            onUndoAction={canvasActions.handleUndoChatAction}
            onViewOnCanvas={canvasActions.handleViewOnCanvas}
            onSendCommand={handleSendCommand}
            onInputChange={c.setInput}
            onToggleMockMode={() => s.setIsMockMode(!s.isMockMode)}
            onAddAttachment={c.addAttachmentUnique}
            onRemoveAttachment={c.removeAttachment}
            onToggleCraftTag={c.toggleCraftTag}
            onActionModeChange={c.setActionMode}
            onAspectRatioChange={s.setAspectRatio}
            onClearChat={() => s.clearProjectChat(s.activeProjectId)}
          />
        )}

        {s.pendingConfirmation && (
          <ChatPendingConfirmModal
            confirmation={s.pendingConfirmation}
            locale={locale}
            onCancel={() => s.setPendingConfirmation(null)}
            onProceed={canvasActions.handleConfirmPendingAction}
          />
        )}
      </div>
    </div>
  );
}

export function ChatTab() {
  return (
    <ChatErrorBoundary>
      <ChatTabInner />
    </ChatErrorBoundary>
  );
}
