"""Local Rekor-compatible transparency log — gov-grade §3.11.

The earlier `sign_release_artifacts.py` produces a hash-linked
`transparency.log` (each entry chains via sha256 of the previous).
This module upgrades the chain to a real RFC 6962 (Certificate
Transparency / Sigstore Rekor) Merkle tree: every leaf is the
canonical sha256 of one signed artifact bundle, and the tree's
root is signed by the release key. Verifiers compute Merkle
inclusion proofs themselves from `rekor.log` and validate them
against the signed root in `rekor.sth` — exactly the property
real Rekor provides, just hosted out of `./rekor/` instead of
`rekor.sigstore.dev`.

Storage layout (gitignored / regenerable):
  rekor/log     — append-only line-per-entry JSON (the "data")
  rekor/sth     — signed tree head (Merkle root + size + signature)

API:
  RekorLog(repo_root)
    .append(record_dict)            -> (log_index, leaf_hash_hex)
    .inclusion_proof(log_index)     -> (leaf, path[], tree_size, root)
    .verify_inclusion_proof(...)    -> bool
    .signed_tree_head(sk)           -> dict (signs current root)
    .verify_signed_tree_head(pk)    -> bool
"""
from __future__ import annotations

import hashlib
import json
import pathlib
from typing import Any

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey, Ed25519PublicKey,
)

# RFC 6962 §2.1 domain separators.
LEAF_PREFIX = b"\x00"
NODE_PREFIX = b"\x01"


def _sha256(*chunks: bytes) -> bytes:
    h = hashlib.sha256()
    for c in chunks:
        h.update(c)
    return h.digest()


def _leaf_hash(data: bytes) -> bytes:
    return _sha256(LEAF_PREFIX, data)


def _node_hash(left: bytes, right: bytes) -> bytes:
    return _sha256(NODE_PREFIX, left, right)


def _merkle_root(leaves: list[bytes]) -> bytes:
    """RFC 6962 Merkle tree root. Empty tree -> sha256 of empty
    string (per spec's empty-tree convention)."""
    if not leaves:
        return _sha256(b"")
    if len(leaves) == 1:
        return leaves[0]
    # Find largest power of 2 <= len-1.
    k = 1
    while k * 2 < len(leaves):
        k *= 2
    return _node_hash(
        _merkle_root(leaves[:k]),
        _merkle_root(leaves[k:]),
    )


def _audit_path(leaves: list[bytes], idx: int) -> list[bytes]:
    """RFC 6962 §2.1.1 path for leaf `idx` (0-indexed) within
    a tree of `len(leaves)`. Returns sibling hashes along the
    path, root-toward-leaf or leaf-toward-root — we use
    leaf-toward-root order so verification walks bottom-up."""
    if len(leaves) <= 1:
        return []
    k = 1
    while k * 2 < len(leaves):
        k *= 2
    if idx < k:
        # Left subtree contains the leaf; right sibling is the
        # full right subtree's root.
        return _audit_path(leaves[:k], idx) + [_merkle_root(leaves[k:])]
    else:
        return _audit_path(leaves[k:], idx - k) + [_merkle_root(leaves[:k])]


def _verify_path(leaf_hash: bytes, idx: int, tree_size: int,
                 path: list[bytes], expected_root: bytes) -> bool:
    """Reconstruct the root from a leaf + audit path and compare."""
    if tree_size == 0:
        return False
    if idx >= tree_size:
        return False
    fn = idx
    sn = tree_size - 1
    r = leaf_hash
    for p in path:
        if sn == 0:
            return False
        if (fn & 1) == 1 or fn == sn:
            r = _node_hash(p, r)
            while not ((fn & 1) == 1 or fn == 0):
                fn >>= 1
                sn >>= 1
            fn >>= 1
            sn >>= 1
        else:
            r = _node_hash(r, p)
            fn >>= 1
            sn >>= 1
    while sn > 0:
        # Sanity: should have consumed everything.
        return False
    return r == expected_root


class RekorLog:
    def __init__(self, repo_root: pathlib.Path):
        self.dir = repo_root / "rekor"
        self.dir.mkdir(parents=True, exist_ok=True)
        self.log_path = self.dir / "log"
        self.sth_path = self.dir / "sth"

    # ── leaves ──
    def _read_entries(self) -> list[dict[str, Any]]:
        if not self.log_path.exists():
            return []
        return [
            json.loads(ln) for ln in self.log_path.read_text().splitlines()
            if ln.strip()
        ]

    def _leaf_bytes(self, entry: dict[str, Any]) -> bytes:
        # Canonical encoding: sorted-keys / no-whitespace JSON.
        return json.dumps(entry, sort_keys=True, separators=(",", ":")).encode()

    def append(self, record: dict[str, Any]) -> tuple[int, str]:
        existing = self._read_entries()
        log_index = len(existing)
        entry = {"logIndex": log_index, "body": record}
        with self.log_path.open("a") as f:
            f.write(json.dumps(entry, sort_keys=True, separators=(",", ":")))
            f.write("\n")
        leaf_hex = _leaf_hash(self._leaf_bytes(entry)).hex()
        return log_index, leaf_hex

    # ── proofs + root ──
    def _leaves(self) -> list[bytes]:
        return [_leaf_hash(self._leaf_bytes(e)) for e in self._read_entries()]

    def root(self) -> bytes:
        return _merkle_root(self._leaves())

    def tree_size(self) -> int:
        return len(self._read_entries())

    def inclusion_proof(self, log_index: int) -> dict[str, Any]:
        leaves = self._leaves()
        if log_index >= len(leaves):
            raise ValueError(f"log_index {log_index} out of range")
        path = _audit_path(leaves, log_index)
        return {
            "logIndex":  log_index,
            "treeSize":  len(leaves),
            "rootHash":  self.root().hex(),
            "auditPath": [p.hex() for p in path],
            "leafHash":  leaves[log_index].hex(),
        }

    def verify_inclusion_proof(self, proof: dict[str, Any]) -> bool:
        try:
            leaf_hash = bytes.fromhex(proof["leafHash"])
            root      = bytes.fromhex(proof["rootHash"])
            path      = [bytes.fromhex(p) for p in proof["auditPath"]]
        except (KeyError, ValueError):
            return False
        return _verify_path(
            leaf_hash, proof["logIndex"], proof["treeSize"], path, root,
        )

    # ── signed tree head ──
    def write_sth(self, sk: Ed25519PrivateKey) -> dict[str, Any]:
        root = self.root()
        size = self.tree_size()
        sth_body = {"treeSize": size, "rootHash": root.hex()}
        sig = sk.sign(json.dumps(sth_body, sort_keys=True, separators=(",", ":")).encode())
        sth = {**sth_body, "signature": sig.hex()}
        self.sth_path.write_text(
            json.dumps(sth, sort_keys=True, separators=(",", ":")) + "\n"
        )
        return sth

    def read_sth(self) -> dict[str, Any] | None:
        if not self.sth_path.exists():
            return None
        return json.loads(self.sth_path.read_text())

    def verify_signed_tree_head(self, pk: Ed25519PublicKey) -> bool:
        sth = self.read_sth()
        if sth is None:
            return False
        try:
            sig = bytes.fromhex(sth["signature"])
        except (KeyError, ValueError):
            return False
        body = {k: v for k, v in sth.items() if k != "signature"}
        # Sanity: stored root must match what we recompute now.
        if sth["rootHash"] != self.root().hex():
            return False
        if sth["treeSize"] != self.tree_size():
            return False
        try:
            pk.verify(sig, json.dumps(body, sort_keys=True, separators=(",", ":")).encode())
        except InvalidSignature:
            return False
        return True
