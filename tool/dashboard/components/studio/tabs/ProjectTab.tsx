"use client";

import { useShallow } from "zustand/react/shallow";
import React, { useMemo, useState } from "react";
import { useTranslations } from "next-intl";
import { useStudioStore, type StudioStore } from "@/store";
import { getRepository } from "@/lib/repositories";
import type { Character } from "@/lib/repositories/types";
import {
  Copy,
  Plus,
  Search,
  Trash2,
} from "@/components/ui/icons";
import { cn } from "@/lib/utils";
import { PolicyEditor } from "@/components/studio/PolicyEditor";
import { CharacterProfileDetail } from "./CharacterProfileDetail";

export function ProjectTab() {
  const t = useTranslations("projectTab");
  const c = useTranslations("common");
  const {
    activeProjectId,
    projects,
    episodes,
    activeEpisodeId,
    selectEpisode,
    characters,
    upsertCharacterInStore,
    removeCharacterFromStore,
    selectedCharacterId,
    setSelectedCharacterId,
    activeCharacterSection,
    setActiveCharacterSection,
    setNodesAndEdges,
    setSelectedNodeId,
    flashNode,
  } = useStudioStore(
    useShallow((s: StudioStore) => ({
      activeProjectId: s.activeProjectId,
      projects: s.projects,
      episodes: s.episodes,
      activeEpisodeId: s.activeEpisodeId,
      selectEpisode: s.selectEpisode,
      characters: s.characters,
      upsertCharacterInStore: s.upsertCharacterInStore,
      removeCharacterFromStore: s.removeCharacterFromStore,
      selectedCharacterId: s.selectedCharacterId,
      setSelectedCharacterId: s.setSelectedCharacterId,
      activeCharacterSection: s.activeCharacterSection,
      setActiveCharacterSection: s.setActiveCharacterSection,
      setNodesAndEdges: s.setNodesAndEdges,
      setSelectedNodeId: s.setSelectedNodeId,
      flashNode: s.flashNode,
    }))
  );

  const [charSearch, setCharSearch] = useState("");
  const [confirmDeleteCharId, setConfirmDeleteCharId] = useState<string | null>(null);
  const [savedToast, setSavedToast] = useState(false);

  const project = projects.find((p) => p.id === activeProjectId);
  const selectedChar = characters.find((ch) => ch.id === selectedCharacterId);

  const filteredCharacters = useMemo(() => {
    const q = charSearch.trim().toLowerCase();
    if (!q) return characters;
    return characters.filter(
      (ch) =>
        ch.name.toLowerCase().includes(q) ||
        ch.id.toLowerCase().includes(q) ||
        ch.voiceId.toLowerCase().includes(q) ||
        ch.role.toLowerCase().includes(q)
    );
  }, [characters, charSearch]);

  const handleSaveCharacter = async (updated: Character) => {
    upsertCharacterInStore(updated);
    setSavedToast(true);
    window.setTimeout(() => setSavedToast(false), 1800);
    try {
      await getRepository().upsertCharacter(activeProjectId, updated);
    } catch {
      // already in store
    }
  };

  const handleAddCharacter = async () => {
    const newId = `char_${Date.now().toString(36)}`;
    const newChar: Character = {
      id: newId,
      projectId: activeProjectId,
      name: "Nhân vật phụ mới",
      role: "supporting",
      description: "Nhân vật phụ trong thế giới khu rừng Sóc Nhỏ TV.",
      heightCm: 120,
      scaleRatio: 1.2,
      voiceId: "soc-nho",
      defaultTempo: 1.0,
      sampleAudioUrl: "/demo/audio/soc-nho_sample.mp3",
      promptLock:
        "3D Pixar-style cute woodland character, 120cm height, warm colors",
      referenceImages: ["/demo/characters/soc_nho_04.png"],
      referenceLibrary: [
        {
          id: "img_init_1",
          url: "/demo/characters/soc_nho_04.png",
          angle: "Chính diện",
          isPrimary: true,
          source: "upload",
          createdAt: new Date().toISOString(),
        },
      ],
      generatedClips: [],
      appearances: [],
      memory: {
        personality: "Thân thiện, vui vẻ",
        catchphrases: ["Chào bạn nha!"],
        habits: ["Hay cười"],
        pastEvents: [],
        pastEventsSummary: [],
      },
    };
    await handleSaveCharacter(newChar);
    setSelectedCharacterId(newChar.id);
    setActiveCharacterSection("profile");
  };

  const handleDuplicateCharacter = async (ch: Character) => {
    const copy: Character = {
      ...ch,
      id: `${ch.id}_copy_${Date.now().toString(36).slice(-3)}`,
      name: `${ch.name} (Bản sao)`,
      role: "supporting",
    };
    await handleSaveCharacter(copy);
  };

  const handleConfirmDeleteCharacter = async (id: string) => {
    removeCharacterFromStore(id);
    if (selectedCharacterId === id) {
      setSelectedCharacterId(null);
    }
    setConfirmDeleteCharId(null);
    try {
      await getRepository().deleteCharacter(activeProjectId, id);
    } catch {
      // ignore
    }
  };

  // Canvas node placement uses store.getState() to avoid subscribing to all nodes/edges on render
  const handleDropCharacterToCanvas = (ch: Character) => {
    const store = useStudioStore.getState();
    const curNodes = store.nodes;
    const curEdges = store.edges;
    const newNodeId = `node_char_${ch.id}_${Date.now().toString(36).slice(-3)}`;
    const newNode = {
      id: newNodeId,
      type: "studioCard",
      position: { x: 370, y: 420 + (curNodes.length % 3) * 40 },
      selected: true,
      data: {
        nodeType: "character",
        status: "idle" as const,
        params: { characterId: ch.id },
        generations: [],
        nextStepHint: `Hồ sơ DNA ${ch.name} (${ch.heightCm} cm)`,
      },
    };
    setNodesAndEdges(
      [...curNodes.map((n) => ({ ...n, selected: false })), newNode],
      curEdges,
      true
    );
    setSelectedNodeId(newNodeId);
    flashNode(newNodeId);
  };

  const handleGeneratePhotoByNode = (ch: Character) => {
    const store = useStudioStore.getState();
    const curNodes = store.nodes;
    const curEdges = store.edges;
    const newNodeId = `node_img_${ch.id}_${Date.now().toString(36).slice(-3)}`;
    const newNode = {
      id: newNodeId,
      type: "studioCard",
      position: { x: 700, y: 240 },
      selected: true,
      data: {
        nodeType: "image_gen",
        status: "succeeded" as const,
        params: {
          model: "gemini-2.5-flash-image",
          aspectRatio: "9:16",
          prompt: `${ch.promptLock}, multi-angle reference turnaround sheet`,
        },
        previewType: "image" as const,
        previewUrl: ch.referenceImages[0] || "/demo/characters/soc_nho_03.png",
        summaryText: `Ảnh tham chiếu mới cho ${ch.name}`,
        generations: [],
      },
    };
    setNodesAndEdges(
      [...curNodes.map((n) => ({ ...n, selected: false })), newNode],
      curEdges,
      true
    );
    setSelectedNodeId(newNodeId);
    flashNode(newNodeId);
  };

  // If a character is selected, render isolated memoized CharacterProfileDetail
  if (selectedChar) {
    return (
      <CharacterProfileDetail
        selectedChar={selectedChar}
        activeProjectId={activeProjectId}
        activeCharacterSection={activeCharacterSection}
        setActiveCharacterSection={setActiveCharacterSection}
        onSaveCharacter={handleSaveCharacter}
        onBackToList={() => setSelectedCharacterId(null)}
        onDropToCanvas={handleDropCharacterToCanvas}
        onGeneratePhotoByNode={handleGeneratePhotoByNode}
        savedToast={savedToast}
      />
    );
  }

  // Default view of ProjectTab: Project overview + Episodes + Character Cards List
  return (
    <div className="p-4 space-y-4 text-footnote">
      {/* Project summary */}
      <section className="rounded-card border border-border bg-card p-3 space-y-2">
        <div className="flex items-center justify-between">
          <h3 className="text-headline font-semibold text-foreground">
            {project?.name || "Sóc Nhỏ TV"}
          </h3>
          <span className="rounded-xs border border-border bg-muted px-2 py-1 font-mono text-body tnum">
            {project?.defaultAspectRatio || "9:16"} ·{" "}
            {project?.defaultVoiceId || "soc-nho"}
          </span>
        </div>
        <p className="text-body text-muted-foreground leading-snug">
          {project?.description}
        </p>
      </section>

      {/* Episodes list */}
      <section className="space-y-2 border-b border-border pb-4">
        <h3 className="font-semibold text-foreground tnum text-footnote">
          {t("episodesTitle", { count: episodes.length })}
        </h3>
        <ul className="space-y-1">
          {episodes.map((ep) => (
            <li key={ep.id}>
              <button
                type="button"
                onClick={() => selectEpisode(ep.id)}
                className={cn(
                  "flex w-full items-center justify-between rounded-ctl border px-3 py-2 text-left text-footnote",
                  ep.id === activeEpisodeId
                    ? "border-foreground bg-accent font-medium text-foreground"
                    : "border-border bg-card text-muted-foreground hover:text-foreground"
                )}
              >
                <span className="truncate">{ep.title}</span>
                <span className="shrink-0 font-mono text-body tnum">
                  {ep.nodeCount} bước
                </span>
              </button>
            </li>
          ))}
        </ul>
      </section>

      {/* Chính sách nội dung xuyên suốt dự án */}
      <section className="space-y-3 border-b border-border pb-4">
        <h3 className="font-semibold text-foreground text-footnote">Chính sách nội dung</h3>
        <p className="text-body leading-relaxed text-muted-foreground">
          Áp dụng cho cả dự án: node Kiểm tra chất lượng luôn đối chiếu lời thoại, caption và kịch bản với chính sách này.
        </p>
        <PolicyEditor />
      </section>

      {/* Characters inside Project */}
      <section className="space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="font-semibold text-foreground tnum text-footnote">
            {t("charactersTitle", { count: characters.length })}
          </h3>
          <button
            type="button"
            onClick={handleAddCharacter}
            className="flex h-7 items-center gap-1 rounded-ctl border border-border bg-card px-3 text-footnote font-medium text-foreground hover:bg-accent"
          >
            <Plus className="h-3 w-3" />
            <span>{t("addCharacterBtn")}</span>
          </button>
        </div>

        <p className="text-body text-muted-foreground leading-snug">
          {t("dragHint")}
        </p>

        {/* Character search */}
        <div className="relative">
          <Search className="pointer-events-none absolute left-2 top-2 h-3.5 w-3.5 text-muted-foreground" />
          <input
            type="search"
            value={charSearch}
            onChange={(e) => setCharSearch(e.target.value)}
            placeholder={t("searchCharacters")}
            className="h-7 w-full rounded-ctl border border-border bg-card pl-8 pr-2 text-footnote text-foreground"
          />
        </div>

        {/* Character cards */}
        <ul className="space-y-2">
          {filteredCharacters.map((ch) => (
            <li
              key={ch.id}
              draggable
              onDragStart={(e) => {
                e.dataTransfer.setData("application/x-studio-character", ch.id);
              }}
              className="rounded-card border border-border bg-card p-3 space-y-2 cursor-grab active:cursor-grabbing hover:border-foreground/50"
            >
              <div
                onClick={() => {
                  setSelectedCharacterId(ch.id);
                  setActiveCharacterSection("profile");
                }}
                className="flex items-center gap-2 cursor-pointer"
              >
                {ch.referenceImages[0] ? (
                  // eslint-disable-next-line @next/next/no-img-element
                  <img decoding="async" loading="lazy"
                    src={ch.referenceImages[0]}
                    alt={ch.name}
                    className="h-11 w-11 shrink-0 rounded-ctl border border-border object-cover object-top"
                  />
                ) : (
                  <span className="flex h-11 w-11 shrink-0 items-center justify-center rounded-ctl border border-border bg-accent font-semibold">
                    {ch.name.charAt(0)}
                  </span>
                )}
                <div className="min-w-0 flex-1">
                  <div className="flex items-center justify-between gap-1">
                    <p className="truncate font-semibold text-foreground text-footnote">
                      {ch.name}
                    </p>
                    <span className="shrink-0 font-mono text-body tnum text-muted-foreground">
                      {ch.heightCm} cm ({ch.scaleRatio}x)
                    </span>
                  </div>
                  <p className="truncate text-body text-muted-foreground">
                    {ch.role === "main"
                      ? t("profile.roleMain")
                      : t("profile.roleSupporting")}{" "}
                    · <code>{ch.voiceId}</code>
                  </p>
                </div>
              </div>

              <div className="flex items-center justify-between border-t border-border/70 pt-2">
                <button
                  type="button"
                  onClick={() => handleDropCharacterToCanvas(ch)}
                  className="rounded-ctl border border-border bg-card px-2 py-1 text-caption font-medium text-foreground hover:bg-accent"
                >
                  + {t("dropToCanvasBtn")}
                </button>
                <div className="flex items-center gap-1">
                  <button
                    type="button"
                    onClick={() => handleDuplicateCharacter(ch)}
                    title={t("duplicateBtn")}
                    aria-label={t("duplicateBtn")}
                    className="rounded-ctl p-1 text-muted-foreground hover:bg-accent hover:text-foreground"
                  >
                    <Copy className="h-3.5 w-3.5" />
                  </button>
                  <button
                    type="button"
                    onClick={() => setConfirmDeleteCharId(ch.id)}
                    title={t("deleteBtn")}
                    aria-label={t("deleteBtn")}
                    className="rounded-ctl p-1 text-muted-foreground hover:bg-accent hover:text-semantic-danger"
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </button>
                </div>
              </div>

              {confirmDeleteCharId === ch.id && (
                <div className="rounded-card border border-semantic-danger/40 bg-accent/50 p-2 space-y-2">
                  <p className="text-body text-semantic-danger">
                    {t("deleteConfirmText", { name: ch.name })}
                  </p>
                  <div className="flex justify-end gap-2">
                    <button
                      type="button"
                      onClick={() => setConfirmDeleteCharId(null)}
                      className="h-7 rounded-ctl border border-border bg-card px-2 text-footnote"
                    >
                      {c("cancel")}
                    </button>
                    <button
                      type="button"
                      onClick={() => handleConfirmDeleteCharacter(ch.id)}
                      className="h-7 rounded-ctl bg-semantic-danger px-2 text-footnote font-medium text-white"
                    >
                      {c("delete")}
                    </button>
                  </div>
                </div>
              )}
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
