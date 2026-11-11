"use client";

import { useCallback } from "react";
import { getRepository } from "@/lib/repositories";
import { generateCharacterDraft } from "@/lib/repositories/character-assets";
import type { Character } from "@/lib/repositories/types";
import { runLocalAgent } from "@/lib/studio-agent";
import type { ChatAttachment, ComposerActionMode } from "@/components/studio/chat/SophisticatedChatInput";
import type { ChatSnapshot, DirectMediaMode } from "../types";

interface Params {
  activeProjectId: string;
  isMockMode: boolean;
  vi: boolean;
  actionMode: ComposerActionMode;
  attachments: ChatAttachment[];
  chatFocus: "character" | "flow" | "project";
  activeChar: Character | null;
  projectNew: boolean;
  routerPush: (path: string) => void;
  setProjectNew: (val: boolean) => void;
  setAttachments: React.Dispatch<React.SetStateAction<ChatAttachment[]>>;
  setCharacters: (chars: Character[]) => void;
  clearProjectChat: (pid: string) => void;
  setNodesAndEdges: (nodes: any[], edges: any[], pushHistory: boolean) => void;
  startRunConfirmation: (promptText: string) => void;
  executeDirectMediaGeneration: (mode: DirectMediaMode, body: string, snap: ChatSnapshot) => string;
}

export function useChatContextActions(p: Params) {
  return useCallback(
    async (raw: string, snapshotBefore: ChatSnapshot): Promise<string | null> => {
      const trimmed = raw.trim();
      const parts = trimmed.split(/\s+/);
      const cmd = parts[0]?.toLowerCase();
      const body = parts.slice(1).join(" ").trim();

      if (cmd === "/grid" || (p.actionMode === "image_list" && (p.attachments.length > 0 || body) && !cmd?.startsWith("/"))) {
        p.executeDirectMediaGeneration("image_list", cmd === "/grid" ? body : trimmed, snapshotBefore);
        p.setAttachments([]);
        return "";
      }
      if (cmd === "/video" || (p.actionMode === "video" && (p.attachments.length > 0 || body) && !cmd?.startsWith("/"))) {
        p.executeDirectMediaGeneration("video", cmd === "/video" ? body : trimmed, snapshotBefore);
        p.setAttachments([]);
        return "";
      }
      if (cmd === "/image" || (p.actionMode === "image" && (p.attachments.length > 0 || body) && !cmd?.startsWith("/"))) {
        p.executeDirectMediaGeneration("image", cmd === "/image" ? body : trimmed, snapshotBefore);
        p.setAttachments([]);
        return "";
      }

      if (cmd === "/run") {
        p.startRunConfirmation(trimmed);
        return "";
      }
      if (cmd === "/clear") {
        p.clearProjectChat(p.activeProjectId);
        return p.vi ? "Đã xóa hội thoại dự án." : "Chat cleared.";
      }
      if (cmd === "/blank") {
        p.setNodesAndEdges([], [], true);
        return p.vi ? "Đã dọn sạch canvas (flow trắng)." : "Cleared canvas (blank flow).";
      }

      if (cmd === "/char") {
        const repo = getRepository();
        const { draft } = await generateCharacterDraft(p.activeProjectId, body || "Nhân vật mới", p.isMockMode);
        const newChar = {
          id: `char_${Date.now().toString(36)}`,
          name: draft.name || "Nhân vật mới",
          role: "main" as const,
          heightCm: draft.heightCm ?? 100,
          description: draft.description || body,
          referenceImages: p.attachments.map((a) => a.url),
          memory: {
            personality: draft.personality || "Vui vẻ",
            habits: draft.habits || [],
            catchphrases: draft.catchphrases || [],
          },
        };
        await repo.upsertCharacter(p.activeProjectId, newChar as any);
        p.setCharacters(await repo.listCharacters(p.activeProjectId));
        p.setAttachments([]);
        return p.vi
          ? `Đã tạo nhân vật “${newChar.name}” (${newChar.heightCm} cm).`
          : `Created character “${newChar.name}” (${newChar.heightCm} cm).`;
      }

      if (p.chatFocus === "character" && p.activeChar && body) {
        const { draft, mode } = await generateCharacterDraft(
          p.activeProjectId,
          `${p.activeChar.name}: ${p.activeChar.description || ""}. Cập nhật: ${body}`,
          p.isMockMode
        );
        const updated = {
          ...p.activeChar,
          description: mode === "gemini" && draft.description ? draft.description : p.activeChar.description,
          referenceImages:
            p.attachments.length > 0
              ? [...(p.activeChar.referenceImages || []), ...p.attachments.map((a) => a.url)]
              : p.activeChar.referenceImages,
          memory: {
            ...p.activeChar.memory,
            personality: mode === "gemini" && draft.personality ? draft.personality : p.activeChar.memory?.personality,
            habits:
              mode === "gemini"
                ? draft.habits ?? p.activeChar.memory?.habits ?? []
                : [...(p.activeChar.memory?.habits ?? []), body],
            catchphrases: mode === "gemini" ? draft.catchphrases ?? p.activeChar.memory?.catchphrases ?? [] : p.activeChar.memory?.catchphrases ?? [],
          },
        };
        await getRepository().upsertCharacter(p.activeProjectId, updated as any);
        p.setCharacters(await getRepository().listCharacters(p.activeProjectId));
        p.setAttachments([]);
        return p.vi
          ? `Đã cập nhật “${p.activeChar.name}” (${mode === "gemini" ? "Gemini" : "bộ nhớ"}). Chiều cao ${p.activeChar.heightCm} cm giữ nguyên.`
          : `Updated “${p.activeChar.name}” (${mode === "gemini" ? "Gemini" : "memory"}). Height locked at ${p.activeChar.heightCm} cm.`;
      }

      if (!cmd && p.chatFocus === "project" && p.projectNew) {
        const r = await runLocalAgent(`tạo dự án ${body}`, { push: p.routerPush });
        p.setProjectNew(false);
        return r ?? (p.vi ? "Đã xử lý." : "Handled.");
      }
      return null;
    },
    [
      p.executeDirectMediaGeneration,
      p.actionMode,
      p.attachments,
      p.startRunConfirmation,
      p.clearProjectChat,
      p.activeProjectId,
      p.vi,
      p.setNodesAndEdges,
      p.isMockMode,
      p.setCharacters,
      p.chatFocus,
      p.activeChar,
      p.projectNew,
      p.routerPush,
      p.setProjectNew,
      p.setAttachments,
    ]
  );
}
