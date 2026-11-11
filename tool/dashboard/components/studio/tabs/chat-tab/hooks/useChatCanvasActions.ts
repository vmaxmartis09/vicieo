"use client";

import { useCallback } from "react";
import { useStudioStore } from "@/store";
import { estimateNodeCostUsd } from "@/lib/core-api";
import type { ChatCanvasActionsParams, ChatSnapshot, DirectMediaMode } from "../types";
import { buildJobFromConfirmation, buildRunConfirmation, buildScheduledPostFromConfirmation } from "../utils/chatConfirmation";
import { buildAddNodeGraphUpdate, buildDirectMediaGraphUpdate } from "../utils/chatGraphBuilders";

export function useChatCanvasActions(p: ChatCanvasActionsParams) {
  const startRunConfirmation = useCallback(
    (promptText: string) => {
      const currentNodes = useStudioStore.getState().nodes;
      const estCost = currentNodes.reduce(
        (acc, n) => acc + estimateNodeCostUsd(n.data.nodeType, n.data.params || {}, p.isMockMode, p.runPolicy),
        0
      );
      p.setPendingConfirmation(buildRunConfirmation(p.activeEpisodeTitle, estCost, p.isMockMode, promptText));
    },
    [p.activeEpisodeTitle, p.isMockMode, p.runPolicy, p.setPendingConfirmation]
  );

  const executeDirectMediaGeneration = useCallback(
    (mode: DirectMediaMode, promptBody: string, snapshotBefore: ChatSnapshot): string => {
      const storeState = useStudioStore.getState();
      const res = buildDirectMediaGraphUpdate({
        mode,
        promptBody,
        currentNodes: storeState.nodes,
        currentEdges: storeState.edges,
        craftTags: p.craftTags,
        attachments: p.attachments,
        aspectRatio: p.aspectRatio,
        chatModel: p.chatModel,
        vi: p.vi,
      });
      p.setNodesAndEdges(res.nextNodes, res.nextEdges, true);
      p.setSelectedNodeId(res.targetNodeId);
      p.flashNode(res.targetNodeId);
      p.say(res.reply, res.targetNodeId, snapshotBefore, res.summaryVi, res.summaryEn);
      return res.targetNodeId;
    },
    [p.craftTags, p.attachments, p.vi, p.aspectRatio, p.setNodesAndEdges, p.setSelectedNodeId, p.flashNode, p.say, p.chatModel]
  );

  const executeStudioAction = useCallback(
    (action: any): { targetNodeId?: string } => {
      if (!action) return {};
      const storeState = useStudioStore.getState();
      if (action.type === "add_node") {
        const res = buildAddNodeGraphUpdate(storeState.nodes, storeState.edges, action, p.vi);
        p.setNodesAndEdges(res.nextNodes, res.nextEdges, true);
        p.setSelectedNodeId(res.targetNodeId);
        p.flashNode(res.targetNodeId);
        return { targetNodeId: res.targetNodeId };
      }
      if (action.type === "set_aspect_ratio" && action.aspectRatio) {
        p.setAspectRatio(action.aspectRatio);
        const firstGen = storeState.nodes.find((n) => n.data.nodeType === "image_gen" || n.data.nodeType === "video_gen");
        if (firstGen) p.flashNode(firstGen.id);
        return { targetNodeId: firstGen?.id };
      }
      if (action.type === "open_tab" && action.tab) {
        p.setActiveRightTab(action.tab);
        return {};
      }
      if (action.type === "open_character") {
        p.setActiveRightTab("project");
        p.setSelectedCharacterId(action.characterId || "koala");
        p.setActiveCharacterSection("profile");
        const charNode = storeState.nodes.find((n) => n.data.nodeType === "character");
        if (charNode) p.flashNode(charNode.id);
        return { targetNodeId: charNode?.id };
      }
      return {};
    },
    [p.vi, p.setNodesAndEdges, p.setSelectedNodeId, p.flashNode, p.setAspectRatio, p.setActiveRightTab, p.setSelectedCharacterId, p.setActiveCharacterSection]
  );

  const handleConfirmPendingAction = useCallback(() => {
    const conf = p.pendingConfirmation;
    if (!conf) return;
    if (conf.kind === "publish") {
      p.addScheduledPost(buildScheduledPostFromConfirmation(p.activeProjectId, conf));
      p.setActiveRightTab("publish");
    } else {
      p.addJob(
        buildJobFromConfirmation({
          activeProjectId: p.activeProjectId,
          chatModel: p.chatModel,
          estimatedCostUsd: conf.estimatedCostUsd,
          isMockMode: p.isMockMode,
          episodeLabel: p.activeEpisodeTitle || p.activeEpisodeId,
        })
      );
      const currentNodes = useStudioStore.getState().nodes;
      for (const n of currentNodes) {
        p.updateNodeExecutionState(n.id, { status: "succeeded", isStale: false });
      }
      if (currentNodes[0]) p.flashNode(currentNodes[0].id);
    }
    p.setPendingConfirmation(null);
  }, [p.pendingConfirmation, p.addScheduledPost, p.activeProjectId, p.setActiveRightTab, p.addJob, p.chatModel, p.isMockMode, p.activeEpisodeTitle, p.activeEpisodeId, p.updateNodeExecutionState, p.flashNode, p.setPendingConfirmation]);

  const handleUndoChatAction = useCallback(
    (msgId: string, snapshot?: any) => {
      if (!snapshot) return;
      p.setNodesAndEdges(snapshot.nodes, snapshot.edges, false);
      if (snapshot.aspectRatio) p.setAspectRatio(snapshot.aspectRatio);
      p.markChatActionUndone(p.activeProjectId, msgId);
    },
    [p.setNodesAndEdges, p.setAspectRatio, p.markChatActionUndone, p.activeProjectId]
  );

  const handleViewOnCanvas = useCallback(
    (targetNodeId: string) => {
      p.setSelectedNodeId(targetNodeId);
      p.flashNode(targetNodeId);
    },
    [p.setSelectedNodeId, p.flashNode]
  );

  return { startRunConfirmation, executeDirectMediaGeneration, executeStudioAction, handleConfirmPendingAction, handleUndoChatAction, handleViewOnCanvas };
}
