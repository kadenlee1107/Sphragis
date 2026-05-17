#!/usr/bin/env python3
"""SP-ISO-004 lint (2026-05-16) — audit which `sys_*` syscall handlers
in src/ reference `cave_policy` (or an explicit no-check annotation).

The week-3-4 audit closed Cave-H6: `sys_connect` was missing a
cave-policy gate. The risk going forward is that a new syscall
handler is added without the same gate. This script enumerates
every `(pub )?fn sys_*` definition and classifies it:

  - PASS:  references `cave_policy` (or `active_has_cap`) within the
           function body, OR sits behind a comment annotation
           `// cave-policy: not-required (<reason>)`.
  - WARN:  no policy reference + no annotation. Reviewer must decide
           whether the handler is a trivial stub (ENOSYS / constant
           return — no policy needed) or a real gap (add the gate +
           annotation).

Exit code:
  0  if every handler is PASS (target state)
  0  if some are WARN and --no-fail-on-warn given (default during
     CI bring-up; flip to fail-on-warn after the existing 93 handlers
     have been triaged)
  1  if WARN handlers exist and --fail-on-warn

Usage:
    python3 scripts/audit_syscall_policy.py           # report mode
    python3 scripts/audit_syscall_policy.py --fail-on-warn

See `.github-workflows-pending/syscall-policy-lint.yml` for CI wiring.
"""

from __future__ import annotations
import argparse
import re
import sys
from pathlib import Path

SYSFN_RE = re.compile(r"^\s*(?:pub\s+)?fn\s+(sys_\w+)\s*\(", re.MULTILINE)
ANNOTATION_RE = re.compile(r"//\s*cave-policy:\s*not-required\s*\(([^)]+)\)")


def scan_file(path: Path) -> list[tuple[int, str, str]]:
    """Return [(line_no, handler_name, classification)] for one file.

    classification ∈ {"PASS-policy", "PASS-annotated", "WARN-no-gate"}.
    """
    out: list[tuple[int, str, str]] = []
    text = path.read_text(encoding="utf-8", errors="replace")
    lines = text.splitlines()

    for m in SYSFN_RE.finditer(text):
        line_no = text[:m.start()].count("\n") + 1
        name = m.group(1)
        # Extract the body roughly: from this line until the next
        # `^fn ` or `^pub fn ` at the same indent, or EOF.
        body = _extract_body(lines, line_no - 1)
        if ANNOTATION_RE.search(body):
            out.append((line_no, name, "PASS-annotated"))
            continue
        if "cave_policy" in body or "active_has_cap" in body or "has_cap(" in body:
            out.append((line_no, name, "PASS-policy"))
            continue
        out.append((line_no, name, "WARN-no-gate"))
    return out


def _extract_body(lines: list[str], start: int) -> str:
    """Naive body extraction: from `fn sys_X(...)` line through the
    matching `}` at the same indent depth. Good enough for static
    handler bodies; not a real Rust parser."""
    if start >= len(lines):
        return ""
    base_indent = len(lines[start]) - len(lines[start].lstrip())
    body_lines = [lines[start]]
    in_body = False
    depth = 0
    for ln in lines[start + 1:]:
        stripped = ln.lstrip()
        if not in_body:
            if "{" in stripped:
                in_body = True
                depth = stripped.count("{") - stripped.count("}")
                body_lines.append(ln)
                if depth == 0:
                    break
            else:
                body_lines.append(ln)
            continue
        depth += stripped.count("{") - stripped.count("}")
        body_lines.append(ln)
        if depth <= 0:
            break
    return "\n".join(body_lines)


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(prog="audit_syscall_policy")
    p.add_argument("--root", default="src", help="Directory to scan (default: src)")
    p.add_argument("--fail-on-warn", action="store_true",
                   help="Exit 1 if any handler is WARN (instead of just reporting)")
    args = p.parse_args(argv)

    root = Path(args.root)
    if not root.is_dir():
        print(f"audit_syscall_policy: {root} is not a directory", file=sys.stderr)
        return 2

    all_results: list[tuple[Path, int, str, str]] = []
    for src in sorted(root.rglob("*.rs")):
        for line_no, name, cls in scan_file(src):
            all_results.append((src, line_no, name, cls))

    pass_policy = [r for r in all_results if r[3] == "PASS-policy"]
    pass_annot = [r for r in all_results if r[3] == "PASS-annotated"]
    warns = [r for r in all_results if r[3] == "WARN-no-gate"]

    print(f"[syscall-policy-lint] scanned {len(all_results)} handlers under {args.root}/")
    print(f"[syscall-policy-lint]   PASS-policy    : {len(pass_policy)}")
    print(f"[syscall-policy-lint]   PASS-annotated : {len(pass_annot)}")
    print(f"[syscall-policy-lint]   WARN-no-gate   : {len(warns)}")

    if warns:
        print("[syscall-policy-lint] handlers without explicit policy gate or annotation:")
        for path, line_no, name, _ in warns:
            print(f"  {path}:{line_no}: {name}")
        print()
        print("[syscall-policy-lint] each WARN must EITHER reference cave_policy /")
        print("[syscall-policy-lint]   active_has_cap / has_cap(...), OR carry a comment")
        print("[syscall-policy-lint]   `// cave-policy: not-required (<reason>)` in the body")
        print("[syscall-policy-lint]   (e.g., for ENOSYS-stub handlers).")

    if warns and args.fail_on_warn:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
