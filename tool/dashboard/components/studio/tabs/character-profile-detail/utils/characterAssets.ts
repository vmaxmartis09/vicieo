import type { StudioStore } from "@/store";
import type { Character, CharacterImageAsset } from "@/lib/repositories/types";

export function resolveReferenceLibrary(selectedChar: Character): CharacterImageAsset[] {
  if (selectedChar.referenceLibrary && selectedChar.referenceLibrary.length > 0) {
    return selectedChar.referenceLibrary;
  }
  return selectedChar.referenceImages.map((url, idx) => ({
    id: `img_${idx}`,
    url,
    angle: idx === 0 ? "Chính diện" : `Góc ${idx + 1}`,
    isPrimary: idx === 0,
    source: "upload" as const,
    createdAt: "2026-11-01T10:00:00Z",
  }));
}

export function appendUploadedReferenceAsset(
  selectedChar: Character,
  dataUrl: string
): Character {
  const newAsset: CharacterImageAsset = {
    id: `ref_${Date.now().toString(36)}`,
    url: dataUrl,
    angle: "Tải lên mới",
    isPrimary: false,
    source: "upload",
    createdAt: new Date().toISOString(),
  };
  const lib = [...(selectedChar.referenceLibrary || []), newAsset];
  return {
    ...selectedChar,
    referenceImages: [
      selectedChar.referenceImages[0] || dataUrl,
      ...lib.map((x) => x.url).slice(1),
    ],
    referenceLibrary: lib,
  };
}

export function markPrimaryReferenceAsset(
  selectedChar: Character,
  assetId: string
): Character {
  const lib = (selectedChar.referenceLibrary || []).map((item) => ({
    ...item,
    isPrimary: item.id === assetId,
  }));
  const primary = lib.find((x) => x.isPrimary);
  const sortedImages = primary
    ? [primary.url, ...lib.filter((x) => !x.isPrimary).map((x) => x.url)]
    : selectedChar.referenceImages;
  return {
    ...selectedChar,
    referenceLibrary: lib,
    referenceImages: sortedImages,
  };
}

export function removeReferenceAsset(
  selectedChar: Character,
  assetId: string
): Character {
  const lib = (selectedChar.referenceLibrary || []).filter((x) => x.id !== assetId);
  return {
    ...selectedChar,
    referenceLibrary: lib,
    referenceImages: lib.map((x) => x.url),
  };
}

export function findReferencingNodes(
  nodes: StudioStore["nodes"],
  selectedChar: Character
) {
  return nodes.filter(
    (n) =>
      (n.data.nodeType === "character" &&
        n.data.params?.characterId === selectedChar.id) ||
      (n.data.nodeType === "voice_tts" &&
        n.data.params?.voiceId === selectedChar.voiceId)
  );
}
