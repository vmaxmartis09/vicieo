"use client";

import React from "react";
import {
  Users,
  Layers,
  FolderGit2,
  MessageSquare,
  X,
} from "@/components/ui/icons";
import { useHubScope } from "@/lib/hub";
import type { Project, Character, Episode } from "@/lib/repositories/types";

interface ChatContextBarProps {
  vi: boolean;
  focus: "character" | "flow" | "project";
  onFocusChange: (focus: "character" | "flow" | "project") => void;
  projects: Project[];
  activeProjectId: string;
  onSwitchProject: (projectId: string) => void;
  showHistory: boolean;
  onToggleHistory: () => void;
  messagesCount: number;
  onCollapse: () => void;
  // Character sub-bar
  characters: Character[];
  activeCharId?: string;
  onSelectCharacter: (charId: string, mode: "create" | "update") => void;
  // Flow sub-bar
  episodes: Episode[];
  activeEpisodeId: string;
  flowMode?: "current" | "new" | "blank";
  onSelectEpisode: (episodeId: string) => void;
  onSetFlowMode: (mode: "current" | "new" | "blank") => void;
  onStartRun: () => void;
  // Project sub-bar
  projectNew: boolean;
  onToggleProjectNew: () => void;
  onOpenPublishTab: () => void;
}

export const ChatContextBar = React.memo(function ChatContextBar({
  vi,
  focus,
  onFocusChange,
  projects,
  activeProjectId,
  onSwitchProject,
  showHistory,
  onToggleHistory,
  messagesCount,
  onCollapse,
  characters,
  activeCharId,
  onSelectCharacter,
  episodes,
  activeEpisodeId,
  flowMode,
  onSelectEpisode,
  onSetFlowMode,
  onStartRun,
  projectNew,
  onToggleProjectNew,
  onOpenPublishTab,
}: ChatContextBarProps) {
  const sel =
    "h-7 min-w-0 flex-1 truncate rounded-ctl border border-input bg-card px-2 text-footnote text-foreground outline-none focus:border-brand";

  return (
    <div className="space-y-2 rounded-t-panel border-b border-border bg-card-elevated px-3 py-2">
      <div className="flex items-center justify-between gap-2 flex-wrap">
        {/* Bộ chọn Phạm vi: Nhân vật / Flow / Dự án */}
        <div
          role="tablist"
          aria-label="Chat context focus"
          className="flex items-center gap-1 rounded-full border border-input/70 bg-accent p-1"
        >
          {(
            [
              ["character", vi ? "Nhân vật" : "Character", Users],
              ["flow", "Flow", Layers],
              ["project", vi ? "Dự án" : "Project", FolderGit2],
            ] as const
          ).map(([k, label, Icon]) => (
            <button
              key={k}
              type="button"
              role="tab"
              aria-selected={focus === k}
              onClick={() => onFocusChange(k)}
              className={`inline-flex h-7 items-center gap-2 rounded-full px-3 text-footnote font-medium transition-all focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring ${
                focus === k
                  ? "bg-card text-foreground shadow-xs font-semibold"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              <Icon className="h-3 w-3" />
              <span>{label}</span>
            </button>
          ))}
        </div>

        {/* Phạm vi Dự án + Bật/tắt Lịch sử hội thoại + Thu gọn */}
        <div className="flex items-center gap-1">
          <div className="flex h-7 items-center gap-1 rounded-full border border-input bg-card px-2 text-footnote">
            <FolderGit2 className="h-3 w-3 text-brand shrink-0" />
            <select
              aria-label="Project"
              className="max-w-[125px] truncate bg-transparent font-medium text-foreground outline-none cursor-pointer text-footnote"
              value={activeProjectId}
              onChange={(e) => onSwitchProject(e.target.value)}
            >
              {projects.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </div>

          <button
            type="button"
            onClick={onToggleHistory}
            className={`inline-flex h-7 items-center gap-1 rounded-full border px-2 text-footnote font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring ${
              showHistory
                ? "border-brand/60 bg-brand/15 text-brand"
                : "border-input bg-card text-foreground/80 hover:bg-accent hover:text-foreground"
            }`}
          >
            <MessageSquare className="h-3 w-3" />
            <span>{vi ? "Hội thoại" : "History"}</span>
            {messagesCount > 0 && (
              <span className="rounded-full bg-brand/20 px-1 text-caption font-semibold text-brand tnum">
                {messagesCount}
              </span>
            )}
          </button>

          <button
            type="button"
            title={vi ? "Thu gọn (Esc)" : "Collapse (Esc)"}
            aria-label={vi ? "Thu gọn" : "Collapse"}
            onClick={onCollapse}
            className="flex h-7 w-7 items-center justify-center rounded-full border border-input bg-card text-foreground/80 hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>

      {/* Thanh Action theo ngữ cảnh: Nhân vật / Flow / Dự án */}
      {focus === "character" && (
        <div className="flex flex-wrap items-center gap-1 pt-1">
          <button
            type="button"
            onClick={() => onSelectCharacter("__new__", "create")}
            className={`h-7 rounded-full border px-3 text-footnote font-medium focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring ${
              activeCharId === "__new__"
                ? "border-primary bg-primary text-primary-foreground"
                : "border-input bg-card text-foreground hover:bg-accent"
            }`}
          >
            ＋ {vi ? "Tạo mới" : "New"}
          </button>
          {characters.map((ch) => (
            <button
              key={ch.id}
              type="button"
              onClick={() => onSelectCharacter(ch.id, "update")}
              className={`h-7 rounded-full border px-3 text-footnote font-medium focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring ${
                activeCharId === ch.id
                  ? "border-primary bg-primary text-primary-foreground"
                  : "border-input bg-card text-foreground hover:bg-accent"
              }`}
            >
              {ch.name} <span className="opacity-70 tnum">({ch.heightCm}cm)</span>
            </button>
          ))}
          <div className="ml-auto flex items-center gap-1">
            <button
              type="button"
              onClick={() => useHubScope.getState().openTab("characters", "create-character")}
              className="h-7 rounded-full border border-input bg-card px-3 text-footnote text-foreground/80 hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
            >
              ＋ {vi ? "Form Nhân vật" : "Character Form"}
            </button>
            <button
              type="button"
              onClick={() => useHubScope.getState().openTab("characters", null, activeCharId ?? null)}
              className="h-7 rounded-full border border-input bg-card px-3 text-footnote text-foreground/80 hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
            >
              {vi ? "Kho Nhân vật" : "Characters"}
            </button>
          </div>
        </div>
      )}

      {focus === "flow" && (
        <div className="flex flex-wrap items-center gap-1 pt-1">
          <select
            aria-label="Flow"
            className={sel}
            value={activeEpisodeId}
            onChange={(e) => {
              onSetFlowMode("current");
              onSelectEpisode(e.target.value);
            }}
          >
            {episodes.map((ep) => (
              <option key={ep.id} value={ep.id}>
                {ep.title}
              </option>
            ))}
          </select>
          <button
            type="button"
            onClick={() => onSetFlowMode("blank")}
            className={`h-7 shrink-0 rounded-ctl border px-3 text-footnote font-medium focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring ${
              flowMode === "blank"
                ? "border-primary bg-primary text-primary-foreground"
                : "border-input bg-card text-foreground hover:bg-accent"
            }`}
          >
            ＋ {vi ? "Trắng" : "Blank"}
          </button>
          <button
            type="button"
            onClick={() => onSetFlowMode("new")}
            className={`h-7 shrink-0 rounded-ctl border px-3 text-footnote font-medium focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring ${
              flowMode === "new"
                ? "border-primary bg-primary text-primary-foreground"
                : "border-input bg-card text-foreground hover:bg-accent"
            }`}
          >
            ＋ {vi ? "Từ mẫu" : "Template"}
          </button>
          <button
            type="button"
            onClick={onStartRun}
            className="h-7 shrink-0 rounded-ctl border border-brand/50 bg-brand/15 px-3 text-footnote font-semibold text-brand hover:bg-brand/25 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
          >
            ▶ {vi ? "Chạy Flow" : "Run"}
          </button>
          <button
            type="button"
            onClick={() => useHubScope.getState().openTab("episodes", null, activeEpisodeId)}
            className="h-7 shrink-0 rounded-ctl border border-input bg-card px-3 text-footnote text-foreground/80 hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
          >
            {vi ? "Danh sách Tập" : "Episodes"}
          </button>
        </div>
      )}

      {focus === "project" && (
        <div className="flex flex-wrap items-center gap-1 pt-1">
          <button
            type="button"
            onClick={onToggleProjectNew}
            className={`h-7 rounded-full border px-3 text-footnote font-medium focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring ${
              projectNew
                ? "border-primary bg-primary text-primary-foreground"
                : "border-input bg-card text-foreground hover:bg-accent"
            }`}
          >
            ＋ {vi ? "Dự án mới" : "New project"}
          </button>
          <button
            type="button"
            onClick={() => useHubScope.getState().openTab("episodes", "create-episode")}
            className="h-7 rounded-full border border-input bg-card px-3 text-footnote text-foreground hover:bg-accent focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
          >
            ＋ {vi ? "Tập mới" : "New episode"}
          </button>
          <button
            type="button"
            onClick={() => useHubScope.getState().openTab("scripts", "compose-script")}
            className="h-7 rounded-full border border-input bg-card px-3 text-footnote text-foreground hover:bg-accent focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
          >
            ＋ {vi ? "Soạn kịch bản" : "Compose script"}
          </button>
          <button
            type="button"
            onClick={() => useHubScope.getState().openTab("projects", null, activeProjectId)}
            className="h-7 rounded-full border border-input bg-card px-3 text-footnote text-foreground/80 hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
          >
            {vi ? "Quản lý Dự án" : "Manage Projects"}
          </button>
          <button
            type="button"
            onClick={onOpenPublishTab}
            className="h-7 rounded-full border border-input bg-card px-3 text-footnote text-foreground/80 hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors"
          >
            {vi ? "Lịch đăng" : "Schedule"}
          </button>
        </div>
      )}
    </div>
  );
});
