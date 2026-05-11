#!/usr/bin/env python3
"""Mac-side wrapper that streams Claude Code's text output to a
remote Chatterbox TTS server, plays audio sentence-by-sentence so
the voice keeps up with the text.

Usage:
  voice_claude.py [--host HOST] [--port PORT] [--ref PATH]
                  [--input -|FILE]

Reads stdin (or a file) line by line, buffers into sentences,
and pipes each sentence to the TTS server over a WebSocket.

The server is at scripts/deploy_chatterbox.sh.
"""
from __future__ import annotations

import argparse
import asyncio
import json
import re
import sys
from queue import Queue
from threading import Thread

import numpy as np
import sounddevice as sd
import websockets

DEFAULT_HOST = "192.168.1.162"
DEFAULT_PORT = 5005
DEFAULT_SAMPLE_RATE = 24000  # Chatterbox default; server reports actual

SENTENCE_END = re.compile(r"[.!?]\s+|[.!?]$|\n\n")
SOFT_BREAK   = re.compile(r"[,;:]\s+")
SOFT_MIN_LEN = 60


def split_sentences(buf: str) -> tuple[list[str], str]:
    """Greedily peel off complete sentences. Return (sentences, leftover)."""
    sentences: list[str] = []
    while True:
        m = SENTENCE_END.search(buf)
        if m:
            cut = m.end()
            piece = buf[:cut].strip()
            if piece:
                sentences.append(piece)
            buf = buf[cut:]
            continue
        # No hard sentence end. If we've accumulated enough text and
        # we hit a soft break, flush early so the audio doesn't lag
        # behind by paragraphs.
        if len(buf) >= SOFT_MIN_LEN:
            m = SOFT_BREAK.search(buf, SOFT_MIN_LEN)
            if m:
                cut = m.end()
                piece = buf[:cut].strip()
                if piece:
                    sentences.append(piece)
                buf = buf[cut:]
                continue
        break
    return sentences, buf


def player_thread(q: Queue, sr: int):
    """Continuous-play thread. Pulls PCM int16 chunks off q, writes
    them into a sounddevice OutputStream. Sentinels: None to stop."""
    with sd.OutputStream(samplerate=sr, channels=1, dtype="int16",
                         blocksize=0, latency="low") as out:
        while True:
            chunk = q.get()
            if chunk is None:
                return
            out.write(chunk)


async def tts_loop(host: str, port: int, ref: str | None,
                   text_q: asyncio.Queue, audio_q: Queue):
    uri = f"ws://{host}:{port}/stream"
    async with websockets.connect(uri, max_size=None) as ws:
        while True:
            sentence = await text_q.get()
            if sentence is None:
                audio_q.put(None)
                return
            payload = {"text": sentence}
            if ref:
                payload["audio_prompt_path"] = ref
            await ws.send(json.dumps(payload))
            pcm_bytes = await ws.recv()
            arr = np.frombuffer(pcm_bytes, dtype=np.int16)
            audio_q.put(arr)


async def amain(args):
    text_q: asyncio.Queue[str | None] = asyncio.Queue()
    audio_q: Queue = Queue()
    sr = args.sample_rate

    player = Thread(target=player_thread, args=(audio_q, sr), daemon=True)
    player.start()

    tts_task = asyncio.create_task(tts_loop(args.host, args.port, args.ref, text_q, audio_q))

    src = sys.stdin if args.input == "-" else open(args.input, "r", encoding="utf-8")
    buf = ""
    loop = asyncio.get_event_loop()
    try:
        while True:
            chunk = await loop.run_in_executor(None, src.readline)
            if not chunk:
                break
            sys.stdout.write(chunk)
            sys.stdout.flush()
            buf += chunk
            sentences, buf = split_sentences(buf)
            for s in sentences:
                await text_q.put(s)
    finally:
        if buf.strip():
            await text_q.put(buf.strip())
        await text_q.put(None)
        await tts_task


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--host", default=DEFAULT_HOST)
    p.add_argument("--port", type=int, default=DEFAULT_PORT)
    p.add_argument("--ref", default=None,
                   help="server-side path to a reference audio clip for voice cloning")
    p.add_argument("--sample-rate", type=int, default=DEFAULT_SAMPLE_RATE)
    p.add_argument("--input", default="-",
                   help="input file or '-' for stdin")
    args = p.parse_args()
    asyncio.run(amain(args))


if __name__ == "__main__":
    main()
