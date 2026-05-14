#!/bin/bash
# 3c-deferred-4 preflight — verify every prerequisite for running the
# full vmnet + Docker macvlan path end-to-end. Doesn't need sudo and
# doesn't touch any state; just fails fast with a clear message if
# something's off.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KERNEL="${ROOT}/target/aarch64-unknown-none/release/sphragis"
STATUS=0

say_ok()  { printf '  \033[32m✓\033[0m %s\n' "$*"; }
say_bad() { printf '  \033[31m✗\033[0m %s\n' "$*"; STATUS=1; }
say_info(){ printf '  \033[36mi\033[0m %s\n' "$*"; }

echo "[preflight] Sphragis packet-pipeline vmnet recipe"
echo

# 1. Kernel built with SPHRAGIS_PASSPHRASE=sphragis-dev
if [ -f "$KERNEL" ]; then
    age=$(stat -f %m "$KERNEL")
    now=$(date +%s)
    delta=$(( now - age ))
    if [ "$delta" -lt 3600 ]; then
        say_ok "kernel built (${delta}s ago)"
    else
        say_info "kernel is ${delta}s old — rebuild if you just changed code"
    fi
else
    say_bad "kernel missing: $KERNEL"
    say_info "build with:  SPHRAGIS_PASSPHRASE=sphragis-dev cargo build --release"
fi

# 2. QEMU present + vmnet backend compiled in
if ! command -v qemu-system-aarch64 >/dev/null; then
    say_bad "qemu-system-aarch64 not on PATH"
    say_info "install:  brew install qemu"
else
    if qemu-system-aarch64 -machine virt -netdev help 2>&1 | grep -q vmnet-host; then
        say_ok "qemu-system-aarch64 has vmnet-host backend"
    else
        say_bad "qemu-system-aarch64 built WITHOUT vmnet support"
        say_info "Homebrew's qemu 10+ ships it; if you built from source, add --enable-vmnet"
    fi
fi

# 3. Docker running
if ! command -v docker >/dev/null; then
    say_bad "docker CLI not on PATH"
else
    if docker info >/dev/null 2>&1; then
        ver=$(docker version --format '{{.Server.Version}}' 2>/dev/null || echo "?")
        say_ok "docker daemon reachable (server $ver)"
    else
        say_bad "docker CLI present but daemon isn't reachable"
        say_info "start OrbStack / Docker Desktop, or run a remote context"
    fi
fi

# 4. batcaved not already listening on :9999 (would steal our daemon's port)
if lsof -nP -iTCP:9999 -sTCP:LISTEN 2>/dev/null | grep -q LISTEN; then
    say_bad "something is already listening on :9999"
    say_info "close it or pick a different port for the test daemon"
else
    say_ok ":9999 is free for batcaved"
fi

# 5. Python modules the test harness needs
missing_py=()
for m in pexpect; do
    if ! python3 -c "import $m" 2>/dev/null; then missing_py+=("$m"); fi
done
if [ "${#missing_py[@]}" -eq 0 ]; then
    say_ok "python3 deps ok (pexpect)"
else
    say_bad "python3 missing: ${missing_py[*]}"
    say_info "install:  pip3 install ${missing_py[*]}"
fi

# 6. Warn about vmnet sudo requirement (not a failure, just info)
if [ "$EUID" -ne 0 ]; then
    say_info "vmnet.framework needs sudo — run the launch script with sudo"
fi

echo
if [ "$STATUS" -eq 0 ]; then
    printf "[preflight] \033[32mOK\033[0m — run scripts/qemu_vmnet_docker_e2e.sh with sudo.\n"
else
    printf "[preflight] \033[31mFAIL\033[0m — fix the above and re-run.\n"
fi
exit $STATUS
