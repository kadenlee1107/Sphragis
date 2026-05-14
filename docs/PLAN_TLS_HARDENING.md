# TLS Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the chain-only-strict HTTPS posture from `DESIGN_TLS_HARDENING.md` — delete `tls_pinning.rs` + the `tls-mode` command + the `fetch_http`/`fetch_url`/`ResearchModeGuard` orphans, replace the chain-fail fallback in `tls.rs` with a single hard error using a new `VerifyError::as_static_str` mapping, make the WM TLS pill a constant, add a `cmd_x509_selftest` smoke, and prove the kernel still boots clean.

**Architecture:** Top-down deletion. Add the new error-mapping method first (additive, can't break anything). Replace the chain-fail block in `tls.rs` next so the new method has a real caller. Then delete orphaned callers (`fetch.rs` orphans, `cmd_tls_mode`). Then convert the surviving consumers (`wm.rs` pill, `net/mod.rs` boot status) to no longer reference `tls_pinning`. Finally delete `tls_pinning.rs`. Then add the selftest, update the boot smoke marker check, run the smoke, write the journal, push.

Each phase ends with `cargo check --target aarch64-unknown-none --release`. Warnings should not rise. The branch is `feat/tls-hardening`, opened from `feat/js-engine-browser-posix` immediately after PR #1 merged.

**Tech Stack:** Rust + Cargo, bare-metal `aarch64-unknown-none`, nightly toolchain, single-binary kernel (no library crate, no `cargo test` — verification is `cargo check` + `qemu_boot_smoke.py` + the new `cmd_x509_selftest` driven manually inside a booted kernel).

**Reference spec:** `DESIGN_TLS_HARDENING.md` (root). Read it before starting. This document is the *how*; the spec is the *why*.

**Pre-deletion HEAD:** `fe981fcd` on branch `feat/tls-hardening` (the spec commit). Verify before tagging.

---

## Phase 0: Safety net

### Task 0.1: Tag the pre-deletion commit

**Files:** none (git operation only).

- [ ] **Step 1: Verify HEAD matches the spec commit**

Run: `git log -1 --format='%H %s'`
Expected: `fe981fcd... 🎯 DESIGN_TLS_HARDENING: chain-only strict HTTPS posture` (or a later commit on `feat/tls-hardening` if other doc work has landed since — that's fine, just confirm the spec is in HEAD's ancestry).

- [ ] **Step 2: Verify branch is `feat/tls-hardening`**

Run: `git branch --show-current`
Expected: `feat/tls-hardening`. If it says `feat/js-engine-browser-posix`, STOP and create the feature branch first: `git switch -c feat/tls-hardening`.

- [ ] **Step 3: Create the rescue tag**

Run: `git tag -a pre-tls-hardening-2026-05-07 -m "Last commit before chain-only-strict TLS hardening. See DESIGN_TLS_HARDENING.md for rationale."`

- [ ] **Step 4: Push the tag**

Run: `git push origin pre-tls-hardening-2026-05-07`
Expected: `* [new tag] pre-tls-hardening-2026-05-07 -> pre-tls-hardening-2026-05-07`.

### Task 0.2: Establish baseline — kernel must build, boot smoke must pass

**Files:** none (verification only).

- [ ] **Step 1: Confirm cargo check passes**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...` (warnings OK, errors not OK). Record the warning count from the line `warning: sphragis (bin "sphragis") generated N warnings` — that's the post-no-browser baseline. Each phase below should match or beat that count.

- [ ] **Step 2: Confirm release build passes**

Run: `cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -5`
Expected: `Finished ...`. Also verify the kernel binary exists: `ls -lh target/aarch64-unknown-none/release/sphragis` should show a multi-MB binary.

- [ ] **Step 3: Confirm boot smoke passes against the pre-deletion kernel**

Run (with a 90s ceiling — the smoke takes ~30s):
```bash
python3 scripts/qemu_boot_smoke.py 2>&1 | tail -10 &
SMOKE_PID=$!
sleep 90
if kill -0 $SMOKE_PID 2>/dev/null; then kill $SMOKE_PID 2>/dev/null; fi
wait $SMOKE_PID 2>/dev/null
```
Expected: `[smoke] PASS — kernel boots, all required subsystems init, no deleted-browser symbols leaked through.` If the smoke fails here, STOP — the baseline is broken before any TLS work, so post-deletion failures won't be attributable to this thread.

---

## Phase 1: Add `VerifyError::as_static_str`

This phase is purely additive — it adds a method that nothing else calls yet. Build stays clean. Lays the groundwork for Phase 2.

### Task 1.1: Add the method

**Files:**
- Modify: `src/net/x509.rs` (around the `pub enum VerifyError` definition at line ~99)

- [ ] **Step 1: Find the enum definition**

Run: `grep -n 'pub enum VerifyError' src/net/x509.rs`
Expected: a single hit reporting the line number (around 99).

- [ ] **Step 2: Add the impl block**

Insert immediately after the closing `}` of the `pub enum VerifyError { ... }` block:

```rust
impl VerifyError {
    /// Map a verifier failure to a debug-friendly static string.
    /// Used by `tls.rs`'s chain-fail branch to abort the handshake with
    /// a specific reason. See DESIGN_TLS_HARDENING.md.
    pub fn as_static_str(&self) -> &'static str {
        match self {
            VerifyError::Parse              => "TLS: chain validation failed: certificate parse error",
            VerifyError::EmptyChain         => "TLS: chain validation failed: empty chain",
            VerifyError::UnsupportedSigAlg  => "TLS: chain validation failed: unsupported signature algorithm",
            VerifyError::HostnameMismatch   => "TLS: chain validation failed: hostname mismatch",
            VerifyError::NotYetValid        => "TLS: chain validation failed: certificate not yet valid",
            VerifyError::Expired            => "TLS: chain validation failed: expired certificate",
            VerifyError::BadSignature       => "TLS: chain validation failed: bad signature",
            VerifyError::UntrustedRoot      => "TLS: chain validation failed: untrusted root",
            VerifyError::ChainIncomplete    => "TLS: chain validation failed: chain incomplete",
        }
    }
}
```

- [ ] **Step 3: Verify build**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`. There may be a `function never used` warning on `as_static_str` — that's fine, Phase 2 calls it.

- [ ] **Step 4: Commit**

Run:
```bash
git add src/net/x509.rs
git commit -m "$(cat <<'EOF'
🎯 tls-hardening: VerifyError::as_static_str — debug-friendly reason mapping

Adds a method on VerifyError that maps each of the 9 variants to a
specific 'TLS: chain validation failed: <reason>' static string.

Phase 1 of the TLS hardening plan. Purely additive; no callers yet
(tls.rs starts using it in Phase 2). See DESIGN_TLS_HARDENING.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 2: Replace `tls.rs` chain-fail block with the hard-error one-liner

This phase wires up Phase 1 and removes the pin-fallback path. After this commit, the only way out of `verify_chain` Err is `return Err(...)` — no fallback, no mode-conditional acceptance.

### Task 2.1: Find the block

**Files:**
- Modify: `src/net/tls.rs` (around lines 985–1055; will shift slightly per branch state)

- [ ] **Step 1: Locate the chain-fail Err arm**

Run: `grep -n 'VerifyOutcome::Err(e)' src/net/tls.rs`
Expected: one hit, around line 993, inside a `match crate::net::x509::verify_chain(...)` block.

- [ ] **Step 2: Read the surrounding match for context**

Run: `sed -n '985,1055p' src/net/tls.rs | head -75`

You should see:
- Line ~985: `match crate::net::x509::verify_chain(leaf, &intermediates, host) {`
- Line ~986–992: `VerifyOutcome::Ok { ... } => { ... uart::puts("[tls] cert chain ok (x509)\n"); }`
- Line ~993: `VerifyOutcome::Err(e) => {`
- Line ~994: `let _ = e;` (currently discards the error)
- Lines ~995–1053: the fallback block — `leaf_info_with_host`, `tls_pinning::check_cert`, `match PinDecision`, `match current_mode()`
- Line ~1054: closing `}` of the `Err` arm
- Line ~1055: closing `}` of the outer `match`

### Task 2.2: Replace the Err arm

**Files:**
- Modify: `src/net/tls.rs` (the Err arm's body, ~62 lines → 1 line)

- [ ] **Step 1: Replace the entire Err arm body**

Use Edit to replace the block. The exact text to find (lines ~993–1054 — adjust the closing `}` count if the surrounding code shifted):

Old:
```rust
                            crate::net::x509::VerifyOutcome::Err(e) => {
                                let _ = e;
                                // V5-CHAIN-001 / V5-CRYPTO-001: even on
                                // chain failure, ALWAYS extract the leaf
                                // SPKI so the CertificateVerify step can
                                // check the peer actually holds the key.
                                // Before this, fallback paths left
                                // peer_spki_len=0 and CertificateVerify
                                // was silently skipped = full MITM bypass.
                                // V11-FRESH-EYES: use leaf_info_with_host so a
                                // cert validly issued for host A cannot be
                                // used against host B via the pin-only
                                // fallback path. Previously the fallback
                                // path didn't re-check hostname at all —
                                // latent MITM risk if STRICT_MODE ever
                                // flipped to false (e.g. dev builds).
                                match crate::net::x509::leaf_info_with_host(leaf, host) {
                                    Ok((spki, alg)) => {
                                        sess.peer_spki_len = spki.len().min(sess.peer_spki.len());
                                        sess.peer_spki[..sess.peer_spki_len]
                                            .copy_from_slice(&spki[..sess.peer_spki_len]);
                                        sess.peer_pubkey_alg = alg as u8;
                                    }
                                    Err(_) => {
                                        return Err("TLS: leaf cert unparseable or hostname mismatch");
                                    }
                                }
                                // V5-WEIRD uart-leak fix: do not distinguish
                                // x509-fail vs pin-ok via log timing. Same
                                // single log line for both outcomes.
                                match crate::net::tls_pinning::check_cert(host, leaf) {
                                    crate::net::tls_pinning::PinDecision::Match => {
                                        tdbg("[tls] leaf accepted (pin)\n");
                                    }
                                    crate::net::tls_pinning::PinDecision::Mismatch => {
                                        // STUMP #101: Open mode logs and proceeds on mismatch.
                                        // Lockdown / Research both abort.
                                        if crate::net::tls_pinning::current_mode()
                                            == crate::net::tls_pinning::Mode::Open
                                        {
                                            uart::puts("[tls] WARN: cert pin MISMATCH but mode=Open — accepting anyway\n");
                                        } else {
                                            return Err("TLS: cert pin mismatch (MITM?)");
                                        }
                                    }
                                    crate::net::tls_pinning::PinDecision::NoPin => {
                                        // Lockdown rejects unpinned hosts. Research
                                        // and Open allow them through with a log.
                                        match crate::net::tls_pinning::current_mode() {
                                            crate::net::tls_pinning::Mode::Lockdown => {
                                                return Err("TLS: no pin / bad chain (lockdown)");
                                            }
                                            crate::net::tls_pinning::Mode::Research => {
                                                tdbg("[tls] leaf accepted (no pin, mode=Research)\n");
                                            }
                                            crate::net::tls_pinning::Mode::Open => {
                                                tdbg("[tls] leaf accepted (no pin, mode=Open)\n");
                                            }
                                        }
                                    }
                                }
                            }
```

New (3 lines, preserving indentation):
```rust
                            crate::net::x509::VerifyOutcome::Err(e) => {
                                return Err(e.as_static_str());
                            }
```

If the exact text doesn't match (because of whitespace drift), use the surrounding context (`VerifyOutcome::Ok { ... }` arm just above, the closing `}` of the outer match below) to anchor. The body of the `Err(e)` arm should collapse to a single `return Err(e.as_static_str());`.

- [ ] **Step 2: Verify build**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -10`

Expected: `Finished ...`. Several functions in `x509.rs` (notably `leaf_info_with_host` if it was only used by the deleted block) may now produce `function never used` warnings — that's fine; Phase 7 will clean them up implicitly when `tls_pinning.rs` deletion lets dead-code analysis re-evaluate.

- [ ] **Step 3: Verify the Err arm shape**

Run: `grep -A 3 'VerifyOutcome::Err' src/net/tls.rs`
Expected: the arm body is exactly `return Err(e.as_static_str());`.

- [ ] **Step 4: Commit**

Run:
```bash
git add src/net/tls.rs
git commit -m "$(cat <<'EOF'
🎯 tls-hardening: chain-fail aborts with verifier reason — no pin fallback

Replaces the ~62-line pin-fallback block in tls.rs's Certificate
handler with a 3-line hard error using VerifyError::as_static_str.
Chain validation failure now aborts the handshake; mode-conditional
Research/Open acceptance paths are gone.

Phase 2 of the TLS hardening plan. tls_pinning.rs still in tree
but no longer called from the live HTTPS path; remaining callers
(fetch.rs ResearchModeGuard, cmd_tls_mode, WM pill, mod.rs boot
status) get cleaned up in Phases 3–7. See DESIGN_TLS_HARDENING.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 3: Delete `fetch.rs` orphans

The browser deletion left every external caller of `fetch_https`/etc. unused, but the helpers + `ResearchModeGuard` are still in tree. This phase deletes them.

### Task 3.1: Inventory the orphans

**Files:**
- Modify: `src/net/fetch.rs`

- [ ] **Step 1: Confirm the orphan list**

Run: `grep -nE '^pub fn fetch_|^fn fetch_|^struct ResearchModeGuard|^impl.*ResearchModeGuard' src/net/fetch.rs`

Expected to show:
- `pub fn fetch_url` (~line 136)
- `pub fn fetch_http` (~line 190)
- `pub fn fetch_post_url` (~line 278)
- `pub fn fetch_post_http` (~line 312)
- `pub fn fetch_post_https` (~line 353) ← KEEP
- `pub fn fetch_https` (~line 505) ← KEEP
- `struct ResearchModeGuard` (~line 64)
- `impl ResearchModeGuard` (~line 69)
- `impl Drop for ResearchModeGuard` (~line 79)

- [ ] **Step 2: Confirm none of the orphans have external callers**

Run: `grep -rn 'fetch_url\|fetch_http\b\|fetch_post_url\|fetch_post_http\b\|ResearchModeGuard\|relax_for_renderer' src/ --include='*.rs' | grep -v 'src/net/fetch.rs'`

Expected: empty. (If any non-`fetch.rs` line shows, STOP and resolve before deleting — there's a caller this plan didn't account for.)

### Task 3.2: Delete `ResearchModeGuard`

**Files:**
- Modify: `src/net/fetch.rs`

- [ ] **Step 1: Delete the struct, impl, and Drop**

Use a Python-style script via Bash to find the extent and delete. Or, find the relevant block manually and use Edit:

```bash
python3 <<'PY'
from pathlib import Path
import re
p = Path("src/net/fetch.rs")
lines = p.read_text().split("\n")
# Find the start of the ResearchModeGuard doc-comment / struct
start = None
for i, ln in enumerate(lines):
    if "STUMP #111 (audit H019): RAII guard" in ln:
        # Walk back to grab the leading `///` block
        j = i
        while j > 0 and lines[j-1].lstrip().startswith("///"):
            j -= 1
        start = j
        break
# Find the end of `impl Drop for ResearchModeGuard`
end = None
saw_drop_impl = False
depth = 0
for i in range(start, len(lines)):
    ln = lines[i]
    if "impl Drop for ResearchModeGuard" in ln:
        saw_drop_impl = True
    if saw_drop_impl:
        for ch in ln:
            if ch == "{": depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    end = i
                    break
        if end is not None:
            break
print(f"Deleting lines {start+1}..{end+1} ({end-start+1} lines)")
new = lines[:start] + lines[end+1:]
# Compact double blanks
out = []
prev_blank = False
for ln in new:
    if ln.strip() == "":
        if prev_blank: continue
        prev_blank = True
    else:
        prev_blank = False
    out.append(ln)
p.write_text("\n".join(out))
print(f"before: {len(lines)} after: {len(out)}")
PY
```

- [ ] **Step 2: Verify the struct and its impls are gone**

Run: `grep -n 'ResearchModeGuard\|relax_for_renderer' src/net/fetch.rs`
Expected: empty.

### Task 3.3: Delete `fetch_url`, `fetch_http`, `fetch_post_url`, `fetch_post_http`

**Files:**
- Modify: `src/net/fetch.rs`

- [ ] **Step 1: Delete each function via the same Python-style approach**

```bash
python3 <<'PY'
from pathlib import Path
import re
p = Path("src/net/fetch.rs")
lines = p.read_text().split("\n")
fns = ["fetch_url", "fetch_http", "fetch_post_url", "fetch_post_http"]

def find_extent(name):
    pat = re.compile(rf"^pub fn {name}\b")
    for i, ln in enumerate(lines):
        if pat.match(ln):
            start = i
            break
    else:
        return None
    depth, seen = 0, False
    for j in range(start, len(lines)):
        for ch in lines[j]:
            if ch == "{": depth += 1; seen = True
            elif ch == "}":
                depth -= 1
                if seen and depth == 0:
                    return (start, j)

def expand_doc(start):
    i = start - 1
    while i >= 0 and (lines[i].lstrip().startswith("///") or lines[i].lstrip().startswith("//") or lines[i].strip() == ""):
        i -= 1
    return i + 1

ranges = []
for name in fns:
    ext = find_extent(name)
    if ext is None:
        print(f"WARN: {name} not found")
        continue
    s, e = ext
    ranges.append((expand_doc(s), e, name))
ranges.sort(reverse=True)
for s, e, n in ranges:
    print(f"  {n}: lines {s+1}..{e+1} ({e-s+1} lines)")
new = lines[:]
for s, e, _ in ranges:
    del new[s:e+1]
out = []
prev_blank = False
for ln in new:
    if ln.strip() == "":
        if prev_blank: continue
        prev_blank = True
    else:
        prev_blank = False
    out.append(ln)
p.write_text("\n".join(out))
print(f"before: {len(lines)} after: {len(out)} removed: {len(lines) - len(out)}")
PY
```

- [ ] **Step 2: Verify the functions are gone**

Run: `grep -nE '^pub fn fetch_(url|http|post_url|post_http)\b' src/net/fetch.rs`
Expected: empty.

### Task 3.4: Refresh the module-level comment

**Files:**
- Modify: `src/net/fetch.rs` (the comment block at the top, lines 1-25 area)

- [ ] **Step 1: Read current comment**

Run: `sed -n '1,25p' src/net/fetch.rs`

You'll see a block including `HTTPS note (STUMP #94)` paragraph that documents the now-deleted renderer-relax behavior.

- [ ] **Step 2: Replace the STUMP #94 paragraph**

Use Edit to find the paragraph starting `// HTTPS note (STUMP #94):` through its trailing blank line, replace with:

```rust
// All HTTPS goes through fetch_https / fetch_post_https. Strict
// chain validation against TRUST_STORE; hybrid PQ on; no fallback
// trust paths. See DESIGN_TLS_HARDENING.md.
```

### Task 3.5: Verify and commit

- [ ] **Step 1: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -10`
Expected: `Finished ...`. The deletions removed every caller of `tls_pinning::set_mode` from `fetch.rs`; some warnings may shift but errors should be zero.

- [ ] **Step 2: Confirm grep is clean for the orphan symbols**

Run: `grep -rn 'fetch_url\|fetch_http\b\|fetch_post_url\|fetch_post_http\b\|ResearchModeGuard\|relax_for_renderer' src/ --include='*.rs'`
Expected: empty.

- [ ] **Step 3: Commit**

Run:
```bash
git add src/net/fetch.rs
git commit -m "$(cat <<'EOF'
🎯 tls-hardening: delete fetch.rs orphans + ResearchModeGuard

Removes:
- struct ResearchModeGuard + relax_for_renderer + Drop impl (the
  renderer-relax backdoor that flipped Lockdown→Research and
  disabled hybrid PQ for the duration of every HTTPS fetch). Zero
  callers since the no-browser pivot.
- pub fn fetch_url (URL dispatcher → http/https)
- pub fn fetch_http
- pub fn fetch_post_url
- pub fn fetch_post_http

Surviving fetch API: fetch_https + fetch_post_https. HTTPS-only.

Module-level comment refreshed to reflect the new posture
(strict chain validation, hybrid PQ on, no fallback paths).

Phase 3 of the TLS hardening plan. See DESIGN_TLS_HARDENING.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 4: Delete `tls-mode` shell command

### Task 4.1: Remove the dispatch arm

**Files:**
- Modify: `src/ui/shell.rs` (around line 134)

- [ ] **Step 1: Locate the dispatch arm**

Run: `grep -n '"tls-mode"' src/ui/shell.rs`
Expected: a single hit around line 134.

- [ ] **Step 2: Delete the dispatch arm**

Use Edit to find the line `"tls-mode" => cmd_tls_mode(parts[1]),` and delete it (including its leading whitespace and trailing newline).

### Task 4.2: Remove the `cmd_tls_mode` function

**Files:**
- Modify: `src/ui/shell.rs`

- [ ] **Step 1: Locate the function**

Run: `grep -n '^fn cmd_tls_mode' src/ui/shell.rs`
Expected: a single hit, around line 3050 (will be lower after Phase 1–3 didn't touch shell.rs much, but precise number depends on the post-no-browser-pivot baseline).

- [ ] **Step 2: Delete the function (including any leading doc comment)**

```bash
python3 <<'PY'
from pathlib import Path
import re
p = Path("src/ui/shell.rs")
lines = p.read_text().split("\n")
start = None
for i, ln in enumerate(lines):
    if re.match(r"^fn cmd_tls_mode\b", ln):
        start = i; break
assert start is not None, "cmd_tls_mode not found"
# Walk back through doc comments
i = start - 1
while i >= 0 and (lines[i].lstrip().startswith("///") or lines[i].lstrip().startswith("//") or lines[i].strip() == ""):
    i -= 1
expanded = i + 1
# Walk forward to closing }
depth, seen = 0, False
end = None
for j in range(start, len(lines)):
    for ch in lines[j]:
        if ch == "{": depth += 1; seen = True
        elif ch == "}":
            depth -= 1
            if seen and depth == 0:
                end = j; break
    if end is not None: break
print(f"Deleting lines {expanded+1}..{end+1} ({end-expanded+1} lines)")
new = lines[:expanded] + lines[end+1:]
out = []
prev_blank = False
for ln in new:
    if ln.strip() == "":
        if prev_blank: continue
        prev_blank = True
    else:
        prev_blank = False
    out.append(ln)
p.write_text("\n".join(out))
PY
```

### Task 4.3: Verify and commit

- [ ] **Step 1: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`.

- [ ] **Step 2: Confirm grep**

Run: `grep -n 'tls-mode\|cmd_tls_mode' src/ui/shell.rs`
Expected: empty.

- [ ] **Step 3: Commit**

Run:
```bash
git add src/ui/shell.rs
git commit -m "$(cat <<'EOF'
🎯 tls-hardening: delete tls-mode shell command

Removes 'tls-mode' dispatch arm + cmd_tls_mode (~25 LOC) from
src/ui/shell.rs. With chain-only-strict TLS, the user-toggleable
Lockdown/Research/Open switch loses meaning — any future relaxed
mode would be a conscious design choice with its own UX, not a
single-keystroke runtime override.

Phase 4 of the TLS hardening plan. tls_pinning::Mode is now
referenced from only two sites (WM pill, net/mod.rs boot status);
both get converted in Phases 5-6 before the module is deleted in
Phase 7. See DESIGN_TLS_HARDENING.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 5: WM TLS pill becomes a constant indicator

### Task 5.1: Replace the match block

**Files:**
- Modify: `src/ui/wm.rs` (around lines 499–505)

- [ ] **Step 1: Locate the pill code**

Run: `grep -n 'tls_pinning\|tls_label' src/ui/wm.rs`
Expected: hits at ~499 (`crate::net::tls_pinning::current_mode()`) and ~500-505 (the match + draw_status_segment call).

- [ ] **Step 2: Replace the match with a constant**

Use Edit. Find the block:

```rust
    let mode = crate::net::tls_pinning::current_mode();
    let (tls_label, tls_color) = match mode {
        crate::net::tls_pinning::Mode::Lockdown => ("LOCKDOWN", CYAN),
        crate::net::tls_pinning::Mode::Research => ("RESEARCH", AMBER),
        crate::net::tls_pinning::Mode::Open     => ("OPEN",     RED),
    };
    x = draw_status_segment(fb, w, x, sy, "TLS", Some(tls_label), tls_color, None);
```

Replace with:

```rust
    // TLS posture is a constant per DESIGN_TLS_HARDENING.md:
    // chain-only strict, hybrid PQ on, no mode toggle. Try LOCK/PQ
    // first; if it pushes adjacent segments off the 1280px bar at
    // boot, fall back to LD/PQ.
    let tls_label = "LOCK/PQ";
    x = draw_status_segment(fb, w, x, sy, "TLS", Some(tls_label), CYAN, None);
```

- [ ] **Step 3: Verify build**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -10`
Expected: `Finished ...`. May produce one warning along the lines of `unused variable: amber` or `unused color constant` if `AMBER`/`RED` were only referenced from the deleted match; ignore for now (Phase 7's deletion of `tls_pinning.rs` won't fix those — they're WM-local consts. If the warning bothers you, leave it; warnings should not rise relative to baseline so check the count.)

- [ ] **Step 4: Update the comment at line 331**

Run: `grep -n "5 live-state segments" src/ui/wm.rs`
Expected: a single hit around line 331.

Use Edit to update the comment from `"5 live-state segments (ENCRYPTED · NET · TLS · AUDIT · UPTIME)"` to `"5 status segments (ENCRYPTED · NET · TLS const · AUDIT · UPTIME)"`. Just clarify that TLS is now a constant indicator rather than live state.

### Task 5.2: Test the pill width visually (optional)

**Files:** none (manual verification).

- [ ] **Step 1: If width may be tight, run a quick visual smoke**

Boot QEMU with display: `python3 scripts/qemu_busybox_baseline.py` (or any qemu launcher that shows the WM). Watch for the TLS pill saying `LOCK/PQ` in cyan in the status bar. If it overflows or pushes other segments off-screen, edit the label to `"LD/PQ"` and rebuild.

If you don't have time for a visual check, default to `LD/PQ` (5 chars matches `OPEN`/`LOCKDOWN` ranges from before, so it's certain to fit). The plan's recommended default if unverified is `LD/PQ`.

### Task 5.3: Commit

- [ ] **Step 1: Commit**

Run:
```bash
git add src/ui/wm.rs
git commit -m "$(cat <<'EOF'
🎯 tls-hardening: WM TLS pill becomes a constant LOCK/PQ indicator

Replaces the runtime match against tls_pinning::Mode (Lockdown/
Research/Open) with a constant 'LOCK/PQ' label in CYAN. With chain-
only-strict + hybrid PQ on as the boot defaults, there's no mode
to toggle and no live state to display.

Comment at line ~331 updated to reflect TLS is a constant indicator
in the status bar, not live state.

Phase 5 of the TLS hardening plan. WM no longer references
tls_pinning. Last reference standing is net/mod.rs's boot status
(Phase 6). See DESIGN_TLS_HARDENING.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 6: Update `net/mod.rs` boot status

### Task 6.1: Replace the boot-status block

**Files:**
- Modify: `src/net/mod.rs` (around lines 60–80)

- [ ] **Step 1: Locate the boot-status block**

Run: `grep -n 'tls_pinning\|TRUST_STORE\|n_pins' src/net/mod.rs`
Expected: hits in a single block around lines 60–75 referencing both `x509::TRUST_STORE.len()` and `tls_pinning::PINS.len()` plus a populate-hint message.

- [ ] **Step 2: Read the block for exact context**

Run: `sed -n '55,85p' src/net/mod.rs`

- [ ] **Step 3: Replace the block**

Use Edit. Find the block (which roughly looks like):

```rust
    // ... [audit comment about empty stores] ...
    let n_trust = crate::net::x509::TRUST_STORE.len();
    let n_pins = crate::net::tls_pinning::PINS.len();
    if n_trust == 0 && n_pins == 0 {
        crate::drivers::uart::puts("  [tls] WARNING: no trust anchors\n");
        crate::drivers::uart::puts("  Populate src/net/x509.rs::TRUST_STORE or src/net/tls_pinning\n");
    } else {
        // ... existing report ...
    }
```

Replace with:

```rust
    let n_trust = crate::net::x509::TRUST_STORE.len();
    crate::drivers::uart::puts("  [tls] trust store: ");
    crate::kernel::mm::print_num(n_trust);
    crate::drivers::uart::puts(" CA roots, chain-only auth, hybrid PQ on\n");
    if n_trust == 0 {
        crate::drivers::uart::puts("  [tls] WARNING: no trust anchors — HTTPS will refuse all peers\n");
    }
```

The exact existing text may differ; preserve the `let n_trust` style and the `crate::kernel::mm::print_num` for numeric output.

- [ ] **Step 4: Verify the boot string fits the smoke's expected marker**

The required marker the boot smoke (Phase 9) will check is:
```
[tls] trust store: 5 CA roots, chain-only auth, hybrid PQ on
```

The format pieces above produce exactly that line when `n_trust == 5`. The space alignment matters: `print_num(5)` outputs `5` (no leading space), then `" CA roots..."` follows. Expected literal output: `  [tls] trust store: 5 CA roots, chain-only auth, hybrid PQ on` (the leading 2-space indent is the existing convention for boot messages).

### Task 6.2: Verify and commit

- [ ] **Step 1: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -10`
Expected: `Finished ...`. There should be no remaining `tls_pinning::` references in `src/net/mod.rs`.

- [ ] **Step 2: Confirm grep**

Run: `grep -n 'tls_pinning' src/net/mod.rs`
Expected: empty.

- [ ] **Step 3: Commit**

Run:
```bash
git add src/net/mod.rs
git commit -m "$(cat <<'EOF'
🎯 tls-hardening: boot status reflects chain-only auth + hybrid PQ on

Replaces the boot-status block that referenced both TRUST_STORE and
tls_pinning::PINS with a single line:

  [tls] trust store: N CA roots, chain-only auth, hybrid PQ on

When N==0, prints a WARNING that HTTPS will refuse all peers (the
audit's 'empty trust store = no auth' guard, restated for the new
chain-only world).

Phase 6 of the TLS hardening plan. After this commit, no surviving
code references tls_pinning::*. Phase 7 deletes the module.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 7: Delete `tls_pinning.rs`

### Task 7.1: Confirm zero callers, then delete

**Files:**
- Delete: `src/net/tls_pinning.rs`
- Modify: `src/net/mod.rs` (remove `pub mod tls_pinning;`)

- [ ] **Step 1: Final grep — must be empty**

Run: `grep -rn 'tls_pinning\|tls_pinning::' src/ --include='*.rs'`
Expected: empty (or only the `pub mod tls_pinning;` line in `src/net/mod.rs`, which we're about to delete).

If there are any other hits, STOP — a previous phase missed a reference. Resolve before continuing.

- [ ] **Step 2: Delete the module declaration**

Run: `grep -n 'pub mod tls_pinning;' src/net/mod.rs`
Expected: a single hit around line 18.

Use Edit to delete that line.

- [ ] **Step 3: Delete the file**

Run: `git rm src/net/tls_pinning.rs`
Expected: `rm 'src/net/tls_pinning.rs'`.

### Task 7.2: Verify and commit

- [ ] **Step 1: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`. The kernel now has zero references to `tls_pinning`.

- [ ] **Step 2: Confirm grep**

Run: `grep -rn 'tls_pinning' src/ --include='*.rs'`
Expected: empty (no exceptions — both the module file and every reference are gone).

- [ ] **Step 3: Commit**

Run:
```bash
git add src/net/tls_pinning.rs src/net/mod.rs
git commit -m "$(cat <<'EOF'
🎯 tls-hardening: delete src/net/tls_pinning.rs

Removes the entire module: Mode enum (Lockdown/Research/Open), Pin
struct, PinDecision enum, PINS static, check_cert(), current_mode(),
set_mode(), is_strict(). 167 LOC.

Also removes the `pub mod tls_pinning;` declaration in
src/net/mod.rs.

After this commit, the kernel has no concept of a 'TLS mode' or
'cert pin'. The only path to a trusted HTTPS handshake is strict
X.509 chain validation against TRUST_STORE; failure aborts.

Phase 7 of the TLS hardening plan. See DESIGN_TLS_HARDENING.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 8: Add `cmd_x509_selftest`

### Task 8.1: Add the selftest function

**Files:**
- Modify: `src/ui/shell.rs` (append a new function at the end of the file, before the last `}` if any module-level closer; or just before the existing `cmd_pq_tls_selftest` to keep selftests grouped).

- [ ] **Step 1: Locate a good insertion point**

Run: `grep -n '^fn cmd_pq_tls_selftest\|^fn cmd_gcm_selftest' src/ui/shell.rs`
Expected: hits around lines 538 (cmd_pq_tls_selftest) and others. Insert `cmd_x509_selftest` immediately before `cmd_pq_tls_selftest` so existing selftests stay grouped.

- [ ] **Step 2: Add the function**

Insert the function (before `fn cmd_pq_tls_selftest` or after `fn cmd_audit_flush`, whichever is more natural in your file). Use Edit:

```rust
/// X.509 chain validator selftest. Exercises the new `as_static_str`
/// error mapping path with two deterministic inputs:
///   1. Trusted root used as a "leaf" with a wrong hostname → expect
///      VerifyOutcome::Err(HostnameMismatch).
///   2. Truncated DER → expect VerifyOutcome::Err(Parse).
///
/// Verifies both that the verifier surfaces the right variant AND
/// that as_static_str returns a debug-friendly string. See
/// DESIGN_TLS_HARDENING.md.
fn cmd_x509_selftest() {
    use crate::net::x509::{verify_chain, VerifyOutcome, VerifyError, TRUST_STORE};

    if TRUST_STORE.is_empty() {
        console::puts("  [x509-selftest] FAIL: TRUST_STORE empty\n");
        return;
    }
    let root_der: &[u8] = TRUST_STORE[0];

    // Case 1: hostname mismatch.
    let case1 = verify_chain(root_der, &[], b"wrong-host.example");
    match case1 {
        VerifyOutcome::Err(VerifyError::HostnameMismatch) => {
            // Confirm as_static_str returns the expected substring.
            let s = VerifyError::HostnameMismatch.as_static_str();
            // Look for "hostname mismatch" in the static string.
            let needle: &[u8] = b"hostname mismatch";
            let mut found = false;
            let bytes = s.as_bytes();
            if bytes.len() >= needle.len() {
                for i in 0..=(bytes.len() - needle.len()) {
                    if &bytes[i..i + needle.len()] == needle {
                        found = true;
                        break;
                    }
                }
            }
            if found {
                console::puts("  [x509-selftest] PASS: hostname-mismatch\n");
            } else {
                console::puts("  [x509-selftest] FAIL: hostname-mismatch (string mismatch)\n");
            }
        }
        VerifyOutcome::Err(other) => {
            console::puts("  [x509-selftest] FAIL: hostname-mismatch (got wrong VerifyError variant: ");
            console::puts(other.as_static_str());
            console::puts(")\n");
        }
        VerifyOutcome::Ok { .. } => {
            console::puts("  [x509-selftest] FAIL: hostname-mismatch (expected Err, got Ok)\n");
        }
    }

    // Case 2: truncated DER.
    let truncated = &root_der[..root_der.len().saturating_sub(5)];
    let case2 = verify_chain(truncated, &[], b"any.example");
    match case2 {
        VerifyOutcome::Err(VerifyError::Parse) => {
            let s = VerifyError::Parse.as_static_str();
            let needle: &[u8] = b"parse error";
            let mut found = false;
            let bytes = s.as_bytes();
            if bytes.len() >= needle.len() {
                for i in 0..=(bytes.len() - needle.len()) {
                    if &bytes[i..i + needle.len()] == needle {
                        found = true;
                        break;
                    }
                }
            }
            if found {
                console::puts("  [x509-selftest] PASS: bad-bytes\n");
            } else {
                console::puts("  [x509-selftest] FAIL: bad-bytes (string mismatch)\n");
            }
        }
        VerifyOutcome::Err(other) => {
            console::puts("  [x509-selftest] FAIL: bad-bytes (got wrong VerifyError variant: ");
            console::puts(other.as_static_str());
            console::puts(")\n");
        }
        VerifyOutcome::Ok { .. } => {
            console::puts("  [x509-selftest] FAIL: bad-bytes (expected Err, got Ok)\n");
        }
    }
}
```

- [ ] **Step 3: Add the dispatch arm**

Run: `grep -n '"pq-tls-selftest"' src/ui/shell.rs`
Expected: a single hit (~line 202).

Use Edit to add a new arm just before the `pq-tls-selftest` arm:

```rust
        "x509-selftest" => cmd_x509_selftest(),
```

### Task 8.2: Verify and commit

- [ ] **Step 1: Build check**

Run: `cargo check --target aarch64-unknown-none --release 2>&1 | tail -5`
Expected: `Finished ...`.

- [ ] **Step 2: Verify dispatch + function are both present**

Run: `grep -n 'x509-selftest\|cmd_x509_selftest' src/ui/shell.rs`
Expected: 3 hits — the dispatch arm, the function definition, and any internal mention.

- [ ] **Step 3: Commit**

Run:
```bash
git add src/ui/shell.rs
git commit -m "$(cat <<'EOF'
🎯 tls-hardening: cmd_x509_selftest — deterministic verifier smoke

Adds a kernel-side selftest exercising the new VerifyError::
as_static_str path. Two deterministic sub-tests:
  1. hostname-mismatch: trusted root with wrong host → expect
     VerifyOutcome::Err(HostnameMismatch); confirm static string
     contains 'hostname mismatch'.
  2. bad-bytes: truncated DER → expect VerifyOutcome::Err(Parse);
     confirm static string contains 'parse error'.

Each sub-test prints PASS or FAIL to UART. Run via 'x509-selftest'
in the sphragis shell.

Phase 8 of the TLS hardening plan. The full chain-fail-aborts-the-
handshake behavior in tls.rs is verified by code review + grep
(no contortion to mock a TLS handshake from the shell). See
DESIGN_TLS_HARDENING.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 9: Update `qemu_boot_smoke.py` with the new required marker

### Task 9.1: Add the new required marker

**Files:**
- Modify: `scripts/qemu_boot_smoke.py`

- [ ] **Step 1: Locate the `required` patterns list**

Run: `grep -n 'required = \[' scripts/qemu_boot_smoke.py`
Expected: a single hit. Read the surrounding `required = [(rb"...", "label"), ...]` block.

- [ ] **Step 2: Add the new entry**

Use Edit to add to the `required` list:

```python
            (rb"\[tls\] trust store: \d+ CA roots, chain-only auth, hybrid PQ on", "tls trust-store boot status"),
```

The regex matches any digit count (so the smoke survives if `TRUST_STORE` grows from 5 in the future). Place it next to the existing `[tls] trust store: ...` matchers if any, or at the end of the list.

- [ ] **Step 3: Verify the script still parses**

Run: `python3 -c "import scripts.qemu_boot_smoke" 2>&1 | head -5`
Expected: empty (clean import) or a `ModuleNotFoundError: No module named 'scripts'` (acceptable — the script isn't structured as a package; just confirm Python parses the file).

Alternative: `python3 -c "import ast; ast.parse(open('scripts/qemu_boot_smoke.py').read())"`
Expected: empty (no syntax error).

### Task 9.2: Run the boot smoke against the post-Phase-8 kernel

- [ ] **Step 1: Build the release kernel**

Run: `cargo build --release --target aarch64-unknown-none --features gicv3 2>&1 | tail -5`
Expected: `Finished ...`.

- [ ] **Step 2: Run the smoke**

```bash
python3 scripts/qemu_boot_smoke.py 2>&1 | tail -10 &
SMOKE_PID=$!
sleep 90
if kill -0 $SMOKE_PID 2>/dev/null; then kill $SMOKE_PID 2>/dev/null; fi
wait $SMOKE_PID 2>/dev/null
```

Expected: `[smoke] PASS — kernel boots, all required subsystems init, no deleted-browser symbols leaked through.`

If the smoke FAILS with `missing required marker: tls trust-store boot status`, it means the boot output did not match the regex. Open the smoke log (`logs/qemu-tests/boot-smoke-*.log`), grep for `[tls] trust store`, and adjust either the boot string in `src/net/mod.rs` (Phase 6) or the regex here to reconcile.

### Task 9.3: Commit

- [ ] **Step 1: Commit**

Run:
```bash
git add scripts/qemu_boot_smoke.py
git commit -m "$(cat <<'EOF'
🎯 tls-hardening: qemu_boot_smoke required marker for chain-only TLS

Adds a required boot-output marker:

  [tls] trust store: \d+ CA roots, chain-only auth, hybrid PQ on

Catches regressions where the boot status reverts to the pre-
hardening pin-based message. Digit-count regex tolerates
TRUST_STORE growth.

Phase 9 of the TLS hardening plan. See DESIGN_TLS_HARDENING.md.

Co-Authored-By: claude-flow <ruv@ruv.net>
EOF
)"
```

---

## Phase 10: Final acceptance + journal entry + push

### Task 10.1: Run the full acceptance grep

**Files:** none (verification only).

- [ ] **Step 1: Run the static grep**

Run:
```bash
rg 'tls_pinning|cmd_tls_mode|fetch_url|fetch_http\b|fetch_post_url|fetch_post_http\b|ResearchModeGuard' src
```

Expected: empty output. If anything returns, name the surviving symbol and find which phase missed it; resolve before continuing.

### Task 10.2: Run x509-selftest manually inside booted Sphragis

**Files:** none (manual verification).

- [ ] **Step 1: Boot Sphragis in QEMU with display + keyboard**

The boot-smoke script auth-gates via virtio-keyboard not serial, so to drive the shell you'll need to type `batman` at the auth gate using the QEMU window's keyboard. Run a launcher that exposes the GUI:

```bash
python3 scripts/qemu_busybox_baseline.py
```
(Or any non-headless qemu launcher in `scripts/`.)

- [ ] **Step 2: At the sphragis prompt, run the selftest**

Type:
```
x509-selftest
```

Expected output:
```
  [x509-selftest] PASS: hostname-mismatch
  [x509-selftest] PASS: bad-bytes
```

If anything says `FAIL`, capture the line and resolve. The most likely cause is a wrong VerifyError variant (the hostname-mismatch path may fall through to `Parse` if the chosen TRUST_STORE entry has unusual SAN encoding — try a different index in `TRUST_STORE` or swap to a known-good DER input).

### Task 10.3: Write SESSION_JOURNAL entry

**Files:**
- Modify: `docs/SESSION_JOURNAL.md` (prepend a new entry at the top, after the format header).

- [ ] **Step 1: Draft the entry**

The entry should cover:
- The strategic context (no-browser pivot orphaned every HTTPS caller, exposed the renderer-relax backdoor, this thread cleaned that up).
- Each phase as a paragraph (commit hashes via `git log --oneline pre-tls-hardening-2026-05-07..HEAD`).
- LOC stats (~250-300 deleted, ~50 added; 1 file deleted, 7 modified).
- State of tree: chain-only HTTPS, hybrid PQ on, no mode toggle, `cmd_x509_selftest` available.
- Tag `pre-tls-hardening-2026-05-07` for revival.
- What's next (per the post-no-browser roadmap memory: scheduler `block_on()`, then captures cleanup).

Match the existing journal voice (terse, technical, honest). ~150-200 lines.

- [ ] **Step 2: Insert at top of journal**

Use Edit to insert the new entry between the format header (around line 12) and the previous newest entry.

- [ ] **Step 3: Commit**

Run:
```bash
git add docs/SESSION_JOURNAL.md
git commit -m "📝 SESSION_JOURNAL: TLS hardening complete

Phase 10 of the TLS hardening plan — handoff point. See
DESIGN_TLS_HARDENING.md for strategy, PLAN_TLS_HARDENING.md for
the per-phase how.

Co-Authored-By: claude-flow <ruv@ruv.net>"
```

### Task 10.4: Push the branch

- [ ] **Step 1: Push**

Run: `git push -u origin feat/tls-hardening 2>&1 | tail -5`
Expected: `* [new branch] feat/tls-hardening -> feat/tls-hardening` (or fast-forward if already pushed).

- [ ] **Step 2: Confirm tag is pushed**

Run: `git push origin pre-tls-hardening-2026-05-07 2>&1 | tail -3`
Expected: `* [up to date]` or new tag confirmation.

### Task 10.5: Hand off to finishing-a-development-branch

- [ ] **Step 1: Invoke the skill**

Use the `superpowers:finishing-a-development-branch` skill to walk through merge / PR / keep-as-is / discard options. Recommended option (matching the no-browser thread): **Push and create a Pull Request** against `feat/js-engine-browser-posix`.

PR title (matching the squash-friendly format from PR #1):
```
tls-hardening: chain-only strict HTTPS, no mode toggles, no pin fallback
```

PR body should reference `DESIGN_TLS_HARDENING.md`, the rescue tag, and summarize the deletions + behavior change.

---

## Done

After Phase 10:
- `feat/tls-hardening` has ~9 commits documenting each phase.
- Tag `pre-tls-hardening-2026-05-07` preserves the pre-deletion state.
- `cargo build --release` produces a working kernel binary; warnings ≤ baseline.
- `qemu_boot_smoke.py` PASSES with the new TLS trust-store marker.
- `cmd_x509_selftest` PASSES both sub-tests inside booted Sphragis.
- Static grep returns empty for all 7 forbidden symbols.
- Journal entry captures the trail; PR opened for review.

The HTTPS posture is now:
- Strict X.509 chain validation against `TRUST_STORE` (5 CA roots).
- Hybrid PQ TLS on at boot.
- No mode toggle, no pin fallback, no dormant alternate-trust machinery.
- Verifier failures abort the handshake with a debug-friendly reason.

Next thread per the post-no-browser roadmap: scheduler `block_on()` (futex/epoll spin replacement).

🦇
