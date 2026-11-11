"use client";

import React from "react";
import type { CharacterProfileDetailProps } from "./types";
import { resolveReferenceLibrary } from "./utils/characterAssets";
import { ProfileHeaderBar } from "./components/ProfileHeaderBar";
import { SectionProfileForm } from "./components/SectionProfileForm";
import { SectionImagesGallery } from "./components/SectionImagesGallery";
import { SectionVideosClips } from "./components/SectionVideosClips";
import { SectionVoiceCloning } from "./components/SectionVoiceCloning";
import { SectionUsageNodes } from "./components/SectionUsageNodes";

export type { CharacterProfileDetailProps } from "./types";

export function CharacterProfileDetail({
  selectedChar,
  activeProjectId,
  activeCharacterSection,
  setActiveCharacterSection,
  onSaveCharacter,
  onBackToList,
  onDropToCanvas,
  onGeneratePhotoByNode,
  savedToast,
}: CharacterProfileDetailProps) {
  const refLib = resolveReferenceLibrary(selectedChar);
  const clips = selectedChar.generatedClips || [];

  return (
    <div className="p-4 space-y-4 text-footnote">
      <ProfileHeaderBar
        selectedChar={selectedChar}
        savedToast={savedToast}
        refCount={refLib.length}
        clipsCount={clips.length}
        activeCharacterSection={activeCharacterSection}
        onSelectSection={setActiveCharacterSection}
        onBackToList={onBackToList}
        onDropToCanvas={onDropToCanvas}
      />

      {activeCharacterSection === "profile" && (
        <SectionProfileForm
          selectedChar={selectedChar}
          onSaveCharacter={onSaveCharacter}
        />
      )}

      {activeCharacterSection === "images" && (
        <SectionImagesGallery
          selectedChar={selectedChar}
          refLib={refLib}
          onSaveCharacter={onSaveCharacter}
          onGeneratePhotoByNode={onGeneratePhotoByNode}
        />
      )}

      {activeCharacterSection === "videos" && (
        <SectionVideosClips clips={clips} />
      )}

      {activeCharacterSection === "voice" && (
        <SectionVoiceCloning
          selectedChar={selectedChar}
          activeProjectId={activeProjectId}
          onSaveCharacter={onSaveCharacter}
        />
      )}

      {activeCharacterSection === "usage" && (
        <SectionUsageNodes selectedChar={selectedChar} />
      )}
    </div>
  );
}
