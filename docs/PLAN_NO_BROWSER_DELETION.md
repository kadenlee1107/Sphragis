# No-Browser Hard-Delete Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove ~30K LOC + ~350 MB of build artifacts from Bat_OS — the native browser engine, Ladybird port, stream-client, and all supporting scripts/tools/ports — while keeping the kernel buildable and bootable at every step.

**Architecture:** Top-down deletion. Remove user-facing entry points first (shell commands, WM app integration), then orphaned engine code, then the kernel-side display path (`ChromiumFb` VFS + `chromium_blit` kthread), then ports/scripts/tools, then dependency cleanup. Each phase ends with `cargo check` and a commit so the tree stays buildable.

**Tech Stack:** Rust + Cargo, bare-metal aarch64 target (`aarch64-unknown-none`), nightly toolchain. The repo is a single-binary kernel; no library crate. Target build: `cargo build --release --target aarch64-unknown-none`.

**Reference spec:** `DESIGN_NO_BROWSER.md` (root). Read it before starting — it explains the *why*. This document is the *how*.

**Pre-deletion HEAD:** `2af9cdd4` on branch `port/ladybird` — verify this before tagging.

---

## Phase 0: Safety net

### Task 0.1: Tag the pre-deletion commit

**Files:** none (git operation only).

- [ ] **Step 1: Verify HEAD matches the spec**

Run: `git log -1 --format='%H %s'`
Expected: `2af9cdd4... 🎯 DESIGN_NO_BROWSER: Bat_OS ships without a browser` (or a later commit if other journal/doc work has landed since — that's fine, just confirm the strategy doc is in HEAD or its ancestry).

- [ ] **Step 2: Create the rescue tag**

Run: `git tag -a pre-no-browser-2026-05-07 -m "Last commit before the hard-delete of native browser, Ladybird port, and stream-client. See DESIGN_NO_BROWSER.md for rationale."`

- [ ] **Step 3: Push the tag**

Run: `git push origin pre-no-browser-2026-05-07`
Expected: `* [new tag] pre-no-browser-2026-05-07 -> pre-no-browser-2026-05-07`

### Task 0.2: Establish baseline — kernel must build now

**Files:** none (verification only).

- [ ] **Step 1: Confirm cargo check passes pre-deletion**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...` (warnings OK, errors not OK). If this fails, STOP — the tree wasn't clean before deletion started, and "after deletion it builds" loses meaning.

- [ ] **Step 2: Note any pre-existing warnings**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | grep "^warning:" | wc -l`
Record the number. After each deletion phase the warning count should not rise — it should fall as dead code is removed.

---

## Phase 1: Discard uncommitted stream-client iter 1 work in surviving files

The 1,613-LOC in-flight diff spans 11 files. Most files are deleted entirely by later phases — those need no special handling. Three files survive (`keyboard.rs`, `tablet.rs`, `tcp.rs`) and have small iter-1 additions that must be reverted.

### Task 1.1: Identify which uncommitted-changed files survive deletion

**Files:** none (analysis only).

- [ ] **Step 1: List the 11 in-flight files and their dispositions**

Run: `git status --short -- 'src/' 'scripts/' | grep -v captures/`

The dispositions:

| File | Survives? | Action |
|---|---|---|
| `scripts/browser_proxy.py` | ❌ Phase 7 deletes | No revert needed |
| `scripts/qemu_ladybird_window.py` | ❌ Phase 7 deletes | No revert needed |
| `src/batcave/linux/vfs.rs` | ✅ | Revert iter-1 lines (Phase 5 also touches this) |
| `src/drivers/virtio/gpu.rs` | ✅ | Revert iter-1 lines (Phase 5 also touches this) |
| `src/drivers/virtio/keyboard.rs` | ✅ | Revert all 10 in-flight lines |
| `src/drivers/virtio/tablet.rs` | ✅ | Revert all 33 in-flight lines |
| `src/net/tcp.rs` | ✅ | Inspect 2-line change; revert if browser-related |
| `src/ui/apps/browser.rs` | ❌ Phase 3 deletes | No revert needed |
| `src/ui/desktop.rs` | ✅ | Revert iter-1 lines (Phase 3 also touches this) |
| `src/ui/shell.rs` | ✅ | Revert iter-1 lines (Phase 2 also touches this) |
| `src/ui/wm.rs` | ✅ | Revert iter-1 lines (Phase 3 also touches this) |

### Task 1.2: Hard-revert the surviving files to HEAD

The simplest correct approach: revert all surviving-and-modified files to HEAD, since later deletion phases will remove the iter-1 work alongside the main browser deletion. Anything that *isn't* iter-1-related on these files would need to be re-added later, but per spec inspection (no non-browser changes are in this in-flight diff), full revert is correct.

- [ ] **Step 1: Inspect tcp.rs change to confirm it's browser-related**

Run: `git diff HEAD -- src/net/tcp.rs`
Expected: a 2-line change related to TCP for the stream-client `cmd_web` POST path. If the change is NOT browser-related (e.g. an unrelated bug fix), STOP and ask for human review — don't blindly revert non-browser work.

- [ ] **Step 2: Revert each surviving file**

Run:
```bash
git checkout HEAD -- \
  src/batcave/linux/vfs.rs \
  src/drivers/virtio/gpu.rs \
  src/drivers/virtio/keyboard.rs \
  src/drivers/virtio/tablet.rs \
  src/net/tcp.rs \
  src/ui/desktop.rs \
  src/ui/shell.rs \
  src/ui/wm.rs
```

- [ ] **Step 3: Verify the working tree is now clean for those files**

Run: `git status --short -- src/ scripts/ | grep -v captures/`
Expected: only the four files marked "❌ Phase X deletes" still show as modified. Surviving files should NOT appear in the output.

- [ ] **Step 4: Verify build still passes**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`. If this fails, the in-flight work had compensating changes that we just split apart — STOP and review.

- [ ] **Step 5: Commit (no code changes, but document the discard)**

The reverts mutate working tree only — nothing to commit until later phases. Skip commit here; the iter-1 discard is recorded in the SESSION_JOURNAL entry written in Phase 10.

---

## Phase 2: Remove shell-command dispatch and `cmd_*` functions

### Task 2.1: Remove the dispatch-table entries

**Files:**
- Modify: `src/ui/shell.rs:139` and `src/ui/shell.rs:156-160`

- [ ] **Step 1: Remove the browser-related match arms**

In `src/ui/shell.rs`, delete these lines (line numbers approximate — search for the strings):

```rust
        "chromium" | "chrome" => cmd_chromium(parts[1], parts[2], parts[3]),
```

```rust
        "web"                   => cmd_web(parts[1]),
        "webwin"                => cmd_webwin(parts[1]),
        "ladybird"              => cmd_ladybird(parts[1], parts[2], parts[3]),
        "dump-dom" | "dom" => cmd_dump_dom(parts[1]),
        "render" => cmd_render(parts[1], &parts),
```

Also search for and remove dispatch entries for: `chromium-version`, `ladybird-js`, `ladybird-dump`. They may share the dispatch block above or be separate — grep for each.

- [ ] **Step 2: Verify no browser-command dispatch remains**

Run:
```bash
grep -nE '"(web|webwin|ladybird|chromium|chrome|dump-dom|dom|render|chromium-version|ladybird-js|ladybird-dump)"' src/ui/shell.rs
```
Expected: no output (or only matches inside `cmd_help()` text, which Task 2.3 cleans up).

### Task 2.2: Remove the `cmd_*` function definitions

**Files:**
- Modify: `src/ui/shell.rs` — delete these functions:
  - `fn cmd_chromium_version()` (~line 3067)
  - `fn cmd_ladybird_js(expr_in: &str)` (~line 3141)
  - `fn cmd_webwin(url: &str)` (~line 3192)
  - `fn cmd_web(url: &str)` (~line 3208)
  - `fn cmd_ladybird_dump(html_in: &str)` (~line 3819)
  - `fn cmd_ladybird(a1: &str, a2: &str, a3: &str)` (~line 3854)
  - `fn cmd_chromium(a1: &str, a2: &str, a3: &str)` (~line 3910)
  - `fn cmd_dump_dom(url: &str)` (~line 4083)
  - `fn cmd_render(url: &str, parts: &[&str; MAX_PARTS])` (~line 4197)

- [ ] **Step 1: Find and delete each function**

For each function in the list, use Read to confirm its start line, find its closing `}`, and delete the whole block including the function-doc comment if present.

The functions may import items from `crate::browser::*`, `crate::net::http::fetch`, `crate::drivers::display::chromium_blit`, etc. Those imports go away too — if an import becomes unused after deletion, the next `cargo check` will warn.

- [ ] **Step 2: Verify no cmd_browser_* functions remain**

Run:
```bash
grep -nE '^fn (cmd_web|cmd_webwin|cmd_ladybird|cmd_chromium|cmd_render|cmd_dump_dom)' src/ui/shell.rs
```
Expected: no output.

### Task 2.3: Update help text + verify build

**Files:**
- Modify: `src/ui/shell.rs` — `fn cmd_help()` at ~line 346.

- [ ] **Step 1: Read the current help text**

Run: `grep -nA 80 'fn cmd_help' src/ui/shell.rs | head -100`

- [ ] **Step 2: Remove any help-text lines mentioning the deleted commands**

Search for any of: `web`, `webwin`, `ladybird`, `chromium`, `render`, `dump-dom`, `chrome` inside the help text and remove their lines. Leave non-browser help intact.

- [ ] **Step 3: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -10`
Expected: `Finished ...`. If errors mention `unresolved import crate::browser::*` or `cannot find function`, those are from `cmd_*` functions or arms not yet removed — go back and clean them up.

- [ ] **Step 4: Commit**

Run:
```bash
git add src/ui/shell.rs
git commit -m "$(cat <<'EOF'
🎯 no-browser: remove shell command dispatch + cmd_* fns

Deletes web, webwin, ladybird, ladybird-js, ladybird-dump,
chromium, chromium-version, render, dump-dom from src/ui/shell.rs.
Per DESIGN_NO_BROWSER.md, Bat_OS no longer has a browser.

Phase 2 of the no-browser deletion plan. Browser app + native
engine + ports + scripts removed in subsequent commits.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 3: Tear down the WM Browser app integration

### Task 3.1: Remove `apps::browser::*` references in `src/ui/desktop.rs`

**Files:**
- Modify: `src/ui/desktop.rs:153, 189, 265-266, 271, 275, 316, 320, 330-333, 347, 350, 358, 408`

- [ ] **Step 1: Read the affected blocks**

Run: `grep -nE 'browser|APP_BROWSER' src/ui/desktop.rs`

The references include: Ctrl+G shortcut, `apps::browser::on_activate()`, `is_remote_mode()` / `set_remote_mode()`, `forward_key_to_proxy()`, `handle_key()`, `tick_remote()`, `repaint_remote_only()`, `remote_seq()`, `wm::APP_BROWSER` render dispatch.

- [ ] **Step 2: Delete each browser block**

For each match, remove the block (key handler arm, heartbeat block, dispatch arm, etc.). Don't leave dangling `else` or empty `match` arms — restructure as needed.

The Ctrl+G shortcut: there may be a lingering comment "Ctrl+G — jump straight to the browser tab" that becomes stale. Remove the comment and the arm.

- [ ] **Step 3: Verify**

Run: `grep -nE 'browser|APP_BROWSER' src/ui/desktop.rs`
Expected: no output.

### Task 3.2: Remove `APP_BROWSER` constant + browser logic in `src/ui/wm.rs`

**Files:**
- Modify: `src/ui/wm.rs:289` (comment), `src/ui/wm.rs:532` (`crate::browser::js::is_enabled()` call), and the `APP_BROWSER` constant + any switch/match using it.

- [ ] **Step 1: Find APP_BROWSER definition**

Run: `grep -nE 'APP_BROWSER|browser' src/ui/wm.rs`

- [ ] **Step 2: Remove the constant and any indexing logic**

Delete the `pub const APP_BROWSER: usize = N;` line. If it shifts other app indices, fix them — re-number the remaining apps to fill the gap (`APP_TERM=0, APP_DASH=1, APP_FILE=2, APP_NET=3, APP_EDIT=4, APP_SEC=5, APP_CHAT=6, APP_CAVE=7` or similar — confirm by reading the file).

- [ ] **Step 3: Remove the `crate::browser::js::is_enabled()` call site**

The comment near line 289 ("Used by the browser app's on_activate so any accidental splits...") and the call at line 532 both go. If `js_on` is used downstream for non-browser purposes, keep that downstream and replace the source with a constant `false`. If it's only used for the browser pill, remove the entire surrounding block.

- [ ] **Step 4: Verify**

Run: `grep -nE 'browser|APP_BROWSER' src/ui/wm.rs`
Expected: no output.

### Task 3.3: Delete `src/ui/apps/browser.rs`

**Files:**
- Delete: `src/ui/apps/browser.rs`

- [ ] **Step 1: Delete the file**

Run: `git rm src/ui/apps/browser.rs`

- [ ] **Step 2: Remove its module declaration**

In `src/ui/apps/mod.rs:2`, delete the line `pub mod browser;`.

### Task 3.4: Build + commit Phase 3

- [ ] **Step 1: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -10`
Expected: `Finished ...`. Errors will likely mention `cannot find module browser` or `cannot find function tick_remote` — check that all four desktop/wm/apps-mod/browser.rs edits landed.

- [ ] **Step 2: Commit**

Run:
```bash
git add src/ui/apps/ src/ui/wm.rs src/ui/desktop.rs
git commit -m "$(cat <<'EOF'
🎯 no-browser: rip out WM Browser app

Removes:
- src/ui/apps/browser.rs (~2.4 KLOC, native + remote-mode renderer)
- pub mod browser in src/ui/apps/mod.rs
- APP_BROWSER constant + render dispatch in src/ui/wm.rs
- crate::browser::js::is_enabled() call site
- Ctrl+G browser shortcut, browser app heartbeat, key
  forwarding, tick_remote/repaint_remote_only, remote-frame poll
  loop in src/ui/desktop.rs

Phase 3 of the no-browser deletion plan.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 4: Delete the native engine `src/browser/`

### Task 4.1: Remove module declaration in `src/main.rs`

**Files:**
- Modify: `src/main.rs:8` (the line `mod browser;`)

- [ ] **Step 1: Find and delete the line**

Run: `grep -n '^mod browser' src/main.rs`
Expected: `8:mod browser;`

Delete that line.

### Task 4.2: Find any remaining `crate::browser::` references

**Files:**
- Modify: any files that import `crate::browser::*` and were missed in earlier phases.

- [ ] **Step 1: Grep for remaining usages**

Run: `grep -rn 'crate::browser::\|use browser::' src/ --include='*.rs'`
Expected: no output. If any remain, they're imports in files that survive (other apps, drivers, etc.) — remove the import line and any code path that uses it.

### Task 4.3: Delete the directory

**Files:**
- Delete: `src/browser/` (entire directory, ~16 KLOC across html/, css/, js/, layout/, paint/, media/, dom.rs, mod.rs)

- [ ] **Step 1: Delete the directory**

Run: `git rm -r src/browser/`

### Task 4.4: Build + commit Phase 4

- [ ] **Step 1: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -10`
Expected: `Finished ...`. The native engine is now gone.

- [ ] **Step 2: Commit**

Run:
```bash
git add src/browser src/main.rs
git commit -m "$(cat <<'EOF'
🎯 no-browser: delete native BatBrowser engine

Removes src/browser/ (~16K LOC of pure-Rust HTML/CSS/JS/layout/
paint/media engine) and its module declaration in src/main.rs.

Per DESIGN_NO_BROWSER.md, the engineering pride is real but
"every byte auditable" + "Chrome-equivalent in pure Rust" is
1-2 years of effort to chase a wrong success metric for a
security-first OS. Reversibility: tag pre-no-browser-2026-05-07
preserves the full engine for git revival.

Phase 4 of the no-browser deletion plan.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 5: Remove `ChromiumFb` VFS node + `chromium_blit` kthread

### Task 5.1: Delete the chromium_blit driver

**Files:**
- Delete: `src/drivers/display/chromium_blit.rs`
- Modify: `src/drivers/display/mod.rs:8` (remove `pub mod chromium_blit;`)

- [ ] **Step 1: Read the driver**

Run: `wc -l src/drivers/display/chromium_blit.rs && head -30 src/drivers/display/chromium_blit.rs`

- [ ] **Step 2: Delete and remove module declaration**

Run:
```bash
git rm src/drivers/display/chromium_blit.rs
```

In `src/drivers/display/mod.rs`, delete `pub mod chromium_blit;` (line 8). The comment at line 4 ("Re-exports display-related bridges. Today: chromium_blit (the /batos/fb0 ...)") goes too — clean up the comment block.

### Task 5.2: Remove the kthread start call in `src/main.rs`

**Files:**
- Modify: `src/main.rs:338`

- [ ] **Step 1: Find and delete**

Run: `grep -n 'chromium_blit' src/main.rs`
Expected: `338:            drivers::display::chromium_blit::start();`

Delete that line.

### Task 5.3: Remove the per-cave-switch reset call in `src/batcave/cave.rs`

**Files:**
- Modify: `src/batcave/cave.rs:1182`

- [ ] **Step 1: Find and delete**

Run: `grep -n 'chromium_blit' src/batcave/cave.rs`
Expected: `1182:    crate::drivers::display::chromium_blit::reset_for_cave_switch();`

Delete that line.

### Task 5.4: Remove `ChromiumFb` from `NodeType` enum and all uses

**Files:**
- Modify: `src/batcave/linux/vfs.rs:26, 223-243, 1049-1180` (multiple sites — see grep)

- [ ] **Step 1: Grep all sites**

Run: `grep -n 'ChromiumFb\|chromium_blit\|FB_MAGIC' src/batcave/linux/vfs.rs`

The deletions span:
- The `ChromiumFb` variant in the `NodeType` enum (~line 26).
- The skip-handling for `ChromiumFb` nodes in directory walks (~lines 223-243).
- `pub const FB_MAGIC` (~line 1057) — keep if used elsewhere; delete if only used by the now-deleted blit code.
- `is_chromium_fb_node` helper (~line 1094).
- `allocate_chromium_fb` / `register_chromium_fb` setup (~lines 1102-1180).

- [ ] **Step 2: Delete each block**

Inside the `NodeType` enum: remove the `ChromiumFb,` variant. Any `match node.node_type` exhaustive matches downstream will fail — fix them by removing the `ChromiumFb` arm.

Inside the directory-walk skip logic: remove the `&& n.node_type != NodeType::ChromiumFb` clauses.

Delete the `FB_MAGIC` const, `is_chromium_fb_node()` fn, and the `allocate_chromium_fb` / `register_chromium_fb` functions wholesale (they're only used by browser-FB plumbing).

- [ ] **Step 3: Verify no remaining references in vfs.rs**

Run: `grep -nE 'ChromiumFb|chromium_blit|FB_MAGIC|is_chromium_fb' src/batcave/linux/vfs.rs`
Expected: no output.

### Task 5.5: Remove `ChromiumFb` branches in `src/batcave/linux/syscall.rs`

**Files:**
- Modify: `src/batcave/linux/syscall.rs:1787-1873, 2139-2141, 2878-2898`

- [ ] **Step 1: Grep all sites**

Run: `grep -n 'ChromiumFb\|chromium_blit' src/batcave/linux/syscall.rs`

Three syscall handlers touch this:
- `sys_write` (~lines 1787-1873): the iter-28 sync-blit-on-write path. Delete the entire `if node.node_type == vfs::NodeType::ChromiumFb { ... }` block including the call to `chromium_blit::tick(fb_base)`.
- `sys_read` (~lines 2139-2141): a `ChromiumFb` short-circuit. Delete that `if` block.
- `sys_mmap` (~lines 2878-2898): the `MAP_SHARED` of the pre-allocated FB region for browser cases. Delete that `if` block.

- [ ] **Step 2: Delete each block**

For each `if node.node_type == vfs::NodeType::ChromiumFb { ... }` block, delete from `if` through the matching closing `}`. Be careful with the surrounding `else if` / `else` — it may need restructuring.

- [ ] **Step 3: Verify**

Run: `grep -nE 'ChromiumFb|chromium_blit' src/batcave/linux/syscall.rs`
Expected: no output.

### Task 5.6: Remove the `/batos/fb0` node setup at boot

**Files:**
- Modify: wherever `allocate_chromium_fb` was called from — likely `src/main.rs` or `src/batcave/init.rs`. Grep to find it.

- [ ] **Step 1: Find the call site**

Run: `grep -rn 'allocate_chromium_fb\|register_chromium_fb' src/ --include='*.rs'`
Expected: only matches inside `src/batcave/linux/vfs.rs` (the now-deleted definitions). If there's still a call site, delete it.

### Task 5.7: Build + commit Phase 5

- [ ] **Step 1: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -10`
Expected: `Finished ...`. The whole `/batos/fb0` framebuffer-write path is gone.

- [ ] **Step 2: Commit**

Run:
```bash
git add src/
git commit -m "$(cat <<'EOF'
🎯 no-browser: remove ChromiumFb VFS + chromium_blit kthread

Removes the kernel-side display-shared-memory plumbing that
existed only to ferry browser-rendered bitmaps to virtio-gpu:

- src/drivers/display/chromium_blit.rs (deleted)
- pub mod chromium_blit in src/drivers/display/mod.rs (removed)
- chromium_blit::start() boot call in src/main.rs (removed)
- chromium_blit::reset_for_cave_switch in src/batcave/cave.rs (removed)
- NodeType::ChromiumFb variant in src/batcave/linux/vfs.rs (removed)
- ChromiumFb branches in sys_write/sys_read/sys_mmap (removed)
- /batos/fb0 boot-time allocation (removed)

WM still uses virtio-gpu directly; no display regression.

Phase 5 of the no-browser deletion plan.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 6: Delete `ports/`

### Task 6.1: Delete browser-port directories

**Files:**
- Delete: `ports/ladybird_port/`, `ports/chromium_port/`, `ports/chromium/`, `ports/netsurf/`

- [ ] **Step 1: Verify the gitignored dirs are local-only**

Run: `git ls-files ports/chromium ports/chromium_port/out 2>&1 | head -5`
Expected: empty (these are gitignored). If non-empty, the gitignore wasn't comprehensive — investigate before deleting.

- [ ] **Step 2: Delete the tracked port directories**

Run:
```bash
git rm -rf ports/ladybird_port ports/chromium_port ports/netsurf
```

- [ ] **Step 3: Delete the gitignored local directories**

Run:
```bash
rm -rf ports/chromium ports/chromium_port/out
```

### Task 6.2: Delete NetSurf component libs + vendored deps

**Files:**
- Delete: `ports/libcss/`, `ports/libdom/`, `ports/libhubbub/`, `ports/libnsfb/`, `ports/libparserutils/`, `ports/libwapcaplet/`, `ports/libnsutils/`, `ports/freetype-2.13.3/`, `ports/libpng-1.6.43/`, `ports/zlib-1.3.1/`, `ports/skia/`, `ports/v8/`

- [ ] **Step 1: Delete**

Run:
```bash
git rm -rf \
  ports/libcss ports/libdom ports/libhubbub ports/libnsfb \
  ports/libparserutils ports/libwapcaplet ports/libnsutils \
  ports/freetype-2.13.3 ports/libpng-1.6.43 ports/zlib-1.3.1 \
  ports/skia ports/v8
```

### Task 6.3: Delete top-of-`ports/` test stubs and archives

**Files:**
- Delete: every loose file in `ports/` that's a browser port artifact.

- [ ] **Step 1: List candidates**

Run: `ls ports/`

The deletion list (verify each before removing):
- `blink_bridge.cpp`, `blink_printf.c`, `blink_stubs.cpp`, `blink_test.cpp`, `blink_tokenizer_test.cpp`
- `css_bridge.cpp`, `css_stubs.c`, `css_test_stubs.cpp`, `css_tokenizer_test.cpp`
- `cxx_test.cpp`, `cxxrt.c`
- `display_test.cpp`
- `freetype_test.c`
- `libblink.a`, `libblink.a.bak`, `libblink_full.a`
- `libc.c`, `libc.o`
- `math_asm.S`, `math_impl.c`, `mmap_test.c`
- `netsurf_css_test.c`, `netsurf_html_css_test.c`, `netsurf_main.c`, `netsurf_render_test.c`
- `png_test.c`, `posix_test.c`
- `skia_render_test.cpp`, `skia_stubs.cpp`, `skia_test.cpp`
- `v8_exec.cpp`, `v8_test.cpp`
- `build_skia.sh`
- `PORTING_NOTES.md` (was a NetSurf-port reference; now stale)

- [ ] **Step 2: Delete**

Run:
```bash
git rm \
  ports/blink_bridge.cpp ports/blink_printf.c ports/blink_stubs.cpp \
  ports/blink_test.cpp ports/blink_tokenizer_test.cpp \
  ports/css_bridge.cpp ports/css_stubs.c ports/css_test_stubs.cpp \
  ports/css_tokenizer_test.cpp \
  ports/cxx_test.cpp ports/cxxrt.c \
  ports/display_test.cpp \
  ports/freetype_test.c \
  ports/libblink.a ports/libblink.a.bak ports/libblink_full.a \
  ports/libc.c ports/libc.o \
  ports/math_asm.S ports/math_impl.c ports/mmap_test.c \
  ports/netsurf_css_test.c ports/netsurf_html_css_test.c \
  ports/netsurf_main.c ports/netsurf_render_test.c \
  ports/png_test.c ports/posix_test.c \
  ports/skia_render_test.cpp ports/skia_stubs.cpp ports/skia_test.cpp \
  ports/v8_exec.cpp ports/v8_test.cpp \
  ports/build_skia.sh \
  ports/PORTING_NOTES.md
```

- [ ] **Step 3: Verify ports/ is empty (or only contains files we deliberately keep)**

Run: `ls ports/ 2>&1`
Expected: empty, or document any survivors.

If empty, remove the directory itself: `rmdir ports/` and stage with `git add -A`.

### Task 6.4: Update `.gitignore`

**Files:**
- Modify: `.gitignore`

- [ ] **Step 1: Read current state**

Run: `cat .gitignore`

- [ ] **Step 2: Remove now-stale entries**

Delete these blocks (search for them):

```
# Vendored Chromium source tree (~1.9 GB, contains a pack file over
# GitHub's 100 MB limit). Re-clone upstream if you need it.
/ports/chromium
```

```
# Docker volumes / build artifacts from the Chromium container work.
/chromium-build
```

```
# Chromium content_shell output — 293 MB PIE binary produced by
# ports/chromium_port/build.sh (in Docker) + manually copied out of
# the batos-chromium-src volume. Not source. Reproduce via:
#   docker run --rm -v batos-chromium-src:/src \
#       -v $(pwd)/ports/chromium_port/out:/out \
#       alpine cp /src/src/out/BatOs/content_shell /out/content_shell
/ports/chromium_port/out/
```

```
.autopilot-session-id
.autopilot-session-started
ports/ladybird_port/out/lib/liblagom-*.so*
```

The autopilot files can stay or go — they're tool state, not browser-specific. Leave them; the autopilot infrastructure survives per the spec.

### Task 6.5: Build + commit Phase 6

- [ ] **Step 1: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -10`
Expected: `Finished ...`. (Cargo doesn't depend on `ports/`, so this is just a sanity check.)

- [ ] **Step 2: Commit**

Run:
```bash
git add -A ports/ .gitignore
git commit -m "$(cat <<'EOF'
🎯 no-browser: delete ports/ (~350 MB)

Removes:
- ports/ladybird_port/ (Ladybird build scaffolding + Dockerfile +
  baked initrd)
- ports/chromium_port/ (Chromium build artifacts including
  content_shell)
- ports/chromium/ (1.3 GB Chromium source — was gitignored, deleted
  the local copy)
- ports/netsurf/ + all NetSurf component libs (libcss, libdom,
  libhubbub, libnsfb, libparserutils, libwapcaplet, libnsutils):
  reference-only, never integrated.
- ports/freetype-2.13.3, ports/libpng-1.6.43, ports/zlib-1.3.1,
  ports/skia, ports/v8: vendored deps for the browser ports.
- ports/{blink,css,skia,v8}_*.cpp/.c/.a/.bak: test stubs and
  archives from the Blink/V8 porting attempts.
- ports/PORTING_NOTES.md: NetSurf-port reference doc, now stale.

.gitignore cleaned up to drop now-irrelevant entries.

Phase 6 of the no-browser deletion plan.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 7: Delete browser scripts and tools

### Task 7.1: Delete browser-related scripts

**Files:**
- Delete: `scripts/browser_proxy.py`, `scripts/qemu_ladybird_window.py`, `scripts/qemu_ladybird_js.py`, `scripts/qemu_chromium_pipeline_smoke.py`, `scripts/qemu_chromium_version.py`, plus any `render_*.py`, `dump_dom.py`, or `mouse_bridge.py` that exist (used to drive the deleted demos).

- [ ] **Step 1: Discover any other browser-related scripts**

Run: `ls scripts/ | grep -iE 'browser|chromium|ladybird|render|dump.dom|mouse_bridge'`

The known list is `browser_proxy.py`, `qemu_ladybird_window.py`, `qemu_ladybird_js.py`, `qemu_chromium_pipeline_smoke.py`, `qemu_chromium_version.py`. Any extras found by the grep that are obviously browser-only (e.g. `render_to_png.py`, `render_live.py`, `dump_dom.py`, `mouse_bridge.py`) get added to the deletion list.

- [ ] **Step 2: Delete**

Run (adjust the file list per Step 1):
```bash
git rm \
  scripts/browser_proxy.py \
  scripts/qemu_ladybird_window.py \
  scripts/qemu_ladybird_js.py \
  scripts/qemu_chromium_pipeline_smoke.py \
  scripts/qemu_chromium_version.py
```

Plus `git rm` any extras the grep found.

### Task 7.2: Delete browser-related tools

**Files:**
- Delete: `tools/bake_ladybird_initrd.sh`, `tools/bake_chromium_initrd.sh`, `tools/bake_chromium_archive.sh`, `tools/bake_chromium.sh`, `tools/audit_chromium_initmap.sh`, `tools/run_chromium.sh`

- [ ] **Step 1: Delete**

Run:
```bash
git rm \
  tools/bake_ladybird_initrd.sh \
  tools/bake_chromium_initrd.sh \
  tools/bake_chromium_archive.sh \
  tools/bake_chromium.sh \
  tools/audit_chromium_initmap.sh \
  tools/run_chromium.sh
```

### Task 7.3: Delete the autopilot Ladybird state file

**Files:**
- Delete: `docs/LADYBIRD_AUTOPILOT.md`

- [ ] **Step 1: Delete**

Run: `git rm docs/LADYBIRD_AUTOPILOT.md`

The autopilot infrastructure (`scripts/autopilot.sh`) survives — it's reusable for non-browser STUMPs.

### Task 7.4: Update the Makefile

**Files:**
- Modify: `Makefile` at repo root.

- [ ] **Step 1: Read the Makefile**

Run: `grep -nE '^[a-z-]+:' Makefile`

Likely browser-related targets (per the scripts sweep): `render`, `render-live`, `dom`, `smoke`, `initrd`. Each may invoke a now-deleted script.

- [ ] **Step 2: Remove browser-only targets**

For each target, decide:
- `render` → invokes `scripts/render_to_png.py` (browser-only). Remove the target.
- `render-live` → invokes `scripts/render_live.py` (browser-only). Remove.
- `dom` → invokes `scripts/dump_dom.py` (browser-only). Remove.
- `smoke` → invokes `scripts/qemu_chromium_pipeline_smoke.py` (browser-only). Remove.
- `initrd` → invokes `tools/bake_chromium_archive.sh`. Remove.

Keep targets that are not browser-specific (`build`, `clean`, `watch`, `info`).

- [ ] **Step 3: Verify the Makefile still parses**

Run: `make -n build 2>&1 | head -10`
Expected: shows the `cargo build` invocation, no errors about missing files.

### Task 7.5: Build + commit Phase 7

- [ ] **Step 1: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`.

- [ ] **Step 2: Commit**

Run:
```bash
git add -A scripts/ tools/ docs/ Makefile
git commit -m "$(cat <<'EOF'
🎯 no-browser: delete browser scripts, tools, autopilot state

Removes:
- scripts/browser_proxy.py: Mac-side stream-client Chromium proxy
- scripts/qemu_ladybird_window.py + qemu_ladybird_js.py
- scripts/qemu_chromium_pipeline_smoke.py + qemu_chromium_version.py
- tools/bake_ladybird_initrd.sh + bake_chromium_*.sh + run_chromium.sh
  + audit_chromium_initmap.sh
- docs/LADYBIRD_AUTOPILOT.md: state file for the now-defunct
  Ladybird-iter autopilot loop

Makefile: removed render, render-live, dom, smoke, initrd targets
that depended on the deleted scripts. Build/clean/watch/info stay.

Autopilot infrastructure (scripts/autopilot.sh) survives — reusable
for non-browser STUMP loops.

Phase 7 of the no-browser deletion plan.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 8: Clean up `Cargo.toml` dependencies

### Task 8.1: Find unused dependencies

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Get the current dep list**

Run: `grep -E '^[a-z][a-z0-9-]+ ?=' Cargo.toml`

- [ ] **Step 2: For each dep, grep src/ for its usage**

Run: `for dep in $(grep -E '^[a-z][a-z0-9-]+ ?=' Cargo.toml | sed 's/[ =].*//'); do echo "--- $dep ---"; grep -rn "use $(echo $dep | tr - _)\|::$(echo $dep | tr - _)::" src/ --include='*.rs' | head -2; done`

Deps that survive (verified used outside browser code):
- `aes`, `ghash`, `sha2`, `chacha20poly1305` — TLS + BatFS
- `ed25519-compact`, `p256`, `p384`, `rsa` — TLS + crypto subsystem
- `x509-cert`, `spki`, `der`, `const-oid` — TLS (even if not yet wired into the live path, they're staged)
- `ml-kem`, `ml-dsa` — TLS hybrid PQ
- `argon2` — KDF for BatFS / passphrases
- `linked_list_allocator` — kernel heap
- `rand_core`, `hybrid_array` — crypto primitives
- `x25519-dalek` — TLS

Deps to inspect (may have been browser-only):
- Any image crate (the native engine had Rust JPEG/PNG decoders in `src/browser/media/`; if those used a crate dep, it goes)
- Any HTML/CSS parser crate (unlikely — the native engine was hand-rolled per the sweep)

- [ ] **Step 3: Run `cargo +nightly udeps` if available**

Run: `cargo +nightly udeps --target aarch64-unknown-none 2>&1 | tail -20`
This auto-detects unused deps. If `cargo-udeps` isn't installed, skip this step — the manual grep covers the obvious cases.

### Task 8.2: Remove unused deps + commit

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: For each confirmed-unused dep, remove its line from `[dependencies]`**

Edit `Cargo.toml` and delete each unused dep's line. Be conservative — if a dep is unused now but plausibly part of the staged-but-not-wired security work (`x509-cert` etc.), leave it. The strategy doc explicitly preserves the TLS validators for future wiring.

- [ ] **Step 2: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`.

- [ ] **Step 3: Commit (if any changes)**

Run:
```bash
git add Cargo.toml Cargo.lock
git commit -m "🎯 no-browser: drop deps used only by deleted browser code

Phase 8 of the no-browser deletion plan.

Co-Authored-By: claude-flow <ruv@ruv.net>"
```

If no deps were removed, skip the commit.

---

## Phase 9: Mark superseded design docs

### Task 9.1: Add SUPERSEDED notice to `DESIGN_BROWSER.md`

**Files:**
- Modify: `DESIGN_BROWSER.md` (root) — prepend a notice at the top.

- [ ] **Step 1: Prepend the notice**

Edit `DESIGN_BROWSER.md` and add at the very top, before the existing first line:

```markdown
> # ⚠️ SUPERSEDED (2026-05-07)
>
> This document describes the native BatBrowser engine, which was
> deleted along with the Ladybird port and stream-client per
> `DESIGN_NO_BROWSER.md`. Bat_OS no longer ships a browser. The
> content below is preserved for historical context — it does NOT
> describe the current state of the OS.
>
> Current strategy: **`DESIGN_NO_BROWSER.md`**.
>
> ---

```

### Task 9.2: Add SUPERSEDED notice to `DESIGN_CHROMIUM.md`

**Files:**
- Modify: `DESIGN_CHROMIUM.md` (root) — replace the existing PARKED notice with a stronger SUPERSEDED notice.

- [ ] **Step 1: Replace the existing PARKED block**

The existing notice (added on 2026-05-07 earlier this session) says PARKED. Replace its first line `> # ⚠️ STATUS: PARKED (as of 2026-05-05)` with `> # ⚠️ SUPERSEDED (2026-05-07): Bat_OS no longer ships a browser.` Keep the rest of the historical content; just retitle.

### Task 9.3: Commit Phase 9

- [ ] **Step 1: Commit**

Run:
```bash
git add DESIGN_BROWSER.md DESIGN_CHROMIUM.md
git commit -m "🎯 no-browser: mark DESIGN_{BROWSER,CHROMIUM} superseded

Both docs preserved for historical context but now point to
DESIGN_NO_BROWSER.md as the active strategy.

Phase 9 of the no-browser deletion plan.

Co-Authored-By: claude-flow <ruv@ruv.net>"
```

---

## Phase 10: Final verification + journal entry + push

### Task 10.1: Full release build

- [ ] **Step 1: Run release build**

Run: `cargo build --release --target aarch64-unknown-none 2>&1 | tail -10`
Expected: `Finished ...`. If the kernel binary doesn't build at this point, something in the deletion was incomplete — go back, find the unresolved reference, fix.

- [ ] **Step 2: Confirm the kernel binary exists**

Run: `ls -lh target/aarch64-unknown-none/release/bat_os* 2>&1`
Expected: a binary with size > 0. Note its size for the journal.

### Task 10.2: Smoke test against a non-browser QEMU script

- [ ] **Step 1: List surviving qemu_*.py scripts**

Run: `ls scripts/qemu_*.py 2>&1`

Pick the simplest one that boots the kernel and exercises the shell. Good candidates (per the existing tooling): NAT-pipeline tests, `qemu_test.py` if present, or any script in `scripts/lib/qemu_boot.py`-driven launchers.

- [ ] **Step 2: If no boot-and-shell smoke survives, write one**

Create `scripts/qemu_boot_smoke.py` (~30 lines): launch QEMU with the kernel, send `help\n` over stdin/serial, confirm a non-empty response, exit. Use `scripts/lib/qemu_boot.py` if it provides boot helpers; otherwise inline a minimal `subprocess.Popen` of `qemu-system-aarch64` with `-machine virt -kernel target/aarch64-unknown-none/release/bat_os -nographic` and pexpect-style stdin handling.

- [ ] **Step 3: Run the smoke**

Run the chosen or newly-written script.
Expected: kernel boots within ~10s, shell prompt appears, `help` prints a non-empty response (one of the surviving commands like `whoami`, `uname`, `status`, etc.).

- [ ] **Step 4: If you wrote `qemu_boot_smoke.py`, commit it**

Run:
```bash
git add scripts/qemu_boot_smoke.py
git commit -m "🎯 no-browser: minimal boot-and-shell smoke for post-pivot

Replaces the deleted Ladybird/Chromium smoke scripts with a
single boot-to-help-prompt verification.

Co-Authored-By: claude-flow <ruv@ruv.net>"
```

### Task 10.3: Write SESSION_JOURNAL entry

**Files:**
- Modify: `docs/SESSION_JOURNAL.md` — prepend a new entry at the top (after the format header).

- [ ] **Step 1: Draft the entry**

The entry should cover:
- The pivot decision (Bat_OS = secure workstation, no browser).
- Total LOC removed (sum the per-phase deletion lines from `git log --oneline pre-no-browser-2026-05-07..HEAD --stat`).
- The 9 phases, one paragraph each.
- The 1,613-LOC stream-client iter 1 explicitly discarded (not committed).
- Tag `pre-no-browser-2026-05-07` for git revival if needed.
- What's next (post-pivot identity: crypto ops, secure files, comms, vetted workloads).

Match the existing journal voice (terse, technical, honest). ~150-200 lines.

- [ ] **Step 2: Insert at top of journal**

Insert between the format header (around line 12) and the previous newest entry.

- [ ] **Step 3: Commit**

Run:
```bash
git add docs/SESSION_JOURNAL.md
git commit -m "📝 SESSION_JOURNAL: no-browser pivot complete

Phase 10 of the no-browser deletion plan.

Co-Authored-By: claude-flow <ruv@ruv.net>"
```

### Task 10.4: Push everything

- [ ] **Step 1: Push the branch**

Run: `git push origin port/ladybird 2>&1 | tail -5`
Expected: `* * * port/ladybird -> port/ladybird` with the new commits.

- [ ] **Step 2: Push the rescue tag (if not already pushed in Phase 0)**

Run: `git push origin pre-no-browser-2026-05-07 2>&1 | tail -5`

- [ ] **Step 3: Verify on GitHub**

Run: `gh repo view --web 2>&1 | head -5` (or visit https://github.com/kadenlee1107/Bat_OS — confirm the latest commit message reflects the no-browser pivot).

---

## Done

After Phase 10:
- `port/ladybird` has ~10 commits documenting the per-phase deletion.
- Tag `pre-no-browser-2026-05-07` preserves the full pre-deletion state.
- `cargo build --release` produces a working kernel binary.
- A non-browser smoke test confirms boot + shell still work.
- Journal entry captures the decision and the deletion narrative.
- DESIGN_BROWSER.md and DESIGN_CHROMIUM.md are marked SUPERSEDED.

Bat_OS is now a secure workstation with no browser. Browsing happens
on the host.

🦇
