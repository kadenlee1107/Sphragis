#!/bin/bash
echo ""
echo "  BAT_OS — Path A Boot Setup"
echo "  ==========================="
echo ""

echo "[1] Checking Bat_OS volume..."
diskutil list | grep Bat_OS
echo ""

echo "[2] Mounting Bat_OS volume..."
diskutil mount "Bat_OS" 2>/dev/null
sleep 1

echo "[3] Configuring boot with kmutil..."
kmutil configure-boot -c /Volumes/Bat_OS/System/Library/Caches/com.apple.kernelcaches/kernelcache -v /Volumes/Bat_OS
echo ""

echo "[4] Blessing volume..."
bless --mount /Volumes/Bat_OS --setBoot --create-snapshot 2>/dev/null
echo ""

echo "Done! Reboot and hold power to see Bat_OS in Startup Manager."
echo ""
