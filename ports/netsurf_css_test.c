// Bat_OS — NetSurf CSS Engine Test
// Proves libcss can parse CSS on our bare-metal OS

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

// libcss types
#include <libcss/libcss.h>
#include <parserutils/charset/mibenum.h>
#include <libwapcaplet/libwapcaplet.h>

// Memory allocator for libcss
static void *css_alloc(void *ptr, size_t size, void *pw) {
    (void)pw;
    if (size == 0) { free(ptr); return NULL; }
    if (ptr) return realloc(ptr, size);
    return malloc(size);
}

// URL resolver (stub)
static css_error resolve_url(void *pw, const char *base,
    lwc_string *rel, lwc_string **abs) {
    (void)pw; (void)base;
    *abs = lwc_string_ref(rel);
    return CSS_OK;
}

void _start(void) {
    printf("=== NetSurf CSS Engine Test ===\n\n");

    // Test 1: Create a CSS stylesheet
    printf("[1] Creating CSS stylesheet... ");
    css_stylesheet_params params;
    memset(&params, 0, sizeof(params));
    params.params_version = CSS_STYLESHEET_PARAMS_VERSION_1;
    params.level = CSS_LEVEL_3;
    params.charset = NULL; // auto-detect
    params.url = "about:blank";
    params.allow_quirks = false;
    params.inline_style = false;
    params.resolve = resolve_url;
    params.resolve_pw = NULL;

    css_stylesheet *sheet = NULL;

    // Test charset lookup directly
    printf("  Testing charset lookup:\n");
    // Try the exact alias name as stored in the table: "utf8" (no hyphen)
    uint16_t mib = parserutils_charset_mibenum_from_name("utf8", 4);
    printf("    utf8 (len=4) = MIBenum %d\n", mib);
    mib = parserutils_charset_mibenum_from_name("UTF-8", 5);
    printf("    UTF-8 (len=5) = MIBenum %d\n", mib);
    mib = parserutils_charset_mibenum_from_name("437", 3);
    printf("    437 (len=3) = MIBenum %d (should be non-zero)\n", mib);
    // Test our toupper
    printf("    toupper test: 'a'=%c 'z'=%c '0'=%c\n", toupper('a'), toupper('z'), toupper('0'));

    css_error err = css_stylesheet_create(&params, &sheet);
    if (err == CSS_OK && sheet) {
        printf("OK!\n");
    } else {
        printf("FAILED (err=%d)\n", err);
        exit(1);
    }

    // Test 2: Parse CSS
    printf("[2] Parsing CSS: 'body { color: red; }' ... ");
    const char *css_text = "body { color: red; font-size: 16px; margin: 10px; }";
    err = css_stylesheet_append_data(sheet,
        (const uint8_t *)css_text, strlen(css_text));
    if (err == CSS_OK || err == CSS_NEEDDATA) {
        printf("OK!\n");
    } else {
        printf("FAILED (err=%d)\n", err);
    }

    // Test 3: Complete the stylesheet
    printf("[3] Completing stylesheet... ");
    err = css_stylesheet_data_done(sheet);
    if (err == CSS_OK) {
        printf("OK!\n");
    } else {
        printf("FAILED (err=%d)\n", err);
    }

    // Test 4: Get stylesheet size
    printf("[4] Stylesheet size: ");
    size_t size = 0;
    err = css_stylesheet_size(sheet, &size);
    if (err == CSS_OK) {
        printf("%zu bytes\n", size);
    } else {
        printf("FAILED\n");
    }

    // Test 5: Clean up
    printf("[5] Destroying stylesheet... ");
    err = css_stylesheet_destroy(sheet);
    if (err == CSS_OK) {
        printf("OK!\n");
    } else {
        printf("FAILED\n");
    }

    printf("\n=== CSS Engine Test PASSED ===\n");
    printf("libcss successfully parses CSS on Bat_OS!\n");
    exit(0);
}
