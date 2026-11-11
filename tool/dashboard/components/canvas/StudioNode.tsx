"use client";

import { enterOnce } from "@/lib/motion";
import React, { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Handle, Position, useReactFlow, useUpdateNodeInternals, type NodeProps, type Node } from "@xyflow/react";
import { useTranslations } from "next-intl";
import { cn } from "@/lib/utils";
import { useStudioStore } from "@/store";
import {
  PORT_TYPE_COLORS,
  getNodeCollapseConfig,
  isNodeDefaultOpen,
  type StudioNodeData,
} from "@/store/slices/canvasSlice";
import {
  FALLBACK_NODE_TYPES,
  estimateNodeCostUsd,
  isPortCompatible,
} from "@/lib/core-api";
import { useCanvasInteraction } from "@/components/canvas/CanvasContext";
import { NodeBody } from "./NodeBody";
import { Reveal } from "./Reveal";
import { applyPortOverride } from "@/lib/node-ports";
import { PortValueInput } from "./PortValueInput";
import { useHubScope } from "@/lib/hub";
import { CharacterCard } from "@/components/canvas/CharacterCard";
import { motion, AnimatePresence } from "framer-motion";
import {
  Play,
  Plus,
  ChevronsDown,
  Sliders,
  ChevronDown,
  ChevronUp,
  Loader2,
  Circle,
  CircleDot,
  CheckCircle2,
  AlertTriangle,
  RotateCcw,
  FileText,
  MessageSquare,
  Images,
  Users,
  Pencil,
  RefreshCw,
  Layers,
  Sparkles,
  Volume2,
  ShieldCheck,
  Film,
  Download,
  Tag,
  Eye,
  EyeOff,
  ExternalLink,
} from "@/components/ui/icons";

export type StudioCardNodeType = Node<StudioNodeData, "studioCard">;
export type StudioNodeType = StudioCardNodeType;
export type DataType = string;
export type { StudioNodeData };

/** Card nguồn (không có đầu vào): có 2 chế độ xem / chỉnh sửa. Xem ai/rules/node-card-style.md */
const SOURCE_TYPES = new Set(["text", "image", "character", "product"]);
/** Card xử lý có 2 chế độ: xem = kết quả; chỉnh sửa = tham số + cổng đầy đủ. */
const PROCESS_TYPES = new Set(["script", "image_gen", "video_gen", "voice_tts", "image_edit", "quality_check", "vision_analyze", "captions", "hyperframes_render", "export", "policy"]);

/** Icon thay cho chữ kiểu dữ liệu ở cổng. */
function portIconOf(portType: string): React.ComponentType<{ className?: string; strokeWidth?: number }> | null {
  if (portType === "text") return FileText;
  if (portType === "image" || portType === "image[]") return Images;
  if (portType === "character") return Users;
  if (portType === "product") return Tag;
  if (portType === "audio") return Volume2;
  if (portType === "video") return Film;
  if (portType === "scenes") return Layers;
  if (portType === "policy") return ShieldCheck;
  if (portType === "dialogue") return MessageSquare;
  if (portType === "voice_clips") return Volume2;
  return null;
}

/** Card mở sẵn chế độ chi tiết khi hiển thị. */
const DETAIL_DEFAULT_TYPES = new Set(["character", "product", "hyperframes_render", "template_render", "export"]);

export const StudioNode = memo(function StudioNode({
  id,
  data,
  selected,
}: NodeProps<StudioCardNodeType>) {
  const t = useTranslations("canvas");
  const nodeCatalog = useStudioStore((s) => s.nodeCatalog);
  const isMockMode = useStudioStore((s) => s.isMockMode);
  const characters = useStudioStore((s) => s.characters);
  const isFlashing = useStudioStore((s) => s.flashingNodeId === id);
  const setSelectedNodeId = useStudioStore((s) => s.setSelectedNodeId);
  const isSelected = useStudioStore((s) => s.selectedNodeId === id);
  const isRunningGraph = useStudioStore((s) => s.isRunningGraph);
  const flashNode = useStudioStore((s) => s.flashNode);
  const { updateNode } = useReactFlow();
  const updateNodeInternals = useUpdateNodeInternals();
  // Nguồn của từng cổng vào: "Node nào · cổng nào" (rút thành chuỗi để chỉ render lại khi nguồn đổi)
  const incomingKey = useStudioStore((s: any) => {
    const rows: string[] = [];
    for (const e of s.edges) {
      if (e.target !== id) continue;
      const src = s.nodes.find((n: any) => n.id === e.source);
      if (!src) continue;
      const base = s.nodeCatalog.find((d: any) => d.typeName === src.data.nodeType);
      const custom = typeof src.data.params?._customName === "string" ? src.data.params._customName : "";
      rows.push([e.targetHandle, e.source, e.sourceHandle, custom || base?.displayName || src.data.nodeType].join("\u0001"));
    }
    return rows.join("\u0002");
  });
  const incoming = useMemo(() => {
    const m = new Map<string, { srcId: string; srcPort: string; srcName: string }[]>();
    if (!incomingKey) return m;
    for (const row of incomingKey.split("\u0002")) {
      const [port, srcId, srcPort, srcName] = row.split("\u0001");
      m.set(port, [...(m.get(port) || []), { srcId, srcPort, srcName }]);
    }
    return m;
  }, [incomingKey]);
  const {
    activeDragPort,
    connectedPorts,
    nodeIssues,
    onOpenQuickAdd,
    onRequestRun,
    onOpenNodeInspector,
    linkMode,
    startLink,
    finishLink,
    stepInfo,
  } = useCanvasInteraction();


  const baseDef =
    nodeCatalog.find((d) => d.typeName === data.nodeType) ||
    FALLBACK_NODE_TYPES.find((d) => d.typeName === data.nodeType) ||
    FALLBACK_NODE_TYPES[0];
  // Cổng/tên ghi đè riêng của node (tuỳ biến trong thanh bên phải)
  const def = useMemo(() => applyPortOverride(baseDef, data.params), [baseDef, data.params]);

  const safeTranslate = (key: string, fallback: string) => {
    try {
      return typeof (t as any).has === "function" && !(t as any).has(key)
        ? fallback
        : t(key as any);
    } catch {
      return fallback;
    }
  };
  const typeLabel = (pt: string) => safeTranslate(`portTypes.${pt}`, pt);

  const categoryLabel = safeTranslate(`categories.${def.category}`, def.category);

  // Compute state key: idle, ready, running, succeeded, failed, stale
  const hasAllRequiredInputs = def.inputPorts
    .filter((p) => p.required)
    .every(
      (p) =>
        connectedPorts.has(`${id}:in:${p.name}`) ||
        (data.params &&
          data.params[p.name] !== undefined &&
          data.params[p.name] !== "")
    );

  const stateKey = data.isStale
    ? "stale"
    : data.status === "idle" && hasAllRequiredInputs
    ? "ready"
    : data.status || "idle";

  const stateLabel = safeTranslate(`status.${stateKey}`, stateKey);
  const p = data.params || {};
  const estCost = useMemo(() => {
    if (isMockMode) return 0.0;
    if (def.typeName === "video_gen") {
      const dur = Number(p.durationSec || 6);
      const variants = Number(p.variants || 1);
      const m = String(p.model || "mock-video").toLowerCase();
      if (m.includes("mock") || m.includes("free") || m.includes("local")) return 0.0;
      const rate = m.includes("ltx")
        ? 0.05
        : m.includes("wan")
        ? 0.05
        : m.includes("hailuo")
        ? 0.08
        : m.includes("kling-v3-std")
        ? 0.084
        : m.includes("kling-v3-pro")
        ? 0.14
        : m.includes("seedance")
        ? 0.12
        : m.includes("veo-3")
        ? 0.12
        : 0.15;
      return Number((dur * rate * variants).toFixed(4));
    }
    return estimateNodeCostUsd(def.typeName, p, isMockMode);
  }, [def.typeName, p, isMockMode]);
  const issues = nodeIssues[id] || [];
  // isFlashing selected from store

  // Compact parameter summary
  const compactParams: string[] = [];
  if (def.typeName === "character") {
    const ch = characters.find((c) => c.id === p.characterId);
    if (ch) {
      compactParams.push(`${ch.name} (${ch.heightCm} cm)`);
      compactParams.push(ch.voiceId);
    } else if (p.characterId) {
      compactParams.push(String(p.characterId));
    }
  } else {
    if (p.model) compactParams.push(String(p.model));
    if (p.aspectRatio) compactParams.push(String(p.aspectRatio));
    if (p.durationSec) compactParams.push(`${p.durationSec}s`);
    if (p.variants && Number(p.variants) > 1) compactParams.push(`x${p.variants}`);
    if (p.resolution) compactParams.push(String(p.resolution));
    if (p.voiceId) compactParams.push(String(p.voiceId));
    if (p.tempo) compactParams.push(`${p.tempo}x`);
    if (p.targetSceneCount) compactParams.push(`${p.targetSceneCount} cảnh`);
    if (p.thresholdScore) compactParams.push(`>= ${p.thresholdScore}`);
    if (p.template) compactParams.push(String(p.template));
    if (p.outputFileName) compactParams.push(String(p.outputFileName));
  }

  const contentPreview =
    typeof p.content === "string" && p.content.trim() ? p.content.trim() : null;

  // Character reference preview if node is character
  const boundChar =
    def.typeName === "character"
      ? characters.find((c) => c.id === p.characterId)
      : undefined;

  const pinnedIdx = data.pinnedGenerationId
    ? data.generations.findIndex((g) => g.id === data.pinnedGenerationId) + 1
    : data.generations.length;


  const CategoryIcon =
    {
      Input: FileText,
      Identity: Users,
      Structure: Layers,
      Intelligence: Sparkles,
      Generation: Sparkles,
      Audio: Volume2,
      Quality: ShieldCheck,
      Composition: Film,
      Output: Download,
    }[def.category as string] || Layers;

  const StatusIcon =
    stateKey === "running"
      ? Loader2
      : stateKey === "succeeded" || stateKey === "completed"
      ? CheckCircle2
      : stateKey === "failed"
      ? AlertTriangle
      : stateKey === "stale"
      ? RotateCcw
      : stateKey === "ready"
      ? CircleDot
      : Circle;
  const statusColor =
    stateKey === "running"
      ? "text-brand"
      : stateKey === "failed"
      ? "text-semantic-danger"
      : stateKey === "succeeded" || stateKey === "completed"
      ? "text-semantic-success"
      : stateKey === "stale"
      ? "text-semantic-warning"
      : stateKey === "ready"
      ? "text-foreground"
      : "text-muted-foreground";


  // Bố cục 3 vùng: NHẬN (trên) · LÀM (giữa) · TRẢ (dưới). Cổng bám theo hàng của vùng tương ứng.
  const HEADER = 54;
  const LABEL = 20;
  const ROW = 24;
  const FOOTER = 26;
  const hasIn = def.inputPorts.length > 0;
  const hasOut = def.outputPorts.length > 0;
  // Chân card chỉ hiện khi có nội dung (chi phí > 0 hoặc có lịch sử tạo); không còn nút "chạy từ đây xuống"
  const hasFooter = !SOURCE_TYPES.has(def.typeName) && (estCost > 0 || data.generations.length > 0);

  const isSourceNode = SOURCE_TYPES.has(def.typeName);
  const isModeNode = isSourceNode || PROCESS_TYPES.has(def.typeName);
  const [charPicker, setCharPicker] = useState(false);
  const [renaming, setRenaming] = useState(false);
  // Tên hiển thị: tên tuỳ đặt > (node nhân vật) tên nhân vật > tên loại node
  const hasCustomName = typeof p._customName === "string" && p._customName.trim() !== "";
  const nodeTitle = hasCustomName
    ? p._customName
    : def.typeName === "character" && boundChar
    ? boundChar.name
    : def.displayName;
  const updateNodeParams = useStudioStore((s) => s.updateNodeParams);
  useEffect(() => {
    if (!selected) setCharPicker(false);
  }, [selected]);
  // Card Text có 2 chế độ: xem (mặc định) / chỉnh sửa (mở rộng). Bấm tiêu đề để đổi.
  // Mỗi card tự quản lý trạng thái của mình (không ảnh hưởng card khác, không tự đóng khi bỏ chọn).
  // Cổng vào đang mở khung chọn nguồn (node khác / tải lên / dự án / nhân vật)
  const [activeInPort, setActiveInPort] = useState<string | null>(null);
  /** Vị trí dọc (px, tính từ đầu thẻ) của ô input đang mở để panel nguồn hiện ngay cạnh */
  const [activeInTop, setActiveInTop] = useState(0);
  // Panel nguồn mở thì cả node nổi lên trên mọi node khác và lắng nghe Esc / click outside để đóng
  useEffect(() => {
    if (activeInPort === null) return;
    updateNode(id, { zIndex: 1000 });

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        setActiveInPort(null);
      }
    };

    const handlePointerDown = (e: PointerEvent) => {
      const target = e.target as HTMLElement;
      if (
        !target.closest("[data-port-popover]") &&
        !target.closest(`[data-in-port="${activeInPort}"]`)
      ) {
        setActiveInPort(null);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("pointerdown", handlePointerDown);

    return () => {
      updateNode(id, { zIndex: 0 });
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("pointerdown", handlePointerDown);
    };
  }, [activeInPort, id, updateNode]);
  // Cấu hình mở/thu gọn theo 3 quy tắc:
  // 1. Những node default là mở: nhân vật, kịch bản, tạo video, Edit video, xuất file
  // 2. Còn lại default là dạng short
  // 3. Nhấn mở mới mở, độc lập 100%, không bị ảnh hưởng bởi node khác
  const isVideoGen = def.typeName === "video_gen";
  const defaultOpen = isNodeDefaultOpen(def.typeName);
  const isCollapsedParam = data.params?._collapsed as boolean | undefined;
  const expanded = isCollapsedParam !== undefined ? !isCollapsedParam : defaultOpen;
  const dragStartPosRef = useRef<{ x: number; y: number } | null>(null);

  const toggleCollapse = useCallback(
    (e?: React.SyntheticEvent) => {
      if (e) {
        e.stopPropagation();
      }
      updateNodeParams(id, { _collapsed: expanded });
      if (!isSelected) {
        setSelectedNodeId(id);
      }
    },
    [id, expanded, isSelected, updateNodeParams, setSelectedNodeId]
  );

  const isBypassed = Boolean(data.params?._bypass);

  // Kiểm tra đủ input: cổng bắt buộc phải được nối hoặc đã có giá trị
  const requiredPorts = def.inputPorts.filter((pt) => pt.required);
  const missingPorts = requiredPorts.filter(
    (pt) =>
      !connectedPorts.has(`${id}:in:${pt.name}`) &&
      (p[pt.name] === undefined || p[pt.name] === "")
  );
  const inputsOk = missingPorts.length === 0;

  // Thông báo lỗi THỐNG NHẤT cho mọi node — luôn hiện ngay dưới card.
  // Ưu tiên: lỗi chạy thật > lỗi kiểm tra đồ thị > thiếu đầu vào (tính trực tiếp từ dây đang nối).
  const missingMessage = inputsOk
    ? null
    : `Thiếu đầu vào bắt buộc: ${missingPorts
        .map((pt) => `${pt.name} (${typeLabel(pt.portType)})`)
        .join(", ")}. Hãy nối node cấp dữ liệu vào.`;
  const staleRunError = data.errorMessage && data.status !== "failed" && inputsOk;
  const errorText =
    (staleRunError ? null : data.errorMessage) ||
    issues[0] ||
    missingMessage ||
    (data.status === "failed" ? "Node chạy thất bại nhưng không có chi tiết lỗi. Xem log chạy để biết thêm." : null);
  const hasError = Boolean(errorText);

  const [guide, setGuide] = useState<"in" | "out" | null>(null);
  const rootRef = useRef<HTMLDivElement>(null);

  // Đo toạ độ dọc chính xác của từng hàng cổng vào/ra theo DOM thật (không lệch khi tiêu đề xuống dòng hoặc nội dung co giãn)
  const [portTops, setPortTops] = useState<{ in: Record<string, number>; out: Record<string, number> }>({
    in: {},
    out: {},
  });

  const measurePorts = useCallback(() => {
    const card = rootRef.current;
    if (!card || !expanded) return;

    const getOffsetCenter = (el: HTMLElement) => {
      let y = el.offsetHeight / 2;
      let curr: HTMLElement | null = el;
      while (curr && curr !== card && card.contains(curr)) {
        y += curr.offsetTop;
        curr = curr.offsetParent as HTMLElement | null;
      }
      return y;
    };

    const nextIn: Record<string, number> = {};
    card.querySelectorAll<HTMLElement>("[data-in-port]").forEach((el) => {
      const name = el.getAttribute("data-in-port");
      if (name && el.offsetHeight > 0) nextIn[name] = getOffsetCenter(el);
    });

    const nextOut: Record<string, number> = {};
    card.querySelectorAll<HTMLElement>("[data-out-port]").forEach((el) => {
      const name = el.getAttribute("data-out-port");
      if (name && el.offsetHeight > 0) nextOut[name] = getOffsetCenter(el);
    });

    setPortTops((prev) => {
      const inKeys = Object.keys(nextIn);
      const outKeys = Object.keys(nextOut);
      const sameIn =
        inKeys.length === Object.keys(prev.in).length &&
        inKeys.every((k) => Math.abs(nextIn[k] - (prev.in[k] ?? -999)) < 0.5);
      const sameOut =
        outKeys.length === Object.keys(prev.out).length &&
        outKeys.every((k) => Math.abs(nextOut[k] - (prev.out[k] ?? -999)) < 0.5);
      if (sameIn && sameOut) return prev;
      return { in: nextIn, out: nextOut };
    });
  }, [expanded]);

  const inTop = (i: number) => {
    const headerH = rootRef.current?.querySelector<HTMLElement>("[data-node-header]")?.offsetHeight || HEADER;
    return headerH + 1 + LABEL + i * ROW + ROW / 2;
  };
  const outBottom = (i: number) =>
    (hasFooter ? FOOTER : 4) + (def.outputPorts.length - 1 - i) * ROW + ROW / 2;

  // Cập nhật toạ độ chân cắm ngay khi node co giãn hoặc đổi nội dung
  useEffect(() => {
    measurePorts();
    updateNodeInternals(id);
    const timer = setTimeout(() => {
      measurePorts();
      updateNodeInternals(id);
    }, 220);
    return () => clearTimeout(timer);
  }, [expanded, isSelected, id, def.inputPorts.length, def.outputPorts.length, hasFooter, nodeTitle, data.summaryText, measurePorts, updateNodeInternals]);

  useEffect(() => {
    const card = rootRef.current;
    if (!card || typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver(() => {
      measurePorts();
      updateNodeInternals(id);
    });
    ro.observe(card);
    return () => ro.disconnect();
  }, [id, measurePorts, updateNodeInternals]);

  useEffect(() => {
    updateNodeInternals(id);
  }, [portTops, id, updateNodeInternals]);

  useEffect(() => {
    if (!guide) return;
    const close = (e: PointerEvent) => {
      if (!rootRef.current?.contains(e.target as globalThis.Node)) setGuide(null);
    };
    document.addEventListener("pointerdown", close);
    return () => document.removeEventListener("pointerdown", close);
  }, [guide]);
  useEffect(() => {
    if (!selected) setGuide(null);
  }, [selected]);

  const linkTargetable =
    linkMode &&
    activeDragPort &&
    activeDragPort.nodeId !== id &&
    (activeDragPort.handleType === "source"
      ? def.inputPorts.some((pt) => isPortCompatible(activeDragPort.portType, pt.portType))
      : def.outputPorts.some((pt) => isPortCompatible(pt.portType, activeDragPort.portType)));

  const portHandle = (
    port: (typeof def.inputPorts)[number],
    index: number,
    side: "in" | "out"
  ) => {
    const isIn = side === "in";
    const portColor = PORT_TYPE_COLORS[port.portType] || "#737373";
    const drag = activeDragPort;
    const other = drag && drag.nodeId !== id && drag.handleType === (isIn ? "source" : "target");
    const compatible =
      other &&
      (isIn
        ? isPortCompatible(drag!.portType, port.portType)
        : isPortCompatible(port.portType, drag!.portType));
    const connected = connectedPorts.has(`${id}:${side}:${port.name}`);
    const missing = isIn && port.required && !connected;
    const measuredTop = isIn ? portTops.in[port.name] : portTops.out[port.name];
    const pos = isIn
      ? !expanded
        ? { top: `${((index + 1) / (def.inputPorts.length + 1)) * 100}%`, transform: "translate(-50%, -50%)" }
        : { top: typeof measuredTop === "number" ? measuredTop : inTop(index), transform: "translate(-50%, -50%)" }
      : !expanded
      ? { top: `${((index + 1) / (def.outputPorts.length + 1)) * 100}%`, bottom: "auto", transform: "translate(50%, -50%)" }
      : typeof measuredTop === "number"
      ? { top: measuredTop, bottom: "auto", transform: "translate(50%, -50%)" }
      : { top: "auto", bottom: outBottom(index), transform: "translate(50%, 50%)" };
    return (
      <Handle
        key={`${side}:${port.name}`}
        id={port.name}
        type={isIn ? "target" : "source"}
        position={isIn ? Position.Left : Position.Right}
        onClick={(e) => {
          if (!linkMode) return;
          e.stopPropagation();
          if (compatible) finishLink(id, port.name, isIn ? "target" : "source");
        }}
        title={`${port.name} · ${typeLabel(port.portType)}${port.required ? " *" : ""}`}
        aria-label={`${port.name} (${typeLabel(port.portType)})`}
        style={{
          ...pos,
          [isIn ? "left" : "right"]: -7,
          width: 15,
          height: 15,
          borderRadius: 4,
          backgroundColor: portIconOf(port.portType) ? "hsl(var(--muted))" : portColor,
          color: portColor,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          border: portIconOf(port.portType) ? (compatible ? "1.5px solid hsl(var(--brand))" : missing ? "1.5px solid hsl(var(--danger))" : "1px solid hsl(var(--border) / 0.8)") : compatible
            ? "2px solid hsl(var(--brand))"
            : missing
            ? "2px solid hsl(var(--danger))"
            : "1.5px solid hsl(var(--card))",
          opacity: other && !compatible ? 0.3 : 1,
          boxShadow: compatible ? `0 0 0 3px ${portColor}44` : missing ? "0 0 0 2px hsl(var(--danger) / 0.25)" : undefined,
          cursor: linkMode && compatible ? "pointer" : undefined,
          transition: "all 150ms",
        }}
      >
        {(() => { const PI = portIconOf(port.portType); return PI ? <PI className="h-2.5 w-2.5 pointer-events-none opacity-80" strokeWidth={1.75} /> : null; })()}
      </Handle>
    );
  };

  const IconBtn = ({
    label,
    onClick,
    children,
    className,
  }: {
    label: string;
    onClick: () => void;
    children: React.ReactNode;
    className?: string;
  }) => (
    <button
      type="button"
      title={label}
      aria-label={label}
      onClick={(e) => {
        e.stopPropagation();
        onClick();
      }}
      className={cn(
        "nodrag flex h-6 w-6 items-center justify-center rounded-ctl text-muted-foreground transition-colors hover:bg-accent hover:text-foreground active:scale-90",
        className
      )}
    >
      {children}
    </button>
  );

  const zoneLabel = (text: string, hint: string) => (
    <div className="flex items-baseline justify-between px-3" style={{ height: LABEL }}>
      <span className="text-caption font-semibold uppercase tracking-wider text-muted-foreground">{text}</span>
      <span className="text-caption text-muted-foreground/70">{hint}</span>
    </div>
  );

  /** Nút + luôn ở giữa mép card: bấm để kéo dây (1 cổng) hoặc chọn cổng (nhiều cổng). Các node nối được sẽ sáng lên. */
  const plusBtn = (side: "in" | "out") => {
    const isIn = side === "in";
    const ports = isIn ? def.inputPorts : def.outputPorts;
    if (ports.length === 0) return null;
    const active = guide === side;
    const begin = (port: (typeof ports)[number]) => {
      setGuide(null);
      startLink({
        nodeId: id,
        portName: port.name,
        portType: port.portType,
        handleType: isIn ? "target" : "source",
      });
    };
    return (
      <>
        <button
          type="button"
          title={isIn ? "Nối đầu vào từ node khác" : "Nối đầu ra tới node khác"}
          aria-label={isIn ? "Nối đầu vào" : "Nối đầu ra"}
          aria-expanded={active}
          onClick={(e) => {
            e.stopPropagation();
            if (ports.length === 1) begin(ports[0]);
            else setGuide(active ? null : side);
          }}
          style={{ top: "50%", [isIn ? "left" : "right"]: -34, transform: "translateY(-50%)" }}
          className={cn(
            "nodrag absolute z-20 flex h-6 w-6 items-center justify-center rounded-full border border-border bg-card text-muted-foreground shadow-sm transition-all hover:scale-110 hover:border-foreground hover:text-foreground",
            active || selected
              ? "opacity-100"
              : "pointer-events-none opacity-0 group-hover/node:pointer-events-auto group-hover/node:opacity-100"
          )}
        >
          <Plus className={cn("h-3 w-3 transition-transform", active && "rotate-45")} />
        </button>
        <AnimatePresence>
          {active && (
            <motion.div
              initial={{ opacity: 0, x: isIn ? 6 : -6, scale: 0.97 }}
              animate={{ opacity: 1, x: 0, scale: 1 }}
              exit={{ opacity: 0, scale: 0.97 }}
              transition={{ duration: 0.15, ease: "easeOut" }}
              style={{ top: "50%", translateY: "-50%", [isIn ? "right" : "left"]: "calc(100% + 40px)" }}
              className="nodrag nowheel absolute z-30 w-56 space-y-1 rounded-panel border border-border bg-card p-2 text-footnote shadow-lg"
            >
              <p className="px-2 pb-1 pt-1 text-caption font-semibold uppercase tracking-wider text-muted-foreground">
                {isIn ? "Chọn đầu vào để nối" : "Chọn đầu ra để nối"}
              </p>
              {ports.map((port) => {
                const connected = connectedPorts.has(`${id}:${side}:${port.name}`);
                return (
                  <div key={port.name} className="flex items-center gap-1">
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation();
                        begin(port);
                      }}
                      className="flex min-w-0 flex-1 items-center gap-2 rounded-ctl px-2 py-2 text-left transition-colors hover:bg-accent"
                    >
                      <span className="h-2 w-2 shrink-0 rounded-full" style={{ backgroundColor: PORT_TYPE_COLORS[port.portType] || "#737373" }} />
                      <span className="truncate font-medium text-foreground">{port.name}</span>
                      <span className="truncate text-muted-foreground">{typeLabel(port.portType)}</span>
                      <span className={cn("ml-auto shrink-0 text-caption", connected ? "text-semantic-success" : isIn && port.required ? "text-semantic-danger" : "text-muted-foreground")}>
                        {connected ? "đã nối" : isIn && port.required ? "cần" : ""}
                      </span>
                    </button>
                    <button
                      type="button"
                      title="Thêm node mới và tự nối"
                      aria-label="Thêm node mới"
                      onClick={(e) => {
                        e.stopPropagation();
                        setGuide(null);
                        onOpenQuickAdd({ anchorNodeId: id, direction: isIn ? "before" : "after", portName: port.name, portType: port.portType });
                      }}
                      className="flex h-6 w-6 shrink-0 items-center justify-center rounded-ctl text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
                    >
                      <Plus className="h-3 w-3" />
                    </button>
                  </div>
                );
              })}
            </motion.div>
          )}
        </AnimatePresence>
      </>
    );
  };

  const outputsDone = data.status === "succeeded" || data.status === "completed";
  const isCharacter = def.typeName === "character";
  // Cổng text: chỉ hiện icon thay cho chữ "text" thừa
  const portType = (port: { name: string; portType: string }) =>
    (() => {
      const PI = portIconOf(port.portType);
      return PI ? <PI className="h-3 w-3 shrink-0" /> : typeLabel(port.portType);
    })();

  const handleNodeAction = (target: HTMLElement, fromFocus = false) => {
    // Bỏ qua khi focus rơi vào chính vỏ card (tránh mở rồi đóng lại ngay khi người dùng bấm header để mở node)
    if (fromFocus && target === rootRef.current) return;
    // Nếu bấm vào header hoặc nút thu gọn ở cạnh dưới thì không can thiệp
    if (target.closest("[data-node-header], [data-collapse-btn]")) return;
    // Bất kỳ action nào khác bên trong body node (input, textarea, port): tự động active và mở rộng
    if (!expanded) {
      updateNodeParams(id, { _collapsed: false });
    }
    if (!isSelected) {
      setSelectedNodeId(id);
    }
  };

  return (
    <motion.div
      ref={rootRef}
      tabIndex={0}
      role="group"
      aria-label={`${def.displayName} (${stateLabel})`}
      initial={enterOnce(`node:${id}`, { opacity: 0 })}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.2, ease: [0.22, 1, 0.36, 1] }}
      onFocusCapture={(e) => {
        handleNodeAction(e.target as HTMLElement, true);
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter" && (e.target as HTMLElement).tagName !== "TEXTAREA") {
          e.preventDefault();
          e.stopPropagation();
          onRequestRun("single_node", id);
        }
      }}
      onClick={(e) => {
        handleNodeAction(e.target as HTMLElement);
      }}
      data-node-card=""
      className={cn(
        "group/node relative rounded-panel border bg-card text-foreground select-none max-w-[calc(100vw-32px)] transition-[width,border-color,box-shadow,opacity] duration-200",
        def.typeName === "template_render" || def.typeName === "hyperframes_render"
          ? "w-[340px]"
          : def.typeName === "video_gen"
          ? expanded
            ? "w-[420px]"
            : "w-[300px]"
          : def.typeName === "image_gen" || def.typeName === "image_edit"
          ? expanded
            ? "w-[360px]"
            : "w-[300px]"
          : expanded
          ? "w-[320px]"
          : "w-[300px]",
        isBypassed && "opacity-50 grayscale-[0.2]",
        stateKey === "stale" && !isBypassed && "opacity-80",
        stateKey === "running" && "node-running",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        linkTargetable && "border-brand ring-4 ring-brand/20",
        linkMode && !linkTargetable && activeDragPort?.nodeId !== id && "opacity-50",
        isFlashing
          ? "border-brand bg-brand/5"
          : hasError
          ? "border-semantic-danger"
          : isSelected
          ? "border-foreground z-20"
          : !linkTargetable && "border-border hover:border-foreground/40"
      )}
      style={{ boxShadow: "none" }}
    >
      {activeInPort !== null && (() => {
        const ap = def.inputPorts.find((pt) => pt.name === activeInPort);
        if (!ap) return null;
        // Ở cấp thẻ (không nằm trong vùng overflow-hidden) để tràn ra ngoài; top khớp hàng input đã bấm
        return (
          <div
            data-port-popover=""
            className="nodrag nowheel absolute left-full z-50 w-[min(340px,calc(100vw-32px))] cursor-default space-y-2 rounded-r-panel border border-l-0 border-border bg-card p-3 text-card-foreground shadow-lg"
            style={{ top: activeInTop }}
            onClick={(e) => e.stopPropagation()}
            onDoubleClick={(e) => e.stopPropagation()}
          >
            {/* Phủ lên đường viền phải của thẻ đúng ở hàng đang chọn: hàng input và panel liền một khối, không có vết */}
            <span aria-hidden className="absolute -left-px top-px h-[24px] w-[2px] bg-card" />
            <div className="flex items-center justify-between text-caption text-muted-foreground">
              <span>
                Nguồn cho <span className="font-mono text-foreground">{ap.name}</span>
              </span>
              <button
                type="button"
                aria-label="Đóng"
                title="Đóng"
                onClick={() => setActiveInPort(null)}
                className="flex h-6 w-6 items-center justify-center rounded-ctl text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
              >
                ✕
              </button>
            </div>
            <PortValueInput nodeId={id} port={ap} />
          </div>
        );
      })()}
      {stateKey === "running" && (
        <div className="pointer-events-none absolute inset-x-2 bottom-0 h-0.5 overflow-hidden rounded-full bg-muted">
          {typeof data.progressPct === "number" ? (
            <motion.div
              className="h-full bg-brand"
              initial={false}
              animate={{ width: `${Math.max(4, Math.min(100, data.progressPct))}%` }}
              transition={{ duration: 0.25, ease: "easeOut" }}
              style={{ willChange: "width" }}
            />
          ) : (
            <div className="h-full w-2/5 rounded-full bg-brand animate-node-indeterminate" />
          )}
        </div>
      )}
      {def.inputPorts.map((pt, i) => portHandle(pt, i, "in"))}
      {def.outputPorts.map((pt, i) => portHandle(pt, i, "out"))}
      {plusBtn("in")}
      {plusBtn("out")}

      {/* Đầu card: hàng 1 = thanh công cụ gọn (bước · loại · trạng thái · nút), hàng 2 = tiêu đề đầy đủ. Header có thể click để đóng/mở mượt mà */}
      <div
        data-node-header=""
        onPointerDown={(e) => {
          dragStartPosRef.current = { x: e.clientX, y: e.clientY };
        }}
        onClick={(e) => {
          const target = e.target as HTMLElement;
          if (target.closest("button, input, textarea, [data-port-popover]")) return;
          if (dragStartPosRef.current) {
            const dx = Math.abs(e.clientX - dragStartPosRef.current.x);
            const dy = Math.abs(e.clientY - dragStartPosRef.current.y);
            if (dx > 6 || dy > 6) return;
          }
          toggleCollapse(e);
        }}
        className="flex flex-col justify-center gap-1 border-b border-border/60 px-3 py-1 cursor-pointer select-none transition-colors hover:bg-muted/30 active:bg-muted/50 rounded-t-panel"
        style={{ minHeight: HEADER }}
        title={expanded ? "Nhấn vào header để thu gọn node" : "Nhấn vào header để mở rộng node"}
      >
        <div className="flex items-center gap-2">
          {(isRunningGraph || stateKey === "running") && stepInfo[id] && (
            <span
              title={stepInfo[id].main ? `Bước ${stepInfo[id].step} · trục chính` : `Bước ${stepInfo[id].step} · hỗ trợ`}
              className={cn(
                "flex h-4 w-4 shrink-0 items-center justify-center rounded-full text-caption font-semibold tnum",
                stepInfo[id].main ? "bg-foreground text-background" : "border border-border text-muted-foreground"
              )}
            >
              {stepInfo[id].step}
            </span>
          )}
          <span title={categoryLabel} className="flex min-w-0 items-center gap-1 text-muted-foreground">
            <CategoryIcon className="h-3 w-3 shrink-0" />
            <span className="truncate text-caption font-medium uppercase tracking-wider">{categoryLabel}</span>
          </span>
          {isBypassed && (
            <span
              title="Node này đang được thiết lập bỏ qua khi chạy"
              className="inline-flex items-center rounded-xs border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-caption font-semibold text-amber-600 dark:text-amber-400 uppercase tracking-wider shrink-0"
            >
              Bỏ qua
            </span>
          )}
          <span className="flex-1" />
          {(def.typeName === "template_render" || def.typeName === "hyperframes_render") && (
            <a
              href={`http://localhost:3002/#project/${(p.template_id as string) || 'koc-review-v1'}`}
              target="_blank"
              rel="noopener noreferrer"
              title="Mở tab mới xem trực tiếp tiến trình editing HyperFrames Studio"
              className="nodrag flex h-6 w-6 items-center justify-center rounded-ctl text-primary transition-colors hover:bg-accent hover:text-foreground shrink-0"
              onClick={(e) => e.stopPropagation()}
            >
              <ExternalLink className="h-3 w-3" />
            </a>
          )}
          {stateKey !== "ready" && stateKey !== "idle" && (
            <span title={`${stateLabel}${stateKey === "running" && typeof data.progressPct === "number" ? ` ${Math.round(data.progressPct)}%` : ""}`} className={cn("flex h-5 w-5 items-center justify-center", statusColor)}>
              <StatusIcon className={cn("h-3 w-3", stateKey === "running" && "animate-spin")} />
            </span>
          )}
          <IconBtn label={t("editInNodeTab")} onClick={() => { setSelectedNodeId(id); onOpenNodeInspector(id); }}>
            <Sliders className="h-3 w-3" />
          </IconBtn>
          <IconBtn label={t("runNode")} onClick={() => onRequestRun("single_node", id)}>
            <Play className="h-3 w-3" />
          </IconBtn>
        </div>
        <div className="group/title relative min-w-0">
          {renaming ? (
            <input
              autoFocus
              defaultValue={nodeTitle}
              onClick={(e) => e.stopPropagation()}
              onDoubleClick={(e) => e.stopPropagation()}
              onBlur={(e) => {
                updateNodeParams(id, { _customName: e.target.value.trim() });
                setRenaming(false);
              }}
              onKeyDown={(e) => {
                e.stopPropagation();
                if (e.key === "Enter") (e.target as HTMLInputElement).blur();
                if (e.key === "Escape") setRenaming(false);
              }}
              className="nodrag w-full rounded-ctl border border-border bg-card px-2 py-1 text-footnote font-semibold outline-none focus:border-foreground"
            />
          ) : (
            <div
              className="flex items-start justify-between gap-2"
              title={`${nodeTitle}${isBypassed ? " (Bỏ qua)" : ""}`}
            >
              <span className="min-w-0 line-clamp-2 break-words text-footnote font-semibold leading-snug tracking-tight text-foreground">
                {nodeTitle}
              </span>
              <button
                type="button"
                title="Sửa tên node"
                aria-label="Sửa tên node"
                onClick={(e) => {
                  e.stopPropagation();
                  setRenaming(true);
                }}
                className="nodrag -mr-1 flex h-6 w-6 shrink-0 items-center justify-center rounded-ctl text-muted-foreground opacity-0 transition-opacity hover:bg-accent hover:text-foreground group-hover/title:opacity-100 focus-visible:opacity-100"
              >
                <Pencil className="h-2.5 w-2.5" />
              </button>
            </div>
          )}
        </div>
      </div>

      {/* 1. NHẬN — chỉ hiện khi node có đầu vào */}
      {hasIn && (
        <div className="border-t border-border/70">
         <Reveal show={expanded}>
          <div className="flex items-center justify-between px-3" style={{ height: LABEL }}>
            <span className="text-caption font-semibold uppercase tracking-wider text-muted-foreground">Nhận</span>
            {requiredPorts.length > 0 ? (
              <span className={cn("flex items-center gap-1 text-caption font-medium", inputsOk ? "text-semantic-success" : "text-semantic-danger")}>
                {inputsOk ? <CheckCircle2 className="h-3 w-3" /> : <AlertTriangle className="h-3 w-3" />}
                {inputsOk ? "Đủ đầu vào" : `Thiếu ${missingPorts.length}/${requiredPorts.length}`}
              </span>
            ) : (
              <span className="text-caption text-muted-foreground/70">Tuỳ chọn</span>
            )}
          </div>
          {def.inputPorts.map((port) => {
            const connected = connectedPorts.has(`${id}:in:${port.name}`);
            const manualVal = p[port.name] ?? (port.name === "references" ? (p.references ?? p.referenceImages) : undefined);
            const filled = connected || (manualVal !== undefined && manualVal !== "" && (!Array.isArray(manualVal) || manualVal.length > 0));
            const srcs = incoming.get(port.name) || [];
            
            const manualSrc = srcs.length === 0 && manualVal != null ? (() => {
              if (typeof manualVal === "string") {
                const trimmed = manualVal.trim();
                if (trimmed.startsWith("{") && trimmed.endsWith("}")) {
                  const inner = trimmed.slice(1, -1).trim();
                  const dotIdx = inner.lastIndexOf(".");
                  return dotIdx > 0 ? inner.slice(0, dotIdx) : inner;
                }
                if (trimmed.includes("/") || trimmed.includes(".")) {
                  const parts = trimmed.split("/");
                  return parts[parts.length - 1] || trimmed;
                }
                return trimmed.length > 15 ? `${trimmed.slice(0, 15)}…` : trimmed;
              }
              if (typeof manualVal === "object") {
                if (Array.isArray(manualVal)) {
                  return manualVal.length > 0 ? `${manualVal.length} ảnh` : null;
                }
                if (manualVal.$ref === "script" && manualVal.label) {
                  return manualVal.label;
                }
                if (manualVal.name) return String(manualVal.name);
              }
              return null;
            })() : null;

            return (
              <div
                key={port.name}
                data-in-port={port.name}
                role="button"
                tabIndex={-1}
                title="Chọn nguồn cho đầu vào này"
                onClick={(e) => {
                  e.stopPropagation();
                  const row = e.currentTarget as HTMLElement;
                  const card = row.closest("[data-node-card]") as HTMLElement | null;
                  if (card) {
                    // offsetTop không bị ảnh hưởng bởi zoom/transform của canvas (getBoundingClientRect thì có)
                    let y = 0;
                    let el: HTMLElement | null = row;
                    while (el && el !== card && card.contains(el)) {
                      y += el.offsetTop;
                      el = el.offsetParent as HTMLElement | null;
                    }
                    setActiveInTop(y);
                  }
                  setActiveInPort((cur) => (cur === port.name ? null : port.name));
                }}
                className={cn(
                  "nodrag flex cursor-pointer items-center gap-2 px-3 text-footnote transition-colors hover:bg-accent/60",
                  activeInPort === port.name && "bg-card shadow-[inset_3px_0_0_hsl(var(--foreground))]"
                )}
                style={{ height: ROW }}
              >
                <span
                  className={cn(
                    "shrink-0 max-w-[100px] truncate font-medium",
                    filled ? "text-foreground" : port.required ? "text-semantic-danger" : "text-muted-foreground"
                  )}
                  title={port.name}
                >
                  {port.name}
                </span>
                {!portIconOf(port.portType) && port.name !== port.portType && (
                  <span className="flex shrink-0 items-center text-caption text-muted-foreground/70">· {portType(port)}</span>
                )}
                {srcs.length > 0 ? (
                  <div className="flex min-w-0 flex-1 items-center gap-1 overflow-hidden">
                    {srcs.slice(0, srcs.length <= 2 ? 2 : 1).map((src) => (
                      <button
                        key={`${src.srcId}:${src.srcPort}`}
                        type="button"
                        title={`Nguồn: ${src.srcName} · ${src.srcPort} — bấm để làm nổi node nguồn`}
                        onClick={(e) => {
                          e.stopPropagation();
                          setSelectedNodeId(src.srcId);
                          flashNode(src.srcId);
                        }}
                        className="nodrag min-w-0 max-w-[96px] truncate rounded-xs bg-accent/70 px-1 font-mono text-caption text-muted-foreground transition-colors hover:bg-brand/15 hover:text-foreground"
                      >
                        ← {src.srcName}
                      </button>
                    ))}
                    {srcs.length > 2 && (
                      <span
                        title={srcs.map((s) => `← ${s.srcName} · ${s.srcPort}`).join("\n")}
                        className="shrink-0 rounded-xs bg-accent px-1 font-mono text-caption text-muted-foreground"
                      >
                        +{srcs.length - 1}
                      </span>
                    )}
                  </div>
                ) : manualSrc ? (
                  <div className="flex min-w-0 flex-1 items-center gap-1 overflow-hidden">
                    <span
                      title={`Nguồn giá trị: ${manualSrc}`}
                      className="nodrag min-w-0 max-w-[110px] truncate rounded-xs border border-brand/30 bg-brand/10 px-2 py-1 font-mono text-caption text-brand font-medium"
                    >
                      ← {manualSrc}
                    </span>
                  </div>
                ) : null}
                <span className="ml-auto flex shrink-0 items-center gap-1 text-caption">
                  {data.inputSummary?.[port.name] && (
                    <span className="max-w-[80px] truncate font-medium text-foreground" title={`Dữ liệu nhận: ${data.inputSummary[port.name]}`}>
                      {data.inputSummary[port.name]}
                    </span>
                  )}
                  {filled ? (
                    <CheckCircle2 className="h-3 w-3 shrink-0 text-semantic-success" />
                  ) : port.required ? (
                    <span className="text-semantic-danger font-medium">thiếu</span>
                  ) : (
                    <span className="text-muted-foreground/60">tuỳ chọn</span>
                  )}
                </span>
              </div>
            );
          })}
         </Reveal>
        </div>
      )}

      {/* 2. LÀM (node nguồn không có đầu vào thì gọi là "Cung cấp") - luôn hiện body kể cả khi thu gọn */}
      <div className="border-t border-border/70 w-full min-w-0">
        <Reveal show={expanded}>
          <div className="flex items-center justify-between px-3" style={{ height: LABEL }}>
            <span className="text-caption font-semibold uppercase tracking-wider text-muted-foreground">{hasIn ? "Làm" : "Cung cấp"}</span>
            {hasIn && <span className="text-caption text-muted-foreground/70">{categoryLabel}</span>}
          </div>
        </Reveal>
        <div className={cn("space-y-2 px-3 pb-2 w-full min-w-0", !expanded ? "pt-1 pb-2" : isModeNode && "pt-2")}>
          <NodeBody
            nodeType={def.typeName}
            nodeId={id}
            data={data}
            character={boundChar}
            compact={!expanded}
            expanded={expanded}
            onPickCharacter={() => setCharPicker((v) => !v)}
            connectedInputs={def.inputPorts.filter((pt) => connectedPorts.has(`${id}:in:${pt.name}`)).map((pt) => pt.name)}
          />
        </div>
      </div>

      {/* 3. TRẢ — chỉ hiện khi node có đầu ra */}
      {hasOut && (
        <div className="border-t border-border/70">
         <Reveal show={expanded}>
          <div className={cn(!hasFooter && "pb-1")}>
          <div className="flex items-center justify-between px-3" style={{ height: LABEL }}>
            <span className="text-caption font-semibold uppercase tracking-wider text-muted-foreground">Trả</span>
            {outputsDone && (
              <span className="flex items-center gap-1 text-caption font-medium text-semantic-success">
                <CheckCircle2 className="h-3 w-3" /> Đã có kết quả
              </span>
            )}
          </div>
          {def.outputPorts.map((port) => {
            const connected = connectedPorts.has(`${id}:out:${port.name}`);
            return (
              <div
                key={port.name}
                data-out-port={port.name}
                className="flex items-center justify-end gap-2 px-3 text-footnote"
                style={{ height: ROW }}
              >
                {data.outputSummary?.[port.name] && (
                  <span className="max-w-[120px] truncate text-caption font-medium text-foreground" title={`Đã sinh ra: ${data.outputSummary[port.name]}`}>
                    {data.outputSummary[port.name]}
                  </span>
                )}
                {!portIconOf(port.portType) && port.name !== port.portType && (
                  <span className="flex shrink-0 items-center text-caption text-muted-foreground/70">{portType(port)}</span>
                )}
                <span className={cn("truncate font-medium", connected ? "text-foreground" : "text-muted-foreground")} title={port.name}>
                  {port.name}
                </span>
              </div>
            );
          })}
          </div>
         </Reveal>
        </div>
      )}

      {data.summaryText && (
        <div className="px-3 pb-1">
          <p className="line-clamp-2 rounded-ctl border border-border/70 bg-muted/20 px-2 py-1 text-footnote text-foreground">{data.summaryText}</p>
        </div>
      )}

      {/* Card Text không cần chân card (chi phí 0, không chạy tiếp từ đây) */}
      <Reveal show={hasFooter && expanded}>
      <div className="flex items-center justify-between px-3 text-xs text-muted-foreground tnum" style={{ height: FOOTER }}>
        <span className="truncate">
          {data.generations.length > 0 ? t("pinnedGen", { index: Math.max(1, pinnedIdx), total: data.generations.length }) : ""}
        </span>
        <span className="flex shrink-0 items-center gap-1">
          {estCost > 0 && <span title={t("estimatedCost", { cost: estCost.toFixed(2) })}>${estCost.toFixed(2)}</span>}
        </span>
      </div>
      </Reveal>

      {/* Bảng chọn nhân vật: tab card bên cạnh, dạng avatar + tạo mới */}
      {isCharacter && charPicker && (
        <div
          onClick={(e) => e.stopPropagation()}
          style={{ left: "calc(100% + 8px)", top: 0 }}
          className="nodrag nowheel absolute z-30 w-[min(380px,calc(100vw-32px))] rounded-panel border border-border bg-card p-2 shadow-lg"
        >
          <p className="px-1 pb-2 text-caption font-semibold uppercase tracking-wider text-muted-foreground">Đổi nhân vật</p>
          <div className="grid max-h-[460px] grid-cols-4 gap-2 overflow-y-auto scroll-quiet">
            {characters.map((c) => {
              const active = c.id === p.characterId;
              return (
                <button
                  key={c.id}
                  type="button"
                  title={`${c.name} · ${c.heightCm}cm`}
                  onClick={() => {
                    updateNodeParams(id, { characterId: c.id });
                    setCharPicker(false);
                  }}
                  className={cn(
                    "flex flex-col items-center gap-1 rounded-card border p-2 text-center transition-colors hover:bg-accent",
                    active ? "border-foreground bg-accent" : "border-border"
                  )}
                >
                  {c.referenceImages?.[0] ? (
                    // eslint-disable-next-line @next/next/no-img-element
                    <img decoding="async" loading="lazy" src={c.referenceImages[0]} alt="" className="h-16 w-16 rounded-full border border-border object-cover object-top" />
                  ) : (
                    <span className="flex h-16 w-16 items-center justify-center rounded-full bg-muted text-base font-semibold text-muted-foreground">
                      {c.name.charAt(0)}
                    </span>
                  )}
                  <span className="w-full truncate text-caption text-foreground">{c.name}</span>
                </button>
              );
            })}
            <button
              type="button"
              title="Tạo nhân vật mới"
              aria-label="Tạo nhân vật mới"
              onClick={() => {
                setCharPicker(false);
                useHubScope.getState().openTab("characters", "create-character");
              }}
              className="flex flex-col items-center justify-center gap-1 rounded-card border border-dashed border-border p-2 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
            >
              <span className="flex h-16 w-16 items-center justify-center rounded-full border border-dashed border-border"><Plus className="h-5 w-5" /></span>
              <span className="text-caption">Tạo mới</span>
            </button>
          </div>
        </div>
      )}

      {/* Thông báo lỗi thống nhất: luôn nằm ngay dưới card, mọi loại node */}
      {errorText && (
        <div
          role="alert"
          data-node-error
          className="absolute inset-x-0 top-full z-10 mt-2 flex items-start gap-2 rounded-card border border-semantic-danger/50 bg-card px-3 py-2 text-footnote leading-snug text-semantic-danger shadow-sm"
        >
          <AlertTriangle className="mt-1 h-3.5 w-3.5 shrink-0" />
          <span className="min-w-0 break-words">{errorText}</span>
        </div>
      )}
      {/* Nút thu gọn / mở rộng ở cạnh dưới (tương tự nút add) */}
      <button
        data-collapse-btn=""
        type="button"
        title={expanded ? "Thu gọn node (dạng short)" : "Mở rộng node"}
        aria-label={expanded ? "Thu gọn node" : "Mở rộng node"}
        onClick={toggleCollapse}
        style={{ left: "50%", bottom: -7, transform: "translateX(-50%)" }}
        className={cn(
          "nodrag absolute z-20 flex h-3.5 w-7 items-center justify-center rounded-xs border border-border/80 bg-card/95 text-muted-foreground/70 shadow-sm transition-all hover:border-foreground hover:text-foreground cursor-pointer hover:bg-card",
          isSelected || !expanded
            ? "opacity-80 hover:opacity-100"
            : "pointer-events-none opacity-0 group-hover/node:pointer-events-auto group-hover/node:opacity-100"
        )}
      >
        {expanded ? (
          <ChevronUp className="h-2.5 w-2.5" />
        ) : (
          <ChevronDown className="h-2.5 w-2.5" />
        )}
      </button>
    </motion.div>
  );
});
