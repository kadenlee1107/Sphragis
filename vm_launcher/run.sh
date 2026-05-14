#!/bin/bash
# Sphragis — Build and launch in native M4 VM
set -e

cd "$(dirname "$0")/.."

echo "[*] Building Sphragis..."
cargo build --release 2>&1 | tail -1

echo "[*] Launching on M4 via Virtualization.framework..."
echo "    (Ctrl+C to exit)"
echo ""

swift vm_launcher/Sphragis_VM.swift target/aarch64-unknown-none/release/sphragis
