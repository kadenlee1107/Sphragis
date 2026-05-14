#!/usr/bin/env python3
"""Sync the Sphragis source tree into an Obsidian vault as per-file notes.

Output goes to ``/Users/kadenlee/SPHRAGIS_VAULT/_generated/`` (or whichever path
``--vault`` points at).  The vault is laid out as:

    <vault>/
      _generated/                  ← everything this script writes
        src/...                    mirrors repo layout, one .md per source file
        scripts/...
        docs/...
        designs/                   top-level DESIGN_*.md
        configs/                   Cargo.toml, build.rs, Makefile, linker.ld
        vendored/                  external/, ports/, boot_chain/ — directory-level
        _index.md                  auto TOC
      Concepts/                    ← hand-written editorial notes (untouched)

Idempotent: rewrites only changed notes, removes orphans for deleted source
files, leaves ``Concepts/`` and any non-``_generated/`` content alone.

Editorial voice rules (in priority order):

  1. Top-of-file Rust ``//!`` module docs and Python module docstrings ARE
     the editorial voice — extract them as the lede of the note when present.
  2. Visible audit/version markers in comments — V<N>-XYZ, STUMP #<N>,
     ``audit-`` references — surface as a "history" callout.
  3. ``pub fn``/``pub struct``/``pub enum`` for Rust, ``def`` and ``class``
     for Python — listed under "Public API" with their first doc-comment line.
  4. ``use crate::…`` imports → cross-links via wikilinks ``[[other_file]]``.
  5. Files with no docstrings get a clearly-labelled stub so a hand-editor
     can spot them.

Run manually:

    python3 scripts/sync_obsidian.py            # full pass
    python3 scripts/sync_obsidian.py --dry-run  # show plan, write nothing
    python3 scripts/sync_obsidian.py --verbose  # print per-file decisions

Or wire into git via ``scripts/install_hooks.sh`` so post-commit and
post-checkout regenerate automatically.
"""
from __future__ import annotations

import argparse
import hashlib
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable

REPO   = Path("/Users/kadenlee/Sphragis")
VAULT  = Path("/Users/kadenlee/SPHRAGIS_VAULT")
GEN    = VAULT / "_generated"

# ---------------------------------------------------------------------------
# Scope: which trees to walk and how to treat them
# ---------------------------------------------------------------------------

# First-party trees — one .md per source file (rich extraction)
FIRST_PARTY_ROOTS = ["src", "scripts", "docs"]

# Vendored trees — also one .md per source file, but with a lighter
# template (path + role + "do not edit by hand" callout). Per the brief,
# we keep coverage truly complete so search/backlinks resolve.
# (ports/ was deleted in the no-browser pivot; remove if it returns.)
VENDORED_PER_FILE_ROOTS = ["external", "boot_chain"]

# Top-level files that become per-file notes under _generated/{group}/
TOP_LEVEL = {
    "designs": [
        "DESIGN.md", "DESIGN_BATCAVES.md", "DESIGN_CRYPTO.md",
        "DESIGN_HTTPS_SYSCALL.md", "DESIGN_NO_BROWSER.md",
        "DESIGN_PACKET_PIPELINE.md", "DESIGN_SCHEDULER_BLOCK_ON.md",
        "DESIGN_TLS_HARDENING.md",
    ],
    "configs": [
        "Cargo.toml", "Cargo.lock", "Makefile", "build.rs",
        "linker.ld", ".gitignore", ".cargo/config.toml",
    ],
    "guides": [
        "CLAUDE.md", "DEPLOY.md", "QUICKSTART.md", "UBUNTU_QUICKSTART.md",
    ],
}

# Vendored / large trees — single dir-level summary, not per-file
VENDORED_ROOTS = ["external", "ports", "boot_chain"]

# Always-skip
SKIP_DIRS = {
    "target", "captures", "logs", ".git", ".claude", ".claude-flow",
    ".ruff_cache", "node_modules", "__pycache__", ".pytest_cache",
}
SKIP_FILES = {".DS_Store", ".autopilot-session-id", ".autopilot-session-started"}

# Skip these extensions outright — build artifacts, binaries, images.
# Keeping them out of the vault keeps search clean.
SKIP_EXTS = {
    ".d", ".o", ".obj", ".a", ".so", ".dylib", ".exe", ".dll",
    ".rlib", ".rmeta", ".timestamp", ".pyc", ".pyo",
    ".bin", ".img", ".dmg", ".iso",
    ".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg",
    ".icns", ".ico",
    ".woff", ".woff2", ".ttf", ".otf",
    ".pdf", ".zip", ".tar", ".gz", ".bz2", ".xz", ".7z",
    ".mp4", ".mp3", ".wav",
}

# Source-file extensions we know how to introspect
EXT_LANG = {
    ".rs":   "rust",
    ".py":   "python",
    ".md":   "markdown",
    ".toml": "toml",
    ".lock": "toml",
    ".sh":   "shell",
    ".ld":   "linker-script",
    ".S":    "asm",
    ".s":    "asm",
    ".yaml": "yaml",
    ".yml":  "yaml",
    ".json": "json",
    ".c":    "c",
    ".h":    "c-header",
    ".cpp":  "cpp",
    ".hpp":  "cpp-header",
    ".cc":   "cpp",
    ".hh":   "cpp-header",
    ".xml":  "xml",
    ".dts":  "device-tree",
    ".dtsi": "device-tree",
}

# Anything else first-party gets a generic-file note (just metadata)


# ---------------------------------------------------------------------------
# Note-content helpers
# ---------------------------------------------------------------------------

@dataclass
class FileFacts:
    """Everything we extract from a source file to build its note."""
    rel_path: Path
    abs_path: Path
    lang: str
    bytes_size: int
    line_count: int
    lede: str = ""              # editorial paragraph from doc comments
    public_api: list[tuple[str, str]] = field(default_factory=list)  # (signature, doc-snippet)
    history: list[str] = field(default_factory=list)  # V-NUMBER / STUMP / audit refs
    related: list[str] = field(default_factory=list)  # cross-links to other repo files
    last_modified: str = ""     # iso date
    last_commit: str = ""       # short sha + message
    vendored: bool = False      # if True, render with vendored template


def _normalize_lede(s: str) -> str:
    """Trim trailing whitespace, collapse 3+ newlines, preserve paragraph
    breaks (double newlines). Result is markdown-safe — caller can drop it
    inside a blockquote and it'll render right."""
    if not s:
        return ""
    # Strip lines, drop trailing whitespace
    lines = [ln.rstrip() for ln in s.splitlines()]
    # Collapse leading/trailing empties
    while lines and not lines[0]:
        lines.pop(0)
    while lines and not lines[-1]:
        lines.pop()
    # Collapse 3+ blank lines to 1
    out: list[str] = []
    blank = 0
    for ln in lines:
        if not ln:
            blank += 1
            if blank <= 1:
                out.append("")
        else:
            blank = 0
            out.append(ln)
    return "\n".join(out)


# ─── Rust extraction ────────────────────────────────────────────────────────

RUST_MODULE_DOC_RE = re.compile(r'^\s*//!(.*)$', re.MULTILINE)
RUST_OUTER_DOC_RE  = re.compile(r'^\s*///(.*)$', re.MULTILINE)
RUST_PUB_RE        = re.compile(
    r'^\s*pub(?:\s+\([^)]*\))?\s+'
    r'(?:async\s+|unsafe\s+|const\s+)*'
    r'(fn|struct|enum|trait|type|mod|static|const)\s+'
    r'([A-Za-z0-9_]+)'
    r'.*?$',
    re.MULTILINE,
)
RUST_USE_CRATE_RE = re.compile(r'^\s*use\s+crate::([A-Za-z0-9_:]+)', re.MULTILINE)
AUDIT_MARKER_RE   = re.compile(r'\b(V\d+-[A-Z]+(?:-\d+)?|STUMP\s*#\s*\d+|audit-\d+)\b')

def extract_rust(text: str) -> dict:
    """Pull module docs, pub API, audit markers, and crate cross-refs.

    Lede priority:
      1. Consecutive top-of-file ``//!`` (Rust module docs) — official.
      2. Fallback: consecutive top-of-file ``//`` lines that LOOK like a file
         header (start with the filename, the project name, or a section
         dash-rule). Many .rs files in this codebase use plain ``//`` for the
         opening editorial paragraph instead of ``//!`` — preserve their voice.
    """
    facts: dict = {"lede": "", "public_api": [], "history": [], "related": []}

    # Pass 1: prefer //! module docs
    mod_doc_lines: list[str] = []
    for line in text.splitlines():
        s = line.lstrip()
        if s.startswith("//!"):
            mod_doc_lines.append(s[3:].lstrip())
            continue
        if mod_doc_lines:
            if not s:
                mod_doc_lines.append("")
                continue
            break
        if s and not (s.startswith("//") or s.startswith("/*") or s.startswith("#!")):
            break
    if mod_doc_lines:
        facts["lede"] = _normalize_lede("\n".join(mod_doc_lines))

    # Pass 2: fallback to leading // header block if no //! found
    if not facts["lede"]:
        header: list[str] = []
        for line in text.splitlines():
            s = line.rstrip()
            if not s.strip():
                if header:
                    header.append("")
                    continue
                else:
                    continue
            ls = s.lstrip()
            if ls.startswith("//") and not ls.startswith("///"):
                # treat as header line
                header.append(ls.lstrip("/").lstrip())
                continue
            break
        if header:
            facts["lede"] = _normalize_lede("\n".join(header))

    # Public API: pub fn/struct/enum/trait/type/mod with optional preceding ///
    lines = text.splitlines()
    pending_doc: list[str] = []
    for ln in lines:
        s = ln.lstrip()
        if s.startswith("///"):
            pending_doc.append(s[3:].lstrip())
            continue
        m = RUST_PUB_RE.match(ln)
        if m:
            kind, name = m.group(1), m.group(2)
            sig = ln.strip().rstrip(";").rstrip("{").strip()
            doc = " ".join(pending_doc).strip() if pending_doc else ""
            facts["public_api"].append((sig, doc))
            pending_doc = []
            continue
        if s and not s.startswith("//"):
            pending_doc = []

    # Audit / version markers anywhere in file
    seen: set[str] = set()
    for m in AUDIT_MARKER_RE.finditer(text):
        marker = m.group(1).strip().replace("  ", " ").upper().replace("STUMP #", "STUMP #")
        if marker not in seen:
            seen.add(marker)
            facts["history"].append(marker)

    # use crate:: imports → related-files
    for m in RUST_USE_CRATE_RE.finditer(text):
        facts["related"].append(m.group(1).split("::")[0])

    return facts


# ─── Python extraction ──────────────────────────────────────────────────────

PY_DEF_RE   = re.compile(r'^(def|class)\s+([A-Za-z_][A-Za-z0-9_]*)', re.MULTILINE)

def extract_python(text: str) -> dict:
    facts: dict = {"lede": "", "public_api": [], "history": [], "related": []}

    # Module docstring — first triple-quoted string after optional shebang/comments
    docstring = ""
    stripped = text.lstrip()
    # skip shebang
    if stripped.startswith("#!"):
        stripped = stripped.split("\n", 1)[1] if "\n" in stripped else ""
    stripped = stripped.lstrip()
    # skip top comments + blank lines
    while True:
        if stripped.startswith("#"):
            stripped = stripped.split("\n", 1)[1] if "\n" in stripped else ""
            stripped = stripped.lstrip()
            continue
        break
    # extract """...""" or '''...'''
    for q in ('"""', "'''"):
        if stripped.startswith(q):
            end = stripped.find(q, len(q))
            if end > 0:
                docstring = stripped[len(q):end].strip()
                break
    facts["lede"] = _normalize_lede(docstring)

    # Public def/class — skip ones starting with _
    for m in PY_DEF_RE.finditer(text):
        kind, name = m.group(1), m.group(2)
        if name.startswith("_"):
            continue
        # Pull the line as signature
        line_start = text.rfind("\n", 0, m.start()) + 1
        line_end   = text.find("\n", m.end())
        sig = text[line_start:line_end if line_end > 0 else len(text)].strip()
        # Try to grab the next docstring line if present
        rest = text[line_end:line_end + 800] if line_end > 0 else ""
        ds = ""
        m2 = re.search(r'"""(.*?)(?:\n|""")', rest, re.S)
        if m2:
            ds = m2.group(1).split("\n", 1)[0].strip()
        facts["public_api"].append((sig, ds))

    # Audit markers
    seen: set[str] = set()
    for m in AUDIT_MARKER_RE.finditer(text):
        mk = m.group(1).strip()
        if mk not in seen:
            seen.add(mk)
            facts["history"].append(mk)

    return facts


# ─── Generic / markdown / config ────────────────────────────────────────────

def extract_markdown(text: str) -> dict:
    """For .md sources we treat the first paragraph as the lede."""
    facts: dict = {"lede": "", "public_api": [], "history": [], "related": []}
    lines = text.splitlines()
    # Skip leading h1 + blank lines, take first non-empty paragraph
    para: list[str] = []
    started = False
    for ln in lines:
        s = ln.strip()
        if not started:
            if not s or s.startswith("#"):
                continue
            started = True
        if not s:
            if para:
                break
            continue
        if s.startswith("#"):
            break
        para.append(s)
    facts["lede"] = _normalize_lede("\n".join(para))
    return facts


def extract_generic(text: str) -> dict:
    return {"lede": "", "public_api": [], "history": [], "related": []}


def extract_c_like(text: str) -> dict:
    """Lede extractor for C / C++ / asm / linker / device-tree.

    Tries (in order):
      1. Top-of-file ``/* ... */`` block — typical license/header banner
      2. Consecutive ``//`` lines at top of file — modern C-family header
      3. Leading ``# ... `` comment block — used by linker/device-tree (.dts)
    """
    facts: dict = {"lede": "", "public_api": [], "history": [], "related": []}

    s = text.lstrip()
    # 1. /* ... */ block
    if s.startswith("/*"):
        end = s.find("*/")
        if end > 0:
            block = s[2:end]
            # strip leading ' * ' on each line
            cleaned: list[str] = []
            for ln in block.splitlines():
                cleaned.append(ln.strip().lstrip("*").lstrip())
            facts["lede"] = _normalize_lede("\n".join(cleaned))
            return facts

    # 2. consecutive // header
    if s.startswith("//"):
        header: list[str] = []
        for line in s.splitlines():
            ls = line.lstrip()
            if ls.startswith("//"):
                header.append(ls.lstrip("/").lstrip())
                continue
            if not ls.strip() and header:
                header.append("")
                continue
            break
        if header:
            facts["lede"] = _normalize_lede("\n".join(header))
            return facts

    # 3. # banner (linker scripts, .dts)
    if s.startswith("#") and not s.startswith("#!"):
        header: list[str] = []
        for line in s.splitlines():
            ls = line.lstrip()
            if ls.startswith("#"):
                header.append(ls.lstrip("#").lstrip())
                continue
            if not ls.strip() and header:
                header.append("")
                continue
            break
        if header:
            facts["lede"] = _normalize_lede("\n".join(header))
            return facts

    return facts


EXTRACTORS = {
    "rust":           extract_rust,
    "python":         extract_python,
    "markdown":       extract_markdown,
    "c":              extract_c_like,
    "c-header":       extract_c_like,
    "cpp":            extract_c_like,
    "cpp-header":     extract_c_like,
    "asm":            extract_c_like,
    "linker-script":  extract_c_like,
    "device-tree":    extract_c_like,
}


# ---------------------------------------------------------------------------
# Walking + collecting facts
# ---------------------------------------------------------------------------

def is_skipped(p: Path) -> bool:
    parts = set(p.parts)
    if parts & SKIP_DIRS:
        return True
    if p.name in SKIP_FILES:
        return True
    return False


def _walk_tree(root: Path, vendored: bool, verbose: bool) -> list[FileFacts]:
    """Walk one tree and return FileFacts for every file we want to note."""
    out: list[FileFacts] = []
    if not root.exists():
        return out
    for p in sorted(root.rglob("*")):
        if not p.is_file() or is_skipped(p.relative_to(REPO)):
            continue
        ext = p.suffix.lower()
        if ext in SKIP_EXTS:
            continue
        if ext == "":
            # extension-less files: only allow well-known textual ones
            if p.name not in {"Makefile", "Dockerfile", "LICENSE", "README"}:
                continue
        elif ext not in EXT_LANG:
            # Unknown extension — skip rather than guess. Add to EXT_LANG to include.
            continue
        facts = build_facts(p)
        facts.vendored = vendored
        out.append(facts)
    return out


def collect_first_party(verbose: bool = False) -> list[FileFacts]:
    out: list[FileFacts] = []

    # 1. first-party trees (rich extraction)
    for root_name in FIRST_PARTY_ROOTS:
        out.extend(_walk_tree(REPO / root_name, vendored=False, verbose=verbose))

    # 2. top-level explicit groups
    for group, names in TOP_LEVEL.items():
        for name in names:
            p = REPO / name
            if p.exists() and p.is_file():
                facts = build_facts(p)
                out.append(facts)

    # 3. vendored trees — per-file but with vendored=True so render path
    #    uses the lighter, "do not edit by hand" template.
    for root_name in VENDORED_PER_FILE_ROOTS:
        out.extend(_walk_tree(REPO / root_name, vendored=True, verbose=verbose))

    if verbose:
        n_fp = sum(1 for f in out if not f.vendored)
        n_v  = sum(1 for f in out if f.vendored)
        print(f"[obsidian-sync]   collected {n_fp} first-party + {n_v} vendored files")
    return out


def build_facts(abs_path: Path) -> FileFacts:
    rel = abs_path.relative_to(REPO)
    ext = abs_path.suffix.lower()
    lang = EXT_LANG.get(ext, "text")

    try:
        text = abs_path.read_text(encoding="utf-8", errors="replace")
    except Exception:
        text = ""

    extractor = EXTRACTORS.get(lang, extract_generic)
    extracted = extractor(text)

    # Last-commit info via git log
    last_modified = ""
    last_commit = ""
    try:
        r = subprocess.run(
            ["git", "log", "-1", "--format=%cs|%h %s", "--", str(rel)],
            cwd=REPO, capture_output=True, text=True, timeout=5,
        )
        if r.returncode == 0 and r.stdout.strip():
            parts = r.stdout.strip().split("|", 1)
            if len(parts) == 2:
                last_modified, last_commit = parts
    except Exception:
        pass

    return FileFacts(
        rel_path=rel,
        abs_path=abs_path,
        lang=lang,
        bytes_size=abs_path.stat().st_size if abs_path.exists() else 0,
        line_count=text.count("\n") + (1 if text and not text.endswith("\n") else 0),
        lede=extracted["lede"],
        public_api=extracted["public_api"],
        history=extracted["history"],
        related=extracted["related"],
        last_modified=last_modified,
        last_commit=last_commit,
    )


# ---------------------------------------------------------------------------
# Note rendering
# ---------------------------------------------------------------------------

GITHUB_BASE = "https://github.com/kadenlee1107/Sphragis/blob/main/"

def vault_path_for(rel: Path) -> Path:
    """Mirror repo path under _generated/ — `tls.rs` → `_generated/src/net/tls.rs.md`.

    For top-level files we route into _generated/{designs,configs,guides}/.
    """
    parts = rel.parts
    if len(parts) == 1:
        # top-level: route by group
        for group, names in TOP_LEVEL.items():
            if rel.name in names:
                return GEN / group / f"{rel.name}.md"
        return GEN / f"{rel.name}.md"
    return GEN / Path(*parts).with_suffix(rel.suffix + ".md")


SUBSYSTEM_VOICE = [
    # (path-prefix, editorial framing)
    ("src/net/tls",       "the kernel TLS stack — handshake, record layer, post-quantum hybrid kex"),
    ("src/net/x509",      "X.509 chain validation that runs in the kernel"),
    ("src/net/https",     "the kernel-mediated HTTPS syscall — caves write HTTP, the kernel encrypts"),
    ("src/net/cave_policy", "per-cave default-deny network policy enforced below the syscall boundary"),
    ("src/net/dns",       "the in-kernel DNS resolver"),
    ("src/net/firewall",  "egress firewall plumbing — paired with cave_policy"),
    ("src/net/",          "part of the from-scratch IPv4/TCP/UDP stack"),
    ("src/security/audit","the audit ring — append-only, encrypted, kernel-only writer"),
    ("src/security/",     "kernel-side security primitives"),
    ("src/auth/",         "the authentication path — passphrase, KDF, lock-screen"),
    ("src/batfs/",        "the encrypted filesystem (BatFS — ChaCha20-Poly1305 + Argon2id)"),
    ("src/cave/",         "the BatCave isolation model — capability-typed processes"),
    ("src/drivers/apple/","an Apple-Silicon-specific driver, reverse-engineered from real M4 hardware"),
    ("src/drivers/",      "a kernel driver"),
    ("src/ui/shell",      "the in-kernel shell"),
    ("src/ui/",           "kernel-side UI rendering and input handling"),
    ("src/arch/",         "architecture-specific kernel code (aarch64)"),
    ("src/kernel/",       "core kernel internals"),
    ("src/main",          "the kernel entry point itself"),
    ("src/lib",           "the kernel library root"),
    ("src/",              "kernel source"),
    ("scripts/qemu_",     "a QEMU-based smoke test invoked from the project's normal CI flow"),
    ("scripts/",          "a build/test helper script"),
    ("docs/M4_GROUND_TRUTH", "the long-form, hand-curated record of every M4 hardware fact this project depends on"),
    ("docs/SESSION_JOURNAL", "the chronological project journal — what was tried, what worked, what's next"),
    ("docs/",             "long-form project documentation"),
]


def _subsystem_phrase(rel_path: Path) -> str:
    s = str(rel_path)
    for prefix, phrase in SUBSYSTEM_VOICE:
        if s.startswith(prefix):
            return phrase
    return ""


def fmt_lede_editorial(facts: FileFacts) -> str:
    """Open the note with a contextual paragraph. Prefer real source-doc voice.
    Failing that, synthesize a grounded paragraph from path-position + audit
    markers — never the generic "auto-stub" line."""
    if facts.lede:
        # Render as a multi-line blockquote, preserving paragraph breaks
        return "\n".join(
            f"> {ln}" if ln else ">"
            for ln in facts.lede.splitlines()
        )

    # Synthesized lede — voice from repo context, not boilerplate
    phrase = _subsystem_phrase(facts.rel_path) or {
        "rust":          "Rust source compiled into the kernel image",
        "python":        "Python helper",
        "toml":          "configuration",
        "linker-script": "linker script consumed by rustc via `-Tlinker.ld`",
        "asm":           "assembly — low-level ARM64 routines that bypass the Rust ABI",
        "shell":         "shell script",
        "yaml":          "configuration",
        "json":          "configuration / data",
        "markdown":      "documentation",
    }.get(facts.lang, "first-party file")

    sentence = f"`{facts.rel_path}` is {phrase}."
    if facts.history:
        # Surface the audit history as part of the editorial framing
        markers = ", ".join(facts.history[:3])
        more = f" and {len(facts.history) - 3} more" if len(facts.history) > 3 else ""
        sentence += (
            f" The source carries audit markers ({markers}{more}), so the "
            f"present shape reflects past incidents — chase those threads if "
            f"you need to understand a counter-intuitive choice."
        )
    else:
        sentence += " No top-of-file doc lives in source yet; consider writing one and re-running the sync."
    return f"> {sentence}"


def fmt_public_api(facts: FileFacts) -> str:
    if not facts.public_api:
        return ""
    out = ["## Public API\n"]
    for sig, doc in facts.public_api[:60]:  # cap to keep notes manageable
        sig_short = sig if len(sig) <= 120 else sig[:117] + "..."
        line = f"- `{sig_short}`"
        if doc:
            line += f" — {doc}"
        out.append(line)
    if len(facts.public_api) > 60:
        out.append(f"- _…and {len(facts.public_api) - 60} more, omitted for note size._")
    return "\n".join(out)


def fmt_history(facts: FileFacts) -> str:
    if not facts.history:
        return ""
    out = [
        "## Audit & version history\n",
        "Markers found in the source — each is a thread worth pulling on if "
        "you want to understand why this file looks the way it does.\n",
    ]
    for marker in facts.history:
        out.append(f"- **{marker}**")
    return "\n".join(out)


def fmt_related(facts: FileFacts, all_paths: set[Path]) -> str:
    if not facts.related:
        return ""
    # Each `related` is a crate-relative module name like "net" or "ui::shell".
    # Resolve to an actual file path under src/ if we can.
    seen: set[str] = set()
    out_links: list[str] = []
    for r in facts.related:
        first = r.split("::")[0]
        if first in seen:
            continue
        seen.add(first)
        # try src/{first}.rs and src/{first}/mod.rs and src/{first}/{first}.rs
        for cand in (f"src/{first}.rs", f"src/{first}/mod.rs"):
            if Path(cand) in all_paths:
                out_links.append(f"[[{cand}]]")
                break
        else:
            out_links.append(f"`crate::{first}` _(no direct file match)_")
    if not out_links:
        return ""
    return "## Related\n\n" + "\n".join(f"- {l}" for l in out_links)


def render_note(facts: FileFacts, all_paths: set[Path]) -> str:
    rel = facts.rel_path
    title = f"`{rel}`"
    note_type = "vendored-file" if facts.vendored else "code-note"
    front = [
        "---",
        f"type: {note_type}",
        f"path: {rel}",
        f"language: {facts.lang}",
        f"lines: {facts.line_count}",
        f"bytes: {facts.bytes_size}",
        f"vendored: {str(facts.vendored).lower()}",
        f"generated: {datetime.now(timezone.utc).isoformat()}",
        "---",
        "",
        f"# {title}",
        "",
    ]
    if facts.vendored:
        front.extend([
            "> ⚠️ **Vendored snapshot — do not hand-edit.** This file came from "
            "an upstream project and lives in the repo for reproducibility. "
            "Upstream changes land via re-import, not in-place edits.",
            "",
        ])
    front.extend([
        fmt_lede_editorial(facts),
        "",
    ])
    sections = [s for s in (
        fmt_public_api(facts),
        fmt_history(facts),
        fmt_related(facts, all_paths),
    ) if s]
    if sections:
        front.append("\n".join(sections))
        front.append("")
    # Reference block at the bottom
    ref = [
        "## Reference",
        "",
        f"- [Source on GitHub]({GITHUB_BASE}{rel})",
    ]
    if facts.last_commit:
        ref.append(f"- Last commit: `{facts.last_commit}` ({facts.last_modified})")
    front.append("\n".join(ref))
    return "\n".join(front).rstrip() + "\n"


# ---------------------------------------------------------------------------
# Vendored directory summaries
# ---------------------------------------------------------------------------

VENDORED_NOTES = {
    "external/m1n1": {
        "title": "external/m1n1",
        "what": (
            "Asahi Linux's m1n1 — the second-stage bootloader that lets us "
            "chainload BAT OS from macOS Recovery without re-flashing firmware "
            "or burning fuses. Vendored snapshot, not a moving target."
        ),
        "why": (
            "m1n1 owns the bring-up of secondary CPUs, the iBoot→kernel handoff, "
            "and the proxy protocol the host uses to drive the M4 over USB-C. "
            "Our `external/m1n1/proxyclient/tools/chainload.py` is patched to "
            "pass `-S` / `--skip-secondary-cpus` because the M4 P-cluster SErrors "
            "on RVBAR writes."
        ),
        "do_not_edit": True,
    },
    "external/asahi-docs": {
        "title": "external/asahi-docs",
        "what": "Asahi Linux's reverse-engineering documentation tree — register layouts, ADT entries, hardware compatible strings.",
        "why": (
            "When `docs/M4_GROUND_TRUTH.md` says 'verified against Asahi docs', "
            "it means the hex was cross-checked here. Treat this as a reference "
            "library; everything we depend on is transcribed into the ground-truth doc."
        ),
        "do_not_edit": True,
    },
    # ports/ was deleted in the no-browser pivot. Stub left here so future
    # imports can re-enable easily; the tree-renderer skips missing dirs.
    "boot_chain": {
        "title": "boot_chain",
        "what": "Boot-chain artifacts: m1n1 stage1 builds, kernel images, and the chainload scripts that move bytes from macOS Recovery into BAT OS's reset vector.",
        "why": (
            "Not source — outputs and helper scripts. The actual code that "
            "lands on the M4 lives in `[[src/main.rs]]` (the kernel) and "
            "`[[external/m1n1]]` (the bootloader). Look here for the integration glue."
        ),
        "do_not_edit": False,
    },
}

def render_vendored(name: str, info: dict) -> str:
    abs_path = REPO / name
    if not abs_path.exists():
        return ""
    n_files = sum(1 for _ in abs_path.rglob("*") if _.is_file())
    size = subprocess.run(
        ["du", "-sh", str(abs_path)], capture_output=True, text=True
    ).stdout.split()[0] if abs_path.exists() else "?"
    edit_note = (
        "\n> ⚠️ **Do not edit by hand.** Vendored snapshot. Upstream changes "
        "land via re-import, not in-place edits.\n"
        if info.get("do_not_edit") else ""
    )
    body = f"""---
type: vendored-tree
path: {name}
files: {n_files}
size: {size}
generated: {datetime.now(timezone.utc).isoformat()}
---

# `{info['title']}`

> {info['what']}
{edit_note}

## Why it's in the repo

{info['why']}

## Reference

- [Tree on GitHub]({GITHUB_BASE}{name})
- {n_files} files · {size} on disk
"""
    return body


# ---------------------------------------------------------------------------
# Index page
# ---------------------------------------------------------------------------

def render_index(facts_list: list[FileFacts]) -> str:
    by_root: dict[str, list[FileFacts]] = {}
    for f in facts_list:
        root = f.rel_path.parts[0] if len(f.rel_path.parts) > 1 else "<top>"
        by_root.setdefault(root, []).append(f)

    lines = [
        "---",
        "type: vault-index",
        f"generated: {datetime.now(timezone.utc).isoformat()}",
        "---",
        "",
        "# BAT OS — vault index",
        "",
        "> Auto-generated. Each entry below is a per-file note covering what "
        "the file does, its public API, audit history, and links to related code. "
        "Hand-written editorial spine lives in `[[Concepts]]/`.",
        "",
        f"**{len(facts_list)} first-party files** indexed across "
        f"{len(by_root)} top-level groupings, plus {len(VENDORED_NOTES)} vendored "
        "directory summaries.",
        "",
    ]
    for root in sorted(by_root.keys()):
        items = sorted(by_root[root], key=lambda f: f.rel_path)
        lines.append(f"## {root}")
        lines.append("")
        for f in items:
            link_path = vault_path_for(f.rel_path).relative_to(GEN).with_suffix("")
            note_link = str(link_path).replace(os.sep, "/")
            note_basename = Path(note_link).name
            lines.append(f"- [[{note_link}|{note_basename}]] · {f.line_count} lines")
        lines.append("")
    lines += [
        "## Vendored",
        "",
    ]
    for name in VENDORED_NOTES:
        slug = name.replace("/", "_")
        lines.append(f"- [[vendored/{slug}|{name}]]")
    lines.append("")
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# I/O: write only on change, prune orphans
# ---------------------------------------------------------------------------

def write_if_changed(path: Path, content: str, dry_run: bool, verbose: bool) -> bool:
    if path.exists():
        old = path.read_text(encoding="utf-8")
        if old == content:
            return False
    if dry_run:
        if verbose:
            print(f"[obsidian-sync]   would write {path.relative_to(VAULT)}")
        return True
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    if verbose:
        print(f"[obsidian-sync]   wrote {path.relative_to(VAULT)}")
    return True


def prune_orphans(written: set[Path], dry_run: bool, verbose: bool) -> int:
    if not GEN.exists():
        return 0
    n = 0
    for p in GEN.rglob("*.md"):
        if p not in written:
            if dry_run:
                if verbose:
                    print(f"[obsidian-sync]   would remove orphan {p.relative_to(VAULT)}")
            else:
                p.unlink()
                if verbose:
                    print(f"[obsidian-sync]   removed orphan {p.relative_to(VAULT)}")
            n += 1
    # also clean up empty dirs
    if not dry_run:
        for p in sorted(GEN.rglob("*"), key=lambda q: -len(q.parts)):
            if p.is_dir() and not any(p.iterdir()):
                p.rmdir()
    return n


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> int:
    global VAULT, GEN
    ap = argparse.ArgumentParser()
    ap.add_argument("--vault",   default=str(VAULT), help="Obsidian vault root")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--verbose", action="store_true")
    args = ap.parse_args()

    VAULT = Path(args.vault)
    GEN   = VAULT / "_generated"

    if not VAULT.exists():
        print(f"[obsidian-sync] FATAL: vault {VAULT} does not exist", file=sys.stderr)
        return 1
    if not REPO.exists():
        print(f"[obsidian-sync] FATAL: repo {REPO} does not exist", file=sys.stderr)
        return 1

    print(f"[obsidian-sync] repo  {REPO}")
    print(f"[obsidian-sync] vault {VAULT}")
    if args.dry_run:
        print("[obsidian-sync] DRY RUN — no files will be written")

    # 1. collect facts
    facts_list = collect_first_party(verbose=args.verbose)
    print(f"[obsidian-sync] indexed {len(facts_list)} first-party files")

    # 2. render notes
    all_paths = {f.rel_path for f in facts_list}
    written: set[Path] = set()
    n_changed = 0
    for f in facts_list:
        target = vault_path_for(f.rel_path)
        content = render_note(f, all_paths)
        if write_if_changed(target, content, args.dry_run, args.verbose):
            n_changed += 1
        written.add(target)

    # 3. vendored summaries
    for name, info in VENDORED_NOTES.items():
        slug = name.replace("/", "_")
        target = GEN / "vendored" / f"{slug}.md"
        content = render_vendored(name, info)
        if not content:
            continue
        if write_if_changed(target, content, args.dry_run, args.verbose):
            n_changed += 1
        written.add(target)

    # 4. index
    target = GEN / "_index.md"
    if write_if_changed(target, render_index(facts_list), args.dry_run, args.verbose):
        n_changed += 1
    written.add(target)

    # 5. prune orphans
    n_pruned = prune_orphans(written, args.dry_run, args.verbose)

    print(f"[obsidian-sync] done — {n_changed} note(s) changed, {n_pruned} orphan(s) pruned")
    return 0


if __name__ == "__main__":
    sys.exit(main())
