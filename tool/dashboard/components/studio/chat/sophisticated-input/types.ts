import type { ChatModel } from "@/store/slices/chatSlice";

export interface ChatAttachment {
  id: string;
  name: string;
  url: string;
  source: "upload" | "canvas" | "character";
  nodeId?: string;
}

export interface CanvasImageItem {
  id: string;
  nodeId: string;
  name: string;
  url: string;
  kind: "image" | "image_list" | "character" | "product";
}

export type ComposerActionMode = "chat" | "image" | "image_list" | "video";

export type ComposerAspectRatio = "9:16" | "16:9" | "1:1" | "4:5";

export interface SlashCommandItem {
  cmd: string;
  hint: string;
  category?: string;
}

export interface SophisticatedChatInputProps {
  input: string;
  onInputChange: (val: string) => void;
  onSubmit: () => void;
  disabled?: boolean;
  model: ChatModel;
  onModelChange: (m: ChatModel) => void;
  isMockMode: boolean;
  onToggleMockMode?: () => void;
  characters: { id: string; name: string; heightCm: number }[];
  canvasImages?: CanvasImageItem[];
  attachments?: ChatAttachment[];
  onAddAttachment?: (att: ChatAttachment) => void;
  onRemoveAttachment?: (id: string) => void;
  craftTags?: string[];
  onToggleCraftTag?: (tag: string) => void;
  actionMode?: ComposerActionMode;
  onActionModeChange?: (mode: ComposerActionMode) => void;
  aspectRatio?: ComposerAspectRatio;
  onAspectRatioChange?: (ratio: ComposerAspectRatio) => void;
  onMentionSelect: (charId: string, charName: string) => void;
  onSlashSelect: (cmd: string) => void;
  onClearChat: () => void;
  slashCommands: SlashCommandItem[];
  placeholder?: string;
  autoFocus?: boolean;
}
