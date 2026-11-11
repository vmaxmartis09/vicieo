"use client";

import React, { useState } from "react";
import { useStudioStore } from "@/store";
import { Reveal } from "../../Reveal";
import { GenericParams, PlatformsView, SlotMappingView } from "../../GenericParams";
import { CanvasChip } from "../../CharacterHero";
import { cn } from "@/lib/utils";
import {
  Bot,
  Check,
  ChevronDown,
  ChevronUp,
  Clapperboard,
  Clock,
  Copy,
  ExternalLink,
  Film,
  Images,
  Music,
  PanelRightOpen,
  Pencil,
  Plus,
  RotateCcw,
  Scissors,
  ScrollText,
  Sliders,
  Smartphone,
  Sparkles,
  Trash2,
  Video,
  Volume2,
} from "@/components/ui/icons";
import { Empty } from "../components/Empty";
import { ImagePreview } from "../components/ImagePreview";
import type { NodeRendererContextProps } from "../types";

const TEMPLATES_INFO_MAP: Record<
  string,
  {
    name: string;
    categoryBadge: string;
    description: string;
    aspect: string;
    duration: string;
    audio: string;
    captions: string;
    scenes: { title: string; dur: string; desc: string }[];
  }
> = {
  "koc-review-v1": {
    name: "KOC Review Sản Phẩm – 5 Sao",
    categoryBadge: "Review KOC",
    description: "Đánh giá chân thực đa khung hình, phân tích ưu nhược điểm & chấm điểm 5 sao",
    aspect: "9:16",
    duration: "auto",
    audio: "chill_lofi",
    captions: "anchor",
    scenes: [
      { title: "1. Intro", dur: "3s", desc: "Giới thiệu KOC & Sản phẩm" },
      { title: "2. Ưu điểm", dur: "15s", desc: "Nêu chi tiết ưu điểm nổi bật" },
      { title: "3. Lưu ý", dur: "8s", desc: "Lưu ý & Nhược điểm chân thực" },
      { title: "4. Chấm sao", dur: "4s", desc: "Tổng kết rating & Lời khuyên" },
    ],
  },
  "tiktok-sales-v1": {
    name: "Bán hàng TikTok – Hook 3s",
    categoryBadge: "Sales & Affiliate",
    description: "Giật tít hook 3 giây đầu, phô diễn tính năng, giá sốc & kêu gọi giỏ hàng",
    aspect: "9:16",
    duration: "auto",
    audio: "trending_pop",
    captions: "bold_popup",
    scenes: [
      { title: "1. Hook", dur: "3s", desc: "Giật tít gây tò mò / Đau đớn" },
      { title: "2. Showcase", dur: "15s", desc: "Trình diễn tính năng cốt lõi" },
      { title: "3. Deal hời", dur: "5s", desc: "Giá sốc / Ưu đãi hôm nay" },
      { title: "4. Kêu gọi", dur: "3s", desc: "Bấm giỏ hàng góc trái" },
    ],
  },
  "shorts-fact-v1": {
    name: "Kiến Thức Thú Vị – Kinetic",
    categoryBadge: "Fact / Shorts",
    description: "Câu hỏi giật gân, hiệu ứng count-up số liệu và sự thật bất ngờ",
    aspect: "9:16",
    duration: "auto",
    audio: "dramatic",
    captions: "kinetic",
    scenes: [
      { title: "1. Câu hỏi", dur: "3s", desc: "BẠN CÓ BIẾT? (Hook)" },
      { title: "2. Con số", dur: "4s", desc: "Count-up ấn tượng" },
      { title: "3. Giải thích", dur: "15s", desc: "Sự thật bất ngờ ít ai biết" },
      { title: "4. Kêu gọi", dur: "3s", desc: "Follow để xem tiếp" },
    ],
  },
  "unboxing-v1": {
    name: "Đập Hộp & Trải Nghiệm Mở Hộp",
    categoryBadge: "Unboxing",
    description: "Mở kiện bưu phẩm nguyên seal, cận cảnh phụ kiện & phán quyết đáng tiền",
    aspect: "9:16",
    duration: "auto",
    audio: "acoustic",
    captions: "anchor",
    scenes: [
      { title: "1. Khui hộp", dur: "3s", desc: "Khui kiện hàng nguyên seal" },
      { title: "2. Phụ kiện", dur: "10s", desc: "Chi tiết từng món bên trong" },
      { title: "3. Trên tay", dur: "12s", desc: "Cảm nhận chất liệu thực tế" },
      { title: "4. Đánh giá", dur: "4s", desc: "Đáng tiền hay không đáng tiền" },
    ],
  },
  "story-cartoon-v1": {
    name: "Hoạt Hình Thiếu Nhi Sóc Nhỏ",
    categoryBadge: "Hoạt hình thiếu nhi",
    description: "Phim hoạt hình nhân vật Sóc Nhỏ, bài học đạo đức & kỹ năng sống",
    aspect: "16:9",
    duration: "auto",
    audio: "acoustic",
    captions: "karaoke",
    scenes: [
      { title: "1. Bối cảnh", dur: "5s", desc: "Khu rừng Sóc Nhỏ & Bạn bè" },
      { title: "2. Biến cố", dur: "15s", desc: "Gặp tình huống thử thách" },
      { title: "3. Giải quyết", dur: "15s", desc: "Cùng vượt qua khó khăn" },
      { title: "4. Bài học", dur: "5s", desc: "Thông điệp ý nghĩa rút ra" },
    ],
  },
  "affiliate-compare-v1": {
    name: "So Sánh Đối Đầu 2 Sản Phẩm",
    categoryBadge: "So sánh / Review",
    description: "Đặt 2 sản phẩm lên bàn cân so sánh thông số, trải nghiệm & chốt deal",
    aspect: "9:16",
    duration: "auto",
    audio: "trending_pop",
    captions: "bold_popup",
    scenes: [
      { title: "1. So găng", dur: "3s", desc: "Đặt 2 sản phẩm cạnh nhau" },
      { title: "2. So thông số", dur: "12s", desc: "Bảng so sánh tính năng" },
      { title: "3. Ưu nhược", dur: "10s", desc: "Ai hơn ai ở điểm nào" },
      { title: "4. Lời khuyên", dur: "4s", desc: "Nên mua cái nào cho ai" },
    ],
  },
};

interface TemplateVideoEditorProps {
  nodeId: string;
  templateId?: string;
  params: Record<string, any>;
  updateNodeParams: (nodeId: string, patch: Record<string, any>) => void;
}

function TemplateVideoEditor({
  nodeId,
  templateId,
  params,
  updateNodeParams,
}: TemplateVideoEditorProps) {
  const currentId = templateId || "koc-review-v1";
  const info = TEMPLATES_INFO_MAP[currentId] || {
    name: currentId,
    categoryBadge: "Template",
    aspect: "9:16",
    duration: "auto",
    audio: "chill_lofi",
    captions: "anchor",
    scenes: [
      { title: "1. Mở đầu", dur: "5s", desc: "Mở đầu" },
      { title: "2. Nội dung", dur: "15s", desc: "Nội dung" },
    ],
  };

  const aspect = params.aspect || (info.aspect.includes("16:9") ? "16:9" : "9:16");
  const duration = params.duration || "auto";
  const bgm = params.bgm || "chill_lofi";
  const captions = params.captions || "anchor";
  const scenes: { title: string; dur?: string; desc?: string }[] =
    Array.isArray(params.scenes) && params.scenes.length > 0 ? params.scenes : info.scenes;

  const [copied, setCopied] = useState(false);
  const setSelectedNodeId = useStudioStore((s) => s.setSelectedNodeId);
  const setActiveRightTab = useStudioStore((s) => s.setActiveRightTab);
  const setRightPanelCollapsed = useStudioStore((s) => s.setRightPanelCollapsed);

  // Tính tổng thời lượng timeline
  const totalSeconds = scenes.reduce((acc, s) => {
    const n = parseInt(s.dur?.replace(/\D/g, "") || "5", 10) || 5;
    return acc + n;
  }, 0);

  // Sinh câu prompt súc tích
  const buildPrompt = () => {
    const durStr = duration === "auto" ? `${totalSeconds}s (Auto)` : `${duration}s`;
    const scenesSummary = scenes.map((s) => `${s.title}${s.dur ? `(${s.dur})` : ""}`).join(" > ");
    return `HyperFrames [${currentId}]: ${aspect} | ${durStr} | BGM:${bgm} | Sub:${captions}${scenesSummary ? ` | Flow: ${scenesSummary}` : ""}`;
  };

  const activePrompt = params.edit_prompt || buildPrompt();

  // Lệnh CLI chung (Universal & tương thích cả Antigravity lẫn Claude)
  const universalCliCommand = `agy run-node --id ${nodeId} --prompt "${activePrompt}"`;

  const handleOpenStudio = (e: React.MouseEvent) => {
    e.stopPropagation();
    setSelectedNodeId(nodeId);
    setActiveRightTab("node");
    setRightPanelCollapsed(false);
  };

  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(universalCliCommand);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="space-y-1.5">
      {/* 4-Line Compact Interactive Prompt Input Box */}
      <div
        role="button"
        tabIndex={0}
        onClick={handleOpenStudio}
        title="Bấm để mở Studio Panel đầy đủ bên phải (Preview, Timeline, Audio, Slots)"
        className="nodrag group/box rounded-ctl border border-border/80 bg-background/85 hover:bg-background hover:border-brand/70 p-2 shadow-2xs transition-all cursor-pointer space-y-1.5"
      >
        <div className="flex items-center justify-between text-[11px]">
          <div className="flex items-center gap-1.5 font-semibold text-foreground">
            <Sparkles className="h-3.5 w-3.5 text-brand shrink-0" />
            <span className="truncate">Chỉ thị dựng HyperFrames</span>
          </div>
          <div className="flex items-center gap-1 font-mono text-[10px] text-muted-foreground shrink-0">
            <span className="px-1 py-0.2 rounded bg-muted/60 text-foreground/80 font-medium">{aspect}</span>
            <span className="px-1 py-0.2 rounded bg-muted/60 text-foreground/80 font-medium">
              {duration === "auto" ? `~${totalSeconds}s` : `${duration}s`}
            </span>
          </div>
        </div>

        {/* Khối hiển thị Prompt ~2 dòng (giống ô input có thể click mở) */}
        <div className="font-mono text-[10px] leading-relaxed text-foreground/90 line-clamp-2 select-all bg-card/70 rounded px-2 py-1.5 border border-border/50">
          {activePrompt}
        </div>

        {/* Thanh tác vụ: Copy CLI universal + Nút mở Studio bên phải */}
        <div className="flex items-center justify-between pt-0.5 text-[10px]">
          <button
            type="button"
            onClick={handleCopy}
            className="nodrag inline-flex items-center gap-1 rounded border border-border/70 bg-card px-2 py-0.5 font-medium text-foreground hover:bg-muted transition-colors shadow-2xs"
            title="Sao chép câu lệnh CLI dùng chung cho cả Antigravity và Claude"
          >
            {copied ? (
              <>
                <Check className="h-3 w-3 text-semantic-success" />
                <span className="text-semantic-success font-semibold">Đã chép!</span>
              </>
            ) : (
              <>
                <Copy className="h-3 w-3 text-muted-foreground" />
                <span>Sao chép CLI</span>
              </>
            )}
          </button>

          <span className="inline-flex items-center gap-1 text-primary font-medium group-hover/box:translate-x-0.5 transition-transform">
            <span>Mở Studio bên phải</span>
            <PanelRightOpen className="h-3 w-3" />
          </span>
        </div>
      </div>
    </div>
  );
}

export function ToolboxAndGenericRenderer({
  nodeType,
  nodeId,
  data,
  params: p,
  previewUrl: url,
  lines,
  connectedInputs = [],
  compact,
  expanded,
  updateNodeParams,
}: NodeRendererContextProps) {
  switch (nodeType) {
    case "bg_remove":
      return (
        <div className="space-y-2">
          {url && data.previewType === "image" && (
            <ImagePreview url={url} alt="Tách nền ảnh" compact={compact} />
          )}
          {!url && (
            <Empty icon={<Images className="h-5 w-5" />} text="Ảnh tách nền sẽ hiện ở đây" tall={!compact} />
          )}
          <GenericParams nodeId={nodeId} nodeType={nodeType} params={p} />
        </div>
      );

    case "depth_layers":
      return (
        <div className="space-y-2">
          {url && data.previewType === "image" && (
            <ImagePreview url={url} alt="Tách lớp chiều sâu" compact={compact} />
          )}
          {!url && (
            <Empty icon={<Images className="h-5 w-5" />} text="Lớp chiều sâu sẽ hiện ở đây" tall={!compact} />
          )}
          <GenericParams nodeId={nodeId} nodeType={nodeType} params={p} />
        </div>
      );

    case "handdraw":
      return (
        <div className="space-y-2">
          {url && data.previewType === "video" && (
            <video
              src={url}
              controls
              preload="metadata"
              className="nodrag max-h-48 w-full rounded-ctl bg-black object-contain"
            />
          )}
          {!url && (
            <Empty icon={<Video className="h-5 w-5" />} text="Video vẽ tay sẽ hiện ở đây" tall={!compact} />
          )}
          <GenericParams nodeId={nodeId} nodeType={nodeType} params={p} />
        </div>
      );

    case "scene_cut":
      return (
        <div className="space-y-2">
          {url && data.previewType === "video" && (
            <video
              src={url}
              controls
              preload="metadata"
              className="nodrag max-h-48 w-full rounded-ctl bg-black object-contain"
            />
          )}
          {lines.length > 0 && (
            <div className="rounded-ctl border border-border bg-muted/20 p-2 text-footnote text-muted-foreground">
              {lines.map((l, i) => (
                <p key={i} className="line-clamp-1">{l}</p>
              ))}
            </div>
          )}
          {!url && lines.length === 0 && (
            <Empty icon={<Scissors className="h-5 w-5" />} text="Chưa tách cảnh video" />
          )}
          <GenericParams nodeId={nodeId} nodeType={nodeType} params={p} />
        </div>
      );

    case "voice_clone":
      return (
        <div className="space-y-2">
          {url && data.previewType === "audio" && (
            <div className="pt-1">
              <audio src={url} controls preload="metadata" className="nodrag h-7 w-full" />
            </div>
          )}
          {p.name && (
            <div className="flex flex-wrap items-center gap-2">
              <CanvasChip title="Mã giọng">{p.name}</CanvasChip>
              {p.display_name && <span className="text-footnote text-muted-foreground">{p.display_name}</span>}
              {p.dialect && <CanvasChip title="Phương ngữ">{p.dialect}</CanvasChip>}
            </div>
          )}
          {lines.length > 0 && <p className="text-caption text-muted-foreground leading-snug">{lines[0]}</p>}
          {!url && !p.name && <Empty icon={<Volume2 className="h-5 w-5" />} text="Mẫu giọng clone sẽ hiện ở đây" />}
          <GenericParams nodeId={nodeId} nodeType={nodeType} params={p} />
        </div>
      );

    case "word_captions":
      return (
        <div className="space-y-2">
          {lines.length > 0 && (
            <ul className="space-y-1 rounded-ctl border border-border bg-muted/20 p-2 text-footnote">
              {lines.slice(0, 4).map((l, i) => (
                <li key={`${nodeId}-wcap-${i}`} className="truncate text-muted-foreground">
                  {l}
                </li>
              ))}
            </ul>
          )}
          {url && data.previewType === "audio" && (
            <audio src={url} controls preload="metadata" className="nodrag h-7 w-full" />
          )}
          {lines.length === 0 && !(url && data.previewType === "audio") && (
            <Empty icon={<ScrollText className="h-5 w-5" />} text="Chưa có phụ đề từng từ" />
          )}
          <GenericParams nodeId={nodeId} nodeType={nodeType} params={p} />
        </div>
      );

    case "template_render": {
      return (
        <div className="space-y-2">
          {/* Tool Badge & View Process in New Tab */}
          <div className="flex items-center justify-between gap-1.5 rounded-ctl border border-primary/25 bg-primary/5 px-2 py-1 text-caption">
            <div className="flex items-center gap-1.5 min-w-0">
              <span className="inline-block h-2 w-2 rounded-full bg-semantic-success animate-pulse shrink-0" />
              <span className="font-semibold text-foreground truncate text-[11px]">HeyGen HyperFrames</span>
              <span className="text-muted-foreground shrink-0 text-[9px] bg-muted/60 px-1 py-0.2 rounded font-mono">v0.9.12</span>
            </div>
            <div className="flex items-center gap-1 shrink-0">
              <a
                href={`http://localhost:3002/#project/${p.template_id || 'koc-review-v1'}`}
                target="_blank"
                rel="noopener noreferrer"
                title="Mở tab mới để xem trực tiếp giao diện biên tập timeline HyperFrames Studio trên trình duyệt"
                className="nodrag inline-flex items-center gap-1 rounded-ctl border border-primary/40 bg-primary/10 px-1.5 py-0.5 text-[10px] font-semibold text-primary hover:bg-primary/20 transition-colors shadow-2xs"
                onClick={(e) => e.stopPropagation()}
              >
                <ExternalLink className="h-2.5 w-2.5" />
                <span>Live Edit</span>
              </a>
            </div>
          </div>

          {/* Video đã render (nếu có) */}
          {url && data.previewType === "video" && (
            <video
              src={url}
              controls
              preload="metadata"
              className="nodrag max-h-40 w-full rounded-ctl bg-black object-contain border border-border"
            />
          )}

          {/* Dạng Prompt nhỏ 4 dòng nhấn mở được như input, mở Studio Panel bên phải */}
          <TemplateVideoEditor
            nodeId={nodeId}
            templateId={p.template_id}
            params={p}
            updateNodeParams={updateNodeParams}
          />
        </div>
      );
    }

    case "platform_export": {
      const rawPlatforms = p.platforms || p.target_platforms;
      const currentPlatforms = Array.isArray(rawPlatforms) ? rawPlatforms : ["tiktok"];
      const hasExportSource = Boolean(
        (url && data.previewType === "video") ||
        (data.generations && data.generations.length > 0) ||
        connectedInputs.includes("video") ||
        p.video
      );
      return (
        <div className="space-y-3">
          {url && data.previewType === "video" && (
            <video src={url} controls preload="metadata" className="nodrag max-h-48 w-full rounded-ctl bg-black object-contain" />
          )}
          {!url && <Empty icon={<Video className="h-5 w-5" />} text="Video xuất đa nền tảng sẽ hiện ở đây" tall={!compact} />}
          {hasExportSource && (
            <PlatformsView
              platforms={currentPlatforms}
              onToggle={(plat) => {
                const next = currentPlatforms.includes(plat)
                  ? currentPlatforms.length > 1
                    ? currentPlatforms.filter((x: string) => x !== plat)
                    : currentPlatforms
                  : [...currentPlatforms, plat];
                updateNodeParams(nodeId, { platforms: next, target_platforms: next });
              }}
            />
          )}
          <GenericParams nodeId={nodeId} nodeType={nodeType} params={p} excludeKeys={["platforms", "target_platforms"]} />
        </div>
      );
    }

    default:
      return (
        <div className="space-y-2">
          {url && data.previewType === "image" && <ImagePreview url={url} alt={nodeType} compact={compact} />}
          {url && data.previewType === "video" && (
            <video src={url} controls preload="metadata" className="nodrag max-h-48 w-full rounded-ctl bg-black" />
          )}
          {url && data.previewType === "audio" && (
            <audio src={url} controls preload="metadata" className="nodrag h-7 w-full" />
          )}
          <GenericParams nodeId={nodeId} nodeType={nodeType} params={p} />
        </div>
      );
  }
}
