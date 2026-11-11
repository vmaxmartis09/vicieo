"use client";

import React, { useRef } from "react";
import { useLocale, useTranslations } from "next-intl";
import { MODEL_OPTIONS, getModePlaceholder } from "./constants";
import { useVoiceDictation } from "./hooks/useVoiceDictation";
import { useComposerMenus } from "./hooks/useComposerMenus";
import { AutocompletePopovers } from "./components/AutocompletePopovers";
import { PlusActionPopover } from "./components/PlusActionPopover";
import { SkillCraftPopover } from "./components/SkillCraftPopover";
import { ModelSettingsPopover } from "./components/ModelSettingsPopover";
import { ModeSwitchPopover } from "./components/ModeSwitchPopover";
import { AttachmentsBar } from "./components/AttachmentsBar";
import { ComposerToolbar } from "./components/ComposerToolbar";
import type { SophisticatedChatInputProps } from "./types";

export type { ChatAttachment, CanvasImageItem, ComposerActionMode } from "./types";

export function SophisticatedChatInput({
  input,
  onInputChange,
  onSubmit,
  disabled = false,
  model,
  onModelChange,
  isMockMode,
  onToggleMockMode,
  characters,
  canvasImages = [],
  attachments = [],
  onAddAttachment,
  onRemoveAttachment,
  craftTags = [],
  onToggleCraftTag,
  actionMode = "chat",
  onActionModeChange,
  aspectRatio = "9:16",
  onAspectRatioChange,
  onMentionSelect,
  onSlashSelect,
  onClearChat,
  slashCommands,
  placeholder,
  autoFocus = false,
}: SophisticatedChatInputProps) {
  const t = useTranslations("chatTab");
  const isVi = useLocale() === "vi";

  const containerRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const menus = useComposerMenus(containerRef, textareaRef, input, autoFocus, onAddAttachment);
  const { isListening, toggleSpeechRecognition } = useVoiceDictation(input, onInputChange, isVi);
  const currentModelObj = MODEL_OPTIONS.find((m) => m.id === model) || MODEL_OPTIONS[0];

  const isSlash = /^\/\S*$/.test(input);
  const isMention = /(?:^|\s)@(\S*)$/.test(input);
  const mentionQuery = /(?:^|\s)@(\S*)$/.exec(input)?.[1]?.toLowerCase() ?? "";
  const filteredChars = characters.filter((c) => c.name.toLowerCase().includes(mentionQuery));
  const filteredCanvasImgs = canvasImages.filter((img) => img.name.toLowerCase().includes(mentionQuery));

  return (
    <div ref={containerRef} className="relative w-full">
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        multiple
        className="hidden"
        onChange={menus.handleFileUpload}
      />

      <AutocompletePopovers
        isSlash={isSlash}
        isMention={isMention}
        input={input}
        isVi={isVi}
        slashCommands={slashCommands}
        filteredChars={filteredChars}
        filteredCanvasImgs={filteredCanvasImgs}
        onSlashSelect={onSlashSelect}
        onMentionSelect={onMentionSelect}
        onAddAttachment={onAddAttachment}
        onInputChange={onInputChange}
        onFocusTextarea={() => textareaRef.current?.focus()}
      />

      {menus.plusMenuOpen && (
        <PlusActionPopover
          isVi={isVi}
          canvasImages={canvasImages}
          onTriggerUpload={() => fileInputRef.current?.click()}
          onAddAttachment={onAddAttachment}
          onActionModeChange={onActionModeChange}
          onClearChat={onClearChat}
          onClose={() => menus.setPlusMenuOpen(false)}
          onFocusTextarea={() => textareaRef.current?.focus()}
        />
      )}

      {menus.skillMenuOpen && (
        <SkillCraftPopover
          isVi={isVi}
          craftTags={craftTags}
          slashCommands={slashCommands}
          onToggleCraftTag={onToggleCraftTag}
          onSlashSelect={onSlashSelect}
          onClose={() => menus.setSkillMenuOpen(false)}
          onFocusTextarea={() => textareaRef.current?.focus()}
        />
      )}

      {menus.modelMenuOpen && (
        <ModelSettingsPopover
          isVi={isVi}
          model={model}
          modelTab={menus.modelTab}
          aspectRatio={aspectRatio}
          actionMode={actionMode}
          onModelTabChange={menus.setModelTab}
          onModelChange={onModelChange}
          onAspectRatioChange={onAspectRatioChange}
          onActionModeChange={onActionModeChange}
          onClose={() => menus.setModelMenuOpen(false)}
        />
      )}

      {menus.modeMenuOpen && (
        <ModeSwitchPopover
          isVi={isVi}
          isMockMode={isMockMode}
          onToggleMockMode={onToggleMockMode}
          onClose={() => menus.setModeMenuOpen(false)}
        />
      )}

      <div className="flex flex-col">
        <AttachmentsBar
          isVi={isVi}
          actionMode={actionMode}
          attachments={attachments}
          craftTags={craftTags}
          onActionModeChange={onActionModeChange}
          onRemoveAttachment={onRemoveAttachment}
          onToggleCraftTag={onToggleCraftTag}
        />

        <textarea
          ref={textareaRef}
          rows={1}
          value={input}
          onChange={(e) => onInputChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
              e.preventDefault();
              menus.closeAllMenus();
              onSubmit();
            }
          }}
          placeholder={placeholder || getModePlaceholder(actionMode, isVi)}
          className="w-full min-h-14 sm:min-h-10 resize-none border-0 bg-transparent px-4 py-2 text-body text-foreground placeholder:text-muted-foreground/75 outline-none focus:outline-none focus-visible:outline-none focus:ring-0 leading-snug"
        />

        <ComposerToolbar
          isVi={isVi}
          sendLabel={t("sendBtn")}
          plusMenuOpen={menus.plusMenuOpen}
          skillMenuOpen={menus.skillMenuOpen}
          modelMenuOpen={menus.modelMenuOpen}
          modeMenuOpen={menus.modeMenuOpen}
          craftTagsCount={craftTags.length}
          currentModelName={currentModelObj.name}
          aspectRatio={aspectRatio}
          isMockMode={isMockMode}
          isListening={isListening}
          disabled={disabled}
          canSubmit={Boolean(input.trim() || attachments.length > 0)}
          actionMode={actionMode}
          onTogglePlusMenu={() => menus.toggleMenu("plus")}
          onToggleSkillMenu={() => menus.toggleMenu("skill")}
          onToggleModelMenu={() => menus.toggleMenu("model")}
          onToggleModeMenu={() => menus.toggleMenu("mode")}
          onToggleSpeech={toggleSpeechRecognition}
          onSubmitClick={() => {
            menus.closeAllMenus();
            onSubmit();
          }}
        />
      </div>
    </div>
  );
}
