import type { Node, Edge } from "@xyflow/react";
import type {
  NodeDefinition,
  NodeStatus,
  GraphValidationResult,
  PortType,
  RunPolicy,
  RunPresetMode,
} from "@/lib/core-contracts";
import type { Episode } from "@/lib/repositories/types";

export interface NodeGenerationRecord {
  id: string;
  createdAt: string;
  previewUrl?: string;
  previewType?: "image" | "video" | "audio" | "text";
  summaryText?: string;
  costUsd: number;
  seed?: number | null;
  model?: string;
}

export interface StudioNodeData extends Record<string, unknown> {
  nodeType: string;
  label?: string;
  status: NodeStatus;
  isStale?: boolean;
  params: Record<string, any>;
  progressPct?: number;
  /** Dữ liệu cổng vào nhận được / cổng ra sinh ra ở lượt chạy gần nhất, vd. {lines: "30 câu thoại"} */
  inputSummary?: Record<string, string>;
  outputSummary?: Record<string, string>;
  lastLog?: string;
  errorMessage?: string;
  actualCostUsd?: number;
  previewUrl?: string;
  previewType?: "image" | "video" | "audio" | "text";
  summaryText?: string;
  generations: NodeGenerationRecord[];
  pinnedGenerationId?: string | null;
  nextStepHint?: string;
}

export const PORT_TYPE_COLORS: Record<PortType, string> = {
  text: "#737373",
  script: "#737373",
  scenes: "#737373",
  captions: "#737373",
  character: "#525252",
  product: "#7a4e8a",
  image: "#1b6c80",
  "image[]": "#1b6c80",
  video: "#b4531a",
  composition: "#b4531a",
  audio: "#a86f0c",
  policy: "#9f3a38",
  dialogue: "#a86f0c",
  voice_clips: "#a86f0c",
};

/** Cấu hình thu gọn mặc định theo loại node thay cho rẽ nhánh if rời rạc */
export interface NodeCollapseConfig {
  /** Luôn mở, không thu gọn */
  alwaysOpen?: boolean;
  /** Tự thu gọn khi mất tiêu điểm (bỏ chọn) */
  autoCollapseOnBlur?: boolean;
  /** Mặc định thu gọn ban đầu */
  defaultCollapsed?: boolean;
}

/**
 * 5 nhóm node default là MỞ:
 * 1. nhân vật: character, cast
 * 2. kịch bản: script, director_script, script_draft, script_from_blueprint
 * 3. tạo video: video_gen
 * 4. Edit video: template_render, hyperframes_render, compose
 * 5. xuất file: platform_export, export
 *
 * Tất cả các node còn lại default là dạng SHORT (defaultCollapsed: true).
 * Nhấn mở mới mở, độc lập 100%, không bị ảnh hưởng bởi node khác!
 */
export const DEFAULT_OPEN_NODE_TYPES = new Set([
  "character",
  "cast",
  "script",
  "director_script",
  "script_draft",
  "script_from_blueprint",
  "video_gen",
  "template_render",
  "hyperframes_render",
  "compose",
  "platform_export",
  "export",
]);

export function isNodeDefaultOpen(nodeType: string): boolean {
  return DEFAULT_OPEN_NODE_TYPES.has(nodeType);
}

export const NODE_COLLAPSE_CONFIG: Record<string, NodeCollapseConfig> = {
  character: { defaultCollapsed: false },
  cast: { defaultCollapsed: false },
  script: { defaultCollapsed: false },
  director_script: { defaultCollapsed: false },
  video_gen: { defaultCollapsed: false },
  template_render: { defaultCollapsed: false },
  hyperframes_render: { defaultCollapsed: false },
  compose: { defaultCollapsed: false },
  platform_export: { defaultCollapsed: false },
  export: { defaultCollapsed: false },
};

export const DEFAULT_NODE_COLLAPSE_CONFIG: NodeCollapseConfig = {
  alwaysOpen: false,
  autoCollapseOnBlur: false,
  defaultCollapsed: true, // Mọi node khác default là dạng short
};

export function getNodeCollapseConfig(nodeType: string): NodeCollapseConfig {
  const isOpen = isNodeDefaultOpen(nodeType);
  return {
    alwaysOpen: false,
    autoCollapseOnBlur: false, // Bỏ hoàn toàn autoCollapseOnBlur theo yêu cầu 3
    defaultCollapsed: !isOpen, // Node ngoài 5 nhóm trên mặc định là dạng short
  };
}

/** Đồ thị mẫu 9 bước giữ nguyên cấu trúc nhưng bỏ nội dung riêng của Sóc Nhỏ khi dùng cho dự án khác. */

export interface ProjectWorkspace {
  episodes: Episode[];
  activeEpisodeId: string;
  graphsByEpisode: Record<string, { nodes: Node<StudioNodeData>[]; edges: Edge[] }>;
  nodes: Node<StudioNodeData>[];
  edges: Edge[];
}

export interface CanvasSlice {
  selectedNodeId: string | null;
  aspectRatio: "9:16" | "16:9" | "1:1" | "4:5";
  zoomLevel: number;
  isMockMode: boolean;
  runPolicy: RunPolicy;
  setRunPolicy: (policy: Partial<RunPolicy> | RunPolicy) => void;
  setRunPreset: (preset: RunPresetMode) => void;
  episodes: Episode[];
  activeEpisodeId: string;
  nodeCatalog: NodeDefinition[];
  nodeCatalogFromCore: boolean;
  nodes: Node<StudioNodeData>[];
  edges: Edge[];
  graphsByEpisode: Record<
    string,
    { nodes: Node<StudioNodeData>[]; edges: Edge[] }
  >;
  undoStack: Array<{ nodes: Node<StudioNodeData>[]; edges: Edge[] }>;
  redoStack: Array<{ nodes: Node<StudioNodeData>[]; edges: Edge[] }>;
  saveState: "saved" | "saving" | "unsaved";
  isRunningGraph: boolean;
  hasActiveRun?: boolean;
  setHasActiveRun?: (active: boolean) => void;
  validationResult: GraphValidationResult | null;
  /** Mỗi dự án có không gian làm việc riêng (tập + đồ thị); chuyển dự án sẽ tráo cả canvas. */
  projectWorkspaces: Record<string, ProjectWorkspace>;
  switchProject: (projectId: string) => void;
  /** Dự án đã nạp đồ thị từ Core (hoặc xác nhận trống). Chỉ lưu tự động khi đã nạp, để không ghi đè đồ thị đã lưu bằng canvas trống. */
  loadedGraphs: Record<string, boolean>;
  loadProjectGraph: (projectId: string) => Promise<void>;
  setSelectedNodeId: (id: string | null) => void;
  setAspectRatio: (ratio: "9:16" | "16:9" | "1:1" | "4:5") => void;
  setZoomLevel: (zoom: number) => void;
  setIsMockMode: (mock: boolean) => void;
  setEpisodes: (episodes: Episode[]) => void;
  selectEpisode: (episodeId: string) => void;
  createEpisode: (
    title: string,
    opts?: {
      blank?: boolean;
      customGraph?: { nodes: Node<StudioNodeData>[]; edges: Edge[] };
      autoSelect?: boolean;
    }
  ) => Episode;
  setNodeCatalog: (defs: NodeDefinition[], fromCore: boolean) => void;
  /** Nạp lại các loại node tuỳ biến (dùng chung + theo dự án) và gộp vào catalog */
  refreshCustomNodeTypes: (projectId: string) => Promise<void>;
  setNodesAndEdges: (
    nodes: Node<StudioNodeData>[],
    edges: Edge[],
    recordHistory?: boolean
  ) => void;
  pushUndoSnapshot: () => void;
  /** Thay các node Nhân vật cố định bằng 1 node Dàn diễn viên tự theo kịch bản của tập (nối lại dây). */
  convertCharactersToCast: () => boolean;
  /** Phân rã node Dàn diễn viên thành từng node Nhân vật (bật Nói) cho các nhân vật đang có trong tập, nối lại dây. */
  expandCastToCharacters: (castNodeId: string, characterIds: string[]) => boolean;
  /** Chọn kịch bản tập → tự vào chế độ Dàn diễn viên: đổi nhân vật cố định thành Dàn diễn viên, hoặc thêm node Dàn diễn viên nếu flow chưa có. */
  ensureCastForEpisode: (scriptNodeId: string) => boolean;
  undoCanvas: () => void;
  redoCanvas: () => void;
  setAllNodesCollapsed: (collapsed: boolean) => void;
  toggleAllNodesCollapsed: () => void;
  loadCanvasTemplate: (template: "soc_nho_story" | "blank") => void;
  updateNodeParams: (nodeId: string, patch: Record<string, any>) => void;
  updateNodeExecutionState: (
    nodeId: string,
    patch: Partial<StudioNodeData>
  ) => void;
  pinNodeGeneration: (nodeId: string, generationId: string) => void;
  markDownstreamStale: (sourceNodeId: string) => void;
  setSaveState: (state: "saved" | "saving" | "unsaved") => void;
  setIsRunningGraph: (running: boolean) => void;
  setValidationResult: (res: GraphValidationResult | null) => void;
}
