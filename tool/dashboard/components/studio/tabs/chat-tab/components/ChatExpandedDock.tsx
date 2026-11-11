"use client";

import React from "react";
import type { Character, Episode, Project } from "@/lib/repositories/types";
import type { ChatContext, ChatMessage, ChatModel } from "@/store/slices/chatSlice";
import { ChatContextBar } from "../../../chat/ChatContextBar";
import { ChatMessageList } from "../../../chat/ChatMessageList";
import {
  SophisticatedChatInput,
  type CanvasImageItem,
  type ChatAttachment,
  type ComposerActionMode,
} from "../../../chat/SophisticatedChatInput";
import type { ChatSlashCommand } from "../constants";
import { QuickPromptsStrip } from "./QuickPromptsStrip";

interface Props {
  vi: boolean;
  chatContext: ChatContext;
  projects: Project[];
  activeProjectId: string;
  characters: Character[];
  activeChar: Character | null;
  episodes: Episode[];
  activeEpisodeId: string;
  showHistory: boolean;
  messages: ChatMessage[];
  sending: boolean;
  projectNew: boolean;
  quickPrompts: string[];
  input: string;
  isMockMode: boolean;
  canvasImages: CanvasImageItem[];
  attachments: ChatAttachment[];
  craftTags: string[];
  actionMode: ComposerActionMode;
  aspectRatio: "9:16" | "16:9" | "1:1" | "4:5";
  slashCommands: ChatSlashCommand[];
  shouldFocusInput: boolean;
  endRef: React.RefObject<HTMLDivElement | null>;
  onSetChatContext: (patch: Partial<ChatContext>) => void;
  onSwitchProject: (id: string) => void;
  onToggleHistory: () => void;
  onCollapse: () => void;
  onSelectEpisode: (epId: string) => void;
  onStartRun: () => void;
  onToggleProjectNew: () => void;
  onOpenPublishTab: () => void;
  onUndoAction: (msgId: string, snapshot?: any) => void;
  onViewOnCanvas: (nodeId: string) => void;
  onSendCommand: (rawPrompt?: string) => void;
  onInputChange: (val: string) => void;
  onToggleMockMode: () => void;
  onAddAttachment: (att: ChatAttachment) => void;
  onRemoveAttachment: (id: string) => void;
  onToggleCraftTag: (tag: string) => void;
  onActionModeChange: (mode: ComposerActionMode) => void;
  onAspectRatioChange: (r: "9:16" | "16:9" | "1:1" | "4:5") => void;
  onClearChat: () => void;
}

export function ChatExpandedDock({
  vi,
  chatContext,
  projects,
  activeProjectId,
  characters,
  activeChar,
  episodes,
  activeEpisodeId,
  showHistory,
  messages,
  sending,
  projectNew,
  quickPrompts,
  input,
  isMockMode,
  canvasImages,
  attachments,
  craftTags,
  actionMode,
  aspectRatio,
  slashCommands,
  shouldFocusInput,
  endRef,
  onSetChatContext,
  onSwitchProject,
  onToggleHistory,
  onCollapse,
  onSelectEpisode,
  onStartRun,
  onToggleProjectNew,
  onOpenPublishTab,
  onUndoAction,
  onViewOnCanvas,
  onSendCommand,
  onInputChange,
  onToggleMockMode,
  onAddAttachment,
  onRemoveAttachment,
  onToggleCraftTag,
  onActionModeChange,
  onAspectRatioChange,
  onClearChat,
}: Props) {
  return (
    <div
      data-chat-dock=""
      className="flex flex-col rounded-panel border border-input bg-card-elevated shadow-2xl ring-1 ring-foreground/15 transition-all duration-200"
    >
      <ChatContextBar
        vi={vi}
        focus={chatContext.focus}
        onFocusChange={(focus) => onSetChatContext({ focus })}
        projects={projects}
        activeProjectId={activeProjectId}
        onSwitchProject={onSwitchProject}
        showHistory={showHistory}
        onToggleHistory={onToggleHistory}
        messagesCount={messages.length}
        onCollapse={onCollapse}
        characters={characters}
        activeCharId={activeChar?.id}
        onSelectCharacter={(charId, characterMode) => onSetChatContext({ characterId: charId, characterMode })}
        episodes={episodes}
        activeEpisodeId={activeEpisodeId}
        flowMode={chatContext.flowMode}
        onSelectEpisode={(epId) => {
          onSetChatContext({ flowMode: "current" });
          onSelectEpisode(epId);
        }}
        onSetFlowMode={(flowMode) => onSetChatContext({ flowMode })}
        onStartRun={onStartRun}
        projectNew={projectNew}
        onToggleProjectNew={onToggleProjectNew}
        onOpenPublishTab={onOpenPublishTab}
      />

      {showHistory && (
        <ChatMessageList
          messages={messages}
          sending={sending}
          vi={vi}
          onUndoAction={onUndoAction}
          onViewOnCanvas={onViewOnCanvas}
          endRef={endRef}
        />
      )}

      <QuickPromptsStrip quickPrompts={quickPrompts} disabled={sending} onSelect={(qp) => onSendCommand(qp)} />

      <div className="rounded-b-panel bg-card">
        <SophisticatedChatInput
          input={input}
          onInputChange={onInputChange}
          onSubmit={() => onSendCommand()}
          disabled={sending}
          model={chatContext.model}
          onModelChange={(m: ChatModel) => onSetChatContext({ model: m })}
          isMockMode={isMockMode}
          onToggleMockMode={onToggleMockMode}
          characters={characters}
          canvasImages={canvasImages}
          attachments={attachments}
          onAddAttachment={onAddAttachment}
          onRemoveAttachment={onRemoveAttachment}
          craftTags={craftTags}
          onToggleCraftTag={onToggleCraftTag}
          actionMode={actionMode}
          onActionModeChange={onActionModeChange}
          aspectRatio={aspectRatio}
          onAspectRatioChange={onAspectRatioChange}
          onMentionSelect={(charId, charName) => {
            onInputChange(input.replace(/@\S*$/, `@${charName} `));
            onSetChatContext({ characterId: charId, characterMode: "update" });
          }}
          onSlashSelect={(cmd) => {
            if (cmd === "/video") onActionModeChange("video");
            else if (cmd === "/image") onActionModeChange("image");
            else if (cmd === "/grid") onActionModeChange("image_list");
            onInputChange(`${cmd} `);
          }}
          onClearChat={onClearChat}
          slashCommands={slashCommands}
          autoFocus={shouldFocusInput}
        />
      </div>
    </div>
  );
}
