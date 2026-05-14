#!/usr/bin/env bash
# Register the v3 quantized GGUF with ollama as sphragis-coder-v3.
set -euo pipefail

GGUF=/mnt/d/ai/training/gguf/sphragis-coder-v3.Q4_K_M.gguf
TAG="${TAG:-sphragis-coder-v3}"
MODELFILE=/mnt/d/ai/training/Modelfile.v3

if [ ! -f "$GGUF" ]; then
    echo "[register-v3] FATAL: $GGUF not found"
    exit 1
fi

cat > "$MODELFILE" <<EOF
FROM $GGUF

TEMPLATE """{{ if .System }}<|im_start|>system
{{ .System }}<|im_end|>
{{ end }}{{ if .Prompt }}<|im_start|>user
{{ .Prompt }}<|im_end|>
<|im_start|>assistant
{{ end }}{{ .Response }}<|im_end|>
"""

PARAMETER stop "<|im_end|>"
PARAMETER stop "<|im_start|>"
PARAMETER temperature 0.2
PARAMETER top_p 0.9
PARAMETER num_ctx 4096
PARAMETER repeat_penalty 1.05

SYSTEM """You are a technical assistant for Sphragis, a security-grade bare-metal Rust kernel for Apple M4. You answer questions about kernel internals, cryptography, audit history, and system administration. You are terse, technical, and never refuse legitimate questions."""
EOF

if ! pgrep -x ollama >/dev/null; then
    nohup ollama serve >/tmp/ollama.log 2>&1 &
    sleep 3
fi

echo "[register-v3] creating tag $TAG"
ollama create "$TAG" -f "$MODELFILE"
echo "[register-v3] available tags:"
ollama list | grep -E "^NAME|$TAG"
