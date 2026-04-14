# NetSurf Port — Status & Notes

## Source
- Cloned from git://git.netsurf-browser.org/netsurf.git
- Main repo: 407,438 lines of C
- Framebuffer frontend: 6,832 lines

## Dependencies Required
1. **libcss** — CSS parser + selector engine (~50K lines)
2. **libdom** — W3C DOM implementation (~30K lines)
3. **libhubbub** — HTML5 parser (~15K lines)
4. **libparserutils** — Parsing utilities (~5K lines)
5. **libwapcaplet** — String interning (~2K lines)
6. **libnsfb** — Framebuffer abstraction (~10K lines)
7. **libnsutils** — Utilities (~2K lines)
8. **libnsgif** — GIF decoder (~5K lines)
9. **libnsbmp** — BMP decoder (~3K lines)
10. **curl/fetch** — HTTP client (can use our TCP/TLS stack)
11. **FreeType** — Font rendering (can use our TrueType renderer)

Total external deps: ~120K lines additional C code

## What We Can Replace
- curl → Our TCP/IP + TLS 1.3 stack (already works)
- FreeType → Our TrueType rasterizer (989 lines Rust)
- libnsfb → Direct framebuffer via our VirtIO GPU driver

## What We Need to Implement
1. Full C standard library (beyond minilib.h)
   - snprintf, sscanf, qsort, bsearch
   - realloc, strdup, strndup
   - fopen/fclose/fread/fwrite (file I/O)
   - ctype.h (isalpha, isdigit, etc.)
2. POSIX functions
   - stat, opendir/readdir
   - gettimeofday

## Alternative: Keep Improving BatBrowser
Our own browser already renders Wikipedia with:
- 1,918 DOM nodes
- TrueType fonts
- 108KB gzip decompression
- HTTPS + TLS 1.3
- Link navigation

Adding CSS2.1 to our engine might be faster than porting NetSurf.
Focus areas for our engine:
1. CSS property parsing from <style> tags
2. Float layout (left/right)
3. Table layout
4. Inline formatting context
5. CSS selectors (class, id, tag, descendant)
