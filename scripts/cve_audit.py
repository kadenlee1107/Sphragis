#!/usr/bin/env python3
"""Check Bat_OS's Cargo.lock against OSV.dev for known CVEs.

gov-grade §3.24 (continuous monitoring & vulnerability mgmt).
Hits OSV.dev's batch query endpoint (free, no auth, no rate limit
for normal-scale use) with one (cargo, name, version) tuple per
crate in Cargo.lock and reports any CVE / GHSA / RUSTSEC advisory
that affects the locked version.

Exits 0 if no advisories. Exits 1 if any advisory matches (so CI
can gate on it). Writes a JSON summary to `cve_audit.json` for
the supply-chain log.

Usage:
    python3 scripts/cve_audit.py              # scan, print summary
    python3 scripts/cve_audit.py --json       # JSON to stdout
    python3 scripts/cve_audit.py --quiet      # only exit code
"""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
import urllib.request
from datetime import datetime, timezone

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
LOCKFILE = REPO_ROOT / "Cargo.lock"
OUT_PATH = REPO_ROOT / "cve_audit.json"
IGNORE_PATH = REPO_ROOT / "cve_audit.ignore"

OSV_BATCH_URL = "https://api.osv.dev/v1/querybatch"


def load_ignore() -> dict[str, str]:
    """Parse `cve_audit.ignore` into {advisory_id: rationale}.
    Format: one record per non-comment, non-blank line, advisory
    ID followed by `:` and a free-text rationale describing why
    the finding doesn't apply to Bat_OS's actual usage."""
    if not IGNORE_PATH.exists():
        return {}
    out: dict[str, str] = {}
    for ln in IGNORE_PATH.read_text().splitlines():
        ln = ln.strip()
        if not ln or ln.startswith("#"):
            continue
        if ":" in ln:
            adv_id, _, rationale = ln.partition(":")
            out[adv_id.strip()] = rationale.strip()
    return out


def parse_cargo_lock(path: pathlib.Path) -> list[dict]:
    """Hand-roll Cargo.lock parsing — same shape as gen_sbom.py to
    avoid pulling in toml/tomli."""
    text = path.read_text()
    blocks = re.split(r"\n(?=\[\[package\]\])", text)
    pkgs: list[dict] = []
    for block in blocks:
        if not block.lstrip().startswith("[[package]]"):
            continue
        pkg: dict[str, str] = {}
        for line in block.splitlines():
            line = line.strip()
            if not line or line == "[[package]]" or "=" not in line:
                continue
            key, _, val = line.partition("=")
            val = val.strip()
            if val.startswith('"') and val.endswith('"'):
                pkg[key.strip()] = val[1:-1]
        if "name" in pkg and "version" in pkg:
            pkgs.append(pkg)
    return pkgs


def query_osv(packages: list[dict]) -> dict:
    """One batch POST to OSV.dev. Returns the parsed JSON."""
    queries = [
        {"package": {"ecosystem": "crates.io", "name": p["name"]},
         "version": p["version"]}
        for p in packages
    ]
    payload = json.dumps({"queries": queries}).encode()
    req = urllib.request.Request(
        OSV_BATCH_URL, data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read().decode())


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--json", action="store_true",
                    help="emit JSON summary to stdout instead of human-readable")
    ap.add_argument("--quiet", action="store_true",
                    help="suppress all output; rely on exit code")
    args = ap.parse_args()

    if not LOCKFILE.exists():
        print(f"[err] {LOCKFILE} not found", file=sys.stderr)
        return 2
    packages = parse_cargo_lock(LOCKFILE)
    if not packages:
        print(f"[err] no packages parsed from Cargo.lock", file=sys.stderr)
        return 2

    try:
        results = query_osv(packages)
    except Exception as e:
        print(f"[err] OSV query failed: {e}", file=sys.stderr)
        return 2

    ignored_map = load_ignore()
    hits: list[dict] = []
    ignored: list[dict] = []
    raw_results = results.get("results", [])
    for pkg, res in zip(packages, raw_results):
        advisories = res.get("vulns", []) if isinstance(res, dict) else []
        for adv in advisories:
            entry = {
                "crate":   pkg["name"],
                "version": pkg["version"],
                "id":      adv.get("id", "UNKNOWN"),
                "summary": adv.get("summary", ""),
                "aliases": adv.get("aliases", []),
            }
            # An advisory is suppressed only if THE ID OR ANY ALIAS
            # is in cve_audit.ignore — protects against the
            # common case where OSV references the bug under a
            # different ID than the RUSTSEC one we'd documented.
            ignore_keys = [entry["id"], *entry["aliases"]]
            matched_key = next((k for k in ignore_keys if k in ignored_map), None)
            if matched_key is not None:
                entry["ignored_under"] = matched_key
                entry["ignore_rationale"] = ignored_map[matched_key]
                ignored.append(entry)
            else:
                hits.append(entry)

    summary = {
        "timestamp":      datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "packages_scanned": len(packages),
        "advisories_found": len(hits),
        "advisories":      hits,
        "ignored_count":   len(ignored),
        "ignored":         ignored,
    }
    OUT_PATH.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")

    if args.json:
        print(json.dumps(summary, indent=2, sort_keys=True))
    elif not args.quiet:
        print(f"[cve-audit] scanned {len(packages)} crate(s) against OSV.dev")
        if ignored:
            print(f"[cve-audit] {len(ignored)} advisory hit(s) suppressed via cve_audit.ignore:")
            for h in ignored:
                print(f"  - {h['crate']} {h['version']}: {h['id']} "
                      f"({h['ignore_rationale'][:60]})")
        if not hits:
            print(f"[cve-audit] PASS — no unsuppressed advisories affect the locked versions")
        else:
            print(f"[cve-audit] FAIL — {len(hits)} new advisory hit(s):")
            for h in hits:
                print(f"  - {h['crate']} {h['version']}: {h['id']}")
                if h["summary"]:
                    print(f"      {h['summary'][:80]}")

    return 1 if hits else 0


if __name__ == "__main__":
    sys.exit(main())
