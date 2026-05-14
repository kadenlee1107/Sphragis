#!/usr/bin/env python3
"""In-toto v0.9 attestations for Sphragis build steps — gov-grade §3.11.

`sign_release_artifacts.py` attests the OUTPUT artifacts. This
script goes a level deeper: each step of the supply chain
(generate SBOM, run reproducible build, sign artifacts) emits
its own signed in-toto `link` document binding inputs to
outputs by sha256. The collection forms a verifiable chain —
"the artifact named X was produced by step Y from inputs A,B,C,
attested by key K".

In-toto link schema (we follow v0.9, the same one slsa-verifier
consumes):
    {
      "_type":     "link",
      "name":      "<step name>",
      "command":   ["<argv>"],
      "materials": {"<path>": {"sha256": "<hex>"}, ...},
      "products":  {"<path>": {"sha256": "<hex>"}, ...},
      "byproducts": {"return-value": 0, "stdout": "", "stderr": ""},
      "environment": {"git_sha": "<rev>"}
    }
And the wrapper:
    {
      "signatures": [{"keyid": "<pk hex>", "sig": "<sig hex>"}],
      "signed": <link object above>
    }

Output goes to `attestations/<step>.intoto.jsonl` (one signed link
per file). The collection is committed alongside the SBOM + repro
sha + Rekor log.

Usage:
    python3 scripts/intoto_attest.py attest <step-name> \
      --materials path1 path2 \
      --products  path3 path4 \
      [--command "..."]
    python3 scripts/intoto_attest.py verify <step-name>
    python3 scripts/intoto_attest.py verify-all
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import subprocess
import sys
from datetime import datetime, timezone

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey, Ed25519PublicKey,
)

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent
KEY_PATH = REPO_ROOT / "release.key"
ATTEST_DIR = REPO_ROOT / "attestations"


def load_sk() -> Ed25519PrivateKey:
    if not KEY_PATH.exists():
        print(f"[err] {KEY_PATH} missing", file=sys.stderr); sys.exit(1)
    return Ed25519PrivateKey.from_private_bytes(KEY_PATH.read_bytes())


def pub_hex(sk: Ed25519PrivateKey) -> str:
    return sk.public_key().public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    ).hex()


def sha256_hex(path: pathlib.Path) -> str:
    if not path.exists():
        print(f"[err] missing artifact: {path}", file=sys.stderr); sys.exit(1)
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_sha() -> str:
    try:
        r = subprocess.run(
            ["git", "-C", str(REPO_ROOT), "rev-parse", "--short=12", "HEAD"],
            check=True, capture_output=True, text=True,
        )
        return r.stdout.strip()
    except subprocess.CalledProcessError:
        return "unknown"


def canonical(obj) -> bytes:
    return json.dumps(obj, sort_keys=True, separators=(",", ":")).encode()


def do_attest(args) -> int:
    ATTEST_DIR.mkdir(parents=True, exist_ok=True)
    sk = load_sk()

    materials = {
        m: {"sha256": sha256_hex(REPO_ROOT / m)} for m in args.materials
    }
    products = {
        p: {"sha256": sha256_hex(REPO_ROOT / p)} for p in args.products
    }
    link = {
        "_type":      "link",
        "name":       args.step,
        "command":    args.command or [args.step],
        "materials":  materials,
        "products":   products,
        "byproducts": {"timestamp": datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")},
        "environment": {"git_sha": git_sha()},
    }
    body = canonical(link)
    sig = sk.sign(body)
    envelope = {
        "signatures": [{"keyid": pub_hex(sk), "sig": sig.hex()}],
        "signed":     link,
    }
    out = ATTEST_DIR / f"{args.step}.intoto.json"
    out.write_text(canonical(envelope).decode() + "\n")
    print(f"[attest] {args.step}: {len(materials)} material(s), "
          f"{len(products)} product(s)")
    for p, m in products.items():
        print(f"[attest]   <- {p}: sha256={m['sha256'][:16]}...")
    print(f"[attest] wrote {out.relative_to(REPO_ROOT)}")
    return 0


def verify_one(step: str, pk: Ed25519PublicKey) -> bool:
    path = ATTEST_DIR / f"{step}.intoto.json"
    if not path.exists():
        print(f"[err] no attestation: {path}", file=sys.stderr); return False
    env = json.loads(path.read_text())
    body = canonical(env["signed"])
    sig = bytes.fromhex(env["signatures"][0]["sig"])
    try:
        pk.verify(sig, body)
    except InvalidSignature:
        print(f"[err] {step}: signature verification failed", file=sys.stderr)
        return False
    # Sanity: re-check every product's sha256 against the live file.
    for p, m in env["signed"]["products"].items():
        live = sha256_hex(REPO_ROOT / p)
        if live != m["sha256"]:
            print(f"[err] {step}: product {p} sha256 drifted "
                  f"({m['sha256'][:16]}... -> {live[:16]}...)",
                  file=sys.stderr)
            return False
    print(f"[verify] {step}: signature OK, {len(env['signed']['products'])} "
          f"product(s) sha256-match")
    return True


def do_verify(args) -> int:
    sk = load_sk()
    pk = sk.public_key()
    if args.step == "__all__":
        if not ATTEST_DIR.exists():
            print(f"[err] no attestations dir: {ATTEST_DIR}", file=sys.stderr)
            return 1
        steps = sorted(p.stem.replace(".intoto", "") for p in ATTEST_DIR.glob("*.intoto.json"))
        if not steps:
            print(f"[err] attestations dir is empty", file=sys.stderr); return 1
        ok = all(verify_one(s, pk) for s in steps)
        return 0 if ok else 1
    return 0 if verify_one(args.step, pk) else 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="cmd", required=True)

    a = sub.add_parser("attest")
    a.add_argument("step")
    a.add_argument("--materials", nargs="*", default=[])
    a.add_argument("--products",  nargs="*", default=[])
    a.add_argument("--command",   nargs="*", default=None)
    a.set_defaults(func=do_attest)

    v = sub.add_parser("verify")
    v.add_argument("step")
    v.set_defaults(func=do_verify)

    va = sub.add_parser("verify-all")
    va.set_defaults(func=lambda args: do_verify(argparse.Namespace(step="__all__")))

    args = ap.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
