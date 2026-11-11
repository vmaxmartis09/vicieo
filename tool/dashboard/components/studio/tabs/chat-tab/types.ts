import type { Edge, Node } from "@xyflow/react";
import type { StudioNodeData } from "@/store/slices/canvas/types";
import type { ChatAttachment, ComposerActionMode } from "@/components/studio/chat/SophisticatedChatInput";

export interface ChatSnapshot {
  nodes: Node<StudioNodeData>[];
  edges: Edge[];
  aspectRatio?: "9:16" | "16:9" | "1:1" | "4:5";
}

export type DirectMediaMode = "image" | "image_list" | "video";

export type SayAssistantFn = (
  text: string,
  targetNodeId?: string,
  snapshotBefore?: ChatSnapshot,
  summaryVi?: string,
  summaryEn?: string
) => void;

export interface ChatCanvasActionsParams {
  activeProjectId: string;
  activeEpisodeId: string;
  activeEpisodeTitle?: string;
  aspectRatio: "9:16" | "16:9" | "1:1" | "4:5";
  isMockMode: boolean;
  runPolicy: any;
  pendingConfirmation: any;
  chatModel: string;
  craftTags: string[];
  attachments: ChatAttachment[];
  vi: boolean;
  say: SayAssistantFn;
  setNodesAndEdges: (nodes: any[], edges: any[], pushHistory: boolean) => void;
  setSelectedNodeId: (id: string | null) => void;
  flashNode: (id: string) => void;
  setAspectRatio: (ratio: "9:16" | "16:9" | "1:1" | "4:5") => void;
  setActiveRightTab: (tab: any) => void;
  setSelectedCharacterId: (id: string | null) => void;
  setActiveCharacterSection: (sec: any) => void;
  setPendingConfirmation: (conf: any) => void;
  markChatActionUndone: (projectId: string, msgId: string) => void;
  addScheduledPost: (post: any) => void;
  addJob: (job: any) => void;
  updateNodeExecutionState: (nodeId: string, patch: any) => void;
}

export interface ChatSendParams {
  input: string;
  sending: boolean;
  attachments: ChatAttachment[];
  craftTags: string[];
  actionMode: ComposerActionMode;
  vi: boolean;
  activeProjectId: string;
  activeEpisodeId: string;
  aspectRatio: "9:16" | "16:9" | "1:1" | "4:5";
  isMockMode: boolean;
  chatModel: string;
  chatCharacterId: string | null;
  chatCharacterMode: "update" | "create";
  setInput: (v: string) => void;
  setExpanded: (v: boolean) => void;
  setShowHistory: (v: boolean) => void;
  setSending: (v: boolean) => void;
  addChatMessage: (pid: string, msg: any) => void;
  setPendingConfirmation: (conf: any) => void;
  runContextAction: (raw: string, snap: ChatSnapshot) => Promise<string | null>;
  executeStudioAction: (action: any) => { targetNodeId?: string };
  say: SayAssistantFn;
  routerPush: (path: string) => void;
}
