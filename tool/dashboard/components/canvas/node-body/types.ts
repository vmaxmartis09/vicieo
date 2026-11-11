import type { StudioNodeData } from "@/store/slices/canvasSlice";
import type { Character } from "@/lib/repositories";

export interface NodeBodyProps {
  nodeType: string;
  nodeId: string;
  data: StudioNodeData;
  character?: Character;
  /** Cổng vào đã nối (tên cổng) */
  connectedInputs: string[];
  /** Thu gọn vừa phải */
  compact?: boolean;
  /** Card Text: chế độ chỉnh sửa (mở rộng) */
  expanded?: boolean;
  /** Card nhân vật: mở bảng chọn nhân vật bên cạnh */
  onPickCharacter?: () => void;
}

export interface NodeRendererContextProps {
  nodeType: string;
  nodeId: string;
  data: StudioNodeData;
  params: Record<string, any>;
  previewUrl?: string;
  lines: string[];
  connectedInputs: string[];
  compact?: boolean;
  expanded?: boolean;
  updateNodeParams: (id: string, patch: Record<string, any>) => void;
}
