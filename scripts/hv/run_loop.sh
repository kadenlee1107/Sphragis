#!/bin/bash
# Canonical Sphragis 2-cycle demo launcher.
# Solves the quoting problem where `sg dialout -c "..."` mangles
# `$'batman;;\t*9\r'` into literal `$batman\r`. A script file is
# read by bash (not sh), so ANSI-C quoting lands correctly.
#
# Usage (from repo root):
#   sg dialout -c scripts/hv/run_loop.sh
#
# Env overrides supported (set before invocation):
#   M1N1DEVICE        — default /dev/ttyACM1
#   SPHRAGIS_HV_LOOP_MAX — default 2
#   SPHRAGIS_HV_STIM_GAP_S — default 25
#
set -u
: ${M1N1DEVICE:=/dev/ttyACM1}
: ${SPHRAGIS_HV_LOOP_MAX:=2}
: ${SPHRAGIS_HV_STIM_GAP_S:=25}
export M1N1DEVICE SPHRAGIS_HV_LOOP_MAX SPHRAGIS_HV_STIM_GAP_S
export SPHRAGIS_HV_LOOP=1
export SPHRAGIS_HV_BOOTSTRAP_CHAINLOAD=1
export SPHRAGIS_KEEP_FB=1
export SPHRAGIS_HV_STIMULUS=$'batman;;\t\t\t\t\t\t\t\t\t\r'
echo "[launcher] STIMULUS hex=$(printf '%s' "$SPHRAGIS_HV_STIMULUS" | xxd -p)"
REPO=$(cd "$(dirname "$0")/../.." && pwd)
exec /usr/bin/python3 "$REPO/scripts/hv/sphragis_hv_interactive.py"
