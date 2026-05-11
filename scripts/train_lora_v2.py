#!/usr/bin/env python3
"""LoRA v2 — improvements stacked on the v1 baseline run.

Diffs from v1:

  * **trl.SFTTrainer with `messages` records** instead of the v1
    `{instruction, input, output}` triple. Lets us train both
    single-turn and multi-turn (tool-call) examples uniformly with
    Qwen2.5's native chat template.
  * **Sample packing**: short records concat to `max_seq_length`,
    so we don't waste compute on padding. ~3× more useful gradient
    per step on this dataset (mean record is ~250 tokens, packing
    fills 2048).
  * **DoRA** (decomposed LoRA) — small but free quality bump.
  * **NEFTune** noise on embeddings — +1-2 pts on instruction
    following from the original paper, no inference cost.
  * **Rank 64 / alpha 128**, up from 16/32. More capacity for the
    larger, more diverse v2 corpus.
  * **max_seq_length 2048**, up from 1536 — needed because the
    multi-turn tool-call records have system + user + tool_call +
    tool + assistant turns concatenated.
  * Same QLoRA / bf16 / paged-AdamW-8bit / gradient-checkpointing
    setup as v1; same `target_modules` set.

Run (on the inference host, in WSL with HF_TOKEN set):

    ~/ai-venv/bin/pip install -q --upgrade trl peft transformers datasets
    ~/ai-venv/bin/python scripts/train_lora_v2.py
"""
import json
import os
from pathlib import Path

import torch
from datasets import Dataset
from peft import LoraConfig, prepare_model_for_kbit_training
from transformers import (
    AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig,
)
from trl import SFTConfig, SFTTrainer

# ── Config ───────────────────────────────────────────────────────────
BASE_MODEL = "Qwen/Qwen2.5-Coder-7B-Instruct"
DATASET    = "/mnt/d/ai/training/bat_os_lora_dataset_v2.jsonl"
OUT_DIR    = "/mnt/d/ai/training/output_v2"
MAX_LEN    = 2048

LORA_RANK    = 64
LORA_ALPHA   = 128
LORA_DROPOUT = 0.05
LORA_TARGETS = [
    "q_proj", "k_proj", "v_proj", "o_proj",
    "gate_proj", "up_proj", "down_proj",
]

BATCH         = 1
GRAD_ACCUM    = 16          # effective batch 16
LEARNING_RATE = 2e-4
EPOCHS        = 3
WARMUP_RATIO  = 0.03
SAVE_STEPS    = 200
LOG_STEPS     = 10
NEFTUNE_ALPHA = 5.0

os.environ.setdefault("CUDA_VISIBLE_DEVICES", "0")


def load_dataset() -> Dataset:
    rows = []
    with open(DATASET, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))
    return Dataset.from_list(rows)


def main():
    print(f"[v2-train] base   {BASE_MODEL}", flush=True)
    print(f"[v2-train] data   {DATASET}", flush=True)
    print(f"[v2-train] out    {OUT_DIR}", flush=True)
    print(f"[v2-train] rank={LORA_RANK} alpha={LORA_ALPHA} max_len={MAX_LEN}", flush=True)

    tokenizer = AutoTokenizer.from_pretrained(BASE_MODEL, trust_remote_code=False)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    bnb = BitsAndBytesConfig(
        load_in_4bit=True,
        bnb_4bit_quant_type="nf4",
        bnb_4bit_use_double_quant=True,
        bnb_4bit_compute_dtype=torch.bfloat16,
    )
    model = AutoModelForCausalLM.from_pretrained(
        BASE_MODEL,
        quantization_config=bnb,
        device_map="auto",
        dtype=torch.bfloat16,
        trust_remote_code=False,
    )
    model.config.use_cache = False
    model = prepare_model_for_kbit_training(model)

    lora = LoraConfig(
        r=LORA_RANK,
        lora_alpha=LORA_ALPHA,
        target_modules=LORA_TARGETS,
        lora_dropout=LORA_DROPOUT,
        bias="none",
        task_type="CAUSAL_LM",
        use_dora=True,
    )

    ds = load_dataset()
    print(f"[v2-train] {len(ds)} records", flush=True)

    args = SFTConfig(
        output_dir=OUT_DIR,
        num_train_epochs=EPOCHS,
        per_device_train_batch_size=BATCH,
        gradient_accumulation_steps=GRAD_ACCUM,
        gradient_checkpointing=True,
        gradient_checkpointing_kwargs={"use_reentrant": False},
        optim="paged_adamw_8bit",
        learning_rate=LEARNING_RATE,
        lr_scheduler_type="cosine",
        warmup_ratio=WARMUP_RATIO,
        bf16=True,
        logging_steps=LOG_STEPS,
        save_steps=SAVE_STEPS,
        save_total_limit=3,
        report_to="none",
        dataloader_num_workers=2,
        ddp_find_unused_parameters=False,
        max_length=MAX_LEN,
        packing=True,
        neftune_noise_alpha=NEFTUNE_ALPHA,
    )

    trainer = SFTTrainer(
        model=model,
        args=args,
        train_dataset=ds,
        peft_config=lora,
        processing_class=tokenizer,
    )

    n_train = sum(p.numel() for p in trainer.model.parameters() if p.requires_grad)
    n_total = sum(p.numel() for p in trainer.model.parameters())
    print(f"[v2-train] trainable: {n_train:,} / {n_total:,} ({100*n_train/n_total:.2f}%)", flush=True)

    trainer.train()

    final = str(Path(OUT_DIR) / "final")
    trainer.save_model(final)
    tokenizer.save_pretrained(final)
    print(f"[v2-train] saved -> {final}", flush=True)


if __name__ == "__main__":
    main()
