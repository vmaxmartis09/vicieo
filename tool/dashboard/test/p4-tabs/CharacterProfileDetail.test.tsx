import "@testing-library/jest-dom/vitest";
import React from "react";
import { describe, it, expect, beforeAll, afterEach, afterAll, beforeEach, vi } from "vitest";
import { render, screen, waitFor, cleanup, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { NextIntlClientProvider } from "next-intl";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import viMessages from "@/messages/vi.json";
import { CORE_URL } from "@/lib/repositories/core";
import { useStudioStore } from "@/store";
import type { Character } from "@/lib/repositories/types";
import type { CharacterProfileSection } from "@/store/slices/rightPanelSlice";
import { CharacterProfileDetail } from "@/components/studio/tabs/CharacterProfileDetail";
import {
  appendUploadedReferenceAsset,
  resolveReferenceLibrary,
} from "@/components/studio/tabs/character-profile-detail/utils/characterAssets";

const sampleCharacter: Character = {
  id: "soc_nho",
  projectId: "soc-nho-tv",
  name: "Sóc Nhỏ (Bi)",
  role: "main",
  heightCm: 100,
  scaleRatio: 1.0,
  voiceId: "soc-nho",
  description: "Nhân vật chính dễ thương",
  promptLock: "Cute 3D chibi squirrel Bi",
  referenceImages: ["/demo/characters/soc_nho_01.png"],
  referenceLibrary: [
    {
      id: "img_1",
      url: "/demo/characters/soc_nho_01.png",
      angle: "Chính diện",
      isPrimary: true,
      source: "upload",
      createdAt: "2026-11-01T10:00:00Z",
    },
    {
      id: "img_2",
      url: "/demo/characters/soc_nho_02.png",
      angle: "Góc nghiêng",
      isPrimary: false,
      source: "generated",
      model: "gemini-2.5-flash-image",
      prompt: "Side view",
      createdAt: "2026-11-01T11:00:00Z",
    },
  ],
  generatedClips: [
    {
      id: "clip_1",
      url: "/demo/clips/ep1_s1.mp4",
      episodeId: "ep1",
      episodeTitle: "Tập 1",
      sceneLabel: "Cảnh 1",
      durationSec: 6,
      createdAt: "2026-11-01T12:00:00Z",
    },
  ],
  memory: {
    personality: "Tinh nghịch",
    catchphrases: ["Ủa gì zợ!"],
    habits: ["Gãi tai"],
    pastEvents: ["Đi khám răng cùng Koala"],
    pastEventsSummary: ["Đi khám răng cùng Koala"],
  },
  appearances: [
    {
      episodeId: "ep1",
      episodeTitle: "Tập 1: Chiếc Răng Sâu",
      scenes: ["Cảnh 1", "Cảnh 2"],
    },
  ],
};

let voicePreviewCalled = false;

const server = setupServer(
  http.post(`${CORE_URL}/api/voice/preview`, () => {
    voicePreviewCalled = true;
    return new HttpResponse(new Blob(["fake-mp3"], { type: "audio/mpeg" }), { status: 200 });
  })
);

function Harness({
  initialSection = "profile",
  onSaveCharacter = vi.fn().mockResolvedValue(undefined),
  onBackToList = vi.fn(),
  onDropToCanvas = vi.fn(),
}: {
  initialSection?: CharacterProfileSection;
  onSaveCharacter?: (ch: Character) => Promise<void>;
  onBackToList?: () => void;
  onDropToCanvas?: (ch: Character) => void;
}) {
  const [section, setSection] = React.useState<CharacterProfileSection>(initialSection);
  const [charState, setCharState] = React.useState<Character>(sampleCharacter);
  return (
    <NextIntlClientProvider locale="vi" messages={viMessages}>
      <CharacterProfileDetail
        selectedChar={charState}
        activeProjectId="soc-nho-tv"
        activeCharacterSection={section}
        setActiveCharacterSection={setSection}
        onSaveCharacter={async (next) => {
          setCharState(next);
          await onSaveCharacter(next);
        }}
        onBackToList={onBackToList}
        onDropToCanvas={onDropToCanvas}
        onGeneratePhotoByNode={vi.fn()}
        savedToast={true}
      />
    </NextIntlClientProvider>
  );
}

beforeAll(() => {
  server.listen();
  if (!URL.createObjectURL) {
    URL.createObjectURL = vi.fn(() => "blob:mock-audio");
  }
});
beforeEach(() => {
  voicePreviewCalled = false;
  useStudioStore.setState({
    nodes: [
      {
        id: "n_char",
        position: { x: 0, y: 0 },
        data: { nodeType: "character", title: "Nhân vật", params: { characterId: "soc_nho" } },
      },
      {
        id: "n_voice",
        position: { x: 200, y: 0 },
        data: { nodeType: "voice_tts", title: "Giọng đọc", params: { voiceId: "soc-nho" } },
      },
    ] as any,
    setSelectedNodeId: vi.fn(),
    flashNode: vi.fn(),
  } as any);
});
afterEach(() => {
  server.resetHandlers();
  cleanup();
});
afterAll(() => server.close());

describe("CharacterProfileDetail characterization tests", () => {
  it("renders profile fields, updates height scaleRatio, and navigates all 5 sections", async () => {
    const user = userEvent.setup();
    const onSaveCharacter = vi.fn().mockResolvedValue(undefined);
    const onBackToList = vi.fn();
    const onDropToCanvas = vi.fn();

    render(
      <Harness
        onSaveCharacter={onSaveCharacter}
        onBackToList={onBackToList}
        onDropToCanvas={onDropToCanvas}
      />
    );

    // Header + saved toast
    expect(screen.getByText("Sóc Nhỏ (Bi)")).toBeInTheDocument();
    expect(screen.getByText("Đã lưu hồ sơ nhân vật")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /Quay lại danh sách nhân vật/i }));
    expect(onBackToList).toHaveBeenCalledTimes(1);

    await user.click(screen.getByRole("button", { name: /Đặt lên canvas/i }));
    expect(onDropToCanvas).toHaveBeenCalledWith(sampleCharacter);

    // Height change updates scaleRatio cleanly
    const heightInput = screen.getByLabelText("Chiều cao khóa (cm)");
    fireEvent.change(heightInput, { target: { value: "150" } });
    expect(onSaveCharacter).toHaveBeenLastCalledWith(
      expect.objectContaining({ heightCm: 150, scaleRatio: 1.5 })
    );

    // Section B: Images + Lightbox + Set primary + Delete
    await user.click(screen.getByRole("tab", { name: /Ảnh \(2\)/i }));
    expect(screen.getByText("Góc: Chính diện")).toBeInTheDocument();

    await user.click(screen.getAllByRole("button", { name: "Xem lớn" })[0]);
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Đóng" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Đặt làm ảnh chính" }));
    expect(onSaveCharacter).toHaveBeenLastCalledWith(
      expect.objectContaining({
        referenceImages: ["/demo/characters/soc_nho_02.png", "/demo/characters/soc_nho_01.png"],
      })
    );

    await user.click(screen.getAllByRole("button", { name: "Xóa" })[1]);
    expect(onSaveCharacter).toHaveBeenLastCalledWith(
      expect.objectContaining({
        referenceImages: ["/demo/characters/soc_nho_01.png"],
      })
    );

    // Section C: Videos
    await user.click(screen.getByRole("tab", { name: /Video \(1\)/i }));
    expect(screen.getByText("6s")).toBeInTheDocument();

    // Section D: Voice test via OmniVoice
    await user.click(screen.getByRole("tab", { name: "Giọng" }));
    await user.click(screen.getByRole("button", { name: /Đọc thử qua OmniVoice/i }));
    await waitFor(() => expect(voicePreviewCalled).toBe(true));

    // Section E: Usage & referencing nodes
    await user.click(screen.getByRole("tab", { name: "Dùng ở đâu" }));
    expect(screen.getByText("Tập 1: Chiếc Răng Sâu")).toBeInTheDocument();
    expect(screen.getByText("n_char")).toBeInTheDocument();
    expect(screen.getByText("n_voice")).toBeInTheDocument();

    await user.click(screen.getAllByRole("button", { name: "Xem trên canvas" })[0]);
    expect(useStudioStore.getState().setSelectedNodeId).toHaveBeenCalledWith("n_char");
    expect(useStudioStore.getState().flashNode).toHaveBeenCalledWith("n_char");
  });

  it("pure helper resolveReferenceLibrary falls back cleanly and appendUploadedReferenceAsset preserves primary", () => {
    const fallbackLib = resolveReferenceLibrary({
      ...sampleCharacter,
      referenceLibrary: [],
      referenceImages: ["/a.png", "/b.png"],
    });
    expect(fallbackLib).toHaveLength(2);
    expect(fallbackLib[0].isPrimary).toBe(true);
    expect(fallbackLib[0].angle).toBe("Chính diện");

    const updated = appendUploadedReferenceAsset(sampleCharacter, "data:image/png;base64,abc");
    expect(updated.referenceLibrary).toHaveLength(3);
    expect(updated.referenceImages[0]).toBe("/demo/characters/soc_nho_01.png");
  });
});
