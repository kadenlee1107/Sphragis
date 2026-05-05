#!/bin/bash
# tools/audit_chromium_initmap.sh — STUMP #152 chunk 1
#
# Walks the Chromium port's content_shell + every lib_runtime ELF and
# dumps the data the loader needs to drive DT_INIT_ARRAY execution
# correctly:
#
#   - DT_INIT_ARRAY size (number of constructors)
#   - DT_PREINIT_ARRAY (main exe only, run before DT_INIT_ARRAY)
#   - PT_TLS memsz + alignment (static-TLS budget contributor)
#   - DT_NEEDED (dependency edges for init order)
#
# Usage:
#   tools/audit_chromium_initmap.sh > docs/CHROMIUM_INITMAP.txt
#
# The output is consumed by:
#   - the loader's static-TLS-budget tuning (sum tls_memsz + headroom)
#   - the init-order DFS (if we ever drive constructors ourselves
#     instead of letting ld-linux do it)
#
# Plan agent's verdict: ld-linux drives init order for us, so the
# DT_NEEDED graph is informational only — useful for sanity-checking
# the loaded set against what content_shell actually links to.

set -euo pipefail

PORT_OUT="${PORT_OUT:-ports/chromium_port/out}"
SHELL_BIN="$PORT_OUT/content_shell"
LIB_DIR="$PORT_OUT/lib_runtime"

# Pick the readelf we have. Homebrew llvm ships llvm-readelf.
READELF=""
for cand in readelf llvm-readelf /opt/homebrew/Cellar/llvm/*/bin/llvm-readelf; do
    if command -v "$cand" > /dev/null 2>&1; then
        READELF="$cand"
        break
    fi
done
if [[ -z "$READELF" ]]; then
    echo "ERROR: no readelf or llvm-readelf in PATH" >&2
    exit 1
fi

print_one() {
    local elf="$1"
    local name
    name=$(basename "$elf")
    echo "─── $name ────────────────────────────────────────"
    echo "  size: $(wc -c < "$elf" | tr -d ' ') bytes"

    # DT_INIT_ARRAY size in bytes (each entry is 8 bytes on 64-bit).
    local ia_size
    ia_size=$("$READELF" -d "$elf" 2>/dev/null \
              | awk '/INIT_ARRAYSZ/{gsub(/[^0-9]/,"",$3); print $3; exit}')
    if [[ -n "$ia_size" ]]; then
        echo "  DT_INIT_ARRAY entries: $((ia_size / 8))"
    else
        echo "  DT_INIT_ARRAY entries: 0"
    fi

    # DT_PREINIT_ARRAY is main-exe only.
    local pia_size
    pia_size=$("$READELF" -d "$elf" 2>/dev/null \
               | awk '/PREINIT_ARRAYSZ/{gsub(/[^0-9]/,"",$3); print $3; exit}')
    if [[ -n "$pia_size" ]]; then
        echo "  DT_PREINIT_ARRAY entries: $((pia_size / 8))"
    fi

    # PT_TLS memsz/align/filesz from program headers.
    "$READELF" -l "$elf" 2>/dev/null \
      | awk '/TLS/ {tls=1; next}
             tls==1 && /MemSiz/ { for(i=1;i<=NF;i++) if($i~/MemSiz/) mem=i+1; tls=2; next }
             tls==2 { print "  PT_TLS memsz: 0x" $4 " (filesz 0x" $3 ", align " $NF ")"; tls=0 }' \
      || true

    # DT_NEEDED libs.
    local needed
    needed=$("$READELF" -d "$elf" 2>/dev/null \
             | awk '/NEEDED/{gsub(/[\[\]]/,""); print $5}' \
             | tr '\n' ' ' \
             || true)
    if [[ -n "$needed" ]]; then
        echo "  DT_NEEDED: $needed"
    fi
}

echo "═══ Chromium init-map audit ═══"
echo "(STUMP #152 chunk 1 output)"
echo "$(date)"
echo

print_one "$SHELL_BIN"
echo
for lib in "$LIB_DIR"/*.so* ; do
    [[ -f "$lib" ]] || continue
    print_one "$lib"
done
echo
echo "═══ Summary ═══"
total_ctors=0
total_tls_kb=0
for elf in "$SHELL_BIN" "$LIB_DIR"/*.so* ; do
    [[ -f "$elf" ]] || continue
    ia=$("$READELF" -d "$elf" 2>/dev/null \
         | awk '/INIT_ARRAYSZ/{gsub(/[^0-9]/,"",$3); print $3; exit}')
    [[ -n "$ia" ]] && total_ctors=$((total_ctors + ia / 8))
    # Best-effort TLS sum (hex from readelf)
    tls=$("$READELF" -l "$elf" 2>/dev/null \
          | awk '/TLS/{getline; gsub(/0x/,""); printf "%d", strtonum("0x"$4); exit}')
    [[ -n "$tls" ]] && total_tls_kb=$((total_tls_kb + tls))
done
echo "  total constructors across all loaded ELFs: $total_ctors"
echo "  total static-TLS bytes (rough sum):        $total_tls_kb"
echo "  current LOADED_TLS_PAGES = 4 (16 KB)"
if [[ $total_tls_kb -gt 16384 ]]; then
    echo "  ⚠ TLS budget needed: $((total_tls_kb / 1024 + 64)) KB (with 64 KB headroom)"
    echo "    → loader.rs LOADED_TLS_PAGES needs to grow"
fi
