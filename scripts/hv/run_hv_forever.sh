#!/usr/bin/env bash
# Single entry point for running Sphragis under HV on M4 with automatic
# recovery after every reset. Usage:
#
#   scripts/hv/run_hv_forever.sh                # run forever
#   SPHRAGIS_HV_MAX_CYCLES=5 scripts/hv/run_hv_forever.sh
#
# Rebuilds patched m1n1 if the tree is newer than the .macho, then
# hands off to scripts/hv/sphragis_hv_supervisor.py. The supervisor
# handles the reset-reboot-chainload-run loop; Ctrl+C to exit.
#
# Assumes the Mac is either already at stock m1n1 OR will be once the
# first run triggers a reset. If the Mac booted into macOS, the
# supervisor will say so — hold power, pick m1n1 in the boot picker.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
M1N1_SRC_STAMP="$(stat -c '%Y' "$ROOT/external/m1n1/src/hv.c" 2>/dev/null || echo 0)"
M1N1_MACHO="$ROOT/external/m1n1/build/m1n1.macho"

if [[ ! -f "$M1N1_MACHO" || "$M1N1_SRC_STAMP" -gt "$(stat -c '%Y' "$M1N1_MACHO")" ]]; then
    echo "[run_hv_forever] m1n1 sources newer than build — rebuilding"
    make -C "$ROOT/external/m1n1" -j4
fi

if [[ ! -f "$ROOT/target/sphragis_apple.bin" ]]; then
    echo "[run_hv_forever] no sphragis_apple.bin yet — building with SPHRAGIS_PASSPHRASE=batman"
    ( cd "$ROOT" && SPHRAGIS_PASSPHRASE=batman bash build_apple.sh )
fi

# The supervisor runs as the calling user but chainload.py inside it
# needs to sudo -n (passwordless) and the HV session needs sg dialout.
# Both have to be pre-authorized by the user; we don't try to fix
# them here, just fail loudly if the expectation is broken.

exec /usr/bin/python3 "$ROOT/scripts/hv/sphragis_hv_supervisor.py"
