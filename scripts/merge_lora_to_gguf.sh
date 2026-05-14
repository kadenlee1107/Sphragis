#!/usr/bin/env bash
# Merge the trained LoRA adapter into the base model and convert to
# Q4_K_M GGUF for ollama. Runs entirely in WSL on the inference host.
#
# Inputs:
#   /mnt/d/ai/training/output/final/         — adapter dir from train_lora.py
#   Qwen/Qwen2.5-Coder-7B-Instruct (HF cache) — base model
# Outputs:
#   /mnt/d/ai/training/merged/               — bf16 merged HF model
#   /mnt/d/ai/training/gguf/sphragis-coder.Q4_K_M.gguf
#
# Run:
#   bash scripts/merge_lora_to_gguf.sh
set -euo pipefail

ADAPTER=/mnt/d/ai/training/output/final
MERGED=/mnt/d/ai/training/merged
GGUF_DIR=/mnt/d/ai/training/gguf
GGUF_OUT="$GGUF_DIR/sphragis-coder.Q4_K_M.gguf"
LLAMA_CPP="${LLAMA_CPP:-$HOME/llama.cpp}"
HF_HOME="${HF_HOME:-/mnt/d/ai/.cache/huggingface}"

mkdir -p "$MERGED" "$GGUF_DIR"

if [ ! -d "$ADAPTER" ]; then
    echo "[merge] FATAL: adapter not found at $ADAPTER"
    echo "[merge] did train_lora.py finish? check /mnt/d/ai/training/output/train.log"
    exit 1
fi

echo "[merge] step 1/3 — merge LoRA adapter into base model (bf16)"
HF_HOME="$HF_HOME" "$HOME/ai-venv/bin/python" - <<'PYEOF'
import os, sys
from pathlib import Path
import torch
from peft import PeftModel
from transformers import AutoModelForCausalLM, AutoTokenizer

BASE  = "Qwen/Qwen2.5-Coder-7B-Instruct"
ADAPT = "/mnt/d/ai/training/output/final"
OUT   = "/mnt/d/ai/training/merged"

print(f"[merge-py] loading base {BASE}")
tok = AutoTokenizer.from_pretrained(BASE)
base = AutoModelForCausalLM.from_pretrained(
    BASE, torch_dtype=torch.bfloat16, device_map="cpu"
)
print(f"[merge-py] applying adapter {ADAPT}")
merged = PeftModel.from_pretrained(base, ADAPT).merge_and_unload()
print(f"[merge-py] saving merged model to {OUT}")
merged.save_pretrained(OUT, safe_serialization=True)
tok.save_pretrained(OUT)
print("[merge-py] done")
PYEOF

echo "[merge] step 2/3 — convert merged HF model to GGUF (f16 intermediate)"
if [ ! -d "$LLAMA_CPP" ]; then
    echo "[merge] cloning llama.cpp to $LLAMA_CPP"
    git clone --depth 1 https://github.com/ggerganov/llama.cpp "$LLAMA_CPP"
fi
# llama.cpp's convert script needs gguf and a few transformers bits;
# ensure they're in the venv.
"$HOME/ai-venv/bin/pip" install -q gguf sentencepiece protobuf 2>&1 | tail -3 || true
"$HOME/ai-venv/bin/python" "$LLAMA_CPP/convert_hf_to_gguf.py" "$MERGED" \
    --outfile "$GGUF_DIR/sphragis-coder.f16.gguf" \
    --outtype f16

echo "[merge] step 3/3 — quantize f16 -> Q4_K_M"
if [ ! -x "$LLAMA_CPP/build/bin/llama-quantize" ]; then
    echo "[merge] building llama-quantize"
    cmake -S "$LLAMA_CPP" -B "$LLAMA_CPP/build" -DGGML_CUDA=OFF >/dev/null
    cmake --build "$LLAMA_CPP/build" --target llama-quantize -j
fi
"$LLAMA_CPP/build/bin/llama-quantize" \
    "$GGUF_DIR/sphragis-coder.f16.gguf" \
    "$GGUF_OUT" \
    Q4_K_M

ls -la "$GGUF_OUT"
echo "[merge] complete -> $GGUF_OUT"
