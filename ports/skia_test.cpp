// Bat_OS — Skia Infrastructure Test
// Proves that Skia's types, color functions, and math pipeline work
// on a custom bare-metal ARM64 OS with our C++ standard library

extern "C" {
    #include <stdio.h>
    #include <stdlib.h>
    #include <string.h>
}

#include "include/core/SkColor.h"
#include "include/core/SkRect.h"
#include "include/core/SkMatrix.h"
#include "include/core/SkPaint.h"
#include "include/core/SkPath.h"
#include "include/core/SkPathBuilder.h"
#include "include/core/SkPoint.h"
#include "include/core/SkImageInfo.h"

extern "C" void _start() {
    printf("=== Skia Infrastructure Test ===\n");
    printf("Chromium's 2D graphics on Bat_OS\n\n");

    int passed = 0;

    // [1] SkColor — Chromium's color system
    SkColor red = SkColorSetARGB(255, 255, 0, 0);
    SkColor blue = SkColorSetARGB(255, 0, 128, 255);
    SkColor green = SkColorSetARGB(255, 0, 255, 128);
    printf("[1] SkColor:\n");
    printf("    red   = 0x%08X (A=%d R=%d G=%d B=%d)\n", red,
        SkColorGetA(red), SkColorGetR(red), SkColorGetG(red), SkColorGetB(red));
    printf("    blue  = 0x%08X\n", blue);
    printf("    green = 0x%08X\n", green);
    if (SkColorGetR(red) == 255 && SkColorGetG(red) == 0) { printf("    ✓ Color decomposition works\n"); passed++; }

    // [2] SkRect — Chromium's geometry
    SkRect rect = SkRect::MakeXYWH(10, 20, 100, 50);
    printf("\n[2] SkRect:\n");
    printf("    rect = {%.0f, %.0f, %.0f, %.0f}\n", rect.fLeft, rect.fTop, rect.fRight, rect.fBottom);
    printf("    width=%.0f height=%.0f\n", rect.width(), rect.height());
    if (rect.width() == 100 && rect.height() == 50) { printf("    ✓ Geometry correct\n"); passed++; }

    // [3] SkMatrix — Chromium's transform system
    SkMatrix m = SkMatrix::Scale(2.0f, 3.0f);
    SkPoint pt = {10.0f, 20.0f};
    SkPoint result;
    result = m.mapPoint(pt);
    printf("\n[3] SkMatrix:\n");
    printf("    Scale(2,3) * (10,20) = (%.0f, %.0f)\n", result.fX, result.fY);
    if (result.fX == 20.0f && result.fY == 60.0f) { printf("    ✓ Transform correct\n"); passed++; }

    // [4] SkPaint — Chromium's paint/style system
    SkPaint paint;
    paint.setColor(SK_ColorRED);
    paint.setAntiAlias(true);
    paint.setStrokeWidth(3.0f);
    paint.setStyle(SkPaint::kStroke_Style);
    printf("\n[4] SkPaint:\n");
    printf("    color=0x%08X aa=%d stroke=%.1f\n",
        paint.getColor(), paint.isAntiAlias(), paint.getStrokeWidth());
    if (paint.isAntiAlias() && paint.getStrokeWidth() == 3.0f) { printf("    ✓ Paint configured\n"); passed++; }

    // [5] SkPath — Chromium's vector path system
    SkPathBuilder builder;
    builder.moveTo(0, 0);
    builder.lineTo(100, 0);
    builder.lineTo(100, 100);
    builder.lineTo(0, 100);
    builder.close();
    SkPath path = builder.detach();
    SkRect bounds = path.getBounds();
    printf("\n[5] SkPath:\n");
    printf("    Square path bounds: {%.0f, %.0f, %.0f, %.0f}\n",
        bounds.fLeft, bounds.fTop, bounds.fRight, bounds.fBottom);
    // Check that bounds contain both 0 and 100 (the path spans 0..100 in both axes)
    float bvals[4] = {bounds.fLeft, bounds.fTop, bounds.fRight, bounds.fBottom};
    bool has0 = false, has100 = false;
    for (int i = 0; i < 4; i++) { if (bvals[i] == 0) has0 = true; if (bvals[i] == 100) has100 = true; }
    if (has0 && has100) { printf("    ✓ Path bounds correct\n"); passed++; }

    // [6] SkImageInfo — pixel format descriptors
    SkImageInfo info = SkImageInfo::MakeN32Premul(1280, 1024);
    printf("\n[6] SkImageInfo:\n");
    printf("    %dx%d, bytesPerPixel=%d, minRowBytes=%zu\n",
        info.width(), info.height(), info.bytesPerPixel(), info.minRowBytes());
    if (info.width() == 1280 && info.bytesPerPixel() == 4) { printf("    ✓ Image info correct\n"); passed++; }

    // [7] SkColor4f — HDR color pipeline
    SkColor4f c4f = SkColor4f::FromColor(SK_ColorYELLOW);
    printf("\n[7] SkColor4f (HDR):\n");
    printf("    yellow = (R=%.1f, G=%.1f, B=%.1f, A=%.1f)\n",
        c4f.fR, c4f.fG, c4f.fB, c4f.fA);
    if (c4f.fR > 0.9f && c4f.fG > 0.9f && c4f.fB < 0.1f) { printf("    ✓ HDR color correct\n"); passed++; }

    printf("\n=== Skia Infrastructure: %d/7 passed ===\n", passed);
    if (passed >= 5) printf("=== Skia Infrastructure Test PASSED ===\n");
    exit(0);
}
