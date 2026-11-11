"use client";

import { useCallback } from "react";
import { useStudioStore } from "@/store";
import { estimateNodeCostUsd, sendChatToCore } from "@/lib/core-api";
import { runLocalAgent } from "@/lib/studio-agent";
import type { ChatSendParams, ChatSnapshot } from "../types";
import { buildPendingConfirmationFromMeta } from "../utils/chatConfirmation";

export function useChatSend(p: ChatSendParams) {
  return useCallback(
    async (rawPrompt?: string) => {
      const promptText = (rawPrompt ?? p.input).trim();
      if ((!promptText && p.attachments.length === 0) || p.sending) return;
      if (!rawPrompt) p.setInput("");
      p.setExpanded(true);
      p.setShowHistory(true);
      p.setSending(true);

      const modePrefix = p.actionMode === "video" ? (p.vi ? "Tạo Video từ ảnh" : "Video from image") : (p.vi ? "Tạo Ảnh" : "Generate Image");
      const displayPrompt = p.attachments.length > 0 ? `${promptText || modePrefix} [${p.attachments.map((a) => `@${a.name}`).join(", ")}]` : promptText;

      p.addChatMessage(p.activeProjectId, {
        id: `msg_u_${Date.now().toString(36)}`,
        role: "user",
        text: displayPrompt,
        timestamp: new Date().toISOString(),
      });

      const storeState = useStudioStore.getState();
      const snapshotBefore: ChatSnapshot = { nodes: [...storeState.nodes], edges: [...storeState.edges], aspectRatio: p.aspectRatio };

      try {
        const ctxReply = await p.runContextAction(promptText, snapshotBefore);
        if (ctxReply !== null) {
          if (ctxReply) p.say(ctxReply);
          return;
        }

        const local = await runLocalAgent(promptText, { push: p.routerPush });
        if (local !== null) {
          p.addChatMessage(p.activeProjectId, {
            id: `msg_a_${Date.now().toString(36)}`,
            role: "assistant",
            text: local,
            timestamp: new Date().toISOString(),
            mode: "provider",
          });
          return;
        }

        const enrichedPrompt = p.craftTags.length > 0 ? `${promptText}\n[Craft: ${p.craftTags.join(", ")}]` : promptText;
        const res = await sendChatToCore(p.activeProjectId, enrichedPrompt, {
          model: p.chatModel,
          flow: p.activeEpisodeId,
          characterId: p.chatCharacterId,
          characterMode: p.chatCharacterMode,
        });
        const firstAction = res.actions?.[0];

        if (res.requiresConfirmation && res.confirmationMeta) {
          const liveNodes = useStudioStore.getState().nodes;
          const estCost = liveNodes.reduce((acc, n) => acc + estimateNodeCostUsd(n.data.nodeType, n.data.params || {}, p.isMockMode), 0);
          p.setPendingConfirmation(buildPendingConfirmationFromMeta(res.confirmationMeta, promptText, estCost, p.isMockMode));
          p.addChatMessage(p.activeProjectId, {
            id: `msg_a_${Date.now().toString(36)}`,
            role: "assistant",
            text: res.reply,
            timestamp: new Date().toISOString(),
            mode: (res.mode as any) || "rule_based_mock",
            actionCard: firstAction
              ? { id: `act_${Date.now().toString(36)}`, summaryVi: firstAction.summaryVi || "Đang chờ xác nhận...", summaryEn: firstAction.summaryEn || "Awaiting confirmation...", snapshotBefore }
              : undefined,
          });
          return;
        }

        const { targetNodeId } = p.executeStudioAction(firstAction);
        p.addChatMessage(p.activeProjectId, {
          id: `msg_a_${Date.now().toString(36)}`,
          role: "assistant",
          text: res.reply,
          timestamp: new Date().toISOString(),
          mode: (res.mode as any) || "rule_based_mock",
          actionCard: firstAction
            ? {
                id: `act_${Date.now().toString(36)}`,
                summaryVi: firstAction.summaryVi || (p.vi ? "Đã thực hiện thao tác trên Studio" : "Executed action"),
                summaryEn: firstAction.summaryEn || "Executed action on Studio canvas",
                targetNodeId,
                snapshotBefore,
              }
            : undefined,
        });
      } catch (err) {
        p.addChatMessage(p.activeProjectId, {
          id: `msg_a_${Date.now().toString(36)}`,
          role: "assistant",
          text: p.vi ? `Lỗi: ${err instanceof Error ? err.message : String(err)}` : `Error: ${err instanceof Error ? err.message : String(err)}`,
          timestamp: new Date().toISOString(),
          mode: "provider",
        });
      } finally {
        p.setSending(false);
      }
    },
    [p.input, p.attachments, p.sending, p.actionMode, p.vi, p.activeProjectId, p.addChatMessage, p.aspectRatio, p.runContextAction, p.say, p.routerPush, p.craftTags, p.chatModel, p.chatCharacterId, p.chatCharacterMode, p.activeEpisodeId, p.isMockMode, p.setPendingConfirmation, p.executeStudioAction, p.setInput, p.setExpanded, p.setShowHistory, p.setSending]
  );
}
