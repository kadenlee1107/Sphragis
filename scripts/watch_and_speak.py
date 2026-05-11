#!/usr/bin/env python3
"""Watch the active Claude Code session JSONL and speak new assistant text.

Tails the most-recently-modified session file under
`~/.claude/projects/-Users-kadenlee-Bat-OS/`. For every new line whose
`type == "assistant"`, it pulls the text blocks out of `message.content`,
strips non-speakable Markdown (asterisks, code fences, table cells,
link syntax, etc.), and pipes the cleaned text through `voice_claude.py`
which streams it sentence-by-sentence to the remote Chatterbox server.

Usage:
  scripts/watch_and_speak.py
  scripts/watch_and_speak.py --from-start    # speak the existing transcript too

Stop with Ctrl-C.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from pathlib import Path

PROJECTS_DIR = Path.home() / ".claude" / "projects" / "-Users-kadenlee-Bat-OS"
VOICE_CMD = [sys.executable, "-u", str(Path(__file__).resolve().parent / "voice_claude.py")]

# Strip Markdown that doesn't make sense spoken. Tables and code blocks get
# omitted entirely because spelling out pipe chars and backticks is awful.
CODE_BLOCK   = re.compile(r"```[\s\S]*?```", re.MULTILINE)
INLINE_CODE  = re.compile(r"`([^`]+)`")
BOLD         = re.compile(r"\*{1,3}([^*\n]+)\*{1,3}")
ITALIC_UND   = re.compile(r"_+([^_\n]+)_+")
MD_LINK      = re.compile(r"\[([^\]]+)\]\([^)]+\)")
HEADER       = re.compile(r"^#+\s+", re.MULTILINE)
TABLE_ROW    = re.compile(r"^\s*\|.*\|\s*$", re.MULTILINE)
HR           = re.compile(r"^\s*---+\s*$", re.MULTILINE)
BULLET       = re.compile(r"^\s*[-*+]\s+", re.MULTILINE)
NUMBERED     = re.compile(r"^\s*\d+\.\s+", re.MULTILINE)
HTML_TAG     = re.compile(r"<[^>]+>")
MULTIBLANK   = re.compile(r"\n{3,}")


def speakable(text: str) -> str:
    text = CODE_BLOCK.sub(" ", text)
    text = INLINE_CODE.sub(r"\1", text)
    text = BOLD.sub(r"\1", text)
    text = ITALIC_UND.sub(r"\1", text)
    text = MD_LINK.sub(r"\1", text)
    text = HEADER.sub("", text)
    text = TABLE_ROW.sub("", text)
    text = HR.sub("", text)
    text = BULLET.sub("", text)
    text = NUMBERED.sub("", text)
    text = HTML_TAG.sub("", text)
    text = MULTIBLANK.sub("\n\n", text)
    return text.strip()


def latest_session() -> Path | None:
    files = sorted(
        PROJECTS_DIR.glob("*.jsonl"),
        key=lambda p: p.stat().st_mtime,
        reverse=True,
    )
    return files[0] if files else None


def tail(path: Path, *, start_offset: int):
    """Yield raw line bytes as they're appended. Detects file rotation."""
    pos = start_offset
    seen_path = path
    while True:
        # If the active session changed (newer JSONL appeared), follow it.
        newest = latest_session()
        if newest and newest != seen_path:
            print(f"[speak] following new session: {newest.name}", file=sys.stderr)
            seen_path = newest
            pos = 0
        try:
            size = seen_path.stat().st_size
        except FileNotFoundError:
            time.sleep(0.5)
            continue
        if size < pos:
            pos = 0          # truncation/rotation in place
        if size > pos:
            with open(seen_path, "rb") as f:
                f.seek(pos)
                chunk = f.read()
                pos = f.tell()
            for raw in chunk.splitlines():
                if raw:
                    yield raw
        time.sleep(0.5)


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument(
        "--from-start",
        action="store_true",
        help="speak every assistant turn in the existing transcript before tailing",
    )
    p.add_argument("--host", default="127.0.0.1")
    p.add_argument("--port", type=int, default=5005)
    args = p.parse_args()

    path = latest_session()
    if path is None:
        print(f"[speak] no session jsonl in {PROJECTS_DIR}", file=sys.stderr)
        return 1
    print(f"[speak] watching {path.name}", file=sys.stderr)

    start = 0 if args.from_start else path.stat().st_size

    voice = subprocess.Popen(
        VOICE_CMD + ["--host", args.host, "--port", str(args.port)],
        stdin=subprocess.PIPE,
        text=True,
        bufsize=1,
    )

    try:
        for raw in tail(path, start_offset=start):
            try:
                entry = json.loads(raw)
            except json.JSONDecodeError:
                continue
            if entry.get("type") != "assistant":
                continue
            for block in entry.get("message", {}).get("content", []):
                if not isinstance(block, dict):
                    continue
                if block.get("type") == "text":
                    spoken = speakable(block.get("text", ""))
                    if spoken:
                        assert voice.stdin is not None
                        voice.stdin.write(spoken + "\n\n")
                        voice.stdin.flush()
    except KeyboardInterrupt:
        pass
    finally:
        if voice.stdin:
            voice.stdin.close()
        voice.wait()

    return 0


if __name__ == "__main__":
    sys.exit(main())
