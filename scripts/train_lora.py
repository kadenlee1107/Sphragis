#!/usr/bin/env python3
"""LoRA fine-tune Qwen2.5-Coder-7B on Bat_OS-specific data.

Designed to run on a single RTX 5070 (12 GB VRAM) using QLoRA:
4-bit nf4 quantization + LoRA adapters + bf16 mixed precision +
gradient checkpointing.

Inputs:
  /mnt/d/ai/training/bat_os_lora_dataset.jsonl  — built by build_lora_dataset.py
Outputs:
  /mnt/d/ai/training/output/                    — adapter checkpoints, logs
  /mnt/d/ai/training/output/final/              — merged final adapter

Run:
  ~/ai-venv/bin/python ~/Bat_OS/scripts/train_lora.py
or, for unattended training that survives SSH disconnect:
  tmux new -d -s lora "~/ai-venv/bin/python ~/Bat_OS/scripts/train_lora.py 2>&1 | tee /mnt/d/ai/training/output/train.log"
"""
import json, os
from pathlib import Path

import torch
from datasets import Dataset
from peft import LoraConfig, get_peft_model, prepare_model_for_kbit_training
from transformers import (
    AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig,
    TrainingArguments, DataCollatorForLanguageModeling, Trainer,
)

# ───────────────────────────────────────────────────────────────────────
# Config
# ───────────────────────────────────────────────────────────────────────
BASE_MODEL = "Qwen/Qwen2.5-Coder-7B-Instruct"
DATASET    = "/mnt/d/ai/training/bat_os_lora_dataset.jsonl"
OUT_DIR    = "/mnt/d/ai/training/output"
MAX_LEN    = 1536       # tight enough for 12 GB VRAM with QLoRA + grad ckpt

LORA_RANK  = 16
LORA_ALPHA = 32
LORA_DROPOUT = 0.05
LORA_TARGET_MODULES = [
    "q_proj", "k_proj", "v_proj", "o_proj",
    "gate_proj", "up_proj", "down_proj",
]

# Hyperparameters tuned for 12 GB VRAM + 32 GB system RAM
BATCH_SIZE_PER_GPU = 1
GRAD_ACCUM         = 16    # effective batch = 16
LEARNING_RATE      = 2e-4
EPOCHS             = 3
WARMUP_RATIO       = 0.03
SAVE_STEPS         = 100
LOG_STEPS          = 10

# Pin training to the 5070 (cuda:0). The 1660 Ti (cuda:1) stays free for the desktop.
os.environ.setdefault("CUDA_VISIBLE_DEVICES", "0")


def load_dataset() -> Dataset:
    """Load the JSONL into a HF Dataset."""
    rows = []
    with open(DATASET, "r", encoding="utf-8") as f:
        for line in f:
            rows.append(json.loads(line))
    return Dataset.from_list(rows)


def format_chat(rec, tokenizer) -> dict:
    """Format one record using Qwen's chat template.

    Concatenates instruction + input as the user message, output as the
    assistant message.
    """
    user_msg = rec["instruction"]
    if rec.get("input"):
        user_msg += "\n\n" + rec["input"]
    messages = [
        {"role": "system", "content":
            "You are a technical assistant for Bat_OS, a security-grade bare-metal Rust kernel for Apple M4. "
            "You answer questions about kernel internals, cryptography, audit history, and system administration. "
            "You are terse, technical, and never refuse legitimate questions."},
        {"role": "user", "content": user_msg},
        {"role": "assistant", "content": rec["output"]},
    ]
    text = tokenizer.apply_chat_template(messages, tokenize=False, add_generation_prompt=False)
    return {"text": text}


def main():
    print(f"[train] loading tokenizer + base model ({BASE_MODEL})...")
    print(f"[train] HF_HOME = {os.environ.get('HF_HOME', '~/.cache/huggingface (default)')}")

    tokenizer = AutoTokenizer.from_pretrained(BASE_MODEL, trust_remote_code=False)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    bnb_config = BitsAndBytesConfig(
        load_in_4bit=True,
        bnb_4bit_quant_type="nf4",
        bnb_4bit_use_double_quant=True,
        bnb_4bit_compute_dtype=torch.bfloat16,
    )

    model = AutoModelForCausalLM.from_pretrained(
        BASE_MODEL,
        quantization_config=bnb_config,
        device_map="auto",
        torch_dtype=torch.bfloat16,
        trust_remote_code=False,
    )
    model.config.use_cache = False
    model = prepare_model_for_kbit_training(model)

    lora_config = LoraConfig(
        r=LORA_RANK,
        lora_alpha=LORA_ALPHA,
        target_modules=LORA_TARGET_MODULES,
        lora_dropout=LORA_DROPOUT,
        bias="none",
        task_type="CAUSAL_LM",
    )
    model = get_peft_model(model, lora_config)

    n_train = sum(p.numel() for p in model.parameters() if p.requires_grad)
    n_total = sum(p.numel() for p in model.parameters())
    print(f"[train] trainable params: {n_train:,} / {n_total:,} ({100*n_train/n_total:.2f}%)")

    print(f"[train] loading dataset from {DATASET}...")
    raw_ds = load_dataset()
    print(f"[train] {len(raw_ds)} raw records")

    formatted = raw_ds.map(lambda r: format_chat(r, tokenizer), remove_columns=raw_ds.column_names)

    def tokenize_fn(batch):
        out = tokenizer(
            batch["text"],
            truncation=True,
            max_length=MAX_LEN,
            padding=False,
        )
        return out

    tokenized = formatted.map(tokenize_fn, batched=True, remove_columns=["text"])
    tokenized = tokenized.filter(lambda r: len(r["input_ids"]) > 16)
    print(f"[train] {len(tokenized)} tokenized records (after length filter)")

    collator = DataCollatorForLanguageModeling(tokenizer=tokenizer, mlm=False)

    args = TrainingArguments(
        output_dir=OUT_DIR,
        num_train_epochs=EPOCHS,
        per_device_train_batch_size=BATCH_SIZE_PER_GPU,
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
        remove_unused_columns=False,
    )

    trainer = Trainer(
        model=model,
        processing_class=tokenizer,
        args=args,
        train_dataset=tokenized,
        data_collator=collator,
    )

    print(f"[train] starting training ({EPOCHS} epochs, eff. batch={BATCH_SIZE_PER_GPU * GRAD_ACCUM})")
    trainer.train()

    final_dir = str(Path(OUT_DIR) / "final")
    print(f"[train] saving final adapter to {final_dir}")
    trainer.save_model(final_dir)
    tokenizer.save_pretrained(final_dir)
    print(f"[train] done.")


if __name__ == "__main__":
    main()
