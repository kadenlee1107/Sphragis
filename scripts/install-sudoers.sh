#!/usr/bin/env bash
# One-shot: install a NOPASSWD sudoers rule for the m1n1 chainload.
# Scoped: only `python3 chainload.py ...` gets passwordless sudo.
# Run as: sudo bash scripts/install-sudoers.sh
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
    echo "ERROR: run with sudo (sudo bash $0)" >&2
    exit 1
fi

DST=/etc/sudoers.d/sphragis-chainload
CMD='/usr/bin/python3 /home/kaden-lee/code/Sphragis/external/m1n1/proxyclient/tools/chainload.py *'
echo "kaden-lee ALL=(ALL) NOPASSWD:SETENV: $CMD" > "$DST"
chmod 440 "$DST"
visudo -c -f "$DST"
echo "installed: $DST"
