#!/bin/bash
# Bat_OS — Skia Minimal Build Script
# Compiles the software rasterizer subset of Skia for ARM64
set -e

SKIA=/Users/kadenlee/Bat_OS/ports/skia
OUT=/tmp/skia_obj
INCLUDE=/Users/kadenlee/Bat_OS/include

mkdir -p ${OUT}

CC="clang++ --target=aarch64-linux-gnu -ffreestanding -nostdlib -O2 -mstrict-align -fPIC \
    -fno-exceptions -fno-rtti -std=c++17 \
    -isystem ${INCLUDE}/c++ \
    -isystem ${INCLUDE} \
    -I${SKIA} -I${SKIA}/include -I${SKIA}/src \
    -DSK_BUILD_FOR_UNIX \
    -DSK_ASSUME_GL=0 \
    -DSK_ENABLE_PRECOMPILE=0 \
    -DSK_DISABLE_EFFECT_DESERIALIZATION \
    -DSK_DISABLE_TRACING \
    -DSK_R32_SHIFT=0 \
    -DSK_SUPPORT_GPU=0 \
    -DSK_GL=0 \
    -DSK_VULKAN=0 \
    -DSK_METAL=0 \
    -DSK_DAWN=0 \
    -DSK_DIRECT3D=0 \
    -DSK_GRAPHITE=0 \
    -DSK_GANESH=0 \
    -Wno-c99-designator \
    -Wno-unused-parameter \
    -Wno-sign-compare"

COMPILED=0
ERRORS=0

compile() {
    local src=$1
    local obj=${OUT}/$(basename $src .cpp).o
    eval ${CC} -c "$src" -o "$obj" 2>/tmp/skia_err.txt
    if [ $? -eq 0 ]; then
        COMPILED=$((COMPILED + 1))
    else
        echo "FAIL: $(basename $src): $(head -1 /tmp/skia_err.txt)"
        ERRORS=$((ERRORS + 1))
    fi
}

echo "=== Building Skia core (software rasterizer) ==="

# Base module
for f in ${SKIA}/src/base/*.cpp; do compile "$f"; done

# Core module (the big one)
for f in ${SKIA}/src/core/*.cpp; do compile "$f"; done

# Image module (SkSurface, SkImage — raster path only)
for f in SkImage.cpp SkImage_Lazy.cpp SkImage_Raster.cpp SkSurface.cpp SkSurface_Base.cpp SkSurface_Null.cpp SkSurface_Raster.cpp; do
    [ -f "${SKIA}/src/image/${f}" ] && compile "${SKIA}/src/image/${f}"
done

# Effects (basic shaders)
for f in ${SKIA}/src/effects/*.cpp; do compile "$f"; done

# Shaders
for f in ${SKIA}/src/shaders/*.cpp; do compile "$f"; done

# Ports (platform layer)
for f in SkDebug_stdio.cpp SkMemory_malloc.cpp SkOSFile_stdio.cpp SkOSFile_none.cpp; do
    [ -f "${SKIA}/src/ports/${f}" ] && compile "${SKIA}/src/ports/${f}"
done

echo ""
echo "=== Results: ${COMPILED} compiled, ${ERRORS} errors ==="
echo "Object files: $(ls ${OUT}/*.o 2>/dev/null | wc -l)"
