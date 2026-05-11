#!/usr/bin/env python3
"""Audit Cargo.lock against the RustSec advisory database.

Pulls the latest advisory-db (https://github.com/RustSec/advisory-db)
and matches every locked crate against its known vulnerabilities.
Reports per-crate findings with CVE, severity, fixed-version, and
the advisory ID for cross-reference.

Why not just `cargo audit`: we want this to run as a release gate
inside the repo without depending on an extra cargo plugin that
isn't part of the toolchain. Hand-rolled pure-Python script is
self-contained and easy to vendor into CI.

Run:
    python3 scripts/check_dependencies.py
    python3 scripts/check_dependencies.py --no-fetch     # use cached db
    python3 scripts/check_dependencies.py --json out.json

Exit code 0 = no findings. 1 = vulnerable crates found.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LOCK = REPO / "Cargo.lock"
CACHE = REPO / ".advisory_db_cache"

PKG_RE   = re.compile(r"^\s*\[\[package\]\]\s*$")
FIELD_RE = re.compile(r"^\s*(\w+)\s*=\s*\"?([^\"]+)\"?")


def parse_cargo_lock(path: Path) -> list[dict]:
    pkgs: list[dict] = []
    cur: dict | None = None
    with path.open("r", encoding="utf-8") as f:
        for line in f:
            if PKG_RE.match(line):
                if cur and cur.get("name"):
                    pkgs.append(cur)
                cur = {}
                continue
            if cur is None:
                continue
            m = FIELD_RE.match(line)
            if m:
                cur[m.group(1)] = m.group(2).strip()
    if cur and cur.get("name"):
        pkgs.append(cur)
    return pkgs


def fetch_advisory_db(no_fetch: bool) -> Path:
    """Clone or update the RustSec advisory-db. Returns the path to
    the local checkout root."""
    if no_fetch and CACHE.exists():
        return CACHE
    if not CACHE.exists():
        print(f"[deps] cloning RustSec advisory-db -> {CACHE.relative_to(REPO)}")
        subprocess.run([
            "git", "clone", "--depth", "1",
            "https://github.com/RustSec/advisory-db.git", str(CACHE),
        ], check=True)
    else:
        print(f"[deps] updating advisory-db cache")
        subprocess.run(["git", "-C", str(CACHE), "pull", "--quiet"], check=False)
    return CACHE


def parse_toml_frontmatter(text: str) -> dict:
    """Parse the TOML metadata block at the top of a RustSec advisory.
    Advisories use a Markdown file with TOML+YAML+plain-text mixed in;
    we want only the [advisory] / [versions] tables."""
    if not text.startswith("```toml"):
        return {}
    end = text.find("```", 7)
    if end == -1:
        return {}
    block = text[7:end]
    out: dict = {"advisory": {}, "versions": {}}
    section = None
    for line in block.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1]
            out.setdefault(section, {})
            continue
        m = re.match(r"^(\w+)\s*=\s*(.*)$", line)
        if not m or section is None:
            continue
        key, raw = m.group(1), m.group(2).strip()
        # Strip quotes / array brackets crudely.
        if raw.startswith('"') and raw.endswith('"'):
            raw = raw[1:-1]
        elif raw.startswith('['):
            # Parse a simple TOML array of strings.
            inner = raw.strip("[]")
            raw = [
                v.strip().strip('"')
                for v in inner.split(",") if v.strip()
            ]
        out[section][key] = raw
    return out


def scan_advisories(db_root: Path) -> dict[str, list[dict]]:
    """Walk crates/<name>/*.md advisories. Returns a map
    name -> [advisory_dict, ...]."""
    crates_dir = db_root / "crates"
    if not crates_dir.exists():
        print(f"[deps] advisory-db missing crates dir at {crates_dir}")
        return {}
    out: dict[str, list[dict]] = {}
    for adv_file in crates_dir.glob("*/*.md"):
        crate = adv_file.parent.name
        try:
            meta = parse_toml_frontmatter(adv_file.read_text(encoding="utf-8"))
        except Exception:
            continue
        adv = meta.get("advisory", {})
        if not adv:
            continue
        out.setdefault(crate, []).append({
            "id": adv.get("id", adv_file.stem),
            "title": adv.get("title", ""),
            "severity": adv.get("severity", "unknown"),
            "cve": adv.get("cve", ""),
            "fixed": meta.get("versions", {}).get("patched", []),
            "url": adv.get("url", ""),
            "_file": str(adv_file.relative_to(db_root)),
        })
    return out


def version_in_unpatched_range(installed: str, patched: list[str]) -> bool:
    """Coarse check: if `installed` doesn't match any of the patched
    ranges, treat as vulnerable. RustSec patched is a list of semver
    range strings like '>= 0.5.4'. We do a string-startswith fallback
    if we can't parse the range. False positives are acceptable
    (better than missing a real one)."""
    if not patched:
        return True
    for rule in patched:
        if not isinstance(rule, str):
            continue
        rule = rule.strip()
        m = re.match(r"^>=?\s*(\S+)$", rule)
        if m and version_cmp(installed, m.group(1)) >= 0:
            return False
        m = re.match(r"^=\s*(\S+)$", rule)
        if m and installed == m.group(1):
            return False
    return True


def version_cmp(a: str, b: str) -> int:
    pa = re.split(r"[.\-+]", a)
    pb = re.split(r"[.\-+]", b)
    for x, y in zip(pa, pb):
        try:
            xi, yi = int(x), int(y)
            if xi != yi:
                return -1 if xi < yi else 1
        except ValueError:
            if x != y:
                return -1 if x < y else 1
    if len(pa) != len(pb):
        return -1 if len(pa) < len(pb) else 1
    return 0


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--no-fetch", action="store_true",
                   help="use cached advisory-db; don't git pull")
    p.add_argument("--json", default=None,
                   help="write machine-readable report to this path")
    args = p.parse_args()

    if not LOCK.exists():
        print(f"[deps] {LOCK} not found")
        return 2

    db_root = fetch_advisory_db(args.no_fetch)
    advisories = scan_advisories(db_root)
    pkgs = parse_cargo_lock(LOCK)

    findings: list[dict] = []
    for pkg in pkgs:
        name = pkg["name"]
        ver  = pkg.get("version", "")
        for adv in advisories.get(name, []):
            if version_in_unpatched_range(ver, adv["fixed"]):
                findings.append({
                    "crate": name,
                    "version": ver,
                    "advisory_id": adv["id"],
                    "title": adv["title"],
                    "severity": adv["severity"],
                    "cve": adv["cve"],
                    "fixed_in": adv["fixed"],
                    "url": adv["url"],
                })

    if args.json:
        Path(args.json).write_text(json.dumps(findings, indent=2),
                                   encoding="utf-8")
        print(f"[deps] wrote {args.json}")

    if not findings:
        print(f"[deps] scanned {len(pkgs)} crates against {sum(len(v) for v in advisories.values())} advisories")
        print("[deps] no findings")
        return 0

    print(f"\n[deps] {len(findings)} vulnerable crate(s):")
    for f in findings:
        print(f"\n  {f['crate']} {f['version']}  ({f['advisory_id']})")
        if f["cve"]:
            print(f"    cve:      {f['cve']}")
        print(f"    severity: {f['severity']}")
        print(f"    title:    {f['title']}")
        if f["fixed_in"]:
            print(f"    fixed in: {', '.join(f['fixed_in']) if isinstance(f['fixed_in'], list) else f['fixed_in']}")
        if f["url"]:
            print(f"    url:      {f['url']}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
