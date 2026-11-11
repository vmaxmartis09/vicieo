"use client";

import { useCallback, useState } from "react";
import { showAlert } from "@/lib/confirm";

export function useVoiceDictation(
  input: string,
  onInputChange: (val: string) => void,
  isVi: boolean
) {
  const [isListening, setIsListening] = useState(false);

  const toggleSpeechRecognition = useCallback(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!SpeechRecognition) {
      showAlert(
        "Ghi âm giọng nói",
        isVi ? "Trình duyệt chưa hỗ trợ ghi âm trực tiếp." : "Speech recognition not supported in this browser.",
        "warning"
      );
      return;
    }

    if (isListening) {
      setIsListening(false);
      return;
    }

    try {
      const recognition = new SpeechRecognition();
      recognition.lang = isVi ? "vi-VN" : "en-US";
      recognition.interimResults = false;
      recognition.maxAlternatives = 1;

      recognition.onstart = () => setIsListening(true);
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      recognition.onresult = (event: any) => {
        const transcript = event.results[0][0].transcript;
        if (transcript) {
          onInputChange(input ? `${input} ${transcript}` : transcript);
        }
      };
      recognition.onerror = () => setIsListening(false);
      recognition.onend = () => setIsListening(false);

      recognition.start();
    } catch {
      setIsListening(false);
    }
  }, [input, isListening, isVi, onInputChange]);

  return { isListening, toggleSpeechRecognition };
}
