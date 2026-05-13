#!/usr/bin/env python3
"""Emit a CycloneDX 1.5 SBOM for the Bat_OS build — gov-grade §3.11
supply-chain hygiene.

Walks `Cargo.lock` (the single source of truth for what actually
shipped) and emits one component per `[[package]]` entry, populating
the SHA-256 hashes from each package's `checksum` field. Vendored
dependencies under `external/` are marked with the local-path
purl-extension so a downstream consumer can tell registry pulls
apart from in-tree forks.

Output: `sbom.cdx.json` at the repo root by default. CycloneDX 1.5
schema is permissively-licensed and tool-supported (cyclonedx-cli,
SPDX converters, NVD scanner glue). The file is checked in alongside
each release tag.

Usage:
    python3 scripts/gen_sbom.py [--out PATH]
"""
from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
import subprocess
import sys
from datetime import datetime, timezone

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent


def parse_cargo_lock(path: pathlib.Path) -> list[dict]:
    """Hand-parse Cargo.lock. Avoids pulling toml/tomli into the
    script's dep surface — Cargo.lock has a strict, predictable
    structure that's safe to parse with regex on `[[package]]`
    blocks."""
    text = path.read_text()
    blocks = re.split(r"\n(?=\[\[package\]\])", text)
    out: list[dict] = []
    for block in blocks:
        if not block.lstrip().startswith("[[package]]"):
            continue
        pkg: dict[str, object] = {}
        for line in block.splitlines():
            line = line.strip()
            if not line or line == "[[package]]":
                continue
            if "=" not in line:
                continue
            key, _, val = line.partition("=")
            key = key.strip()
            val = val.strip()
            if val.startswith('"') and val.endswith('"'):
                pkg[key] = val[1:-1]
            elif val.startswith("["):
                # dependencies — collect lines until "]"
                deps: list[str] = []
                # cargo encodes dependencies inline OR over multiple
                # lines. The block-level split is fine; the
                # individual `dependencies = [...]` value may span
                # lines too, but for SBOM purposes we only need the
                # name list. Pull names with a separate pass below.
                pkg[key] = val
        # Re-extract dependencies cleanly.
        deps_match = re.search(r"dependencies\s*=\s*\[\s*([^\]]*?)\s*\]",
                               block, re.DOTALL)
        if deps_match:
            raw = deps_match.group(1)
            # Each dep entry is `"<name>[ <version>]"`. Match the
            # opening quote followed by an identifier; the optional
            # space-separated version is ignored. The earlier
            # `"([^" ]+)` regex also matched the comma-newline that
            # follows each closing quote — fixed by anchoring the
            # capture class to identifier chars.
            names = re.findall(r'"([A-Za-z][A-Za-z0-9_-]*)', raw)
            pkg["dependencies"] = names
        out.append(pkg)
    return out


def purl_for(pkg: dict, vendored_names: set[str]) -> str:
    name = pkg.get("name", "")
    version = pkg.get("version", "")
    source = pkg.get("source", "")
    base = f"pkg:cargo/{name}@{version}"
    if name in vendored_names:
        return f"{base}?vendored=true"
    if source.startswith("registry+https://github.com/rust-lang/crates.io-index"):
        return base
    if source.startswith("git+"):
        return f"{base}?vcs={source[4:]}"
    return base


def vendored_dirs() -> set[str]:
    ext = REPO_ROOT / "external"
    if not ext.is_dir():
        return set()
    return {p.name for p in ext.iterdir() if p.is_dir()}


def git_short_sha() -> str:
    try:
        r = subprocess.run(
            ["git", "-C", str(REPO_ROOT), "rev-parse", "--short=12", "HEAD"],
            check=True, capture_output=True, text=True,
        )
        return r.stdout.strip()
    except subprocess.CalledProcessError:
        return "unknown"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--out", default=str(REPO_ROOT / "sbom.cdx.json"))
    args = ap.parse_args()

    lockfile = REPO_ROOT / "Cargo.lock"
    if not lockfile.exists():
        print("[gen-sbom] Cargo.lock not found", file=sys.stderr)
        return 1

    packages = parse_cargo_lock(lockfile)
    vendored = vendored_dirs()

    components = []
    for pkg in packages:
        name = pkg.get("name", "")
        version = pkg.get("version", "")
        if not name or not version:
            continue
        comp = {
            "type": "library",
            "bom-ref": f"{name}@{version}",
            "name": name,
            "version": version,
            "purl": purl_for(pkg, vendored),
            "scope": "required",
        }
        checksum = pkg.get("checksum")
        if checksum:
            comp["hashes"] = [{"alg": "SHA-256", "content": checksum}]
        deps = pkg.get("dependencies", [])
        if deps:
            comp["properties"] = [
                {"name": "dependency.name", "value": d}
                for d in deps
            ]
        components.append(comp)

    now = datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    sbom = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "serialNumber": f"urn:uuid:{hashlib.sha256(f'{now}-{git_short_sha()}'.encode()).hexdigest()[:32]}",
        "version": 1,
        "metadata": {
            "timestamp": now,
            "tools": [
                {
                    "vendor": "Bat_OS",
                    "name": "scripts/gen_sbom.py",
                    "version": "1.0",
                }
            ],
            "component": {
                "type": "operating-system",
                "name": "bat_os",
                "version": git_short_sha(),
                "purl": f"pkg:generic/bat_os@{git_short_sha()}",
            },
        },
        "components": components,
    }

    out_path = pathlib.Path(args.out)
    out_path.write_text(json.dumps(sbom, indent=2, sort_keys=True) + "\n")
    print(f"[gen-sbom] wrote {out_path}")
    print(f"[gen-sbom]   components: {len(components)}")
    print(f"[gen-sbom]   vendored:   {len(vendored)} ({', '.join(sorted(vendored)) or 'none'})")
    print(f"[gen-sbom]   build sha:  {git_short_sha()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
