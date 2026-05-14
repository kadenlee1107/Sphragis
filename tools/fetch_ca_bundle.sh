#!/bin/bash
# tools/fetch_ca_bundle.sh — STUMP #139
#
# Re-fetches the curated root CA DER bundle that Sphragis ships in
# `src/net/ca_certs/`. Roots rarely rotate, but when they do (or
# when STUMP #140 unlocks RSA leaf verification and we want to add
# more) this is the one-shot refresh.
#
# Each CA publishes their root cert at a stable URL. We fetch the
# DER form directly so there's no PEM-decode step in the build.
#
# Usage:
#   bash tools/fetch_ca_bundle.sh
#
# Then `make build` to bake the updated DERs into the kernel.

set -euo pipefail

DEST="src/net/ca_certs"
mkdir -p "$DEST"

declare -a ROOTS=(
    "isrg_root_x1.der|https://letsencrypt.org/certs/isrgrootx1.der"
    "isrg_root_x2.der|https://letsencrypt.org/certs/isrg-root-x2.der"
    "amazon_root_ca1.der|https://www.amazontrust.com/repository/AmazonRootCA1.cer"
    "digicert_global_root_ca.der|https://cacerts.digicert.com/DigiCertGlobalRootCA.crt"
    "digicert_global_root_g2.der|https://cacerts.digicert.com/DigiCertGlobalRootG2.crt"
)

for entry in "${ROOTS[@]}"; do
    file="${entry%%|*}"
    url="${entry##*|}"
    echo "[ca_bundle] fetching $file from $url"
    curl -sS -o "$DEST/$file" "$url"

    # Sanity-check it's a valid DER cert before we trust it.
    if ! openssl x509 -inform der -in "$DEST/$file" -noout -subject \
            > /dev/null 2>&1; then
        echo "[ca_bundle] ERROR: $file failed openssl validation" >&2
        exit 1
    fi
    subject=$(openssl x509 -inform der -in "$DEST/$file" -noout -subject \
                  | sed 's/^subject=//')
    size=$(wc -c < "$DEST/$file" | awk '{print $1}')
    echo "  ✓ $size bytes — $subject"
done

echo
echo "[ca_bundle] done. $(ls -1 "$DEST"/*.der | wc -l | tr -d ' ') roots in $DEST."
echo "  Run \`make build\` to bake the new bundle into the kernel."
