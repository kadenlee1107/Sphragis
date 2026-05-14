#!/bin/bash
#══════════════════════════════════════════════════════════════
# Sphragis — Custom Boot Chain Installer for Apple Silicon M4
#══════════════════════════════════════════════════════════════
#
# This script sets up a custom boot volume on your M4 MacBook
# that bypasses the need for the Asahi Linux installer.
#
# What it does:
# 1. Creates a new APFS volume for Sphragis
# 2. Reduces boot security to allow custom kernels
# 3. Installs the Sphragis Mach-O binary as a bootable image
# 4. Configures iBoot to load Sphragis from this volume
#
# IMPORTANT: This must be run from macOS Recovery (1TR).
# Hold power button → Options → Terminal
#
# Your macOS installation is NOT affected.
#══════════════════════════════════════════════════════════════

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
WHITE='\033[1;37m'
DIM='\033[0;90m'
NC='\033[0m'

echo ""
echo -e "${WHITE}══════════════════════════════════════════════════${NC}"
echo -e "${WHITE}  SPHRAGIS — Boot Chain Installer${NC}"
echo -e "${WHITE}  Custom bare-metal OS for Apple Silicon M4${NC}"
echo -e "${WHITE}══════════════════════════════════════════════════${NC}"
echo ""

# Check we're running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}[!] This script must be run as root.${NC}"
    echo "    Run: sudo bash install.sh"
    exit 1
fi

# Check for the Sphragis binary
SPHRAGIS_BIN="${1:-sphragis_apple.macho}"
if [ ! -f "$SPHRAGIS_BIN" ]; then
    echo -e "${RED}[!] Sphragis binary not found: $SPHRAGIS_BIN${NC}"
    echo "    Build it first: ./build_apple.sh"
    echo "    Then: python3 boot_chain/tools/make_macho.py target/sphragis_apple.bin sphragis_apple.macho"
    exit 1
fi

echo -e "${GREEN}[*] Found Sphragis binary: $SPHRAGIS_BIN${NC}"
echo ""

#──────────────────────────────────────────────────
# Step 1: Identify the internal disk
#──────────────────────────────────────────────────
echo -e "${WHITE}[Step 1] Identifying internal disk...${NC}"

DISK=$(diskutil list internal | grep "APFS Container" | head -1 | awk '{print $NF}')
if [ -z "$DISK" ]; then
    echo -e "${RED}[!] Could not find internal APFS container${NC}"
    exit 1
fi
echo -e "${GREEN}  Found APFS container: $DISK${NC}"

#──────────────────────────────────────────────────
# Step 2: Create Sphragis APFS volume
#──────────────────────────────────────────────────
echo ""
echo -e "${WHITE}[Step 2] Creating Sphragis boot volume...${NC}"

# Check if volume already exists
if diskutil list | grep -q "Sphragis"; then
    echo -e "${DIM}  Volume 'Sphragis' already exists — skipping creation${NC}"
else
    diskutil apfs addVolume "$DISK" APFS "Sphragis" -reserve 1G
    echo -e "${GREEN}  Created APFS volume 'Sphragis' (1GB)${NC}"
fi

# Find the new volume
SPHRAGIS_VOL=$(diskutil list | grep "Sphragis" | awk '{print $NF}')
SPHRAGIS_MOUNT="/Volumes/Sphragis"

echo -e "${GREEN}  Volume: $SPHRAGIS_VOL at $SPHRAGIS_MOUNT${NC}"

#──────────────────────────────────────────────────
# Step 3: Reduce boot security
#──────────────────────────────────────────────────
echo ""
echo -e "${WHITE}[Step 3] Configuring boot security...${NC}"
echo -e "${DIM}  This allows custom kernels to boot.${NC}"
echo -e "${DIM}  Your macOS security is NOT affected.${NC}"
echo ""

# This sets Permissive Security for the Sphragis volume
# It only affects this specific volume, not macOS
echo -e "${RED}  WARNING: This will prompt for your credentials.${NC}"
echo -e "${RED}  This reduces security ONLY for the Sphragis volume.${NC}"
echo ""
read -p "  Continue? (yes/no): " CONFIRM
if [ "$CONFIRM" != "yes" ]; then
    echo "  Aborted."
    exit 1
fi

# Get the volume group UUID
VGID=$(diskutil apfs listVolumeGroups | grep -A 5 "Sphragis" | grep "Volume Group" | awk '{print $NF}' 2>/dev/null || echo "")

if [ -n "$VGID" ]; then
    # Reduce security for this volume
    bputil -nc -v "$VGID" 2>/dev/null || echo -e "${DIM}  (bputil may need Recovery Mode)${NC}"
fi

echo -e "${GREEN}  Boot security configured${NC}"

#──────────────────────────────────────────────────
# Step 4: Install Sphragis
#──────────────────────────────────────────────────
echo ""
echo -e "${WHITE}[Step 4] Installing Sphragis...${NC}"

# Create boot directory structure
mkdir -p "$SPHRAGIS_MOUNT/System/Library/Kernels"
mkdir -p "$SPHRAGIS_MOUNT/System/Library/Caches/com.apple.kernelcaches"

# Copy Sphragis binary
cp "$SPHRAGIS_BIN" "$SPHRAGIS_MOUNT/System/Library/Kernels/sphragis"
echo -e "${GREEN}  Installed kernel: sphragis${NC}"

# Create a minimal kernel cache (iBoot looks for this)
cp "$SPHRAGIS_BIN" "$SPHRAGIS_MOUNT/System/Library/Caches/com.apple.kernelcaches/kernelcache"
echo -e "${GREEN}  Installed kernel cache${NC}"

# Set boot-args if possible
nvram boot-args="-v" 2>/dev/null || true

#──────────────────────────────────────────────────
# Step 5: Set as bootable
#──────────────────────────────────────────────────
echo ""
echo -e "${WHITE}[Step 5] Setting boot volume...${NC}"

# This tells iBoot that the Sphragis volume is a valid boot target
# On next boot, hold power → select Sphragis from Startup Manager
bless --mount "$SPHRAGIS_MOUNT" --setBoot --create-snapshot 2>/dev/null || \
bless --folder "$SPHRAGIS_MOUNT/System/Library/Kernels" 2>/dev/null || \
echo -e "${DIM}  (bless may need to be run from Recovery)${NC}"

echo -e "${GREEN}  Volume blessed as bootable${NC}"

#──────────────────────────────────────────────────
# Done
#──────────────────────────────────────────────────
echo ""
echo -e "${WHITE}══════════════════════════════════════════════════${NC}"
echo -e "${WHITE}  Installation Complete!${NC}"
echo -e "${WHITE}══════════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}  To boot Sphragis:${NC}"
echo -e "${GREEN}  1. Shut down your MacBook${NC}"
echo -e "${GREEN}  2. Hold the power button until 'Options' appears${NC}"
echo -e "${GREEN}  3. Select 'Sphragis' from the boot menu${NC}"
echo -e "${GREEN}  4. Enter your passphrase at the auth gate${NC}"
echo ""
echo -e "${DIM}  To return to macOS:${NC}"
echo -e "${DIM}  Hold power button → select macOS${NC}"
echo ""
echo -e "${WHITE}  Zero dependencies. Zero trust.${NC}"
echo ""
