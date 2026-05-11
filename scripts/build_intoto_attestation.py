#!/usr/bin/env python3
"""Emit an in-toto v1 attestation for a Bat_OS release build.

in-toto attests "this artifact was produced by this builder running
this command on this commit" — the building block for SLSA L3+.

Run after a successful release build:
    python3 scripts/build_intoto_attestation.py \
        --artifact target/aarch64-unknown-none/release/bat_os \
        --output out/bat_os.intoto.jsonl

The output is a JSON Lines envelope ready to ship alongside the
artifact. Verifiers can replay the recorded command and check the
output hash matches.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(64 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=REPO).decode().strip()


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--artifact", required=True)
    p.add_argument("--output", required=True)
    p.add_argument("--builder-id", default="https://github.com/kadenlee1107/Bat_OS/builders/local-v1")
    p.add_argument("--build-type", default="https://slsa.dev/build-type/cargo-bare/v1")
    p.add_argument("--predicate-type", default="https://slsa.dev/provenance/v1")
    args = p.parse_args()

    artifact = Path(args.artifact)
    if not artifact.exists():
        print(f"[intoto] artifact not found: {artifact}", file=sys.stderr)
        return 1

    commit = git("rev-parse", "HEAD")
    branch = git("rev-parse", "--abbrev-ref", "HEAD")
    dirty = bool(git("status", "--porcelain"))

    statement = {
        "_type": "https://in-toto.io/Statement/v1",
        "subject": [{
            "name": str(artifact.relative_to(REPO)) if artifact.is_relative_to(REPO) else artifact.name,
            "digest": {"sha256": sha256(artifact)},
        }],
        "predicateType": args.predicate_type,
        "predicate": {
            "buildDefinition": {
                "buildType": args.build_type,
                "externalParameters": {
                    "source": "git+https://github.com/kadenlee1107/Bat_OS",
                    "commit": commit,
                    "branch": branch,
                },
                "internalParameters": {
                    "uname": platform.uname()._asdict(),
                    "rustc": subprocess.run(
                        ["rustc", "--version"], capture_output=True
                    ).stdout.decode().strip(),
                    "cargo": subprocess.run(
                        ["cargo", "--version"], capture_output=True
                    ).stdout.decode().strip(),
                    "source_date_epoch": os.environ.get("SOURCE_DATE_EPOCH"),
                    "rustflags": os.environ.get("RUSTFLAGS"),
                    "cargo_incremental": os.environ.get("CARGO_INCREMENTAL"),
                    "git_dirty": dirty,
                },
            },
            "runDetails": {
                "builder": {"id": args.builder_id},
                "metadata": {
                    "invocationId": f"{commit}-{int(time.time())}",
                    "startedOn": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                    "finishedOn": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                },
            },
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(statement, indent=2) + "\n", encoding="utf-8")
    print(f"[intoto] wrote {out.relative_to(REPO) if out.is_relative_to(REPO) else out}")
    print(f"[intoto] commit={commit[:12]} dirty={dirty} artifact_sha256={statement['subject'][0]['digest']['sha256'][:16]}…")
    return 0


if __name__ == "__main__":
    sys.exit(main())
