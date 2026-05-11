#!/usr/bin/env python3
"""Grade a Bat_OS LoRA against bat_os_evals.jsonl.

Sends each question to an OpenAI-compatible inference endpoint
(ollama, vLLM, etc) and grades the answer with substring-match
rules. Prints per-category and per-question results.

Usage:
  ./run_evals.py                                  # default ollama at 127.0.0.1:11434, model 'bat-os-coder'
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

EVALS = Path(__file__).resolve().parent / "bat_os_evals.jsonl"

SYSTEM_PROMPT = (
    "You are a technical assistant for Bat_OS, a security-grade bare-metal "
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


def ask_model(host: str, port: int, model: str, question: str, timeout: int) -> str:
    body = json.dumps({
        "model": model,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": question},
        ],
        "stream": False,
        "temperature": 0.2,
    }).encode("utf-8")
    last_err: Exception | None = None
    for attempt in range(3):
        try:
            req = urllib.request.Request(
                f"http://{host}:{port}/v1/chat/completions",
                data=body,
                headers={"Content-Type": "application/json"},
            )
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                payload = json.loads(resp.read())
            return payload["choices"][0]["message"]["content"]
        except (urllib.error.URLError, OSError, http.client.RemoteDisconnected) as e:
            last_err = e
            time.sleep(2 + attempt * 3)
    raise last_err if last_err else RuntimeError("ask_model failed without exception")


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
    p.add_argument("--model", default="bat-os-coder")
    p.add_argument("--timeout", type=int, default=120)
    p.add_argument("--filter", default="", help="run only IDs starting with this prefix")
    p.add_argument("--show-answers", action="store_true",
                   help="print full answers, not just pass/fail")
    args = p.parse_args()

    rows = load_evals()
    if args.filter:
        rows = [r for r in rows if r["id"].startswith(args.filter)]

    print(f"[evals] running {len(rows)} questions against {args.host}:{args.port}/{args.model}")
    print()

    by_cat: dict[str, list[bool]] = {}
    fails: list[tuple[dict, str, str]] = []
    t0 = time.time()

    for rec in rows:
        try:
            answer = ask_model(args.host, args.port, args.model, rec["question"], args.timeout)
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
