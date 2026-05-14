#!/usr/bin/env python3
"""Grade a Sphragis LoRA against sphragis_evals.jsonl.

Sends each question to an OpenAI-compatible inference endpoint
(ollama, vLLM, etc) and grades the answer with substring-match
rules. Prints per-category and per-question results.

Usage:
  ./run_evals.py                                  # default ollama at 127.0.0.1:11434, model 'sphragis-coder'
  ./run_evals.py --host 192.168.1.162 --port 11434
  ./run_evals.py --model qwen2.5-coder:7b         # to grade the un-fine-tuned baseline
"""
from __future__ import annotations

import argparse
import http.client
import json
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

EVALS = Path(__file__).resolve().parent / "sphragis_evals.jsonl"

SYSTEM_PROMPT = (
    "You are a technical assistant for Sphragis, a security-grade bare-metal "
    "Rust kernel for Apple M4. You answer questions about kernel internals, "
    "cryptography, audit history, and system administration. You are terse, "
    "technical, and never refuse legitimate questions. If you do not know "
    "something or it does not exist in our codebase, say so plainly."
)


def load_evals() -> list[dict]:
    rows: list[dict] = []
    with EVALS.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))
    return rows


def _post(host: str, port: int, body: bytes, timeout: int) -> dict:
    """One HTTP POST with retries on transient disconnect."""
    last_err: Exception | None = None
    for attempt in range(3):
        try:
            req = urllib.request.Request(
                f"http://{host}:{port}/v1/chat/completions",
                data=body,
                headers={"Content-Type": "application/json"},
            )
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                return json.loads(resp.read())
        except (urllib.error.URLError, OSError, http.client.RemoteDisconnected) as e:
            last_err = e
            time.sleep(2 + attempt * 3)
    raise last_err if last_err else RuntimeError("post failed without exception")


def ask_model(host: str, port: int, model: str, question: str, timeout: int,
              rag_context: str = "",
              tool_specs: list | None = None,
              tool_dispatch=None,
              max_tool_calls: int = 3) -> str:
    """Ask the model. If `tool_specs` and `tool_dispatch` are given,
    run a ReAct-style loop: model emits tool_calls -> we execute ->
    feed results back -> repeat until model returns plain text.
    Bounded by max_tool_calls so runaway loops can't hang the eval."""
    user_content = (rag_context + question) if rag_context else question
    messages: list[dict] = [
        {"role": "system", "content": SYSTEM_PROMPT},
        {"role": "user", "content": user_content},
    ]
    payload_base: dict = {
        "model": model,
        "stream": False,
        "temperature": 0.2,
    }
    if tool_specs:
        payload_base["tools"] = tool_specs
        payload_base["tool_choice"] = "auto"

    for _hop in range(max_tool_calls + 1):
        body = json.dumps({**payload_base, "messages": messages}).encode("utf-8")
        resp = _post(host, port, body, timeout)
        choice = resp["choices"][0]
        msg = choice.get("message", {})
        finish = choice.get("finish_reason", "")
        tool_calls = msg.get("tool_calls") or []

        if tool_calls and tool_dispatch:
            # Append the assistant turn that requested the calls, then
            # dispatch each call and append the result as a tool msg.
            messages.append({
                "role": "assistant",
                "content": msg.get("content") or "",
                "tool_calls": tool_calls,
            })
            for call in tool_calls:
                fn = call.get("function", {})
                name = fn.get("name", "")
                args = fn.get("arguments", "{}")
                result = tool_dispatch(name, args)
                messages.append({
                    "role": "tool",
                    "tool_call_id": call.get("id", ""),
                    "name": name,
                    "content": result,
                })
            continue

        # Plain text response — we're done.
        return msg.get("content") or ""

    return msg.get("content") or "[NETWORK ERROR] tool loop exceeded max_tool_calls"


def grade(rec: dict, answer: str) -> tuple[bool, str]:
    """Return (passed, reason)."""
    text = answer.lower()
    must_mention = [s.lower() for s in rec.get("must_mention", [])]
    must_not    = [s.lower() for s in rec.get("must_not_mention", [])]

    rule = rec.get("eval", "all_substrings")

    if rule == "all_substrings":
        missing = [s for s in must_mention if s not in text]
        if missing:
            return False, f"missing: {missing}"
    elif rule == "any_substrings":
        if must_mention and not any(s in text for s in must_mention):
            return False, f"none of {must_mention} present"
    elif rule == "regex":
        import re
        for pat in must_mention:
            if not re.search(pat, text):
                return False, f"regex /{pat}/ no match"
    else:
        return False, f"unknown eval rule: {rule}"

    bad = [s for s in must_not if s in text]
    if bad:
        return False, f"forbidden present: {bad}"

    return True, "ok"


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--host", default="127.0.0.1")
    p.add_argument("--port", type=int, default=11434)
    p.add_argument("--model", default="sphragis-coder")
    p.add_argument("--timeout", type=int, default=120)
    p.add_argument("--filter", default="", help="run only IDs starting with this prefix")
    p.add_argument("--show-answers", action="store_true",
                   help="print full answers, not just pass/fail")
    p.add_argument("--rag", action="store_true",
                   help="prepend top-3 BM25 retrieval from docs/rag_corpus before each question")
    p.add_argument("--tools", action="store_true",
                   help="enable tool-use loop (read_file, grep_source, read_concept_note)")
    args = p.parse_args()

    rows = load_evals()
    if args.filter:
        rows = [r for r in rows if r["id"].startswith(args.filter)]

    corpus = None
    if args.rag:
        from rag import Corpus
        corpus = Corpus.load()
        print(f"[evals] RAG enabled: {corpus.n} docs, {len(corpus.df)} terms")

    tool_specs = None
    tool_dispatch = None
    if args.tools:
        from tools import TOOL_SPECS, dispatch as _dispatch
        tool_specs = TOOL_SPECS
        tool_dispatch = _dispatch
        print(f"[evals] tools enabled: {[t['function']['name'] for t in tool_specs]}")

    print(f"[evals] running {len(rows)} questions against {args.host}:{args.port}/{args.model}")
    print()

    by_cat: dict[str, list[bool]] = {}
    fails: list[tuple[dict, str, str]] = []
    t0 = time.time()

    for rec in rows:
        rag_ctx = corpus.context_block(rec["question"], k=3) if corpus else ""
        try:
            answer = ask_model(args.host, args.port, args.model,
                               rec["question"], args.timeout,
                               rag_context=rag_ctx,
                               tool_specs=tool_specs,
                               tool_dispatch=tool_dispatch)
        except Exception as e:
            answer = f"[ASK_MODEL ERROR] {type(e).__name__}: {e}"

        ok, reason = grade(rec, answer)
        marker = "PASS" if ok else "FAIL"
        print(f"  [{marker}] {rec['id']:<14} {rec['question'][:80]}")
        if not ok:
            fails.append((rec, answer, reason))
        if args.show_answers or not ok:
            for line in answer.splitlines()[:6]:
                print(f"           > {line}")
        by_cat.setdefault(rec["category"], []).append(ok)

    dt = time.time() - t0

    print()
    print(f"[evals] {sum(b for r in by_cat.values() for b in r)}/{len(rows)} passed in {dt:.1f}s")
    print()
    print("[evals] by category:")
    for cat in sorted(by_cat):
        results = by_cat[cat]
        passed = sum(results)
        total  = len(results)
        pct    = 100.0 * passed / total if total else 0.0
        bar    = "#" * int(pct / 5)
        print(f"  {cat:<14} {passed:>2}/{total:<2} ({pct:5.1f}%) {bar}")

    if fails:
        print()
        print(f"[evals] failure breakdown ({len(fails)}):")
        for rec, answer, reason in fails:
            print(f"  - {rec['id']}: {reason}")

    return 0 if not fails else 1


if __name__ == "__main__":
    sys.exit(main())
