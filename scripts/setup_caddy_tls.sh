#!/usr/bin/env bash
# Stand up caddy in front of ollama with a self-signed TLS cert,
# then print the SHA-256 fingerprint to paste into
# src/ai/policy.rs::PINNED_CERT_SHA256.
#
# Per DESIGN_AI_AGENT.md, the kernel pins this fingerprint and any
# deviation aborts the connection. The cert is self-signed because
# Bat_OS' own CA is its trust root for inference traffic — we don't
# want a third-party CA in this path.
#
# Outputs:
#   /etc/caddy/bat-os-coder.crt              — self-signed cert (PEM)
#   /etc/caddy/bat-os-coder.key              — private key (PEM, 0400)
#   /etc/caddy/Caddyfile                     — caddy config
#   pinned_cert_sha256.txt                   — fingerprint to paste
#
# Run (on the inference host, with sudo):
#   sudo bash scripts/setup_caddy_tls.sh
set -euo pipefail

CERT_DIR=/etc/caddy
HOST="${INFERENCE_HOST:-bat-os-coder.local}"
PORT="${INFERENCE_PORT:-443}"

mkdir -p "$CERT_DIR"

if [ ! -f "$CERT_DIR/bat-os-coder.crt" ]; then
    echo "[caddy] generating self-signed cert for $HOST"
    openssl req -x509 -nodes -newkey ec:<(openssl ecparam -name prime256v1) \
        -keyout "$CERT_DIR/bat-os-coder.key" \
        -out    "$CERT_DIR/bat-os-coder.crt" \
        -days 365 \
        -subj "/CN=$HOST" \
        -addext "subjectAltName=DNS:$HOST"
    chmod 0400 "$CERT_DIR/bat-os-coder.key"
fi

echo "[caddy] writing Caddyfile -> $CERT_DIR/Caddyfile"
cat > "$CERT_DIR/Caddyfile" <<EOF
{
    auto_https off
}

:$PORT {
    tls $CERT_DIR/bat-os-coder.crt $CERT_DIR/bat-os-coder.key

    @ai path /v1/chat/completions /v1/models /api/*
    handle @ai {
        reverse_proxy 127.0.0.1:11434
    }

    handle {
        respond "Bat_OS coder endpoint" 200
    }

    log {
        output file /var/log/caddy/bat-os-coder.log
        format json
    }
}
EOF

echo "[caddy] (re)starting service"
if command -v systemctl >/dev/null 2>&1; then
    systemctl reload caddy 2>/dev/null || systemctl restart caddy || true
else
    pkill caddy || true
    nohup caddy run --config "$CERT_DIR/Caddyfile" > /var/log/caddy/run.log 2>&1 &
fi

FPR=$(openssl x509 -in "$CERT_DIR/bat-os-coder.crt" -fingerprint -sha256 -noout \
    | sed 's/.*=//' | tr -d ':' | tr 'A-Z' 'a-z')

echo
echo "================================================================"
echo "PINNED CERT SHA-256 (paste into src/ai/policy.rs):"
echo
echo "$FPR"
echo
echo "as a Rust array literal:"
python3 - <<PYEOF
fp = "$FPR"
b = [int(fp[i:i+2], 16) for i in range(0, 64, 2)]
print("pub const PINNED_CERT_SHA256: [u8; 32] = [")
for i in range(0, 32, 8):
    row = ', '.join(f"0x{x:02x}" for x in b[i:i+8])
    print(f"    {row},")
print("];")
PYEOF
echo "================================================================"
echo "$FPR" > pinned_cert_sha256.txt
echo "[caddy] saved fingerprint -> ./pinned_cert_sha256.txt"
echo "[caddy] caddy listening on https://$HOST:$PORT/v1/chat/completions"
