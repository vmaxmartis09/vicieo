"use client";

import { useShallow } from "zustand/react/shallow";
import { useStudioStore, type StudioStore } from "@/store";

export function useChatStoreBindings() {
  return useStudioStore(
    useShallow((st: StudioStore) => ({
      activeProjectId: st.activeProjectId,
      chatByProject: st.chatByProject,
      addChatMessage: st.addChatMessage,
      markChatActionUndone: st.markChatActionUndone,
      pendingConfirmation: st.pendingConfirmation,
      setPendingConfirmation: st.setPendingConfirmation,
      nodes: st.nodes,
      nodesLength: st.nodes.length,
      aspectRatio: st.aspectRatio,
      setAspectRatio: st.setAspectRatio,
      setNodesAndEdges: st.setNodesAndEdges,
      selectedNodeId: st.selectedNodeId,
      setSelectedNodeId: st.setSelectedNodeId,
      flashNode: st.flashNode,
      setActiveRightTab: st.setActiveRightTab,
      setSelectedCharacterId: st.setSelectedCharacterId,
      setActiveCharacterSection: st.setActiveCharacterSection,
      addScheduledPost: st.addScheduledPost,
      isMockMode: st.isMockMode,
      setIsMockMode: st.setIsMockMode,
      runPolicy: st.runPolicy,
      updateNodeExecutionState: st.updateNodeExecutionState,
      projects: st.projects,
      characters: st.characters,
      setCharacters: st.setCharacters,
      episodes: st.episodes,
      activeEpisodeId: st.activeEpisodeId,
      selectEpisode: st.selectEpisode,
      switchProject: st.switchProject,
      chatContext: st.chatContext,
      setChatContext: st.setChatContext,
      clearProjectChat: st.clearProjectChat,
      addJob: st.addJob,
      isRunningGraph: st.isRunningGraph,
      hasActiveRun: Boolean(st.hasActiveRun || st.isRunningGraph),
    }))
  );
}
