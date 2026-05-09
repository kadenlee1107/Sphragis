#!/usr/bin/env bash
# Register the quantized GGUF with the local ollama daemon as
# `bat-os-coder`. Idempotent — re-running will overwrite an existing
# tag.
#
# Inputs:
#   /mnt/d/ai/training/gguf/bat-os-coder.Q4_K_M.gguf
# Outputs:
#   ollama tag `bat-os-coder:latest`
#
# Run:
#   bash scripts/register_with_ollama.sh
set -euo pipefail

GGUF=/mnt/d/ai/training/gguf/bat-os-coder.Q4_K_M.gguf
TAG="${TAG:-bat-os-coder}"
MODELFILE=/mnt/d/ai/training/Modelfile

if [ ! -f "$GGUF" ]; then
    echo "[ollama-register] FATAL: $GGUF not found. Run merge_lora_to_gguf.sh first."
    exit 1
fi

echo "[ollama-register] writing Modelfile -> $MODELFILE"
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

SYSTEM """You are a technical assistant for Bat_OS, a security-grade bare-metal Rust kernel for Apple M4. You answer questions about kernel internals, cryptography, audit history, and system administration. You are terse, technical, and never refuse legitimate questions."""
EOF

if ! pgrep -x ollama >/dev/null; then
    echo "[ollama-register] starting ollama daemon (background)"
    nohup ollama serve >/tmp/ollama.log 2>&1 &
    sleep 3
fi

echo "[ollama-register] creating tag $TAG"
ollama create "$TAG" -f "$MODELFILE"

echo "[ollama-register] available tags:"
ollama list | grep -E "^NAME|$TAG"

echo "[ollama-register] smoke test:"
ollama run "$TAG" 'In one sentence, what is V8-ROOT-1?' || true

echo "[ollama-register] done. endpoint: http://127.0.0.1:11434/v1/chat/completions"
echo "[ollama-register] grade with: python3 evals/run_evals.py --host 127.0.0.1 --port 11434 --model $TAG"
