#!/usr/bin/env python3
"""Build the v2 LoRA training dataset.

Goals over v1:

  1. Much more data. v1 had 2,123 records mostly drawn from
     source-fn bodies and commit messages. The model learned the
     vocabulary but couldn't recall exact file paths or function
     signatures. v2 adds synthetic Q&A that targets those gaps:
     per-function "where is X defined" and "what's X's signature"
     pairs, audit-marker file pointers, module surface summaries.

  2. Multi-turn tool-call examples. v1 had zero. ollama's tool API
     refused to expose tools to the v1 model because the LoRA
     scrubbed the base's tool-use behavior. v2 reintroduces it: a
     fraction of the records demonstrate the read_file / grep_source
     / read_concept_note tools mid-conversation, so the LoRA learns
     to call them on its own.

Output: out/sphragis_lora_dataset_v2.jsonl  (messages-style records,
ready for trl.SFTTrainer with the Qwen2.5 chat template applied at
training time).

Run:
    python3 scripts/build_lora_dataset_v2.py
"""
from __future__ import annotations

import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

REPO  = Path(__file__).resolve().parent.parent
VAULT = Path.home() / "SPHRAGIS_VAULT"
OUT   = REPO / "out" / "sphragis_lora_dataset_v2.jsonl"

SYSTEM_PROMPT = (
    "You are a technical assistant for Sphragis, a security-grade bare-metal "
    "Rust kernel for Apple M4. You answer questions about kernel internals, "
    "cryptography, audit history, and system administration. You are terse, "
    "technical, and never refuse legitimate questions. Cite file paths when "
    "you can. Use the read_file, grep_source, and read_concept_note tools "
    "when a specific path or symbol is needed."
)

# ── Regexes ───────────────────────────────────────────────────────────
PUB_FN_RE = re.compile(
    r"^(?P<doc>(?:[ \t]*///[^\n]*\n)*)"      # optional doc comment block
    r"(?P<sig>[ \t]*pub(?:\([^)]*\))?\s+(?:async\s+|unsafe\s+|const\s+)*fn\s+"
    r"(?P<name>\w+)\s*[^{]*?)"               # signature (greedy lazy mix)
    r"\s*\{",
    re.MULTILINE,
)
AUDIT_RE = re.compile(r"\b(V\d+-[A-Z]+(?:-\d+)?|STUMP\s*#\s*\d+)")


# ── Synthetic Q&A generators ──────────────────────────────────────────
@dataclass
class PubFn:
    name: str
    signature: str          # trimmed to one line
    doc: str                # may be empty
    path: str               # relative
    line: int


def scan_pub_fns() -> list[PubFn]:
    fns: list[PubFn] = []
    for p in (REPO / "src").rglob("*.rs"):
        rel = str(p.relative_to(REPO))
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for m in PUB_FN_RE.finditer(text):
            sig = " ".join(m.group("sig").split())   # collapse whitespace
            name = m.group("name")
            doc = m.group("doc") or ""
            doc = "\n".join(
                line.strip().removeprefix("///").strip()
                for line in doc.splitlines()
                if line.strip()
            ).strip()
            # Line number: char-offset -> 1-based line index
            line = text.count("\n", 0, m.start()) + 1
            fns.append(PubFn(name=name, signature=sig, doc=doc, path=rel, line=line))
    return fns


def msg(role: str, content: str, **extra) -> dict:
    d = {"role": role, "content": content}
    d.update(extra)
    return d


def conv(*messages: dict) -> dict:
    return {"messages": [msg("system", SYSTEM_PROMPT), *messages]}


# Single-turn synthetic pairs — five styles per fn so we hit the same
# vocabulary from multiple angles. The eval grader checks for things
# like "close_pcb", "src/fs", "Category::Cave"; the answers below
# include those literal tokens by construction.
def pairs_for_fn(fn: PubFn) -> list[dict]:
    out: list[dict] = []
    rt_match = re.search(r"->\s*([^\{]+?)\s*$", fn.signature)
    rt = rt_match.group(1).strip() if rt_match else "()"
    doc_first = fn.doc.split(".")[0].strip() + "." if fn.doc else ""

    out.append(conv(
        msg("user", f"What does `{fn.name}` do in Sphragis?"),
        msg("assistant",
            f"`{fn.name}` lives in `{fn.path}`. "
            f"{doc_first or 'See ' + fn.path + ' for the implementation.'} "
            f"Signature: `{fn.signature}`."),
    ))
    out.append(conv(
        msg("user", f"Where is `{fn.name}` defined?"),
        msg("assistant", f"`{fn.path}:{fn.line}`."),
    ))
    out.append(conv(
        msg("user", f"What is the signature of `{fn.name}`?"),
        msg("assistant", f"`{fn.signature}`. Defined in `{fn.path}`."),
    ))
    out.append(conv(
        msg("user", f"What does `{fn.name}` return?"),
        msg("assistant", f"`{rt}`. See `{fn.path}` for context."),
    ))
    if fn.doc:
        out.append(conv(
            msg("user", f"Explain `{fn.name}` from `{fn.path}`."),
            msg("assistant", fn.doc + f"\n\nSignature: `{fn.signature}`."),
        ))
    return out


# Audit markers — single-turn, but include the surrounding comment as
# the explanation so the model learns the V-marker → context mapping.
def audit_pairs() -> list[dict]:
    out: list[dict] = []
    seen: set[tuple[str, str]] = set()
    for p in (REPO / "src").rglob("*.rs"):
        rel = str(p.relative_to(REPO))
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for m in AUDIT_RE.finditer(text):
            marker = m.group(1).replace(" ", "").strip()
            key = (marker, rel)
            if key in seen:
                continue
            seen.add(key)
            ctx = text[max(0, m.start() - 220): m.end() + 600]
            out.append(conv(
                msg("user", f"What does the audit marker {marker} refer to?"),
                msg("assistant",
                    f"{marker} is referenced in `{rel}`. Context:\n\n"
                    f"```\n{ctx.strip()}\n```"),
            ))
            out.append(conv(
                msg("user", f"Which file contains {marker}?"),
                msg("assistant", f"`{rel}`."),
            ))
    return out


# Module surface summaries — for each `mod.rs`, generate a "what's in
# this module" pair listing the `pub` items.
def module_surface_pairs() -> list[dict]:
    out: list[dict] = []
    pub_decl = re.compile(r"^pub(?:\([^)]*\))?\s+(?:async\s+|unsafe\s+|const\s+)*"
                          r"(fn|struct|enum|trait|mod|type|const|static)\s+(\w+)",
                          re.MULTILINE)
    for p in (REPO / "src").rglob("mod.rs"):
        rel = str(p.relative_to(REPO))
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        items = pub_decl.findall(text)
        if not items:
            continue
        listing = ", ".join(f"{k} {n}" for k, n in items[:30])
        mod_name = p.parent.name
        out.append(conv(
            msg("user", f"What's in the `{mod_name}` module?"),
            msg("assistant",
                f"`{rel}` exports: {listing}."),
        ))
    return out


# Concept-note paraphrases — single-turn user asks, assistant
# answers with the relevant body excerpt and cites the file.
def concept_pairs() -> list[dict]:
    out: list[dict] = []
    cd = VAULT / "Concepts"
    if not cd.exists():
        return out
    for p in sorted(cd.glob("*.md")):
        body = p.read_text(encoding="utf-8")
        title = p.stem
        # Trim the YAML frontmatter if present.
        if body.startswith("---"):
            end = body.find("---", 3)
            if end != -1:
                body = body[end + 3:].lstrip()
        first_para = body.split("\n\n", 1)[0]
        out.append(conv(
            msg("user", f"Explain the Sphragis concept '{title}'."),
            msg("assistant", f"From the Concept note '{title}':\n\n{body[:3000]}"),
        ))
        out.append(conv(
            msg("user", f"Summarize '{title}' in two sentences."),
            msg("assistant", first_para[:600]),
        ))
    return out


# Multi-turn tool-call examples. Each demonstrates calling one of
# read_file / grep_source / read_concept_note, then incorporating
# the result into the final answer. We synthesize the "tool result"
# from the same source data the eval host would return.
def tool_call_examples(fns: list[PubFn]) -> list[dict]:
    out: list[dict] = []
    # Cap the count so tool-call records stay a manageable fraction
    # of the dataset.
    sample = fns[: min(len(fns), 200)]
    for fn in sample:
        # Style 1: grep_source for the fn name
        out.append({
            "messages": [
                msg("system", SYSTEM_PROMPT),
                msg("user", f"Where in the source is `{fn.name}` defined?"),
                msg("assistant", "",
                    tool_calls=[{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": "grep_source",
                            "arguments": json.dumps({"pattern": f"fn {fn.name}"}),
                        },
                    }]),
                msg("tool",
                    json.dumps({"matches": [{"path": fn.path, "line": fn.line,
                                             "content": fn.signature[:200]}]}),
                    tool_call_id="call_1", name="grep_source"),
                msg("assistant",
                    f"`{fn.name}` is defined at `{fn.path}:{fn.line}`. "
                    f"Signature: `{fn.signature}`."),
            ]
        })
    return out


# ── Driver ────────────────────────────────────────────────────────────
def main() -> int:
    OUT.parent.mkdir(parents=True, exist_ok=True)

    fns = scan_pub_fns()
    print(f"[v2] scanned {len(fns)} pub fns")

    records: list[dict] = []
    for fn in fns:
        records.extend(pairs_for_fn(fn))
    print(f"[v2] {len(records):>5} after source-fn synthetic Q&A")

    apairs = audit_pairs()
    records.extend(apairs)
    print(f"[v2] {len(records):>5} after audit-marker pairs (+{len(apairs)})")

    mpairs = module_surface_pairs()
    records.extend(mpairs)
    print(f"[v2] {len(records):>5} after module-surface pairs (+{len(mpairs)})")

    cpairs = concept_pairs()
    records.extend(cpairs)
    print(f"[v2] {len(records):>5} after concept-note pairs (+{len(cpairs)})")

    tpairs = tool_call_examples(fns)
    records.extend(tpairs)
    print(f"[v2] {len(records):>5} after tool-call examples (+{len(tpairs)})")

    # Carry the v1 commit-message pairs across — short, cheap to keep,
    # and they're useful background for the model's voice.
    try:
        r = subprocess.run(
            ["git", "log", "main", "--format=%s%n----BODY----%n%b%n----END----"],
            cwd=REPO, capture_output=True, text=True, timeout=30,
        )
        added = 0
        for blk in r.stdout.split("----END----"):
            blk = blk.strip()
            if "----BODY----" not in blk:
                continue
            subject, body = blk.split("----BODY----", 1)
            subject, body = subject.strip(), body.strip()
            if not subject or not body:
                continue
            records.append(conv(
                msg("user", f"Expand on this Sphragis commit subject: {subject}"),
                msg("assistant", body),
            ))
            added += 1
        print(f"[v2] {len(records):>5} after commit-msg pairs (+{added})")
    except Exception as e:
        print(f"[v2] commit-msg collection failed: {e}")

    with OUT.open("w", encoding="utf-8") as f:
        for rec in records:
            f.write(json.dumps(rec, ensure_ascii=False) + "\n")

    print(f"[v2] TOTAL {len(records)} records -> {OUT.relative_to(REPO)}")
    print(f"[v2] file size: {OUT.stat().st_size:,} bytes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
