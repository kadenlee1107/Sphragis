#!/usr/bin/env bash
# One-shot: make /dev/m1n1 always symlink the PROXY interface (iface 0),
# not the virtual-UART interface (iface 2). The existing rule matches
# by VID/PID only and picks whichever interface udev processes last —
# often the vUART, which silently consumes our chainload bytes.
# Run as: sudo bash scripts/fix-udev.sh
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
    echo "ERROR: run with sudo (sudo bash $0)" >&2
    exit 1
fi

cat > /etc/udev/rules.d/99-m1n1.rules <<'EOF'
# m1n1 USB CDC serial — proxy interface only.
# m1n1 exposes two CDC-ACM pipes:
#   - Interface 0 = PROXY (bidirectional REQ/REPLY, what chainload.py talks to)
#   - Interface 2 = virtual UART (m1n1 serial debug console, one-way)
# Symlink only the proxy.
SUBSYSTEM=="tty", ATTRS{idVendor}=="1209", ATTRS{idProduct}=="316d", \
    ATTRS{bInterfaceNumber}=="00", \
    SYMLINK+="m1n1", GROUP="dialout", MODE="0660"
EOF

udevadm control --reload-rules
udevadm trigger --subsystem-match=tty
echo "udev rule updated + reloaded"
ls -la /dev/m1n1 2>/dev/null || echo "(no /dev/m1n1 yet — will appear on next m1n1 enumeration)"
