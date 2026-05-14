#!/bin/bash
#══════════════════════════════════════════════
# Sphragis — One-Shot Deploy Script
# Run this from Recovery Mode Terminal:
#   cd /Volumes/CanonA_0007 && bash deploy.sh
#══════════════════════════════════════════════

echo ""
echo "  SPHRAGIS DEPLOYER"
echo "  ==============="
echo ""

# Find the Mach-O binary
if [ ! -f ./sphragis_apple.macho ]; then
    echo "[!] sphragis_apple.macho not found in current directory"
    exit 1
fi
echo "[+] Found sphragis_apple.macho"

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
echo "[*] Creating Sphragis volume..."
if diskutil apfs list | grep -q "Sphragis"; then
    echo "[*] Volume already exists — skipping"
else
    diskutil apfs addVolume "$CONTAINER" APFS "Sphragis" -reserve 1000000000
fi

# Wait for mount
sleep 2

# Check mount
if [ ! -d "/Volumes/Sphragis" ]; then
    echo "[!] /Volumes/Sphragis not mounted — trying to mount..."
    diskutil mount "Sphragis"
    sleep 2
fi

# Install kernel
echo "[*] Installing Sphragis kernel..."
mkdir -p /Volumes/Sphragis/System/Library/Kernels
mkdir -p /Volumes/Sphragis/System/Library/Caches/com.apple.kernelcaches
cp ./sphragis_apple.macho /Volumes/Sphragis/System/Library/Kernels/sphragis
cp ./sphragis_apple.macho /Volumes/Sphragis/System/Library/Caches/com.apple.kernelcaches/kernelcache
echo "[+] Kernel installed"

# Security
echo "[*] Reducing boot security for Sphragis volume..."
bputil -nc 2>/dev/null || echo "[*] bputil: may need manual confirmation"
csrutil authenticated-root disable 2>/dev/null || echo "[*] csrutil: may need manual confirmation"

# Bless
echo "[*] Blessing boot volume..."
bless --mount /Volumes/Sphragis --setBoot --create-snapshot 2>/dev/null || \
bless --folder /Volumes/Sphragis/System/Library/Kernels 2>/dev/null || \
echo "[*] bless: may need manual configuration"

echo ""
echo "  ══════════════════════════════════════"
echo "  DONE! Reboot and hold power button"
echo "  Select 'Sphragis' from Startup Manager"
echo "  Passphrase: sphragis-dev"
echo "  ══════════════════════════════════════"
echo ""
