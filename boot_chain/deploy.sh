#!/bin/bash
#══════════════════════════════════════════════
# Bat_OS — One-Shot Deploy Script
# Run this from Recovery Mode Terminal:
#   cd /Volumes/CanonA_0007 && bash deploy.sh
#══════════════════════════════════════════════

echo ""
echo "  BAT_OS DEPLOYER"
echo "  ==============="
echo ""

# Find the Mach-O binary
if [ ! -f ./bat_os_apple.macho ]; then
    echo "[!] bat_os_apple.macho not found in current directory"
    exit 1
fi
echo "[+] Found bat_os_apple.macho"

# Find the right APFS container (the one with Macintosh HD)
echo "[*] Finding APFS container..."
CONTAINER=""
for disk in $(diskutil apfs list | grep "Container Reference" | awk '{print $NF}'); do
    if diskutil apfs list "$disk" 2>/dev/null | grep -q "Macintosh HD"; then
        CONTAINER="$disk"
        break
    fi
done

if [ -z "$CONTAINER" ]; then
    echo "[!] Could not find Macintosh HD container"
    echo "[*] Listing all containers:"
    diskutil apfs list | grep -E "Container Reference|Volume Name"
    echo ""
    echo "Enter the container disk (e.g. disk3):"
    read CONTAINER
fi

echo "[+] Using container: $CONTAINER"

# Create volume
echo "[*] Creating Bat_OS volume..."
if diskutil apfs list | grep -q "Bat_OS"; then
    echo "[*] Volume already exists — skipping"
else
    diskutil apfs addVolume "$CONTAINER" APFS "Bat_OS" -reserve 1000000000
fi

# Wait for mount
sleep 2

# Check mount
if [ ! -d "/Volumes/Bat_OS" ]; then
    echo "[!] /Volumes/Bat_OS not mounted — trying to mount..."
    diskutil mount "Bat_OS"
    sleep 2
fi

# Install kernel
echo "[*] Installing Bat_OS kernel..."
mkdir -p /Volumes/Bat_OS/System/Library/Kernels
mkdir -p /Volumes/Bat_OS/System/Library/Caches/com.apple.kernelcaches
cp ./bat_os_apple.macho /Volumes/Bat_OS/System/Library/Kernels/bat_os
cp ./bat_os_apple.macho /Volumes/Bat_OS/System/Library/Caches/com.apple.kernelcaches/kernelcache
echo "[+] Kernel installed"

# Security
echo "[*] Reducing boot security for Bat_OS volume..."
bputil -nc 2>/dev/null || echo "[*] bputil: may need manual confirmation"
csrutil authenticated-root disable 2>/dev/null || echo "[*] csrutil: may need manual confirmation"

# Bless
echo "[*] Blessing boot volume..."
bless --mount /Volumes/Bat_OS --setBoot --create-snapshot 2>/dev/null || \
bless --folder /Volumes/Bat_OS/System/Library/Kernels 2>/dev/null || \
echo "[*] bless: may need manual configuration"

echo ""
echo "  ══════════════════════════════════════"
echo "  DONE! Reboot and hold power button"
echo "  Select 'Bat_OS' from Startup Manager"
echo "  Passphrase: batman"
echo "  ══════════════════════════════════════"
echo ""
