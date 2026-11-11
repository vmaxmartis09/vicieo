import "@testing-library/jest-dom/vitest";
import React from "react";
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { render, screen, cleanup, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { NextIntlClientProvider } from "next-intl";
import viMessages from "@/messages/vi.json";
import { useConfirmStore } from "@/lib/confirm";
import {
  SophisticatedChatInput,
  type ChatAttachment,
  type CanvasImageItem,
} from "@/components/studio/chat/SophisticatedChatInput";

const SAMPLE_CHARS = [
  { id: "soc_nho", name: "Sóc Nhỏ (Bi)", heightCm: 100 },
  { id: "koala", name: "Bác sĩ Koala", heightCm: 150 },
];

const SAMPLE_CANVAS_IMAGES: CanvasImageItem[] = [
  {
    id: "img_1",
    nodeId: "node_img_1",
    name: "Ảnh Rừng Xanh",
    url: "/demo/forest.png",
    kind: "image",
  },
];

const SAMPLE_SLASH = [
  { cmd: "/video", hint: "Tạo Video từ ảnh tham chiếu" },
  { cmd: "/run", hint: "Chạy flow hiện tại" },
  { cmd: "/clear", hint: "Xóa hội thoại" },
];

function renderInput(
  overrides: Partial<React.ComponentProps<typeof SophisticatedChatInput>> = {}
) {
  const onInputChange = vi.fn();
  const onSubmit = vi.fn();
  const onModelChange = vi.fn();
  const onToggleMockMode = vi.fn();
  const onAddAttachment = vi.fn();
  const onRemoveAttachment = vi.fn();
  const onToggleCraftTag = vi.fn();
  const onActionModeChange = vi.fn();
  const onAspectRatioChange = vi.fn();
  const onMentionSelect = vi.fn();
  const onSlashSelect = vi.fn();
  const onClearChat = vi.fn();

  const utils = render(
    <NextIntlClientProvider locale="vi" messages={viMessages}>
      <SophisticatedChatInput
        input=""
        onInputChange={onInputChange}
        onSubmit={onSubmit}
        model="gemini-3.8-flash"
        onModelChange={onModelChange}
        isMockMode={true}
        onToggleMockMode={onToggleMockMode}
        characters={SAMPLE_CHARS}
        canvasImages={SAMPLE_CANVAS_IMAGES}
        attachments={[]}
        onAddAttachment={onAddAttachment}
        onRemoveAttachment={onRemoveAttachment}
        craftTags={[]}
        onToggleCraftTag={onToggleCraftTag}
        actionMode="chat"
        onActionModeChange={onActionModeChange}
        aspectRatio="9:16"
        onAspectRatioChange={onAspectRatioChange}
        onMentionSelect={onMentionSelect}
        onSlashSelect={onSlashSelect}
        onClearChat={onClearChat}
        slashCommands={SAMPLE_SLASH}
        {...overrides}
      />
    </NextIntlClientProvider>
  );

  return {
    ...utils,
    onInputChange,
    onSubmit,
    onModelChange,
    onToggleMockMode,
    onAddAttachment,
    onRemoveAttachment,
    onToggleCraftTag,
    onActionModeChange,
    onAspectRatioChange,
    onMentionSelect,
    onSlashSelect,
    onClearChat,
  };
}

beforeEach(() => {
  useConfirmStore.setState({ current: null });
  delete (window as any).SpeechRecognition;
  delete (window as any).webkitSpeechRecognition;
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("SophisticatedChatInput characterization tests", () => {
  it("shows slash command autocomplete menu and selects command", async () => {
    const user = userEvent.setup();
    const { onSlashSelect } = renderInput({ input: "/vi" });

    expect(screen.getByText("/video")).toBeInTheDocument();
    expect(screen.queryByText("/run")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /\/video/i }));
    expect(onSlashSelect).toHaveBeenCalledWith("/video");
  });

  it("shows @ mention menu with canvas images and characters", async () => {
    const user = userEvent.setup();
    const { onAddAttachment, onMentionSelect, onInputChange } = renderInput({
      input: "Kiểm tra @",
    });

    expect(screen.getByText("Ảnh Rừng Xanh")).toBeInTheDocument();
    expect(screen.getByText("Sóc Nhỏ (Bi)")).toBeInTheDocument();
    expect(screen.getByText("Bác sĩ Koala")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /Ảnh Rừng Xanh/i }));
    expect(onAddAttachment).toHaveBeenCalledWith(
      expect.objectContaining({ id: "att_cv_img_1", nodeId: "node_img_1", source: "canvas" })
    );
    expect(onInputChange).toHaveBeenCalledWith("Kiểm tra @Ảnh Rừng Xanh ");

    await user.click(screen.getByRole("button", { name: /Sóc Nhỏ \(Bi\)/i }));
    expect(onMentionSelect).toHaveBeenCalledWith("soc_nho", "Sóc Nhỏ (Bi)");
  });

  it("opens '+', 'Skill', 'Model/Ratio', and 'Mode' popovers and dispatches selections", async () => {
    const user = userEvent.setup();
    const {
      onAddAttachment,
      onActionModeChange,
      onClearChat,
      onToggleCraftTag,
      onModelChange,
      onAspectRatioChange,
      onToggleMockMode,
    } = renderInput({ input: "Cảnh mới" });

    // '+' menu -> pick canvas image, pick mode, or clear chat
    await user.click(screen.getByRole("button", { name: "Đính kèm ảnh hoặc chọn chế độ" }));
    expect(screen.getByText("Tải ảnh lên từ máy (Upload local)")).toBeInTheDocument();
    await user.click(screen.getByTitle("Ảnh Rừng Xanh"));
    expect(onAddAttachment).toHaveBeenCalledWith(
      expect.objectContaining({ nodeId: "node_img_1", source: "canvas" })
    );

    await user.click(screen.getByRole("button", { name: "Đính kèm ảnh hoặc chọn chế độ" }));
    await user.click(screen.getByRole("button", { name: /Ảnh → Tạo Video/i }));
    expect(onActionModeChange).toHaveBeenCalledWith("video");

    await user.click(screen.getByRole("button", { name: "Đính kèm ảnh hoặc chọn chế độ" }));
    await user.click(screen.getByRole("button", { name: "Xóa cuộc trò chuyện này" }));
    expect(onClearChat).toHaveBeenCalledTimes(1);

    // 'Skill' menu -> craft preset toggle
    await user.click(screen.getByRole("button", { name: /Skill/i }));
    await user.click(screen.getByRole("button", { name: "Phong cách: 3D Hoạt hình" }));
    expect(onToggleCraftTag).toHaveBeenCalledWith("3D Pixar Pixar-style");

    // 'Model · Ratio' popover -> model switch & media ratio tab
    await user.click(screen.getByRole("button", { name: /Gemini 3\.8 Flash/i }));
    await user.click(screen.getByRole("button", { name: /Gemini 2\.5 Pro/i }));
    expect(onModelChange).toHaveBeenCalledWith("gemini-2.5-pro");

    await user.click(screen.getByRole("button", { name: /Gemini 3\.8 Flash/i }));
    await user.click(screen.getByRole("button", { name: "Ảnh / Video / Tỉ lệ" }));
    await user.click(screen.getByRole("button", { name: "16:9" }));
    expect(onAspectRatioChange).toHaveBeenCalledWith("16:9");

    // 'Hỏi trước' / Turbo popover
    await user.click(screen.getByRole("button", { name: /Hỏi trước/i }));
    await user.click(screen.getByRole("button", { name: /Turbo · Live/i }));
    expect(onToggleMockMode).toHaveBeenCalledTimes(1);
  });

  it("handles voice dictation fallback alert and SpeechRecognition dictation result", async () => {
    const user = userEvent.setup();
    const { onInputChange } = renderInput({ input: "" });

    // Without SpeechRecognition -> shows confirm/alert warning
    await user.click(screen.getByRole("button", { name: "Voice input" }));
    expect(useConfirmStore.getState().current?.title).toBe("Ghi âm giọng nói");

    // With mocked SpeechRecognition -> starts and appends transcript
    let instance: any = null;
    class MockSpeechRecognition {
      lang = "";
      interimResults = false;
      maxAlternatives = 1;
      onstart: (() => void) | null = null;
      onresult: ((e: any) => void) | null = null;
      onerror: (() => void) | null = null;
      onend: (() => void) | null = null;
      start() {
        instance = this;
        this.onstart?.();
      }
    }
    (window as any).SpeechRecognition = MockSpeechRecognition;

    await user.click(screen.getByRole("button", { name: "Voice input" }));
    expect(instance).not.toBeNull();
    expect(instance.lang).toBe("vi-VN");

    act(() => {
      instance.onresult?.({ results: [[{ transcript: "Sóc Nhỏ đi học" }]] });
      instance.onend?.();
    });
    expect(onInputChange).toHaveBeenCalledWith("Sóc Nhỏ đi học");
  });

  it("renders attachments strip and supports submit on Enter or button click", async () => {
    const user = userEvent.setup();
    const attachments: ChatAttachment[] = [
      { id: "att_1", name: "Bi Pose", url: "/demo/bi.png", source: "upload" },
    ];
    const { onRemoveAttachment, onToggleCraftTag, onActionModeChange, onSubmit } = renderInput({
      input: "Tạo cảnh rừng",
      attachments,
      craftTags: ["Golden Hour Rim Light"],
      actionMode: "video",
    });

    expect(screen.getByText("Bi Pose")).toBeInTheDocument();
    expect(screen.getByText("Golden Hour Rim Light")).toBeInTheDocument();
    expect(screen.getByText("Chế độ: Ảnh → Video")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Remove Bi Pose" }));
    expect(onRemoveAttachment).toHaveBeenCalledWith("att_1");

    await user.click(screen.getByRole("button", { name: "Remove Golden Hour Rim Light" }));
    expect(onToggleCraftTag).toHaveBeenCalledWith("Golden Hour Rim Light");

    await user.click(screen.getByRole("button", { name: "Reset mode" }));
    expect(onActionModeChange).toHaveBeenCalledWith("chat");

    await user.click(screen.getByRole("button", { name: "Gửi" }));
    expect(onSubmit).toHaveBeenCalledTimes(1);
  });
});
