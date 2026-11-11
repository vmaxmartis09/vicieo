import type { CharacterProfileSection } from "@/store/slices/rightPanelSlice";
import type { Character } from "@/lib/repositories/types";

export interface CharacterProfileDetailProps {
  selectedChar: Character;
  activeProjectId: string;
  activeCharacterSection: CharacterProfileSection;
  setActiveCharacterSection: (s: CharacterProfileSection) => void;
  onSaveCharacter: (ch: Character) => Promise<void>;
  onBackToList: () => void;
  onDropToCanvas: (ch: Character) => void;
  onGeneratePhotoByNode: (ch: Character) => void;
  savedToast: boolean;
}
