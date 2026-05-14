#!/usr/bin/env python3
"""Build LoRA training dataset from Sphragis source + docs + vault.

Produces a JSONL file at out/sphragis_lora_dataset.jsonl with one
{"instruction": ..., "input": ..., "output": ...} record per line.
Intended for HuggingFace SFTTrainer or trl's LoRA fine-tune flow.

Dataset composition (see DESIGN_AI_AGENT.md "Training data"):
  * Source files: synthesize "What does function X do?" → docstring + body
  * Audit markers: "What is V8-ROOT-1?" → surrounding comment + linked code
  * Concept notes: "Tell me about <topic>" → Concept note body
  * Commit messages: subject → body + diff stat
"""
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

REPO  = Path(__file__).resolve().parent.parent
VAULT = Path.home() / "SPHRAGIS_VAULT"
OUT   = REPO / "out" / "sphragis_lora_dataset.jsonl"

AUDIT_RE = re.compile(r'(V\d+-[A-Z]+(?:-\d+)?|STUMP\s*#\s*\d+)')
RUST_FN_RE = re.compile(
    r'^(///[^\n]*\n)*\s*(pub\s+(?:async\s+|unsafe\s+|const\s+)*fn\s+\w+[^{]+)\{',
    re.MULTILINE,
)


def collect_source_pairs() -> list[dict]:
    """For each pub fn in src/, emit a (signature → body) pair."""
    out = []
    for p in (REPO / "src").rglob("*.rs"):
        text = p.read_text(encoding="utf-8", errors="replace")
        for m in RUST_FN_RE.finditer(text):
            sig = m.group(2).strip()
            start = m.start()
            body_end = min(start + 2000, len(text))
            body = text[start:body_end]
            rel = p.relative_to(REPO)
            out.append({
                "instruction": "In Sphragis, what does the following function do?",
                "input": sig,
                "output": f"From `{rel}`:\n\n```rust\n{body}\n```",
            })
    return out


def collect_audit_pairs() -> list[dict]:
    """For each V-marker, emit a (marker → surrounding-comment) pair."""
    out = []
    seen: set[str] = set()
    for p in (REPO / "src").rglob("*.rs"):
        text = p.read_text(encoding="utf-8", errors="replace")
        for m in AUDIT_RE.finditer(text):
            marker = m.group(1).strip()
            if marker in seen:
                continue
            seen.add(marker)
            ctx = text[max(0, m.start() - 200): m.end() + 600]
            rel = p.relative_to(REPO)
            out.append({
                "instruction": f"What does the audit marker {marker} refer to in Sphragis?",
                "input": "",
                "output": f"From `{rel}`:\n\n{ctx}",
            })
    return out


def collect_concept_pairs() -> list[dict]:
    """Each Concept note → its full body."""
    out = []
    if not (VAULT / "Concepts").exists():
        return out
    for p in (VAULT / "Concepts").glob("*.md"):
        text = p.read_text(encoding="utf-8")
        title = p.stem
        out.append({
            "instruction": f"Explain the Sphragis concept '{title}'.",
            "input": "",
            "output": text,
        })
    return out


def collect_design_pairs() -> list[dict]:
    """Each top-level DESIGN_*.md or docs/PLAN_*.md → its full body."""
    out = []
    for p in REPO.glob("DESIGN_*.md"):
        text = p.read_text(encoding="utf-8")
        out.append({
            "instruction": f"Summarize the Sphragis design doc {p.name}.",
            "input": "",
            "output": text,
        })
    for p in (REPO / "docs").glob("PLAN_*.md"):
        text = p.read_text(encoding="utf-8")
        out.append({
            "instruction": f"What is the Sphragis implementation plan in {p.name}?",
            "input": "",
            "output": text,
        })
    return out


def collect_commit_pairs() -> list[dict]:
    """Each commit on main → (subject → body)."""
    out = []
    r = subprocess.run(
        ["git", "log", "main", "--format=%s%n----BODY----%n%b%n----END----"],
        cwd=REPO, capture_output=True, text=True, timeout=30,
    )
    if r.returncode != 0:
        return out
    blocks = r.stdout.split("----END----")
    for blk in blocks:
        blk = blk.strip()
        if "----BODY----" not in blk:
            continue
        subject, body = blk.split("----BODY----", 1)
        subject, body = subject.strip(), body.strip()
        if not subject or not body:
            continue
        out.append({
            "instruction": "Expand on this Sphragis commit subject.",
            "input": subject,
            "output": body,
        })
    return out


def main() -> int:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    pairs: list[dict] = []
    src_pairs   = collect_source_pairs()
    audit_pairs = collect_audit_pairs()
    cnpt_pairs  = collect_concept_pairs()
    desn_pairs  = collect_design_pairs()
    cmt_pairs   = collect_commit_pairs()
    pairs.extend(src_pairs)
    pairs.extend(audit_pairs)
    pairs.extend(cnpt_pairs)
    pairs.extend(desn_pairs)
    pairs.extend(cmt_pairs)

    with OUT.open("w", encoding="utf-8") as f:
        for p in pairs:
            f.write(json.dumps(p, ensure_ascii=False) + "\n")

    print(f"[lora-dataset] {len(src_pairs):>5} source-fn pairs")
    print(f"[lora-dataset] {len(audit_pairs):>5} audit-marker pairs")
    print(f"[lora-dataset] {len(cnpt_pairs):>5} concept-note pairs")
    print(f"[lora-dataset] {len(desn_pairs):>5} design-doc pairs")
    print(f"[lora-dataset] {len(cmt_pairs):>5} commit-msg pairs")
    print(f"[lora-dataset] {len(pairs):>5} TOTAL → {OUT.relative_to(REPO)}")
    print(f"[lora-dataset] file size: {OUT.stat().st_size:,} bytes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
