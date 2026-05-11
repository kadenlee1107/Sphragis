#!/usr/bin/env bash
# Sign a Bat_OS release artifact with minisign.
#
# Why minisign (and not cosign / Sigstore-keyless)?
#   * Minisign produces a tiny detached signature that's trivial to
#     verify offline with a single Ed25519 public key. No CT log
#     dependency, no OIDC, no third-party trust root.
#   * Sigstore-keyless is great for public OSS distribution but
#     binds your signing identity to a public OIDC IdP. That doesn't
#     fit Bat_OS's air-gap and government-grade positioning.
#   * For a follow-up: ALSO produce a Sigstore signature for the
#     public Github release flow, alongside the minisign one. They
#     compose; verifiers can require either or both.
#
# Setup (one-time, on the signer's machine):
#   minisign -G -p $HOME/.bat_os/release_key.pub \
#                -s $HOME/.bat_os/release_key.sec
#   cp $HOME/.bat_os/release_key.pub keys/release.pub
#
# Sign a release artifact:
#   bash scripts/sign_release.sh target/aarch64-unknown-none/release/bat_os
#
# Verify (anywhere):
#   minisign -V -p keys/release.pub -m <artifact> -x <artifact>.minisig
set -euo pipefail

ARTIFACT="${1:?usage: sign_release.sh <artifact>}"
SIG="$ARTIFACT.minisig"
SECKEY="${MINISIGN_SECKEY:-$HOME/.bat_os/release_key.sec}"
PUBKEY="${MINISIGN_PUBKEY:-keys/release.pub}"

if ! command -v minisign >/dev/null 2>&1; then
    echo "[sign] minisign not installed — brew install minisign / apt install minisign"
    exit 2
fi
if [ ! -f "$ARTIFACT" ]; then
    echo "[sign] artifact not found: $ARTIFACT"
    exit 2
fi
if [ ! -f "$SECKEY" ]; then
    echo "[sign] signing key not found at $SECKEY"
    echo "       generate with: minisign -G -s $SECKEY -p ${PUBKEY:-keys/release.pub}"
    exit 2
fi

minisign -S -s "$SECKEY" -m "$ARTIFACT" -x "$SIG" \
    -t "bat_os release $(date -u +%Y-%m-%dT%H:%M:%SZ) sha256=$(shasum -a 256 "$ARTIFACT" | awk '{print $1}')"

echo "[sign] signed -> $SIG"
echo "[sign] verify with:"
echo "       minisign -V -p $PUBKEY -m $ARTIFACT -x $SIG"
