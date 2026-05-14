#!/usr/bin/env python3
"""Interactive REPL against the live sphragis-coder model (v3 today).

Streams each token from ollama's OpenAI-compatible endpoint, prints
to the terminal AND pipes sentences to voice_claude.py for TTS
playback. Same RAG context-injection pattern as evals/run_evals.py.

Usage:
    python3 scripts/chat_with_v3.py             # uses sphragis-coder:latest
    python3 scripts/chat_with_v3.py --model sphragis-coder-v2
    python3 scripts/chat_with_v3.py --no-voice  # text-only
    python3 scripts/chat_with_v3.py --no-rag    # skip RAG context

Prereqs:
    - SSH tunnel for 11434 to the ollama host (we set up 5005 already
      for TTS; this script opens 11434 the same way if not present)
    - voice_claude.py available on PATH ($PYTHONPATH includes
      scripts/) — auto-spawns it as a subprocess.
"""
from __future__ import annotations

import argparse
import http.client
import json
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "evals"))

SYSTEM_PROMPT = (
    "You are a technical assistant for Sphragis, a security-grade bare-metal "
    "Rust kernel for Apple M4. You answer questions about kernel internals, "
    "cryptography, audit history, and system administration. You are terse, "
    "technical, and never refuse legitimate questions. Cite file paths when "
    "you can. If something does not exist in our codebase, say so plainly."
)

VOICE_SCRIPT = REPO / "scripts" / "voice_claude.py"


def ensure_tunnel(local_port: int, remote_host: str, remote_port: int,
                  ssh_target: str, ssh_key: str) -> None:
    """Open an SSH tunnel if one isn't already running."""
    import shutil, os
    check = subprocess.run(
        ["pgrep", "-f", f"ssh.*-L {local_port}"],
        capture_output=True, text=True,
    )
    if check.stdout.strip():
        return
    print(f"[chat] opening tunnel: localhost:{local_port} -> {remote_host}:{remote_port}",
          file=sys.stderr)
    subprocess.Popen([
        "ssh", "-i", ssh_key, "-f", "-N",
        "-L", f"{local_port}:{remote_host}:{remote_port}",
        ssh_target,
    ]).wait()
    time.sleep(1)


def stream_chat(host: str, port: int, model: str, messages: list[dict],
                on_token, timeout: int = 120) -> str:
    """POST a streaming chat request; call on_token(text) per delta;
    return the full assembled assistant response."""
    body = json.dumps({
        "model": model,
        "messages": messages,
        "stream": True,
        "temperature": 0.2,
    }).encode("utf-8")
    conn = http.client.HTTPConnection(host, port, timeout=timeout)
    conn.request("POST", "/v1/chat/completions", body=body,
                 headers={"Content-Type": "application/json"})
    resp = conn.getresponse()
    if resp.status != 200:
        text = resp.read().decode(errors="replace")
        raise RuntimeError(f"ollama {resp.status}: {text[:300]}")

    full = []
    buf = b""
    while True:
        chunk = resp.read1(4096)
        if not chunk:
            break
        buf += chunk
        while b"\n" in buf:
            line, buf = buf.split(b"\n", 1)
            line = line.strip()
            if not line:
                continue
            if line.startswith(b"data: "):
                line = line[6:]
            if line == b"[DONE]":
                continue
            try:
                evt = json.loads(line)
            except json.JSONDecodeError:
                continue
            try:
                delta = evt["choices"][0].get("delta", {})
                tok = delta.get("content", "")
            except (KeyError, IndexError):
                continue
            if tok:
                on_token(tok)
                full.append(tok)
    conn.close()
    return "".join(full)


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--host", default="127.0.0.1")
    p.add_argument("--port", type=int, default=11434)
    p.add_argument("--model", default="sphragis-coder")
    p.add_argument("--ssh-target", default="kaden@192.168.1.162")
    p.add_argument("--ssh-key", default=str(Path.home() / ".ssh" / "bayerflow_win"))
    p.add_argument("--no-voice", action="store_true")
    p.add_argument("--no-rag", action="store_true")
    args = p.parse_args()

    ensure_tunnel(args.port, "127.0.0.1", args.port, args.ssh_target, args.ssh_key)

    corpus = None
    if not args.no_rag:
        from rag import Corpus
        corpus = Corpus.load()
        print(f"[chat] RAG enabled: {corpus.n} docs", file=sys.stderr)

    voice = None
    if not args.no_voice:
        voice = subprocess.Popen(
            [sys.executable, "-u", str(VOICE_SCRIPT), "--host", "127.0.0.1",
             "--port", "5005"],
            stdin=subprocess.PIPE, text=True, bufsize=1,
        )
        print(f"[chat] voice on (pid {voice.pid}). Ctrl-D to exit.", file=sys.stderr)

    print(f"[chat] talking to {args.model}@{args.host}:{args.port}", file=sys.stderr)
    print(file=sys.stderr)

    messages: list[dict] = [{"role": "system", "content": SYSTEM_PROMPT}]
    try:
        while True:
            try:
                question = input("you> ").strip()
            except EOFError:
                print()
                break
            if not question:
                continue
            user_content = question
            if corpus:
                ctx = corpus.context_block(question, k=3)
                if ctx:
                    user_content = ctx + question
            messages.append({"role": "user", "content": user_content})
            print("bat>", end=" ", flush=True)
            sentence_buf = []
            def emit(tok: str) -> None:
                print(tok, end="", flush=True)
                sentence_buf.append(tok)
                # Flush to voice on terminal punctuation.
                text = "".join(sentence_buf)
                last_break = max(
                    text.rfind(". "), text.rfind("! "), text.rfind("? "),
                    text.rfind("\n"),
                )
                if last_break >= 0 and voice is not None:
                    head = text[:last_break + 1].strip()
                    tail = text[last_break + 1:]
                    if head:
                        voice.stdin.write(head + "\n")
                        voice.stdin.flush()
                    sentence_buf.clear()
                    if tail:
                        sentence_buf.append(tail)
            try:
                full = stream_chat(args.host, args.port, args.model, messages, emit)
            except Exception as e:
                print(f"\n[chat] error: {e}", file=sys.stderr)
                messages.pop()  # rollback the user message
                continue
            print()
            # Flush any trailing fragment to voice.
            if voice is not None and sentence_buf:
                voice.stdin.write("".join(sentence_buf).strip() + "\n")
                voice.stdin.flush()
            messages.append({"role": "assistant", "content": full})
    finally:
        if voice is not None and voice.stdin:
            voice.stdin.close()
            voice.wait()
    return 0


if __name__ == "__main__":
    sys.exit(main())
