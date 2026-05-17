#!/usr/bin/env bash
# Sphragis release-verifier (SP-BLD-005).
#
# Verifies a Sphragis release artifact against its sigstore signature
# + Rekor transparency-log entry. Wraps `cosign verify-blob` with the
# Sphragis-specific identity assertions so an operator doesn't have to
# remember the exact certificate-identity-regexp / OIDC-issuer values.
#
# Usage:
#   tools/release-verifier/verify.sh <artifact> <artifact.sig> <artifact.crt>
#
# Example:
#   tools/release-verifier/verify.sh sphragis sphragis.sig sphragis.crt
#
# Exit code 0 = signature valid + Fulcio cert chains to sigstore root +
#               cert identity matches Sphragis repo + Rekor entry found.
# Exit code 1 = any verification step failed.
#
# Requires: cosign v2+ (https://github.com/sigstore/cosign).

set -euo pipefail

if [ $# -ne 3 ]; then
    echo "usage: $0 <artifact> <artifact.sig> <artifact.crt>" >&2
    exit 2
fi

ARTIFACT="$1"
SIGNATURE="$2"
CERTIFICATE="$3"

if ! command -v cosign >/dev/null 2>&1; then
    echo "[release-verifier] cosign not found; install from https://github.com/sigstore/cosign" >&2
    exit 2
fi

# cosign verify-blob checks:
#   1. Signature is valid over the artifact bytes.
#   2. Certificate chains to the sigstore Fulcio root.
#   3. Certificate identity matches our --certificate-identity-regexp
#      (the github.com/kadenlee1107/Sphragis OIDC subject).
#   4. The signature is recorded in the Rekor transparency log
#      (validated by default; --insecure-ignore-tlog to disable).
cosign verify-blob \
    --certificate "$CERTIFICATE" \
    --signature "$SIGNATURE" \
    --certificate-identity-regexp '^https://github.com/kadenlee1107/Sphragis/' \
    --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
    "$ARTIFACT"

echo "[release-verifier] PASS — $ARTIFACT verified against sigstore + Rekor"
