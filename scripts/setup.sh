#!/usr/bin/env bash
# Bat_OS — one-time Ubuntu setup. Run once after `git clone`.
# Idempotent: rerunning is safe and cheap.
set -euo pipefail

log() { echo -e "\033[1;34m[setup]\033[0m $*"; }
err() { echo -e "\033[1;31m[setup] ERROR:\033[0m $*" >&2; exit 1; }

# ─── apt packages ──────────────────────────────────────────────────

log "Updating apt cache"
sudo apt update -qq

log "Installing apt packages"
sudo apt install -y \
    python3-construct python3-serial \
    gcc-aarch64-linux-gnu \
    openssh-server \
    curl git gh \
    ffmpeg v4l-utils \
    tmux htop \
    build-essential pkg-config \
    python3-pip

# ─── Tailscale (for the Mac↔Ubuntu private network) ───────────────

if ! command -v tailscale >/dev/null; then
    log "Installing Tailscale"
    curl -fsSL https://tailscale.com/install.sh | sh
else
    log "Tailscale already installed"
fi

# ─── Serial port access ────────────────────────────────────────────

if ! groups "$USER" | grep -q dialout; then
    log "Adding $USER to dialout group (for /dev/ttyACM* access)"
    sudo usermod -a -G dialout "$USER"
    log "  → Log out + back in for group change to take effect."
fi

# Stable /dev/m1n1 symlink when m1n1's composite device is plugged in.
UDEV_RULE=/etc/udev/rules.d/99-m1n1.rules
if [ ! -f "$UDEV_RULE" ]; then
    log "Installing udev rule for m1n1 (VID 1209 / PID 316D)"
    sudo tee "$UDEV_RULE" > /dev/null <<'EOF'
# m1n1 USB CDC serial — consistent naming + group access
SUBSYSTEM=="tty", ATTRS{idVendor}=="1209", ATTRS{idProduct}=="316d", \
    SYMLINK+="m1n1", GROUP="dialout", MODE="0660"
EOF
    sudo udevadm control --reload-rules
    sudo udevadm trigger
fi

# ─── Rust toolchain ────────────────────────────────────────────────

if ! command -v rustup >/dev/null; then
    log "Installing rustup + Rust"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
fi

log "Ensuring Rust nightly + rust-src + aarch64 target"
rustup toolchain install nightly --profile minimal
rustup component add rust-src --toolchain nightly
rustup target add aarch64-unknown-none-softfloat --toolchain nightly
# Also install stable for bat_os itself (rust-toolchain.toml usually pins it)
rustup toolchain install stable --profile default

# ─── Claude Code (optional — user can skip) ────────────────────────

if ! command -v claude >/dev/null; then
    log "Claude Code is not installed. Install manually with:"
    log "  curl -fsSL https://claude.ai/install.sh | sh"
    log "  (or whichever path Anthropic publishes currently)"
fi

# ─── Build dirs ────────────────────────────────────────────────────

mkdir -p logs captures

log "Done. Useful next commands:"
log "  sudo tailscale up          # join the private network"
log "  ./scripts/chainload.sh X   # chainload Bat_OS binary X"
log "  ./scripts/rebuild.sh       # cargo build the Apple binary"
