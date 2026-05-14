#!/bin/bash
echo ""
echo "  SPHRAGIS — Path A Boot Setup"
echo "  ==========================="
echo ""

echo "[1] Checking Sphragis volume..."
diskutil list | grep Sphragis
echo ""

echo "[2] Mounting Sphragis volume..."
diskutil mount "Sphragis" 2>/dev/null
sleep 1

echo "[3] Configuring boot with kmutil..."
kmutil configure-boot -c /Volumes/Sphragis/System/Library/Caches/com.apple.kernelcaches/kernelcache -v /Volumes/Sphragis
echo ""

echo "[4] Blessing volume..."
bless --mount /Volumes/Sphragis --setBoot --create-snapshot 2>/dev/null
echo ""

echo "Done! Reboot and hold power to see Sphragis in Startup Manager."
echo ""
