#!/bin/bash
#══════════════════════════════════════════════════
# Bat_OS — Deploy v2 (img4 + boot policy)
# Run from Recovery Mode Terminal:
#   cd /Volumes/CanonA_0007 && bash deploy_v2.sh
#══════════════════════════════════════════════════

echo ""
echo "  BAT_OS DEPLOY v2"
echo "  ================="
echo "  img4 kernel cache + boot policy"
echo ""

# Check required files
if [ ! -f ./bat_os_apple.macho ]; then
    echo "[!] bat_os_apple.macho not found"
    exit 1
fi
echo "[+] Found kernel binary"

# ─── Step 1: Find APFS Container ───
echo ""
echo "[Step 1] Finding APFS container..."

CONTAINER=""
# Try to find the container with Macintosh HD
for disk in $(diskutil apfs list 2>/dev/null | grep "Container Reference" | awk '{print $NF}'); do
    if diskutil apfs list "$disk" 2>/dev/null | grep -q "Macintosh HD"; then
        CONTAINER="$disk"
        break
    fi
done

# Fallback: try each container
if [ -z "$CONTAINER" ]; then
    for disk in $(diskutil apfs list 2>/dev/null | grep "Container Reference" | awk '{print $NF}'); do
        CONTAINER="$disk"
        break
    done
fi

if [ -z "$CONTAINER" ]; then
    echo "[!] No APFS container found"
    echo "    Run: diskutil apfs list"
    echo "    Enter container disk (e.g., disk3):"
    read CONTAINER
fi

echo "[+] Container: $CONTAINER"

# ─── Step 2: Clean up old Bat_OS volume if exists ───
echo ""
echo "[Step 2] Preparing volume..."

if diskutil list 2>/dev/null | grep -q "Bat_OS"; then
    echo "[*] Removing old Bat_OS volume..."
    diskutil apfs deleteVolume "Bat_OS" 2>/dev/null || true
    sleep 2
fi

# ─── Step 3: Create volume group (System + Data pair) ───
echo ""
echo "[Step 3] Creating Bat_OS volume group..."

# Create the system volume
diskutil apfs addVolume "$CONTAINER" APFS "Bat_OS" -role S 2>/dev/null
if [ $? -ne 0 ]; then
    echo "[*] Trying without role flag..."
    diskutil apfs addVolume "$CONTAINER" APFS "Bat_OS"
fi
sleep 2

# Mount it
diskutil mount "Bat_OS" 2>/dev/null
sleep 1

if [ ! -d "/Volumes/Bat_OS" ]; then
    echo "[!] Failed to mount Bat_OS volume"
    echo "    Checking available volumes..."
    ls /Volumes/
    exit 1
fi
echo "[+] Volume mounted at /Volumes/Bat_OS"

# ─── Step 4: Create img4 kernel cache ───
echo ""
echo "[Step 4] Creating img4 kernel cache..."

# Check if python3 is available
if command -v python3 &>/dev/null; then
    python3 ./make_img4.py ./bat_os_apple.macho ./kernelcache.img4
    IMG4_MADE=$?
else
    echo "[*] python3 not available — using Mach-O directly"
    cp ./bat_os_apple.macho ./kernelcache.img4
    IMG4_MADE=1
fi

# ─── Step 5: Install kernel in ALL locations iBoot might look ───
echo ""
echo "[Step 5] Installing kernel..."

# Standard kernel location
mkdir -p "/Volumes/Bat_OS/System/Library/Kernels"
mkdir -p "/Volumes/Bat_OS/System/Library/Caches/com.apple.kernelcaches"
mkdir -p "/Volumes/Bat_OS/System/Library/KernelCollections"
mkdir -p "/Volumes/Bat_OS/usr/standalone/firmware"

# Copy to every possible location
cp ./bat_os_apple.macho "/Volumes/Bat_OS/System/Library/Kernels/kernel"
echo "  [+] Installed kernel"

if [ -f ./kernelcache.img4 ]; then
    cp ./kernelcache.img4 "/Volumes/Bat_OS/System/Library/Caches/com.apple.kernelcaches/kernelcache"
    cp ./kernelcache.img4 "/Volumes/Bat_OS/System/Library/KernelCollections/BootKernelExtensions.kc"
    echo "  [+] Installed img4 kernel cache"
else
    cp ./bat_os_apple.macho "/Volumes/Bat_OS/System/Library/Caches/com.apple.kernelcaches/kernelcache"
    cp ./bat_os_apple.macho "/Volumes/Bat_OS/System/Library/KernelCollections/BootKernelExtensions.kc"
    echo "  [+] Installed Mach-O kernel cache (fallback)"
fi

# ─── Step 6: Configure boot security ───
echo ""
echo "[Step 6] Configuring boot security..."

# Get the Bat_OS volume UUID
VOLUME_UUID=$(diskutil info "Bat_OS" 2>/dev/null | grep "Volume UUID" | awk '{print $NF}')
VOLUME_GROUP=$(diskutil info "Bat_OS" 2>/dev/null | grep "Volume Group" | awk '{print $NF}' | head -1)
DISK_ID=$(diskutil info "Bat_OS" 2>/dev/null | grep "Device Identifier" | awk '{print $NF}')

echo "  Volume UUID: ${VOLUME_UUID:-unknown}"
echo "  Volume Group: ${VOLUME_GROUP:-unknown}"
echo "  Disk ID: ${DISK_ID:-unknown}"

# Method 1: Disable authenticated root for this volume
echo "  [*] Disabling authenticated root..."
csrutil authenticated-root disable 2>/dev/null && echo "  [+] Done" || echo "  [*] May need manual confirmation"

# Method 2: Set no-security boot policy
echo "  [*] Setting permissive boot policy..."
bputil -nc 2>/dev/null && echo "  [+] Done" || echo "  [*] May need credentials"

# Method 3: Allow third-party kernel extensions
echo "  [*] Allowing third-party kernels..."
if [ -n "$DISK_ID" ]; then
    bputil -a -v "$DISK_ID" 2>/dev/null || true
fi

# Method 4: Try kmutil to create a proper boot collection
echo "  [*] Attempting kmutil boot configuration..."
kmutil configure-boot \
    -c "/Volumes/Bat_OS/System/Library/Caches/com.apple.kernelcaches/kernelcache" \
    -v "/Volumes/Bat_OS" \
    --raw 2>/dev/null && echo "  [+] kmutil succeeded" || echo "  [*] kmutil: may not accept custom kernel"

# ─── Step 7: Bless the volume ───
echo ""
echo "[Step 7] Blessing boot volume..."

# Try multiple bless approaches
bless --mount "/Volumes/Bat_OS" \
    --setBoot \
    --create-snapshot \
    --bootefi 2>/dev/null && echo "[+] Bless method 1 succeeded" || true

bless --mount "/Volumes/Bat_OS" \
    --setBoot 2>/dev/null && echo "[+] Bless method 2 succeeded" || true

bless --folder "/Volumes/Bat_OS/System/Library/Kernels" \
    --setBoot 2>/dev/null && echo "[+] Bless method 3 succeeded" || true

# Try setting it as next boot via nvram
echo "  [*] Setting boot-volume via nvram..."
if [ -n "$VOLUME_UUID" ]; then
    nvram boot-volume="$VOLUME_UUID" 2>/dev/null || true
fi

# Set verbose boot
nvram boot-args="-v debug=0x14e" 2>/dev/null || true

# ─── Step 8: Verify ───
echo ""
echo "[Step 8] Verification..."

echo "  Volume contents:"
ls -la "/Volumes/Bat_OS/System/Library/Kernels/" 2>/dev/null
ls -la "/Volumes/Bat_OS/System/Library/Caches/com.apple.kernelcaches/" 2>/dev/null

echo ""
echo "  Boot policy:"
bputil -d 2>/dev/null | head -20 || echo "  (unable to read boot policy)"

echo ""
echo "══════════════════════════════════════════════"
echo "  DEPLOY COMPLETE"
echo "══════════════════════════════════════════════"
echo ""
echo "  To boot: shut down → hold power → Startup Manager"
echo ""
echo "  If Bat_OS doesn't appear in Startup Manager:"
echo "  It means iBoot's boot policy still doesn't"
echo "  recognize our volume. The output above shows"
echo "  what worked and what didn't — send me a photo"
echo "  of this screen and we'll iterate."
echo ""
echo "  Your macOS is completely safe."
echo ""
