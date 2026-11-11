"use client";

import React, { useState, useRef } from "react";
import { cn } from "@/lib/utils";
import {
  Bot,
  Check,
  Clapperboard,
  Clock,
  Copy,
  ExternalLink,
  Film,
  Music,
  Pencil,
  Play,
  Plus,
  RotateCcw,
  Sliders,
  Smartphone,
  Sparkles,
  Trash2,
  Upload,
  Video,
  Volume2,
} from "@/components/ui/icons";

export const TEMPLATES_CATALOG: Record<
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
    slots: { key: string; label: string; placeholder: string; defaultVal: string }[];
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
    slots: [
      { key: "reviewer", label: "Tên KOC / Reviewer", placeholder: "Gobi (KOC thời trang)...", defaultVal: "Gobi (KOC Thời trang)" },
      { key: "product_name", label: "Tên sản phẩm", placeholder: "Đầm dạ hội / Váy Kaki...", defaultVal: "Set Áo Cổ Vuông Kaki Nữ" },
      { key: "pros", label: "Ưu điểm nổi bật", placeholder: "Vải đứng form, bề mặt chắc chắn...", defaultVal: "Vải đứng form, bề mặt chắc chắn, tôn dáng" },
      { key: "cons", label: "Lưu ý / Nhược điểm", placeholder: "Cần chọn size đúng số đo...", defaultVal: "Cần chọn size S/M/L đúng số đo theo bảng" },
      { key: "rating", label: "Đánh giá sao", placeholder: "5.0 · 31 đánh giá...", defaultVal: "5.0 ★ · 31 đánh giá" },
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
    slots: [
      { key: "hook_text", label: "Câu giật tít Hook", placeholder: "Đừng mua nếu chưa biết...", defaultVal: "Dừng lại 3s nếu bạn đang tìm kiếm set đồ này!" },
      { key: "discount_price", label: "Giá ưu đãi độc quyền", placeholder: "Chỉ 199K...", defaultVal: "Chỉ hôm nay: Giảm 40% + Freeship" },
      { key: "cta_text", label: "Lời kêu gọi CTA", placeholder: "Bấm góc trái...", defaultVal: "Nhấp ngay vào giỏ hàng bên dưới để săn deal!" },
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
    slots: [
      { key: "question", label: "Câu hỏi giật gân", placeholder: "Bạn có biết...", defaultVal: "Bạn có biết điều gì xảy ra nếu uống nước ấm mỗi sáng?" },
      { key: "fact_stat", label: "Con số ấn tượng", placeholder: "80% người...", defaultVal: "Hơn 85% người đã bất ngờ khi biết điều này" },
    ],
  },
  "unboxing-v1": {
    name: "Đập Hộp & Mở Hộp",
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
    slots: [
      { key: "box_condition", label: "Tình trạng kiện hàng", placeholder: "Nguyên seal...", defaultVal: "Đóng gói 2 lớp, nguyên seal chống sốc" },
      { key: "verdict", label: "Phán quyết mở hộp", placeholder: "10/10 đáng tiền...", defaultVal: "Xứng đáng từng xu trong tầm giá" },
    ],
  },
  "story-cartoon-v1": {
    name: "Hoạt Hình Thiếu Nhi Sóc Nhỏ",
    categoryBadge: "Hoạt hình",
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
    slots: [
      { key: "moral_lesson", label: "Bài học đạo đức", placeholder: "Luôn biết chia sẻ...", defaultVal: "Bài học: Chia sẻ niềm vui cùng bạn bè" },
    ],
  },
  "affiliate-compare-v1": {
    name: "So Sánh Đối Đầu 2 Sản Phẩm",
    categoryBadge: "So sánh",
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
    slots: [
      { key: "product_a", label: "Sản phẩm A", placeholder: "Tên sản phẩm A...", defaultVal: "Bản Tiêu Chuẩn (A)" },
      { key: "product_b", label: "Sản phẩm B", placeholder: "Tên sản phẩm B...", defaultVal: "Bản Cao Cấp (B)" },
    ],
  },
};

/** Icon Claude Code CLI */
function ClaudeIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor" className={className}>
      <path d="M16.5 3.5l-4.5 17-2.5-9.5-6-3.5 13-4z" opacity="0.9" fill="#D97757" />
    </svg>
  );
}

/** Icon Google Antigravity CLI */
function AntigravityIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" className={className}>
      <circle cx="12" cy="12" r="9" stroke="url(#agy-grad)" strokeWidth="2.5" />
      <path d="M12 7v10M7 12h10" stroke="url(#agy-grad)" strokeWidth="2.5" strokeLinecap="round" />
      <defs>
        <linearGradient id="agy-grad" x1="3" y1="3" x2="21" y2="21" gradientUnits="userSpaceOnUse">
          <stop stopColor="#38BDF8" />
          <stop offset="0.5" stopColor="#818CF8" />
          <stop offset="1" stopColor="#F472B6" />
        </linearGradient>
      </defs>
    </svg>
  );
}

interface HyperFramesStudioPanelProps {
  nodeId: string;
  params: Record<string, any>;
  updateNodeParams: (nodeId: string, patch: Record<string, any>) => void;
}

export function HyperFramesStudioPanel({
  nodeId,
  params,
  updateNodeParams,
}: HyperFramesStudioPanelProps) {
  const currentId = (params.template_id as string) || "koc-review-v1";
  const info = TEMPLATES_CATALOG[currentId] || {
    name: currentId,
    categoryBadge: "Template",
    description: "Mẫu dựng video chuẩn HyperFrames",
    aspect: "9:16",
    duration: "auto",
    audio: "chill_lofi",
    captions: "anchor",
    scenes: [
      { title: "1. Mở đầu", dur: "5s", desc: "Phân cảnh mở đầu" },
      { title: "2. Nội dung", dur: "15s", desc: "Phân cảnh nội dung chính" },
    ],
    slots: [],
  };

  const aspect = params.aspect || (info.aspect.includes("16:9") ? "16:9" : "9:16");
  const duration = params.duration || "auto";
  const bgm = params.bgm || "chill_lofi";
  const captions = params.captions || "anchor";
  const customAudioName = params.custom_audio_name || "";
  const scenes: { title: string; dur?: string; desc?: string }[] =
    Array.isArray(params.scenes) && params.scenes.length > 0 ? params.scenes : info.scenes;
  const slotMapping = params.slot_mapping || {};
  const cliTool = params.cli_tool || "antigravity"; // "antigravity" | "claude"

  const [copied, setCopied] = useState(false);
  const [selectedSceneIdx, setSelectedSceneIdx] = useState<number | null>(null);
  const audioInputRef = useRef<HTMLInputElement>(null);

  // Tính tổng thời lượng timeline
  const totalSeconds = scenes.reduce((acc, s) => {
    const n = parseInt(s.dur?.replace(/\D/g, "") || "5", 10) || 5;
    return acc + n;
  }, 0);

  // Biên dịch câu prompt tổng thể
  const buildPrompt = (patch: Record<string, any>) => {
    const asp = patch.aspect ?? aspect;
    const dur = patch.duration ?? duration;
    const b = patch.bgm ?? bgm;
    const cap = patch.captions ?? captions;
    const scList = patch.scenes ?? scenes;
    const audName = patch.custom_audio_name ?? customAudioName;

    const bgmStr = audName
      ? `CustomAudio:${audName}`
      : b === "chill_lofi"
      ? "Lofi"
      : b === "trending_pop"
      ? "Pop"
      : b === "acoustic"
      ? "Acoustic"
      : b === "dramatic"
      ? "Dramatic"
      : b === "none"
      ? "Mute"
      : b;

    const capStr =
      cap === "anchor"
        ? "Anchor"
        : cap === "bold_popup"
        ? "Popup"
        : cap === "kinetic"
        ? "Kinetic"
        : cap === "karaoke"
        ? "Karaoke"
        : cap === "none"
        ? "Off"
        : cap;

    const durStr = dur === "auto" ? `${totalSeconds}s (Auto)` : `${dur}s`;
    const scenesSummary = Array.isArray(scList) && scList.length > 0
      ? scList.map((s: any) => `${s.title}${s.dur ? `(${s.dur})` : ""}`).join(" > ")
      : "";

    return `HyperFrames [${currentId}]: ${asp} | ${durStr} | BGM:${bgmStr} | Sub:${capStr}${scenesSummary ? ` | Flow: ${scenesSummary}` : ""}`;
  };

  const handleUpdate = (patch: Record<string, any>) => {
    const isCustomized = Boolean(params.prompt_customized);
    const newPrompt = isCustomized ? params.edit_prompt : buildPrompt(patch);
    updateNodeParams(nodeId, {
      ...patch,
      edit_prompt: newPrompt,
    });
  };

  // Cập nhật phân cảnh trên timeline
  const handleUpdateScene = (idx: number, patch: Partial<{ title: string; dur: string; desc: string }>) => {
    const updated = scenes.map((s, i) => (i === idx ? { ...s, ...patch } : s));
    handleUpdate({ scenes: updated });
  };

  const handleStepDuration = (idx: number, delta: number) => {
    const cur = scenes[idx];
    const rawNum = parseInt(cur.dur?.replace(/\D/g, "") || "5", 10) || 5;
    const nextNum = Math.max(1, rawNum + delta);
    handleUpdateScene(idx, { dur: `${nextNum}s` });
  };

  const handleAddScene = () => {
    const newScene = {
      title: `${scenes.length + 1}. Phân cảnh mới`,
      dur: "5s",
      desc: "Nội dung phân cảnh bổ sung",
    };
    const next = [...scenes, newScene];
    handleUpdate({ scenes: next });
    setSelectedSceneIdx(next.length - 1);
  };

  const handleRemoveScene = (idx: number) => {
    if (scenes.length <= 1) return;
    const next = scenes.filter((_, i) => i !== idx);
    handleUpdate({ scenes: next });
    if (selectedSceneIdx === idx) {
      setSelectedSceneIdx(null);
    } else if (selectedSceneIdx !== null && selectedSceneIdx > idx) {
      setSelectedSceneIdx(selectedSceneIdx - 1);
    }
  };

  const handleResetScenes = () => {
    handleUpdate({ scenes: info.scenes });
    setSelectedSceneIdx(null);
  };

  // Cập nhật giá trị slot động (dynamic values)
  const handleUpdateSlot = (key: string, val: string) => {
    const nextSlots = { ...(slotMapping || {}), [key]: val };
    handleUpdate({ slot_mapping: nextSlots });
  };

  // Upload nhạc tùy chỉnh
  const handleAudioUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    handleUpdate({
      custom_audio_name: file.name,
      bgm: "custom",
    });
  };

  const activePrompt = params.edit_prompt || buildPrompt({});

  // Lệnh CLI chung (Universal & tương thích cả hai)
  const universalCliCommand = `agy run-node --id ${nodeId} --prompt "${activePrompt}"`;

  const handleCopyUniversal = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(universalCliCommand);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const selectedScene = selectedSceneIdx !== null ? scenes[selectedSceneIdx] : null;

  return (
    <div className="space-y-4 text-footnote">
      {/* 1. Header & Live Edit Tab */}
      <div className="flex items-center justify-between border-b border-border pb-3">
        <div className="space-y-0.5 min-w-0">
          <div className="flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-semantic-success animate-pulse" />
            <h3 className="text-callout font-semibold text-foreground">HyperFrames Studio</h3>
          </div>
          <span className="text-caption text-muted-foreground block truncate">
            {info.name} ({info.categoryBadge})
          </span>
        </div>
        <a
          href={`http://localhost:3002/#project/${currentId}`}
          target="_blank"
          rel="noopener noreferrer"
          className="nodrag inline-flex items-center gap-1.5 rounded-ctl border border-primary/50 bg-primary/10 px-2.5 py-1 text-caption font-semibold text-primary hover:bg-primary/20 transition-colors shadow-2xs shrink-0"
        >
          <ExternalLink className="h-3.5 w-3.5" />
          <span>Live Editor</span>
        </a>
      </div>

      {/* 2. Chọn Mẫu Video (Template Switcher) */}
      <div className="space-y-1.5">
        <label className="text-caption font-medium text-muted-foreground">Mẫu video (Template ID)</label>
        <select
          value={currentId}
          onChange={(e) => {
            const nextTmpl = e.target.value;
            const nextInfo = TEMPLATES_CATALOG[nextTmpl];
            handleUpdate({
              template_id: nextTmpl,
              aspect: nextInfo?.aspect || "9:16",
              scenes: nextInfo?.scenes || [],
              slot_mapping: {},
            });
          }}
          className="nodrag w-full h-8 rounded-ctl border border-border/80 bg-card px-2.5 text-footnote font-medium text-foreground outline-none focus:border-brand"
        >
          {Object.entries(TEMPLATES_CATALOG).map(([k, item]) => (
            <option key={k} value={k}>
              {item.name} ({item.categoryBadge})
            </option>
          ))}
        </select>
        <p className="text-[11px] text-muted-foreground leading-relaxed">{info.description}</p>
      </div>

      {/* 3. Live Video Preview Box */}
      <div className="space-y-1.5">
        <div className="flex items-center justify-between text-caption text-muted-foreground">
          <span className="font-medium flex items-center gap-1">
            <Video className="h-3.5 w-3.5 text-brand" />
            <span>Khung xem trước (HyperFrames Daemon)</span>
          </span>
          <span className="font-mono text-[10px] bg-muted/60 px-1.5 py-0.5 rounded">
            Port 3002
          </span>
        </div>

        <div className="relative aspect-[9/16] max-h-56 w-full rounded-ctl border border-border bg-black/90 flex flex-col items-center justify-center overflow-hidden shadow-2xs mx-auto">
          <div className="absolute inset-0 flex flex-col items-center justify-center text-center p-3 text-muted-foreground gap-2">
            <Video className="h-8 w-8 text-primary/70 animate-pulse" />
            <div className="text-caption font-medium text-foreground">
              {info.name}
            </div>
            <span className="text-[10px] text-muted-foreground">
              Đang kết nối live stream timeline HyperFrames
            </span>
            <a
              href={`http://localhost:3002/#project/${currentId}`}
              target="_blank"
              rel="noopener noreferrer"
              className="mt-1 inline-flex items-center gap-1 px-2 py-0.5 text-[10px] rounded bg-primary/20 text-primary hover:bg-primary/30 transition-colors"
            >
              <span>Xem toàn màn hình</span>
              <ExternalLink className="h-2.5 w-2.5" />
            </a>
          </div>
        </div>
      </div>

      {/* 4. Cấu hình định dạng & Thuộc tính cơ bản */}
      <div className="grid grid-cols-2 gap-2 text-caption">
        {/* Tỉ lệ */}
        <div className="space-y-1">
          <label className="text-[11px] font-medium text-muted-foreground flex items-center gap-1">
            <Smartphone className="h-3 w-3 text-brand" />
            <span>Khung hình</span>
          </label>
          <select
            value={aspect}
            onChange={(e) => handleUpdate({ aspect: e.target.value })}
            className="nodrag w-full h-7 rounded-ctl border border-border/80 bg-card px-2 text-[11px] font-medium text-foreground outline-none focus:border-brand"
          >
            <option value="9:16">Dọc 9:16 (TikTok, Shorts)</option>
            <option value="16:9">Ngang 16:9 (YouTube)</option>
            <option value="1:1">Vuông 1:1 (Instagram)</option>
            <option value="4:5">Feed 4:5 (Facebook)</option>
          </select>
        </div>

        {/* Thời lượng */}
        <div className="space-y-1">
          <label className="text-[11px] font-medium text-muted-foreground flex items-center gap-1">
            <Clock className="h-3 w-3 text-emerald-500" />
            <span>Thời lượng</span>
          </label>
          <select
            value={duration}
            onChange={(e) => handleUpdate({ duration: e.target.value })}
            className="nodrag w-full h-7 rounded-ctl border border-border/80 bg-card px-2 text-[11px] font-medium text-foreground outline-none focus:border-brand"
          >
            <option value="auto">Tự động (Theo timeline {totalSeconds}s)</option>
            <option value="15">15 giây (Hook ngắn)</option>
            <option value="30">30 giây (Chuẩn TikTok)</option>
            <option value="60">60 giây (Chi tiết)</option>
            <option value="90">90 giây (Toàn diện)</option>
          </select>
        </div>
      </div>

      {/* 5. Nhạc nền (BGM) & Upload Nhạc Tùy Chỉnh */}
      <div className="space-y-2 border-t border-border/60 pt-3">
        <div className="flex items-center justify-between text-caption">
          <label className="font-semibold text-foreground flex items-center gap-1.5">
            <Music className="h-3.5 w-3.5 text-amber-500" />
            <span>Âm thanh & Nhạc nền (BGM)</span>
          </label>
          {customAudioName && (
            <span className="text-[10px] text-brand font-medium truncate max-w-[150px]">
              {customAudioName}
            </span>
          )}
        </div>

        <div className="grid grid-cols-[1fr_auto] gap-1.5 items-center">
          <select
            value={bgm}
            onChange={(e) => {
              if (e.target.value !== "custom") {
                handleUpdate({ bgm: e.target.value, custom_audio_name: "" });
              }
            }}
            className="nodrag w-full h-7 rounded-ctl border border-border/80 bg-card px-2 text-[11px] font-medium text-foreground outline-none focus:border-brand"
          >
            <option value="chill_lofi">Chill Lofi (-10dB) · Êm dịu review</option>
            <option value="trending_pop">Trending Pop · Sôi động bán hàng</option>
            <option value="acoustic">Upbeat Acoustic · Đập hộp & Vlog</option>
            <option value="dramatic">Dramatic / Hồi hộp · Fact & Khám phá</option>
            <option value="cartoon">Đồng thoại vui nhộn · Hoạt hình</option>
            <option value="none">Không nhạc nền (Chỉ giọng đọc voice)</option>
            {customAudioName && <option value="custom">File tải lên: {customAudioName}</option>}
          </select>

          {/* Nút Upload Nhạc Tùy Chọn */}
          <input
            ref={audioInputRef}
            type="file"
            accept="audio/*"
            className="hidden"
            onChange={handleAudioUpload}
          />
          <button
            type="button"
            onClick={() => audioInputRef.current?.click()}
            className="nodrag inline-flex items-center gap-1 h-7 px-2.5 rounded-ctl border border-border/80 bg-muted/40 hover:bg-muted text-[11px] font-medium text-foreground transition-colors shadow-2xs"
            title="Tải lên tệp âm thanh MP3/WAV riêng của bạn"
          >
            <Upload className="h-3 w-3 text-brand" />
            <span>Tải nhạc</span>
          </button>
        </div>
      </div>

      {/* 6. Kiểu Phụ đề & Typography */}
      <div className="space-y-1.5 border-t border-border/60 pt-3">
        <label className="text-caption font-semibold text-foreground flex items-center gap-1.5">
          <Clapperboard className="h-3.5 w-3.5 text-purple-500" />
          <span>Kiểu phụ đề (Captions)</span>
        </label>
        <select
          value={captions}
          onChange={(e) => handleUpdate({ captions: e.target.value })}
          className="nodrag w-full h-7 rounded-ctl border border-border/80 bg-card px-2 text-[11px] font-medium text-foreground outline-none focus:border-brand"
        >
          <option value="anchor">Anchor Lower-Third (Chân trang tinh tế)</option>
          <option value="bold_popup">Bold Pop-up CTA (Chữ nổi bật giật tít)</option>
          <option value="kinetic">Kinetic Typography (Chuyển động nhịp nhanh)</option>
          <option value="karaoke">Karaoke màu sắc (Tô sáng từng từ)</option>
          <option value="none">Tắt phụ đề</option>
        </select>
      </div>

      {/* 7. Interactive Timeline Multi-Track */}
      <div className="space-y-2 border-t border-border/60 pt-3">
        <div className="flex items-center justify-between text-caption">
          <div className="flex items-center gap-1.5 font-semibold text-foreground">
            <Film className="h-3.5 w-3.5 text-brand" />
            <span>Timeline kịch bản ({scenes.length} cảnh)</span>
            <span className="font-mono text-[10px] text-muted-foreground font-normal">
              · {totalSeconds}s
            </span>
          </div>
          <div className="flex items-center gap-1 text-[10px]">
            <button
              type="button"
              onClick={handleAddScene}
              className="nodrag inline-flex items-center gap-0.5 px-2 py-0.5 rounded border border-border/70 bg-card text-foreground hover:text-brand transition-colors shadow-2xs font-medium"
            >
              <Plus className="h-2.5 w-2.5" />
              <span>Thêm cảnh</span>
            </button>
            {Array.isArray(params.scenes) && (
              <button
                type="button"
                onClick={handleResetScenes}
                className="nodrag p-1 rounded border border-border/70 bg-card text-muted-foreground hover:text-foreground transition-colors"
                title="Khôi phục mặc định"
              >
                <RotateCcw className="h-2.5 w-2.5" />
              </button>
            )}
          </div>
        </div>

        {/* Thanh Sequencer Track */}
        <div className="flex items-center gap-1 w-full">
          {scenes.map((s, idx) => {
            const isSelected = selectedSceneIdx === idx;
            const colorClass =
              idx === 0
                ? "border-sky-500/40 bg-sky-500/10 text-sky-400"
                : idx === scenes.length - 1
                ? "border-amber-500/40 bg-amber-500/10 text-amber-400"
                : "border-indigo-500/40 bg-indigo-500/10 text-indigo-400";

            return (
              <button
                key={idx}
                type="button"
                onClick={() => setSelectedSceneIdx(isSelected ? null : idx)}
                title={`${s.title}: ${s.desc || ""} (Bấm để chỉnh sửa)`}
                className={cn(
                  "nodrag flex-1 min-w-0 h-8 px-2 py-1 rounded border flex items-center justify-between gap-1 text-[11px] font-medium transition-all shadow-2xs select-none",
                  colorClass,
                  isSelected ? "ring-2 ring-brand border-brand font-semibold shadow-xs" : ""
                )}
              >
                <span className="truncate text-foreground font-medium">{s.title}</span>
                <span className="font-mono text-[10px] px-1 py-0.2 rounded bg-black/40 text-foreground/80 shrink-0">
                  {s.dur || "5s"}
                </span>
              </button>
            );
          })}
        </div>

        {/* Bảng chi tiết sửa phân cảnh đang chọn */}
        {selectedScene && selectedSceneIdx !== null && (
          <div className="rounded-ctl border border-brand/50 bg-brand/5 p-2.5 space-y-2 animate-in fade-in duration-150">
            <div className="flex items-center justify-between text-caption font-semibold text-brand">
              <span className="flex items-center gap-1">
                <Pencil className="h-3 w-3" />
                <span>Hiệu chỉnh phân cảnh {selectedSceneIdx + 1}</span>
              </span>
              <div className="flex items-center gap-1.5">
                {scenes.length > 1 && (
                  <button
                    type="button"
                    onClick={() => handleRemoveScene(selectedSceneIdx)}
                    className="nodrag text-muted-foreground hover:text-semantic-error p-0.5 rounded"
                    title="Xóa phân cảnh này"
                  >
                    <Trash2 className="h-3 w-3" />
                  </button>
                )}
                <button
                  type="button"
                  onClick={() => setSelectedSceneIdx(null)}
                  className="nodrag text-muted-foreground hover:text-foreground p-0.5 rounded"
                  title="Xong"
                >
                  <Check className="h-3.5 w-3.5 text-brand" />
                </button>
              </div>
            </div>

            <div className="grid grid-cols-[1fr_auto] gap-2 items-center">
              <input
                type="text"
                value={selectedScene.title}
                onChange={(e) => handleUpdateScene(selectedSceneIdx, { title: e.target.value })}
                placeholder="Tiêu đề phân cảnh..."
                className="nodrag h-7 px-2 text-footnote font-medium bg-card border border-border/80 rounded-ctl outline-none focus:border-brand text-foreground"
              />

              {/* Tăng giảm giây +/- */}
              <div className="flex items-center gap-0.5 bg-card border border-border/80 rounded-ctl px-1.5 h-7">
                <button
                  type="button"
                  onClick={() => handleStepDuration(selectedSceneIdx, -1)}
                  className="nodrag px-1 text-muted-foreground hover:text-foreground text-caption font-bold"
                  title="Giảm 1 giây"
                >
                  -
                </button>
                <input
                  type="text"
                  value={selectedScene.dur || "5s"}
                  onChange={(e) => handleUpdateScene(selectedSceneIdx, { dur: e.target.value })}
                  className="nodrag w-10 text-center font-mono text-[11px] bg-transparent outline-none text-foreground font-semibold"
                />
                <button
                  type="button"
                  onClick={() => handleStepDuration(selectedSceneIdx, 1)}
                  className="nodrag px-1 text-muted-foreground hover:text-foreground text-caption font-bold"
                  title="Tăng 1 giây"
                >
                  +
                </button>
              </div>
            </div>

            <input
              type="text"
              value={selectedScene.desc || ""}
              onChange={(e) => handleUpdateScene(selectedSceneIdx, { desc: e.target.value })}
              placeholder="Chỉ đạo nội dung / hành động cho cảnh..."
              className="nodrag w-full h-6 px-2 text-[11px] bg-card border border-border/70 rounded outline-none focus:border-brand text-muted-foreground placeholder:text-muted-foreground/40"
            />
          </div>
        )}
      </div>

      {/* 8. Select Giá Trị Động Cho Template (Dynamic Slots) */}
      {info.slots && info.slots.length > 0 && (
        <div className="space-y-2.5 border-t border-border/60 pt-3">
          <label className="text-caption font-semibold text-foreground flex items-center gap-1.5">
            <Sliders className="h-3.5 w-3.5 text-brand" />
            <span>Giá trị động phân cảnh (Dynamic Slots)</span>
          </label>
          <div className="space-y-2 bg-muted/20 p-2.5 rounded-ctl border border-border/60">
            {info.slots.map((slot) => {
              const currentVal = (slotMapping[slot.key] as string) || slot.defaultVal;
              return (
                <div key={slot.key} className="space-y-1">
                  <div className="flex items-center justify-between text-[11px]">
                    <span className="font-medium text-foreground">{slot.label}</span>
                    <span className="font-mono text-[10px] text-muted-foreground">{slot.key}</span>
                  </div>
                  <input
                    type="text"
                    value={currentVal}
                    onChange={(e) => handleUpdateSlot(slot.key, e.target.value)}
                    placeholder={slot.placeholder}
                    className="nodrag w-full h-7 px-2 text-footnote bg-card border border-border/80 rounded-ctl outline-none focus:border-brand text-foreground"
                  />
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* 9. AI Prompt & CLI Engine (With Official Icons & Universal Copy) */}
      <div className="space-y-2 border-t border-border/60 pt-3">
        <div className="flex items-center justify-between text-caption">
          <div className="flex items-center gap-1.5 font-semibold text-foreground">
            <Sparkles className="h-3.5 w-3.5 text-brand" />
            <span>Chỉ thị AI & CLI Dựng phim</span>
          </div>

          {/* Switch tool CLI với Icon chính thức */}
          <div className="flex items-center gap-1 rounded-ctl border border-border/60 bg-muted/40 p-0.5 text-[10px] font-mono">
            <button
              type="button"
              onClick={() => handleUpdate({ cli_tool: "antigravity" })}
              className={cn(
                "nodrag inline-flex items-center gap-1 px-2 py-0.5 rounded transition-colors",
                cliTool === "antigravity"
                  ? "bg-brand text-brand-foreground font-semibold shadow-2xs"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              <AntigravityIcon />
              <span>Antigravity</span>
            </button>
            <button
              type="button"
              onClick={() => handleUpdate({ cli_tool: "claude" })}
              className={cn(
                "nodrag inline-flex items-center gap-1 px-2 py-0.5 rounded transition-colors",
                cliTool === "claude"
                  ? "bg-brand text-brand-foreground font-semibold shadow-2xs"
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              <ClaudeIcon />
              <span>Claude</span>
            </button>
          </div>
        </div>

        {/* Input Prompt Biên dịch */}
        <div className="rounded-ctl border border-border/80 bg-card p-2 space-y-1.5 shadow-2xs">
          <textarea
            value={activePrompt}
            onChange={(e) =>
              updateNodeParams(nodeId, {
                edit_prompt: e.target.value,
                prompt_customized: true,
              })
            }
            rows={3}
            placeholder="Chỉ thị biên tập video gửi tới HyperFrames hoặc CLI..."
            className="nodrag w-full bg-transparent text-[11px] font-mono leading-relaxed text-foreground placeholder:text-muted-foreground/50 outline-none resize-none"
          />

          <div className="flex items-center justify-between pt-1 border-t border-border/40 text-[10px]">
            {params.prompt_customized ? (
              <span className="text-amber-500 flex items-center gap-1 font-mono">
                <Pencil className="h-2.5 w-2.5" />
                <span>Đã tùy biến tay</span>
              </span>
            ) : (
              <span className="text-emerald-500/90 flex items-center gap-1 font-mono">
                <Sparkles className="h-2.5 w-2.5" />
                <span>Tự động sync từ panel</span>
              </span>
            )}

            <div className="flex items-center gap-1.5">
              {params.prompt_customized && (
                <button
                  type="button"
                  onClick={() =>
                    updateNodeParams(nodeId, {
                      edit_prompt: undefined,
                      prompt_customized: false,
                    })
                  }
                  className="nodrag text-muted-foreground hover:text-foreground flex items-center gap-0.5"
                  title="Đồng bộ lại từ các thông số trên"
                >
                  <RotateCcw className="h-2.5 w-2.5" />
                  <span>Sync lại</span>
                </button>
              )}

              {/* Nút Sao chép CLI chung (Universal Copy) */}
              <button
                type="button"
                onClick={handleCopyUniversal}
                className="nodrag inline-flex items-center gap-1 rounded border border-border/80 bg-card px-2.5 py-1 text-caption font-semibold text-foreground hover:bg-muted transition-colors shadow-2xs"
                title="Sao chép câu lệnh CLI dùng chung cho cả Antigravity và Claude"
              >
                {copied ? (
                  <>
                    <Check className="h-3 w-3 text-semantic-success" />
                    <span className="text-semantic-success">Đã chép!</span>
                  </>
                ) : (
                  <>
                    <Copy className="h-3 w-3 text-muted-foreground" />
                    <span>Sao chép CLI</span>
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

