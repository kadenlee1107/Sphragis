#!/usr/bin/env python3
"""Generate a CycloneDX 1.5 SBOM for the Bat_OS kernel build.

CycloneDX (over SPDX) chosen for:
  * Stronger tooling support in the Rust ecosystem (cargo-cyclonedx).
  * Native support for cryptographic component manifests.
  * Smaller files for typical Rust workspaces.

Output: out/sbom.json — pin this in every release.

Run after editing Cargo.toml / Cargo.lock:
    python3 scripts/generate_sbom.py

The script intentionally does NOT shell out to cargo-cyclonedx — we
hand-roll the parser so this lives in the repo without a build-tool
dependency. cargo-cyclonedx would produce a richer doc; ours covers
the contract that matters for procurement and CVE triage:

    bomFormat / specVersion / serialNumber
    metadata.timestamp / metadata.tools / metadata.component (root)
    components[] with name / version / purl / type=library + a
        cryptographic component sub-list for the primitives we ship
        (AES, SHA-2, SHA-3, BLAKE3, ChaCha20-Poly1305, XChaCha20-
         Poly1305, AES-XTS, Argon2id, Ed25519, X25519, ML-KEM-768,
         ML-DSA-65, ECDSA P-256, ECDSA P-384, RSA-PSS, HOTP, TOTP).
"""
from __future__ import annotations

import hashlib
import json
import re
import sys
import time
import uuid
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LOCK = REPO / "Cargo.lock"
OUT  = REPO / "out" / "sbom.json"

PURL_TPL = "pkg:cargo/{name}@{version}"

# Bat_OS kernel-side cryptographic component manifest. Matches what
# `src/crypto/` actually exposes. Keep in sync when adding primitives.
CRYPTO_COMPONENTS = [
    # name,                 implements,                                 oid_or_id
    ("aes-128-ctr",         "FIPS 197 (AES) in CTR mode",               "2.16.840.1.101.3.4.1.21"),
    ("aes-256-ctr",         "FIPS 197 (AES) in CTR mode",               "2.16.840.1.101.3.4.1.41"),
    ("aes-128-gcm",         "FIPS 197 + SP 800-38D",                    "2.16.840.1.101.3.4.1.6"),
    ("aes-256-gcm",         "FIPS 197 + SP 800-38D",                    "2.16.840.1.101.3.4.1.46"),
    ("aes-128-xts",         "SP 800-38E XTS-AES",                       "1.3.111.2.1619.0.1.1"),
    ("aes-256-xts",         "SP 800-38E XTS-AES",                       "1.3.111.2.1619.0.1.2"),
    ("sha-256",             "FIPS 180-4",                               "2.16.840.1.101.3.4.2.1"),
    ("sha-384",             "FIPS 180-4",                               "2.16.840.1.101.3.4.2.2"),
    ("sha3-256",            "FIPS 202",                                 "2.16.840.1.101.3.4.2.8"),
    ("sha3-384",            "FIPS 202",                                 "2.16.840.1.101.3.4.2.9"),
    ("sha3-512",            "FIPS 202",                                 "2.16.840.1.101.3.4.2.10"),
    ("shake128",            "FIPS 202 XOF",                             "2.16.840.1.101.3.4.2.11"),
    ("shake256",            "FIPS 202 XOF",                             "2.16.840.1.101.3.4.2.12"),
    ("blake3",              "BLAKE3 hash + keyed MAC + KDF",            "n/a"),
    ("hmac-sha256",         "FIPS 198-1 over SHA-256",                  "1.2.840.113549.2.9"),
    ("hmac-sha384",         "FIPS 198-1 over SHA-384",                  "1.2.840.113549.2.10"),
    ("hkdf-sha256",         "RFC 5869 over SHA-256",                    "n/a"),
    ("hkdf-sha384",         "RFC 5869 over SHA-384",                    "n/a"),
    ("chacha20-poly1305",   "RFC 8439 AEAD",                            "n/a"),
    ("xchacha20-poly1305",  "draft-irtf-cfrg-xchacha-03 AEAD",          "n/a"),
    ("argon2id",            "RFC 9106 password hashing",                "n/a"),
    ("ed25519",             "RFC 8032",                                 "1.3.101.112"),
    ("x25519",              "RFC 7748",                                 "1.3.101.110"),
    ("ecdsa-p256",          "FIPS 186-5 over secp256r1",                "1.2.840.10045.3.1.7"),
    ("ecdsa-p384",          "FIPS 186-5 over secp384r1",                "1.3.132.0.34"),
    ("rsa-pss",             "RFC 8017 RSA-PSS",                         "1.2.840.113549.1.1.10"),
    ("ml-kem-768",          "FIPS 203 (post-quantum KEM)",              "n/a"),
    ("ml-dsa-65",           "FIPS 204 (post-quantum signature)",        "n/a"),
    ("hotp-sha256",         "RFC 4226 HMAC-based OTP",                  "n/a"),
    ("totp-sha256",         "RFC 6238 time-based OTP",                  "n/a"),
]


def parse_cargo_lock(path: Path) -> list[dict]:
    """Walk Cargo.lock and emit one CycloneDX component per [[package]]."""
    pkgs: list[dict] = []
    cur: dict | None = None
    pkg_re   = re.compile(r"^\s*\[\[package\]\]\s*$")
    field_re = re.compile(r"^\s*(\w+)\s*=\s*\"?([^\"]+)\"?")
    with path.open("r", encoding="utf-8") as f:
        for line in f:
            if pkg_re.match(line):
                if cur and cur.get("name"):
                    pkgs.append(cur)
                cur = {}
                continue
            if cur is None:
                continue
            m = field_re.match(line)
            if not m:
                continue
            key, val = m.group(1), m.group(2).strip()
            if key in ("name", "version", "source", "checksum"):
                cur[key] = val
        if cur and cur.get("name"):
            pkgs.append(cur)
    return pkgs


def to_cyclonedx_component(pkg: dict) -> dict:
    name = pkg["name"]
    version = pkg.get("version", "")
    purl = PURL_TPL.format(name=name, version=version)
    comp: dict = {
        "type": "library",
        "bom-ref": f"{name}@{version}",
        "name": name,
        "version": version,
        "purl": purl,
    }
    if (cs := pkg.get("checksum")):
        comp["hashes"] = [{"alg": "SHA-256", "content": cs}]
    if (src := pkg.get("source")):
        if "git+" in src:
            comp["externalReferences"] = [{"type": "vcs", "url": src.split("git+", 1)[1]}]
        elif "registry+" in src:
            comp["externalReferences"] = [{"type": "distribution",
                                           "url": src.split("registry+", 1)[1]}]
    return comp


def crypto_component(name: str, implements: str, oid: str) -> dict:
    return {
        "type": "cryptographic-asset",
        "bom-ref": f"crypto:{name}",
        "name": name,
        "cryptoProperties": {
            "assetType": "algorithm",
            "algorithmProperties": {
                "primitive": name,
                "implementationPlatform": "aarch64-unknown-none",
                "certificationLevel": ["none"],
                "mode": "approved-mode-eligible",
                "implementation": implements,
                "oid": oid,
            },
        },
    }


def main() -> int:
    if not LOCK.exists():
        print(f"[sbom] {LOCK} not found — run `cargo generate-lockfile` first")
        return 1
    OUT.parent.mkdir(parents=True, exist_ok=True)

    pkgs = parse_cargo_lock(LOCK)

    components = [to_cyclonedx_component(p) for p in pkgs]
    components += [crypto_component(n, i, o) for (n, i, o) in CRYPTO_COMPONENTS]

    root = {
        "type": "operating-system",
        "bom-ref": "bat-os",
        "name": "bat_os",
        "version": "0.1.0",
        "description": "Security-grade bare-metal Rust kernel for Apple M4.",
    }

    sbom = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "version": 1,
        "serialNumber": f"urn:uuid:{uuid.uuid4()}",
        "metadata": {
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "tools": [{
                "vendor": "bat_os",
                "name": "generate_sbom.py",
                "version": "1.0",
            }],
            "component": root,
        },
        "components": components,
    }

    OUT.write_text(json.dumps(sbom, indent=2) + "\n", encoding="utf-8")
    digest = hashlib.sha256(OUT.read_bytes()).hexdigest()
    print(f"[sbom] {len(pkgs)} cargo packages + {len(CRYPTO_COMPONENTS)} crypto assets")
    print(f"[sbom] wrote {OUT.relative_to(REPO)} ({OUT.stat().st_size:,} bytes)")
    print(f"[sbom] sha256={digest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
