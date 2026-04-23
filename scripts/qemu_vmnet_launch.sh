#!/bin/bash
# Followup 3c-vmnet: launch Bat_OS with a real macOS vmnet-host NIC
# so Docker containers can share the L2 segment and their frames hit
# Bat_OS's nic 1 NAT forwarder for real (not via a Python socket peer).
#
# Requires sudo because macOS's vmnet.framework gates interface
# creation behind the com.apple.vm.networking entitlement. QEMU from
# Homebrew isn't signed with that entitlement, so sudo is the only
# unprivileged path.
#
# Usage:
#   ./scripts/qemu_vmnet_launch.sh               # default 192.168.77.1/24
#   ./scripts/qemu_vmnet_launch.sh 192.168.78.0  # custom subnet
#
# Containers can then be attached to a Docker macvlan network with
# the same subnet + parent=<vmnet interface> so their traffic lands
# on Bat_OS's nic 1. Example:
#   docker network create -d macvlan \
#       --subnet=192.168.77.0/24 --gateway=192.168.77.1 \
#       -o parent=bridge100 caves
#   docker run --network caves --ip=192.168.77.10 kali:latest
#
# (bridge100 is whatever interface `ifconfig` shows for vmnet; check
# with `ifconfig -l` after a successful launch.)

set -euo pipefail

SUBNET_BASE="${1:-192.168.77}"
START="${SUBNET_BASE}.1"
END="${SUBNET_BASE}.254"
MASK="255.255.255.0"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KERNEL="${ROOT}/target/aarch64-unknown-none/release/bat_os"

if [ ! -f "$KERNEL" ]; then
    echo "kernel missing: $KERNEL"
    echo "build with:  BAT_OS_PASSPHRASE=batman cargo build --release"
    exit 1
fi

echo "[vmnet] launching QEMU with vmnet-host ${START}-${END}/${MASK}"
echo "[vmnet] this will prompt for sudo; vmnet.framework needs privileges"

exec sudo qemu-system-aarch64 \
    -machine virt -cpu max -m 2G \
    -display none \
    -device virtio-gpu-device \
    -device virtio-keyboard-device \
    -netdev user,id=hostnet \
    -device virtio-net-device,netdev=hostnet \
    -netdev "vmnet-host,id=caves,start-address=${START},end-address=${END},subnet-mask=${MASK}" \
    -device virtio-net-device,netdev=caves \
    -serial mon:stdio \
    -kernel "${KERNEL}"
