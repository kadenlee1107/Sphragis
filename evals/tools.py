"""Tool implementations for the Sphragis AI agent — Python side.

Mirrors the six read-only tools the design specifies (see
src/ai/tools.rs for the kernel-side definitions). Used by the eval
harness to measure the impact of giving the model real source-tree
access, before the kernel-side dispatch in tools.rs is wired.

Tool catalog:
    read_file         — return UTF-8 file contents (clipped)
    grep_source       — substring search across src/
    query_audit_ring  — stub: no live audit ring on the Mac
    suggest_command   — pass-through (no execution)
    read_concept_note — return one Concept note from the vault corpus
    list_caves        — stub: no caves on the Mac

The two stubs return polite "not available on the eval host" results
so the model can recover (e.g. fall back to grep_source) without
crashing the tool loop.
"""
from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parent.parent
SRC  = REPO / "src"
CORPUS_DIR = REPO / "docs" / "rag_corpus"

MAX_FILE_BYTES = 4096  # cap each read so the context window stays manageable
MAX_GREP_HITS  = 20


def _bounded_read(path: Path, max_bytes: int = MAX_FILE_BYTES) -> str:
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except FileNotFoundError:
        raise FileNotFoundError(f"{path}")
    if len(text) > max_bytes:
        return text[:max_bytes] + "\n…(truncated)"
    return text


def tool_read_file(args: dict) -> str:
    """args: {path: string}. Returns the file's text, capped."""
    raw = args.get("path", "")
    # Reject anything that escapes the repo root.
    p = (REPO / raw).resolve()
    try:
        p.relative_to(REPO.resolve())
    except ValueError:
        return json.dumps({"error": f"path outside repo: {raw}"})
    if not p.exists():
        return json.dumps({"error": f"not found: {raw}"})
    if not p.is_file():
        return json.dumps({"error": f"not a file: {raw}"})
    try:
        body = _bounded_read(p)
    except Exception as e:
        return json.dumps({"error": str(e)})
    return json.dumps({"path": str(p.relative_to(REPO)), "content": body})


def tool_grep_source(args: dict) -> str:
    """args: {pattern: string, path_glob?: string}.
    Substring search via `grep -nF` for safety (no regex metachars)."""
    pattern = args.get("pattern", "")
    if not pattern:
        return json.dumps({"error": "missing pattern"})
    glob = args.get("path_glob", "**/*.rs")
    targets = sorted(SRC.rglob(glob.replace("src/", "")))
    if not targets:
        targets = list(SRC.rglob("*.rs"))
    matches: list[dict] = []
    for f in targets:
        if not f.is_file():
            continue
        try:
            for i, line in enumerate(f.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
                if pattern in line:
                    matches.append({
                        "path": str(f.relative_to(REPO)),
                        "line": i,
                        "content": line.strip()[:200],
                    })
                    if len(matches) >= MAX_GREP_HITS:
                        return json.dumps({"matches": matches, "truncated": True})
        except Exception:
            continue
    return json.dumps({"matches": matches})


def tool_query_audit_ring(args: dict) -> str:
    """Stub: no live audit ring on the Mac eval host. Returns empty."""
    return json.dumps({
        "entries": [],
        "note": "audit ring not available outside the running kernel",
    })


def tool_suggest_command(args: dict) -> str:
    """Pass-through. Eval doesn't execute; just records the suggestion."""
    return json.dumps({
        "command": "",
        "explanation": "suggest_command is informational; not executed in eval",
    })


def tool_read_concept_note(args: dict) -> str:
    """args: {name: string}. Looks up by slug fragment in docs/rag_corpus/."""
    name = args.get("name", "").lower().strip()
    if not name:
        return json.dumps({"error": "missing name"})
    needle = re.sub(r"[^a-z0-9]+", "_", name).strip("_")
    for p in sorted(CORPUS_DIR.glob("concept_*.md")):
        slug = p.stem.removeprefix("concept_")
        if needle in slug or slug in needle:
            return json.dumps({"name": p.stem, "content": _bounded_read(p, max_bytes=4096)})
    return json.dumps({"error": f"no concept note matched '{name}'"})


def tool_list_caves(args: dict) -> str:
    """Stub: no caves on the Mac eval host."""
    return json.dumps({
        "caves": [],
        "note": "no live caves outside the running kernel",
    })


DISPATCH = {
    "read_file":          tool_read_file,
    "grep_source":        tool_grep_source,
    "query_audit_ring":   tool_query_audit_ring,
    "suggest_command":    tool_suggest_command,
    "read_concept_note":  tool_read_concept_note,
    "list_caves":         tool_list_caves,
}


def dispatch(name: str, args: Any) -> str:
    """args may arrive as dict or JSON-encoded string from ollama."""
    if isinstance(args, str):
        try:
            args = json.loads(args)
        except json.JSONDecodeError:
            args = {}
    fn = DISPATCH.get(name)
    if not fn:
        return json.dumps({"error": f"unknown tool: {name}"})
    try:
        return fn(args or {})
    except Exception as e:
        return json.dumps({"error": f"{type(e).__name__}: {e}"})


# JSON-Schema-shaped tool specs for the chat-completion request. Each
# spec mirrors the parameters in src/ai/tools.rs::TOOLS so the model
# sees the same surface here as on the kernel.
TOOL_SPECS = [
    {
        "type": "function",
        "function": {
            "name": "read_file",
            "description": "Read a UTF-8 file from the Sphragis repo. Returns up to 4 KB.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "path relative to repo root"},
                },
                "required": ["path"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "grep_source",
            "description": "Substring search across src/. Returns up to 20 matches with file:line.",
            "parameters": {
                "type": "object",
                "properties": {
                    "pattern": {"type": "string"},
                    "path_glob": {"type": "string", "description": "optional glob; default '**/*.rs'"},
                },
                "required": ["pattern"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "read_concept_note",
            "description": "Look up a Concept note by name. Returns the note text.",
            "parameters": {
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                },
                "required": ["name"],
            },
        },
    },
]
