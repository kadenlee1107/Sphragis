#!/usr/bin/env python3
"""Sign + verify the release-artifact set — gov-grade §3.11 closure.

`scripts/release_sign.py` exists to sign individual files for the
in-kernel `release-verify` shell command. This script is the
supply-chain bundle wrapper around it: it signs every artifact the
release process produces (`sbom.cdx.json` + `repro.sha256`),
emits sidecar `.sig` files, and appends a hash-linked entry to
`transparency.log` — the local analogue of Sigstore Rekor.

The transparency log is append-only + each entry chains via sha256
of the previous one. A downstream verifier checks (a) every
signature against the baked pubkey, and (b) the log's chain
integrity end-to-end, so an attacker who later substitutes an
artifact must also rewrite every subsequent log entry.

Usage:
    # Sign sbom.cdx.json + repro.sha256, append to transparency.log
    python3 scripts/sign_release_artifacts.py sign

    # Verify everything against ./release.key's pubkey
    python3 scripts/sign_release_artifacts.py verify
"""
from __future__ import annotations

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

# Local Rekor-compatible Merkle log (gov-grade §3.11 step 3).
sys.path.insert(0, str(REPO_ROOT / "scripts"))
from rekor_local import RekorLog

# Order matters: the artifact list is part of the log entry, so both
# sign + verify must agree byte-for-byte on which files go in and in
# what order.
ARTIFACT_PATHS = ["sbom.cdx.json", "repro.sha256"]

KEY_PATH = REPO_ROOT / "release.key"
LOG_PATH = REPO_ROOT / "transparency.log"


def load_signing_key() -> Ed25519PrivateKey:
    if not KEY_PATH.exists():
        print(f"[err] {KEY_PATH} missing — run `release_sign.py keygen` first",
              file=sys.stderr)
        sys.exit(1)
    raw = KEY_PATH.read_bytes()
    return Ed25519PrivateKey.from_private_bytes(raw)


def pubkey_hex_from_key(sk: Ed25519PrivateKey) -> str:
    return sk.public_key().public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    ).hex()


def sha256_file(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_short_sha() -> str:
    try:
        r = subprocess.run(
            ["git", "-C", str(REPO_ROOT), "rev-parse", "--short=12", "HEAD"],
            check=True, capture_output=True, text=True,
        )
        return r.stdout.strip()
    except subprocess.CalledProcessError:
        return "unknown"


def previous_log_hash() -> str:
    """sha256 of the last line of the log (or 64-zeros for genesis)."""
    if not LOG_PATH.exists():
        return "0" * 64
    lines = [ln for ln in LOG_PATH.read_text().splitlines() if ln.strip()]
    if not lines:
        return "0" * 64
    last = json.loads(lines[-1])
    return last["this_log_sha256"]


def canonical_entry_bytes(entry: dict) -> bytes:
    """Hash this against the log chain. Deterministic JSON (sorted
    keys, no whitespace) so a verifier reading the same file
    recomputes the same digest."""
    return json.dumps(entry, sort_keys=True, separators=(",", ":")).encode()


def do_sign() -> int:
    sk = load_signing_key()
    pubkey_hex = pubkey_hex_from_key(sk)
    timestamp = datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    sha = git_short_sha()
    prev_log = previous_log_hash()

    artifact_records = []
    for rel in ARTIFACT_PATHS:
        path = REPO_ROOT / rel
        if not path.exists():
            print(f"[err] missing artifact: {rel}", file=sys.stderr)
            return 1
        data = path.read_bytes()
        sig = sk.sign(data)
        sig_path = path.with_suffix(path.suffix + ".sig")
        sig_path.write_bytes(sig)
        artifact_sha = hashlib.sha256(data).hexdigest()
        print(f"[sign] {rel}: sha256={artifact_sha[:16]}... -> {sig_path.name}")
        artifact_records.append({
            "path":   rel,
            "sha256": artifact_sha,
            "sig":    sig.hex(),
            "size":   len(data),
        })

    log_entry_no_chain = {
        "timestamp":      timestamp,
        "git_sha":        sha,
        "pubkey":         pubkey_hex,
        "artifacts":      artifact_records,
        "prev_log_sha256": prev_log,
    }
    chain_hash = hashlib.sha256(
        canonical_entry_bytes(log_entry_no_chain)
    ).hexdigest()
    log_entry = {**log_entry_no_chain, "this_log_sha256": chain_hash}

    with LOG_PATH.open("a") as f:
        f.write(json.dumps(log_entry, sort_keys=True, separators=(",", ":")))
        f.write("\n")
    print(f"[sign] appended log entry; chain hash now {chain_hash[:16]}...")

    # Append to the Rekor-compatible Merkle log + refresh the
    # signed tree head. Verifiers reconstruct the root from
    # `rekor/log` themselves and check both (a) every inclusion
    # proof and (b) the STH signature. Same shape as real
    # Sigstore Rekor, hosted locally.
    rekor = RekorLog(REPO_ROOT)
    log_index, leaf_hex = rekor.append(log_entry)
    sth = rekor.write_sth(sk)
    print(f"[sign] rekor: logIndex={log_index}, leaf={leaf_hex[:16]}..., "
          f"treeSize={sth['treeSize']}, root={sth['rootHash'][:16]}...")
    print(f"[sign] pubkey: {pubkey_hex}")
    return 0


def do_verify() -> int:
    sk = load_signing_key()
    pk_bytes = sk.public_key().public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    )
    pk = Ed25519PublicKey.from_public_bytes(pk_bytes)
    pk_hex = pk_bytes.hex()

    if not LOG_PATH.exists():
        print(f"[err] no transparency log at {LOG_PATH}", file=sys.stderr)
        return 1

    lines = [ln for ln in LOG_PATH.read_text().splitlines() if ln.strip()]
    if not lines:
        print(f"[err] transparency log is empty", file=sys.stderr)
        return 1

    # ── Chain integrity ──
    prev = "0" * 64
    for i, ln in enumerate(lines):
        entry = json.loads(ln)
        if entry["prev_log_sha256"] != prev:
            print(f"[err] log entry {i}: prev_log_sha256 mismatch — chain broken",
                  file=sys.stderr)
            return 1
        entry_no_chain = {k: v for k, v in entry.items() if k != "this_log_sha256"}
        expected = hashlib.sha256(canonical_entry_bytes(entry_no_chain)).hexdigest()
        if expected != entry["this_log_sha256"]:
            print(f"[err] log entry {i}: this_log_sha256 mismatch — entry tampered",
                  file=sys.stderr)
            return 1
        prev = entry["this_log_sha256"]
    print(f"[verify] transparency log: {len(lines)} entry(ies), chain Ok")

    # ── Most-recent entry's signatures ──
    head = json.loads(lines[-1])
    if head["pubkey"] != pk_hex:
        print(f"[err] head entry signed by different key:\n"
              f"      log: {head['pubkey']}\n"
              f"      key: {pk_hex}",
              file=sys.stderr)
        return 1

    for rec in head["artifacts"]:
        path = REPO_ROOT / rec["path"]
        if not path.exists():
            print(f"[err] artifact missing: {rec['path']}", file=sys.stderr)
            return 1
        live_sha = sha256_file(path)
        if live_sha != rec["sha256"]:
            print(f"[err] artifact sha256 mismatch on {rec['path']}:\n"
                  f"       expected {rec['sha256']}\n"
                  f"       got      {live_sha}",
                  file=sys.stderr)
            return 1
        sig = bytes.fromhex(rec["sig"])
        try:
            pk.verify(sig, path.read_bytes())
        except InvalidSignature:
            print(f"[err] Ed25519 sig verification failed on {rec['path']}",
                  file=sys.stderr)
            return 1
        # Sidecar .sig file must also match.
        sig_path = path.with_suffix(path.suffix + ".sig")
        if not sig_path.exists() or sig_path.read_bytes() != sig:
            print(f"[err] sidecar {sig_path.name} missing or stale",
                  file=sys.stderr)
            return 1
        print(f"[verify]   {rec['path']}: sha256={rec['sha256'][:16]}... + sig OK")

    # ── Rekor: signed tree head + inclusion proofs for every entry ──
    rekor = RekorLog(REPO_ROOT)
    if rekor.tree_size() > 0:
        if not rekor.verify_signed_tree_head(pk):
            print(f"[err] rekor: signed-tree-head verification failed", file=sys.stderr)
            return 1
        sth = rekor.read_sth()
        for i in range(rekor.tree_size()):
            proof = rekor.inclusion_proof(i)
            if not rekor.verify_inclusion_proof(proof):
                print(f"[err] rekor: inclusion proof failed at logIndex={i}",
                      file=sys.stderr)
                return 1
        print(f"[verify] rekor: STH signed; {rekor.tree_size()} inclusion proof(s) verified")
        print(f"[verify]   root={sth['rootHash'][:16]}...")
    else:
        print(f"[verify] rekor: log empty (no entries to prove inclusion of)")

    print(f"[verify] PASS — {len(head['artifacts'])} artifact(s) signed by {pk_hex[:16]}...")
    return 0


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr); return 2
    cmd = sys.argv[1]
    if cmd == "sign":   return do_sign()
    if cmd == "verify": return do_verify()
    print(f"[err] unknown subcommand: {cmd}", file=sys.stderr); return 2


if __name__ == "__main__":
    sys.exit(main())
