#!/usr/bin/env python3
"""Whisper ASR sidecar: `whisper_sidecar.py <audio.wav> <lang|auto> <model>` -> JSON segments on stdout."""
import json
import sys


def main() -> int:
    audio, lang, model_name = sys.argv[1], sys.argv[2], sys.argv[3]
    from faster_whisper import WhisperModel

    model = WhisperModel(model_name, device="cpu", compute_type="int8")
    segments, _ = model.transcribe(audio, language=None if lang == "auto" else lang, word_timestamps=True)
    out = []
    for s in segments:
        words = [{"word": w.word, "start": round(w.start, 2), "end": round(w.end, 2), "probability": round(w.probability, 2)} for w in (s.words or [])]
        out.append({"start": round(s.start, 2), "end": round(s.end, 2), "text": s.text.strip(), "words": words})
    json.dump(out, sys.stdout, ensure_ascii=False)
    return 0


if __name__ == "__main__":
    sys.exit(main())
