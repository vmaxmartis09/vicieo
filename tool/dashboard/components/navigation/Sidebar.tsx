"use client";

import { useShallow } from "zustand/react/shallow";
import React, { useState } from "react";
import { useTranslations } from "next-intl";
import { cn } from "@/lib/utils";
import dynamic from "next/dynamic";
import { Plus, Film, Sparkles } from "@/components/ui/icons";
import { useStudioStore, type StudioStore } from "@/store";
import { Mark } from "@/components/brand/Mark";
import { UserBubble } from "./UserBubble";

const AIModelsLibraryModal = dynamic(
  () => import("@/components/studio/AIModelsLibraryModal").then((m) => m.AIModelsLibraryModal),
  { ssr: false }
);

export function Sidebar() {
  const t = useTranslations("leftRail");
  const c = useTranslations("common");
  const {
    activeProjectId,
    switchProject,
    projects,
    episodes,
    activeEpisodeId,
    selectEpisode,
    createEpisode,
    graphsByEpisode,
  } = useStudioStore(
    useShallow((s: StudioStore) => ({
      activeProjectId: s.activeProjectId,
      switchProject: s.switchProject,
      projects: s.projects,
      episodes: s.episodes,
      activeEpisodeId: s.activeEpisodeId,
      selectEpisode: s.selectEpisode,
      createEpisode: s.createEpisode,
      graphsByEpisode: s.graphsByEpisode,
    }))
  );

  const [creatingEp, setCreatingEp] = useState(false);
  const [newEpTitle, setNewEpTitle] = useState("");
  const [showModelsModal, setShowModelsModal] = useState(false);

  const handleCreateEpisode = (e: React.FormEvent) => {
    e.preventDefault();
    const title = newEpTitle.trim() || t("newEpisodeDefault");
    createEpisode(title);
    setNewEpTitle("");
    setCreatingEp(false);
  };

  return (
    <aside className="hidden md:flex w-[228px] shrink-0 flex-col h-dvh bg-background px-4 pt-3 pb-3 select-none">
      {/* Product mark and name */}
      <div className="flex items-center gap-2 px-1 pb-3">
        <Mark className="h-6 w-6" />
        <span className="text-callout font-semibold tracking-tight text-foreground">
          {c("appName")}
        </span>
      </div>

      {/* Project switcher */}
      <label className="mb-3 block px-1">
        <span className="mb-1 block text-footnote text-muted-foreground">
          {t("projectLabel")}
        </span>
        <select
          value={activeProjectId}
          onChange={(e) => switchProject(e.target.value)}
          className="h-8 w-full cursor-pointer rounded-ctl border border-border bg-card px-2 text-body font-medium text-foreground"
        >
          {projects.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>
      </label>

      {/* Episodes / Graphs header */}
      <div className="mb-2 flex items-center justify-between px-1">
        <span className="text-footnote font-medium text-muted-foreground">
          {t("episodesLabel")}
        </span>
        <button
          type="button"
          onClick={() => setCreatingEp((v) => !v)}
          title={t("newEpisode")}
          aria-label={t("newEpisode")}
          className="flex h-6 w-6 items-center justify-center rounded-ctl border border-border bg-card text-foreground hover:bg-accent"
        >
          <Plus className="h-3.5 w-3.5" />
        </button>
      </div>

      {creatingEp && (
        <form
          onSubmit={handleCreateEpisode}
          className="mb-2 space-y-2 rounded-ctl border border-border bg-card p-2"
        >
          <label className="block text-body text-muted-foreground">
            {t("newEpisodePrompt")}
          </label>
          <input
            type="text"
            value={newEpTitle}
            onChange={(e) => setNewEpTitle(e.target.value)}
            placeholder={t("newEpisodeDefault")}
            autoFocus
            className="h-7 w-full rounded-ctl border border-border bg-background px-2 text-footnote text-foreground"
          />
          <div className="flex justify-end gap-1">
            <button
              type="button"
              onClick={() => setCreatingEp(false)}
              className="h-6 rounded-ctl border border-border px-2 text-body text-muted-foreground hover:text-foreground"
            >
              {c("cancel")}
            </button>
            <button
              type="submit"
              className="h-6 rounded-ctl bg-primary px-2 text-body font-medium text-primary-foreground"
            >
              {c("create")}
            </button>
          </div>
        </form>
      )}

      {/* Episode / Graph list */}
      <nav
        aria-label={t("episodesLabel")}
        className="flex-1 space-y-1 overflow-y-auto px-1 scroll-quiet"
      >
        {episodes.map((ep) => {
          const active = ep.id === activeEpisodeId;
          const g = graphsByEpisode[ep.id];
          const count = g ? g.nodes.length : ep.nodeCount;
          return (
            <button
              key={ep.id}
              type="button"
              onClick={() => selectEpisode(ep.id)}
              aria-current={active ? "page" : undefined}
              className={cn(
                "flex w-full flex-col gap-1 rounded-ctl border px-3 py-2 text-left transition-colors",
                active
                  ? "border-border bg-card font-medium text-foreground"
                  : "border-transparent text-muted-foreground hover:bg-accent/60 hover:text-foreground"
              )}
            >
              <div className="flex items-center gap-2">
                <Film className="h-3.5 w-3.5 shrink-0" strokeWidth={1.5} />
                <span className="truncate text-body">{ep.title}</span>
              </div>
              <div className="flex items-center justify-between pl-5 text-footnote text-muted-foreground tnum">
                <span>
                  {count > 0
                    ? t("nodeCount", { count })
                    : t("emptyGraphTag")}
                </span>
                <span>{t(`episodeStatus.${ep.status}`)}</span>
              </div>
            </button>
          );
        })}
      </nav>

      {/* Kho Mô hình AI (AI Models Library) */}
      <div className="pt-2 border-t border-border/70 space-y-1">
        <button
          type="button"
          onClick={() => setShowModelsModal(true)}
          className="flex w-full items-center justify-between rounded-ctl border border-border/70 bg-card/60 p-2 text-left transition-colors hover:border-foreground/40 hover:bg-accent cursor-pointer group"
        >
          <div className="flex items-center gap-2 min-w-0">
            <div className="flex h-6 w-6 items-center justify-center rounded-ctl bg-brand/10 text-brand shrink-0 group-hover:scale-105 transition-transform">
              <Sparkles className="h-3.5 w-3.5" />
            </div>
            <div className="min-w-0">
              <div className="truncate text-footnote font-semibold text-foreground">Kho Mô hình AI</div>
              <div className="truncate text-caption text-muted-foreground">14+ SOTA Models Detail</div>
            </div>
          </div>
          <span className="rounded-xs bg-brand/10 px-2 py-1 font-mono text-caption font-semibold text-brand shrink-0">
            Detail
          </span>
        </button>
      </div>

      <AIModelsLibraryModal
        isOpen={showModelsModal}
        onClose={() => setShowModelsModal(false)}
      />

      {/* Bottom User Bubble */}
      <div className="mt-2 pt-2 border-t border-border/70">
        <UserBubble />
      </div>
    </aside>
  );
}
