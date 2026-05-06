# Ladybird Port — Autopilot State

**This file is the single source of truth for the autonomous loop.**

Read top-to-bottom every iter. Update the "Current iter" + "Last 5 outcomes"
sections after each commit. Flag NEEDS HUMAN sparingly — only for hardware,
secrets, or genuinely destructive ops.

---

## The Rules (autopilot reads this every fire)

**Session continuity:** The autopilot pins every fire to a single
`--session-id` UUID, so context accumulates. You SHOULD remember your own
prior fixes from earlier iters in this session. If you don't (e.g. after
auto-compaction), re-read this doc + recent commits. The doc is the ground
truth either way.

1. **Default: act, don't ask.** Read this file, do the next concrete step,
   commit + push, update this file. Don't editorialize about progress, don't
   suggest stopping, don't ask if anyone wants to keep going.

2. **When you'd normally ask the user a question, ask GPT instead via
   `mcp__gpt__ask-gpt`.** Log the question + answer to "GPT consultations"
   below. Act on the answer. Skip questions you can answer yourself by
   reading the codebase or recalling from this same session.

3. **NEEDS HUMAN flag.** Write a line starting with `> NEEDS HUMAN:`
   (markdown blockquote, this exact prefix) at the top of "Current iter"
   and exit cleanly **only** for:
   - Hardware Kaden physically controls (M4 boot, USB, signing keys)
   - Secrets / credentials
   - Destructive operations: `git push --force` to main, `rm -rf` outside
     `/tmp`, history rewrites, anything irreversible
   - True ambiguity that GPT also can't answer (rare)

4. **Commit discipline.** One logical change per commit. Build + smoke must
   pass before push. If a smoke regresses, revert the offending commit and
   document why in this file. Never push broken code to silence the loop.

5. **Cap blast radius.** If you've made 5 consecutive failed commits without
   progress, stop and write NEEDS HUMAN with a summary. Don't grind into
   the ground.

6. **No editorializing.** Drop "we made huge progress", "want to call it",
   "get some sleep". Just report facts.

7. **Build environment.** `BAT_OS_ALLOW_UNSIGNED_INITRD=1
   BAT_OS_PASSPHRASE=batman cargo build --release --features gicv3` is the
   standard kernel build. `bash tools/bake_ladybird_initrd.sh
   ports/ladybird_port/out` rebakes the initrd. Smoke is
   `python3 scripts/qemu_ladybird_js.py` (passes when stdout has `\r\n2\r\n`).

8. **Time budget.** If a single docker build exceeds 30 min, abort and
   investigate. If a smoke run exceeds 5 min wall clock, something's wrong.

---

## Current iter

**Iter 25** — FontPlugin needs fonts on a scannable path; fontconfig +
getdents64 must return entries for `/usr/share/fonts/*.ttf`.

**Concrete next step:** Debug why fontconfig's getdents64 on `/usr/share/fonts`
returns zero entries even though 14 TTF files are registered in the VFS.
Likely causes:
1. `sys_getdents64` doesn't iterate child nodes correctly for dirs created
   after `populate_rootfs` (ordering issue — fonts are added in
   `populate_lib_from_archive` which runs after dir creation).
2. `O_DIRECTORY` openat returns success but the fd isn't associated with a
   VFS directory node (fd table entry doesn't store the dir index).
3. Fontconfig reads `/etc/fonts/fonts.conf` first — if absent, it may skip
   scanning even if the dirs exist.

Debug by adding a trace inside `sys_getdents64` for the font dir fd, or by
creating a minimal `/etc/fonts/fonts.conf` in VFS that points to
`/usr/share/fonts`.

**Files likely to touch:**
- `src/batcave/linux/syscall.rs` (getdents64 debug / fix)
- `src/batcave/linux/vfs.rs` (maybe `/etc/fonts/fonts.conf`)

**Success criteria:**
- FontPlugin constructor passes (no VERIFY crash).
- Step `[3/5]` TraversableNavigable starts (may hit new wall inside it).

**On failure:** If getdents64 is fundamentally broken for dynamic dirs, add
a hacky `/etc/fonts/fonts.conf` that hardcodes the path and see if
fontconfig picks it up.

---

## Iter sequence (rough plan, revise as you learn)

| Iter | Goal | Risk |
|---|---|---|
| 24 | HeadlessPageClient + real Page wired into realm.host_defined ✓ | CMake needed LibGC LibGfx LibIPC |
| 25 | FontPlugin needs fonts: fontconfig + getdents64 + VFS | fonts present but not scanned |
| 26 | Document ctor's `parse_css_stylesheet("")` for view transitions | CSS parser may need more setup |
| 27 | HTMLParser::create + parser->run() on real Document | Parser may want EventLoop tasks |
| 28 | Walk Document tree, dump real Element + Text nodes | Likely just works once 27 works |
| 29-30 | LayoutTreeBuilder + compute_layout | Block/inline boxes; needs fonts |
| 31-35 | Painting → Skia surface | Skia mmap-heavy; expect new kernel issues |
| 36+ | Copy Skia BGRA → /batos/fb0 → see pixels in QEMU window | Use existing chromium_blit |

---

## Last 5 outcomes

| Iter | Date | Result | Commit |
|---|---|---|---|
| 24 | 2026-05-05 | HeadlessPageClient + Page + EventLoopPlugin + FontPlugin ✓ build; VFS /usr/share/fonts 14 TTFs; FontPlugin VERIFY crash at fontconfig scan (getdents64 returns 0 entries). | e18dfa45 |
| 23 | 2026-05-05 | VM+Realm bootstrap ✓; Document SIGSEGVs in `principal_host_defined_page` (0x99e9_…). Need real Page. | 57e172e4 |
| 22 | 2026-05-05 | Tree-style HTML token dump (extends iter 21). | 3a7506ed |
| 21 | 2026-05-05 | dump-html-tokens binary built; HTMLTokenizer prints DOCTYPE/StartTag/Char/EndTag. | 53aafca7 |
| 20 | 2026-05-05 | L2 BLOCK fix in is_user_writable; /bin/js exits cleanly. console.log(1+1) → 2. | ae0f303c |
| 19 | 2026-05-05 | anon MAP_FIXED force-overwrites L3 entries (BSS no longer reads file content). | de6c9dfb |

---

## GPT consultations

### 2026-05-05 iter-24 plan: how to bring up minimal Page+PageClient

**Q:** [iter 23 SEGV in Document::create_for_fragment_parsing reading
principal_host_defined_page(realm). Document needs Page+PageClient. What's
the most surgical way to bring up a minimal Page just for headless DOM
construction?]

**A (GPT-5.4):** Build a real `Page` with a dummy `PageClient`, set
`realm.host_defined()` to `PrincipalHostDefined { page }` before
`Document::create_for_fragment_parsing`. Reference `Tests/LibWeb`, not
WebContent (too heavy). Cheap defaults for PageClient virtuals:

- `palette()` → default-constructed palette
- `screen_rect()` → {0,0,800,600}
- `device_pixels_per_css_pixel()` → 1.0f
- `zoom_level()` → 1.0f
- `preferred_color_scheme()` → Light
- `preferred_contrast()` / `preferred_motion()` → NoPreference
- callbacks (`page_did_*`, `did_request_*`) → no-op
- `prompt/alert/confirm` → empty/false
- file pickers → failure

Be careful with methods returning references — back them with stored members,
don't return temporaries. Make sure PageClient outlives Page (probably both
heap-allocated as JS::Cell anyway since PageClient inherits JS::Cell).

No simpler entry point exists; have to satisfy `principal_host_defined_page`.

### 2026-05-05 iter-24: exact Page/PageClient API for headless DOM

**Q:** [What's the exact C++ interface for PageClient, Page::create, and
PrincipalHostDefined in current Ladybird main (May 2026)?]

**A (GPT-5.4):** Don't fabricate signatures — inspect the actual headers in
the container. Key findings after inspection:
- `PageClient` is a `JS::Cell` (line 367 of Page.h), needs
  `GC_DECLARE_ALLOCATOR` + `GC_DEFINE_ALLOCATOR`.
- `Page::create(JS::VM&, GC::Ref<PageClient>)` — takes a GC ref, not raw ptr.
- `PrincipalHostDefined` is a struct needing `(ESO, Intrinsics, Page)` — NOT
  constructible standalone. Use `TraversableNavigable::create_a_new_top_level_traversable`
  instead, which bootstraps the whole chain.
- SVGDecodedImageData::SVGPageClient is the reference implementation.
- `EventLoopPlugin::install` and `FontPlugin::install` must be called before
  TraversableNavigable creation.
- CMakeLists.txt `lagom_utility` needs `LibGC LibGfx LibIPC` added.

---

## Open kernel-side issues (background, no action required unless they fire)

- IRQ delivery under HVF: never observed timer IRQ during 120s smoke runs.
  Cooperative-yield-every-64-syscalls covers most cases. If pure-userland
  hang reappears, this is the suspect.
- `tcgetattr` was patched in libc.so.6 (immediate-ret) at iter 18 — root
  cause never identified. Acceptable workaround.
- `__stack_chk_fail` patched (immediate-ret) at iter 13 — same.
- `tcgetpgrp` patched (immediate-ret) at iter 14 — same.

These three libc patches live in `ports/ladybird_port/out/lib/libc.so.6`.
A pristine glibc would re-introduce iter-13/14/18 walls.
