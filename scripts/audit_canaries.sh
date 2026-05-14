#!/usr/bin/env bash
# Stack canary + mitigation audit for the Sphragis kernel image.
#
# Rust's default codegen emits some stack-protection only on functions
# with arrays larger than the threshold. For a kernel we want the
# stronger guarantee: every function gets a canary, regardless of
# whether it has a stack-allocated array. The `stack-protector=all`
# rustc flag in nightly does this; this script verifies the resulting
# binary actually carries the canary symbols.
#
# Also checks for related mitigations:
#   * BTI markers (ARMv8.5 Branch Target Identification)
#   * PAC instructions (ARMv8.3 Pointer Authentication — paciasp/autiasp)
#   * W^X sections (no segment is both writable and executable)
#
# Run after `cargo build --release`:
#   bash scripts/audit_canaries.sh
#
# Exit 0 = all green; exit 1 = at least one mitigation missing.
set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
ARTIFACT="${1:-$REPO/target/aarch64-unknown-none/release/sphragis}"

if [ ! -f "$ARTIFACT" ]; then
    echo "[canary-audit] artifact not found: $ARTIFACT"
    echo "[canary-audit] usage: $0 [path/to/kernel/elf]"
    exit 2
fi

if ! command -v objdump >/dev/null 2>&1 && ! command -v llvm-objdump >/dev/null 2>&1; then
    echo "[canary-audit] need objdump or llvm-objdump in PATH"
    exit 2
fi

OBJDUMP=$(command -v llvm-objdump || command -v objdump)
READELF=$(command -v llvm-readelf || command -v readelf)

echo "[canary-audit] artifact: $ARTIFACT"
echo "[canary-audit] objdump:  $OBJDUMP"
echo

fail=0

# --- 1. Stack canary symbols ---
# The Rust stack-protector lowers calls into compiler-rt's
# __stack_chk_fail / __stack_chk_guard. Their presence (or absence)
# is a binary signal — either the protector is on for at least some
# functions, or it isn't.
echo "[canary-audit] (1) stack-protector symbols"
chk_fail_count=$("$READELF" --syms "$ARTIFACT" 2>/dev/null | grep -c "__stack_chk_fail" || true)
chk_guard_count=$("$READELF" --syms "$ARTIFACT" 2>/dev/null | grep -c "__stack_chk_guard" || true)
if [ "$chk_fail_count" -gt 0 ] && [ "$chk_guard_count" -gt 0 ]; then
    echo "  ✓ __stack_chk_fail + __stack_chk_guard present"
else
    echo "  ✗ stack canary symbols absent — add"
    echo "    RUSTFLAGS=\"-Z stack-protector=all\" to .cargo/config.toml"
    fail=1
fi
echo

# --- 2. PAC (Pointer Authentication) ---
# `paciasp` and `autiasp` are the standard prologue/epilogue
# instructions that authenticate the return address against the
# function's pointer. If the binary contains zero of either, PAC
# isn't being emitted.
echo "[canary-audit] (2) PAC prologue/epilogue (paciasp / autiasp)"
paciasp_count=$("$OBJDUMP" -d "$ARTIFACT" 2>/dev/null | grep -c paciasp || true)
autiasp_count=$("$OBJDUMP" -d "$ARTIFACT" 2>/dev/null | grep -c autiasp || true)
if [ "$paciasp_count" -gt 0 ] && [ "$autiasp_count" -gt 0 ]; then
    echo "  ✓ paciasp count=$paciasp_count  autiasp count=$autiasp_count"
else
    echo "  ✗ no PAC instructions — add to RUSTFLAGS:"
    echo "    -C target-feature=+pauth -Z branch-protection=pac-ret"
    fail=1
fi
echo

# --- 3. BTI (Branch Target Identification) ---
# `bti c` is the marker at indirect branch targets. With BTI off,
# an attacker who hijacks an indirect branch can land anywhere; with
# BTI on, the target must be a marked instruction or the CPU faults.
echo "[canary-audit] (3) BTI markers (bti c / bti j)"
bti_count=$("$OBJDUMP" -d "$ARTIFACT" 2>/dev/null | grep -cE "bti (c|j|jc)" || true)
if [ "$bti_count" -gt 0 ]; then
    echo "  ✓ bti markers count=$bti_count"
else
    echo "  ✗ no bti markers — add to RUSTFLAGS:"
    echo "    -C target-feature=+bti -Z branch-protection=bti"
    fail=1
fi
echo

# --- 4. W^X (write-xor-execute) over segments ---
# Read the program headers; any segment whose flags contain both
# W and E (RWE = 7) is a W^X violation. JIT runtimes intentionally
# allocate RWE pages, but a static kernel should never.
echo "[canary-audit] (4) W^X — no segment is both writable and executable"
wx_violations=$("$READELF" -l "$ARTIFACT" 2>/dev/null | grep "LOAD" | grep -cE " RWE | WRE " || true)
if [ "$wx_violations" -eq 0 ]; then
    echo "  ✓ no segments with W+E set"
else
    echo "  ✗ $wx_violations segments are both writable AND executable"
    fail=1
fi
echo

# --- Summary ---
if [ "$fail" -eq 0 ]; then
    echo "[canary-audit] ALL CHECKS GREEN"
    exit 0
else
    echo "[canary-audit] FAIL — see notes above"
    exit 1
fi
