import { useCallback, useState } from "react";
import { CORE_URL } from "@/lib/repositories/core";

export function useCharacterVoiceTest(params: {
  voiceId: string;
  sampleAudioUrl?: string;
  defaultSentence: string;
}) {
  const { voiceId, sampleAudioUrl, defaultSentence } = params;
  const [ttsSentence, setTtsSentence] = useState(defaultSentence);
  const [ttsTempo, setTtsTempo] = useState(1.1);
  const [ttsPreviewUrl, setTtsPreviewUrl] = useState<string | null>(null);
  const [ttsSpeaking, setTtsSpeaking] = useState(false);

  const handleTestOmniVoice = useCallback(async () => {
    setTtsSpeaking(true);
    setTtsPreviewUrl(null);
    try {
      const res = await fetch(`${CORE_URL}/api/voice/preview`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          voice_id: voiceId,
          text: ttsSentence,
          tempo: ttsTempo,
        }),
      });
      if (!res.ok) throw new Error("TTS preview failed");
      const blob = await res.blob();
      const url = URL.createObjectURL(blob);
      setTtsPreviewUrl(url);
    } catch {
      setTtsPreviewUrl(sampleAudioUrl || "/demo/audio/soc-nho_sample.mp3");
    } finally {
      setTtsSpeaking(false);
    }
  }, [voiceId, ttsSentence, ttsTempo, sampleAudioUrl]);

  return {
    ttsSentence,
    setTtsSentence,
    ttsTempo,
    setTtsTempo,
    ttsPreviewUrl,
    ttsSpeaking,
    handleTestOmniVoice,
  };
}
