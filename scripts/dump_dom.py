#!/usr/bin/env python3
"""Run Bat_OS's `dump-dom` command on a baked HTML file and print the
DOM tree to stdout.

Usage:
    python3 scripts/dump_dom.py [url]

Default URL: file:///bin/hello.html

This is the text-only sibling of `render_to_png.py`. Use it when you
want the structure of the DOM but don't need pixels — much faster than
the full render path because it skips layout + paint.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from scripts.lib.qemu_boot import boot  # noqa: E402


URL = sys.argv[1] if len(sys.argv) > 1 else "file:///bin/hello.html"


def main() -> int:
    with boot(log_prefix="dumpdom", timeout=60) as session:
        session.run(f"dump-dom {URL}".encode())
        session.expect_prompt(timeout=30)

    raw = session.log.read_text(encoding="utf-8", errors="replace")
    m = re.search(r"=== DOM ===\s*\n(.*?)\n\s*=== END ===", raw, re.DOTALL)
    if not m:
        print(f"[dump-dom] FAILED — no === DOM === markers in {session.log}",
              file=sys.stderr)
        return 1
    print(m.group(1))
    print(f"\n[dump-dom] log: {session.log}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
