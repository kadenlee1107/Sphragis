#!/bin/bash
# 3c-deferred-4 — full real-Docker vmnet end-to-end.
#
#   container (alpine on macvlan) → bridge100 → Bat_OS nic 1 → NAT →
#   nic 0 (slirp) → internet
#
# Must run as root: vmnet.framework is gated behind the
# com.apple.vm.networking entitlement which Homebrew QEMU isn't
# signed with, so sudo is the only unprivileged path. Run
# `scripts/qemu_vmnet_preflight.sh` first (no sudo needed) to catch
# missing prerequisites.
#
# What the script does, in order:
#   1. Starts batcaved.py on :9999 (background, piped to a log).
#   2. Launches QEMU with:
#        nic 0 = -netdev user      (slirp, for daemon TCP control)
#        nic 1 = -netdev vmnet-host on 192.168.77.0/24
#        -serial pipe               (so we can drive the Bat_OS shell)
#   3. Waits for Bat_OS to reach the auth prompt, sends "batman".
#   4. Discovers the vmnet bridge interface (bridgeNN on macOS) and
#      remembers its name for Docker.
#   5. Creates a Docker macvlan network on that bridge with subnet
#      192.168.77.0/24.
#   6. Starts an alpine container at 192.168.77.10 and installs curl.
#   7. In Bat_OS shell: register policy for the cave + NAT binding.
#   8. From inside the container: `curl -sI https://example.com`.
#      That exercises ARP (container → Bat_OS gateway), IPv4 + TCP
#      through the NAT, reply reverse-NAT.
#   9. Checks nat-stats counters: arp-replies ≥ 1, allow ≥ 1.
#  10. Tears everything down on exit.
#
# Heads-up: the test depends on macOS / OrbStack macvlan semantics
# which vary across versions. If it doesn't work first try, check:
#   - `ifconfig` for the actual bridgeNN interface QEMU created
#   - `docker network inspect caves` for IP assignments
#   - batcaved log for FW/CPOL messages

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KERNEL="${ROOT}/target/aarch64-unknown-none/release/bat_os"
LOG_DIR="${ROOT}/logs/vmnet-e2e"
STAMP="$(date +%Y%m%d-%H%M%S)"
mkdir -p "$LOG_DIR"
DAEMON_LOG="${LOG_DIR}/batcaved-${STAMP}.log"
QEMU_LOG="${LOG_DIR}/qemu-${STAMP}.log"

CAVE_NAME="vmnet-kali"
CAVE_IP="192.168.77.10"
SUBNET="192.168.77.0/24"
GATEWAY="192.168.77.1"
NET_NAME="caves-vmnet"
CONTAINER="${CAVE_NAME}-ctr"
IMAGE="alpine:3"

if [ "$EUID" -ne 0 ]; then
    echo "ERROR: this script must run as root (vmnet needs entitlement)"
    exit 1
fi

# Original user, for starting daemon in their context so logs land in their home.
REAL_USER="${SUDO_USER:-$USER}"
REAL_HOME="$(eval echo "~${REAL_USER}")"

cleanup_done=0
cleanup() {
    [ $cleanup_done -ne 0 ] && return
    cleanup_done=1
    echo
    echo "[e2e] cleanup"
    if [ -n "${QEMU_PID:-}" ] && kill -0 "$QEMU_PID" 2>/dev/null; then
        kill -TERM "$QEMU_PID" 2>/dev/null || true
        sleep 1
        kill -KILL "$QEMU_PID" 2>/dev/null || true
    fi
    if [ -n "${DAEMON_PID:-}" ] && kill -0 "$DAEMON_PID" 2>/dev/null; then
        sudo -u "$REAL_USER" kill -TERM "$DAEMON_PID" 2>/dev/null || true
    fi
    sudo -u "$REAL_USER" docker rm -f "$CONTAINER" 2>/dev/null || true
    sudo -u "$REAL_USER" docker network rm "$NET_NAME" 2>/dev/null || true
    echo "[e2e] logs:"
    echo "       qemu:   $QEMU_LOG"
    echo "       daemon: $DAEMON_LOG"
}
trap cleanup EXIT INT TERM

echo "[e2e] starting batcaved on :9999 (user $REAL_USER)"
sudo -u "$REAL_USER" python3 "$ROOT/scripts/batcaved.py" >"$DAEMON_LOG" 2>&1 &
DAEMON_PID=$!
sleep 1
if ! kill -0 "$DAEMON_PID" 2>/dev/null; then
    echo "ERROR: batcaved failed to start; see $DAEMON_LOG"
    exit 1
fi

# Note the current bridge interfaces BEFORE QEMU so we can diff later.
pre_bridges=$(ifconfig -l | tr ' ' '\n' | grep '^bridge' | sort -u || true)

# Launch QEMU with two NICs. Redirect serial stdio to a pipe we can drive.
FIFO_IN="${LOG_DIR}/qemu-in-${STAMP}"
FIFO_OUT="${LOG_DIR}/qemu-out-${STAMP}"
mkfifo "$FIFO_IN" "$FIFO_OUT"

echo "[e2e] launching QEMU with -netdev vmnet-host on $SUBNET"
qemu-system-aarch64 \
    -machine virt -cpu max -m 2G \
    -display none \
    -device virtio-gpu-device \
    -device virtio-keyboard-device \
    -netdev user,id=hostnet \
    -device virtio-net-device,netdev=hostnet \
    -netdev "vmnet-host,id=cavenet,start-address=${GATEWAY},end-address=192.168.77.254,subnet-mask=255.255.255.0" \
    -device virtio-net-device,netdev=cavenet \
    -serial "pipe:${LOG_DIR}/qemu-serial-${STAMP}" \
    -kernel "$KERNEL" \
    >"$QEMU_LOG" 2>&1 &
QEMU_PID=$!

# QEMU with -serial pipe creates two files: .in and .out
SERIAL_IN="${LOG_DIR}/qemu-serial-${STAMP}.in"
SERIAL_OUT="${LOG_DIR}/qemu-serial-${STAMP}.out"

# Wait for QEMU to create the serial pipe + Bat_OS to reach the auth loop.
for i in {1..60}; do
    [ -p "$SERIAL_OUT" ] && break
    sleep 0.3
done
if [ ! -p "$SERIAL_OUT" ]; then
    echo "ERROR: QEMU serial pipe never appeared"; exit 1
fi

echo "[e2e] waiting for Bat_OS auth input loop..."
if ! timeout 60 grep -q "entering input loop" <(tail -f "$QEMU_LOG") 2>/dev/null; then
    echo "ERROR: Bat_OS never reached the auth prompt"; exit 1
fi
echo "batman" > "$SERIAL_IN"
sleep 1

# 4. Find the new bridge interface.
sleep 2  # give vmnet a moment to create it
post_bridges=$(ifconfig -l | tr ' ' '\n' | grep '^bridge' | sort -u || true)
new_bridge=$(comm -13 <(echo "$pre_bridges") <(echo "$post_bridges") | head -1)
if [ -z "$new_bridge" ]; then
    echo "WARN: couldn't detect new bridge interface; using bridge100 as guess"
    new_bridge="bridge100"
fi
echo "[e2e] vmnet bridge: $new_bridge"

# 5. Create Docker macvlan on the new bridge.
echo "[e2e] creating macvlan network '$NET_NAME' on $new_bridge"
sudo -u "$REAL_USER" docker network create -d macvlan \
    --subnet="$SUBNET" \
    --gateway="$GATEWAY" \
    -o parent="$new_bridge" \
    "$NET_NAME" >/dev/null

# 6. Run a container on that network.
echo "[e2e] running alpine at $CAVE_IP (installing curl, may take ~10s)"
sudo -u "$REAL_USER" docker run -d --rm \
    --name "$CONTAINER" \
    --network "$NET_NAME" --ip="$CAVE_IP" \
    "$IMAGE" sh -c 'apk add --no-cache curl >/dev/null 2>&1 && sleep 600' \
    >/dev/null
# Wait for curl to actually be installed.
for i in {1..60}; do
    if sudo -u "$REAL_USER" docker exec "$CONTAINER" which curl >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

# 7. Wire Bat_OS: register cave binding + policy for example.com's IP.
echo "[e2e] registering cave policy in Bat_OS"
# example.com — resolve now so we can write a stable allow rule.
EXAMPLE_IP="$(dig +short example.com | head -1)"
if [ -z "$EXAMPLE_IP" ]; then EXAMPLE_IP="93.184.216.34"; fi
echo "[e2e]   example.com = $EXAMPLE_IP"
printf 'nat-reset\n'                           > "$SERIAL_IN"; sleep 0.3
printf 'nat-bind %s %s\n' "$CAVE_IP" "$CAVE_NAME"     > "$SERIAL_IN"; sleep 0.3
printf 'cpol-add %s %s 443 tcp\n' "$CAVE_NAME" "$EXAMPLE_IP" > "$SERIAL_IN"; sleep 0.3

# 8. Exercise the pipeline from inside the container.
echo "[e2e] running curl -sI https://${EXAMPLE_IP} from the container"
CURL_OUT=$(sudo -u "$REAL_USER" docker exec "$CONTAINER" \
    curl -sI --max-time 10 --resolve "example.com:443:${EXAMPLE_IP}" \
    https://example.com 2>&1 || true)
echo "$CURL_OUT" | head -5

# 9. Check nat-stats.
printf 'nat-stats\n' > "$SERIAL_IN"; sleep 1

STATS=$(tail -50 "$SERIAL_OUT" | tr -d '\r')
echo "--- nat-stats snippet ---"
echo "$STATS" | grep -E "allow|arp-replies|host-frames" || true

PASS=1
if ! echo "$CURL_OUT" | grep -qi "HTTP/"; then
    echo "FAIL: container curl didn't get an HTTP response"
    PASS=0
fi
if ! echo "$STATS" | grep -E "allow: *[1-9]" >/dev/null; then
    echo "FAIL: no allow counter recorded"
    PASS=0
fi
if ! echo "$STATS" | grep -E "arp-replies: *[1-9]" >/dev/null; then
    echo "FAIL: no ARP replies (container should've ARPed gateway)"
    PASS=0
fi

if [ "$PASS" -eq 1 ]; then
    echo "[e2e] PASS: real container traffic flowed through Bat_OS NAT"
    exit 0
else
    echo "[e2e] FAIL — inspect logs"
    exit 1
fi
