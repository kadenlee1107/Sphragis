#!/usr/bin/env bash
# Deploy Chatterbox TTS server on the inference host (the 5070 box).
#
# Stands up a FastAPI server with two endpoints:
#   POST /tts     — one-shot, returns audio/wav of the rendered text
#   WS   /stream  — sentence-by-sentence streaming, returns raw int16 PCM
# Both keep the model warm in VRAM so subsequent inferences are sub-second.
#
# Lessons baked in from the first deploy:
#   * Use python3 (not python3.11). Ubuntu 24.04 ships 3.12 by default.
#   * Patch perth.PerthImplicitWatermarker — it's None on systems without
#     the TF-based watermarker backend (which won't install on a clean
#     CUDA wheel). Substitute the no-op DummyWatermarker.
#   * Upgrade torch to 2.11.0+cu128 AFTER chatterbox-tts installs. The
#     pinned 2.6.0+cu124 does not have kernels for sm_120 (Blackwell /
#     RTX 5070) and crashes on first GPU op.
#   * Use soundfile.write to render WAV. torchaudio.save now requires
#     torchcodec, which isn't bundled.
#
# Run on the inference host (in WSL):
#   bash scripts/deploy_chatterbox.sh
#
# After this finishes, the Mac side can hit:
#   ws://<host>:5005/stream  — sentence-by-sentence streaming
#   POST <host>:5005/tts     — one-shot generate
#
# Voice cloning: drop a 5-10 second clean reference clip at
# /mnt/d/ai/tts/refs/default.wav. The server will use it as the
# audio prompt for every call that doesn't specify its own.
set -euo pipefail

VENV="$HOME/chatterbox-venv"
SERVER_PY=/mnt/d/ai/tts/server.py
PORT="${PORT:-5005}"

mkdir -p /mnt/d/ai/tts /mnt/d/ai/tts/refs

echo "[chatterbox] step 1/4 — venv + base deps"
if [ ! -f "$VENV/bin/python" ]; then
    python3 -m venv "$VENV"
fi
"$VENV/bin/pip" install --quiet --upgrade pip
"$VENV/bin/pip" install --quiet \
    chatterbox-tts \
    fastapi \
    uvicorn \
    websockets \
    soundfile \
    numpy \
    resemble-perth

echo "[chatterbox] step 2/4 — upgrade torch to cu128 for sm_120 (Blackwell)"
"$VENV/bin/pip" install --quiet --upgrade \
    --index-url https://download.pytorch.org/whl/cu128 \
    torch torchaudio torchvision
"$VENV/bin/python" -c "import torch; assert torch.cuda.is_available(), 'CUDA not available'; print('  torch', torch.__version__, 'cuda', torch.version.cuda)"

echo "[chatterbox] step 3/4 — write FastAPI server"
cat > "$SERVER_PY" <<'PYEOF'
"""Chatterbox TTS server. Keeps the model warm; serves /tts and /stream."""
import asyncio
import io
import time
from pathlib import Path

import numpy as np
import soundfile as sf
import torch

# Substitute the no-op DummyWatermarker for the TF-backed one when the
# latter fails to import (e.g. on the CUDA wheel that doesn't ship TF).
import perth
if perth.PerthImplicitWatermarker is None:
    perth.PerthImplicitWatermarker = perth.DummyWatermarker

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.responses import StreamingResponse
from pydantic import BaseModel
from chatterbox.tts import ChatterboxTTS

device = "cuda" if torch.cuda.is_available() else (
    "mps" if torch.backends.mps.is_available() else "cpu"
)
print(f"[chatterbox-server] loading model on {device}", flush=True)
t0 = time.time()
model = ChatterboxTTS.from_pretrained(device=device)
print(f"[chatterbox-server] loaded in {time.time() - t0:.1f}s, sr={model.sr}", flush=True)

REF_DIR = Path("/mnt/d/ai/tts/refs")
REF_DEFAULT = REF_DIR / "default.wav"

app = FastAPI()


class SpeakReq(BaseModel):
    text: str
    audio_prompt_path: str | None = None
    exaggeration: float = 0.4
    cfg_weight: float = 0.5
    temperature: float = 0.7


def _generate(text, ref, exaggeration, cfg_weight, temperature):
    kwargs = dict(text=text, exaggeration=exaggeration,
                  cfg_weight=cfg_weight, temperature=temperature)
    if ref:
        kwargs["audio_prompt_path"] = ref
    elif REF_DEFAULT.exists():
        kwargs["audio_prompt_path"] = str(REF_DEFAULT)
    return model.generate(**kwargs)


@app.post("/tts")
def tts(req: SpeakReq):
    wav = _generate(req.text, req.audio_prompt_path,
                    req.exaggeration, req.cfg_weight, req.temperature)
    buf = io.BytesIO()
    samples = wav.squeeze(0).cpu().numpy()
    sf.write(buf, samples, model.sr, format="WAV", subtype="PCM_16")
    buf.seek(0)
    return StreamingResponse(buf, media_type="audio/wav")


@app.websocket("/stream")
async def stream(ws: WebSocket):
    """One text frame in, one PCM frame back. PCM is little-endian int16,
    mono, at model.sr Hz. Frames pipeline naturally — client can send the
    next sentence while playing the previous one."""
    await ws.accept()
    print("[stream] client connected", flush=True)
    try:
        while True:
            msg = await ws.receive_json()
            text = msg.get("text", "").strip()
            if not text:
                continue
            ref = msg.get("audio_prompt_path")
            ex = float(msg.get("exaggeration", 0.4))
            cw = float(msg.get("cfg_weight", 0.5))
            t  = float(msg.get("temperature", 0.7))
            t0 = time.time()
            wav = await asyncio.to_thread(_generate, text, ref, ex, cw, t)
            dt = time.time() - t0
            pcm = (wav.squeeze(0).cpu().numpy() * 32767).astype(np.int16).tobytes()
            print(f"[stream] {len(text.split())}w in {dt:.2f}s -> {len(pcm)} bytes",
                  flush=True)
            await ws.send_bytes(pcm)
    except WebSocketDisconnect:
        print("[stream] client disconnected", flush=True)


@app.get("/health")
def health():
    return {"ok": True, "device": device, "sample_rate": model.sr}
PYEOF

echo "[chatterbox] step 4/4 — start server on :$PORT"
pkill -f uvicorn 2>/dev/null || true
sleep 1
nohup "$VENV/bin/uvicorn" --app-dir /mnt/d/ai/tts server:app \
    --host 0.0.0.0 --port "$PORT" \
    > /mnt/d/ai/tts/server.log 2>&1 &
SERVER_PID=$!
echo "[chatterbox] PID $SERVER_PID, log /mnt/d/ai/tts/server.log"
sleep 10
echo "[chatterbox] health check:"
curl -sf "http://127.0.0.1:$PORT/health" || echo "[chatterbox] health check failed; see log"
