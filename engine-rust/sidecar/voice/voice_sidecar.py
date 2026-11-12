#!/usr/bin/env python3
"""Thin JSON-in/JSON-out wrapper over gobi VoiceCloner (OmniVoice TTS/clone).

usage: voice_sidecar.py say|clone < request.json   -> last stdout line is a JSON object.
Env: VOICE_GOBI_SRC (dir containing the `gobi` package, default /Volumes/Data/socnhotv/src),
     VOICE_WHISPER_MODEL (default "base"). Run with the repo root .venv python.
"""
import json, os, subprocess, sys

sys.path.insert(0, os.environ.get("VOICE_GOBI_SRC", "/Volumes/Data/socnhotv/src"))
for s in (sys.stdout, sys.stderr):
    s.reconfigure(encoding="utf-8")


def duration(path):
    out = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", path],
                         capture_output=True, text=True, check=True).stdout.strip()
    return round(float(out), 3)


def words(path):
    from faster_whisper import WhisperModel
    m = WhisperModel(os.environ.get("VOICE_WHISPER_MODEL", "base"), device="cpu", compute_type="int8")
    # decode via ffmpeg: the venv's PyAV rejects faster_whisper's open() kwargs
    import numpy as np
    pcm = subprocess.run(["ffmpeg", "-v", "error", "-i", path, "-f", "s16le", "-ac", "1", "-ar", "16000", "-"],
                         capture_output=True, check=True).stdout
    segs, _ = m.transcribe(np.frombuffer(pcm, np.int16).astype(np.float32) / 32768.0, language="vi", word_timestamps=True)
    return [{"word": w.word.strip(), "start": round(w.start, 3), "end": round(w.end, 3)}
            for s in segs for w in (s.words or [])]


def say(req):
    from gobi.services.voice import VoiceCloner
    out = req["out"]
    os.makedirs(os.path.dirname(out), exist_ok=True)
    VoiceCloner(profiles_dir=req["vault"]).speak(
        req["voiceId"], req["text"], out, engine="omnivoice", master=True, tempo_factor=float(req.get("tempo") or 1.05))
    if not os.path.isfile(out) or os.path.getsize(out) == 0:
        raise RuntimeError("TTS produced no audio")
    res = {"durationSec": duration(out), "words": []}
    if req.get("words", True):
        try:
            res["words"] = words(out)
        except Exception as e:  # karaoke timing is best-effort; audio is still valid
            res["wordsError"] = str(e)[:200]
    return res


def clone(req):
    from gobi.services.voice import VoiceCloner
    meta = VoiceCloner(profiles_dir=req["vault"]).create_profile(
        media_path=req["source"], profile_id=req["voiceId"], dialect=req.get("dialect", "south"),
        speaker_name=req.get("name") or req["voiceId"], custom_transcript=req.get("transcript"))
    return {"name": meta.get("speaker_name"), "durationSec": meta.get("duration"), "transcript": meta.get("transcript", "")}


if __name__ == "__main__":
    try:
        req = json.load(sys.stdin)
        print(json.dumps({"ok": True, **{"say": say, "clone": clone}[sys.argv[1]](req)}, ensure_ascii=False))
    except Exception as e:
        print(json.dumps({"ok": False, "error": f"{type(e).__name__}: {e}"[:600]}, ensure_ascii=False))
        sys.exit(1)
