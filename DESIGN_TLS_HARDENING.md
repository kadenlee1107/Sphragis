# DESIGN: TLS Hardening — Chain-Only Strict, No Alternate Trust Paths

**Status:** Active proposal as of 2026-05-07.
**Follows:** `DESIGN_NO_BROWSER.md` (the no-browser pivot orphaned every HTTPS caller and exposed the renderer-relax backdoor).
**Touches:** `src/net/tls.rs`, `src/net/x509.rs`, `src/net/fetch.rs`, `src/net/mod.rs`, `src/ui/wm.rs`, `src/ui/shell.rs`, `scripts/qemu_boot_smoke.py`. Deletes `src/net/tls_pinning.rs`.

## Goal

Make Sphragis's HTTPS posture honest end-to-end. Strict X.509 chain
validation is the **only** path to a trusted handshake. Hybrid PQ TLS
is on. No mode toggles, no pin fallback, no dormant trust machinery.

If pinned or self-hosted endpoints become a real use case later, that
gets a separate explicit pinned-HTTPS API designed at the time — not
a dormant escape hatch sitting in tree.

## Why now

The no-browser pivot deleted every HTTPS caller in the repo
(`cmd_render`, `cmd_dump_dom`, `browser_proxy`, etc.). The validators
in `src/net/x509.rs` were already wired into `tls.rs` and worked, but
two things made the live HTTPS path lie about its security:

1. **`fetch.rs::ResearchModeGuard::relax_for_renderer`** flipped the
   kernel from `Mode::Lockdown` → `Mode::Research` and disabled hybrid
   PQ at the start of every fetch. The "renderer needs to talk to
   unpinned hosts" justification died with the browser. The guard now
   has zero callers but its existence in tree is misleading.
2. **`tls.rs`'s chain-fail branch** falls back to `tls_pinning::
   check_cert` and, in `Research`/`Open` mode with no pin, **accepts
   the connection anyway** with a log line. Combined with the guard
   above, every HTTPS fetch the deleted browser ever made ran in
   "encrypted but not authenticated" mode by design.

Now that the only HTTPS surface area is the canonical
`fetch_https` / `fetch_post_https` API (also currently uncalled, but
intended for future machine-to-machine use), the right move is to
delete the entire alternate-trust apparatus while it's removable
without breaking a live caller.

## Decisions locked in

1. **Auth posture:** Chain-only strict. No pin fallback. If
   `verify_chain` returns `Err`, the handshake aborts with a specific
   reason string.
2. **API surface:** HTTPS-only. Plain HTTP is not a feature of the
   canonical fetch API. Callers needing unauthenticated transport
   reach for raw `crate::net::tcp` and own the implications.
3. **Hybrid PQ default:** ON at boot. `TLS_HYBRID_ENABLED_FLAG`
   already inits to `true`; the only thing that flipped it false was
   the deleted `ResearchModeGuard`.
4. **No pinning machinery in tree:** `tls_pinning.rs` and the
   `tls-mode` shell command both go. Mode (Lockdown/Research/Open) as
   a kernel-wide concept ceases to exist.
5. **WM status pill:** becomes a non-interactive constant. Label
   `LOCK/PQ` if width fits, fall back to `LD/PQ`. Color CYAN.

## What gets deleted

**Module-level:**
- `src/net/tls_pinning.rs` (167 lines): `Mode` enum, `Pin` struct,
  `PinDecision`, `PINS` static, `check_cert`, `current_mode`,
  `set_mode`, `is_strict`, the whole file.
- `pub mod tls_pinning;` declaration in `src/net/mod.rs`.

**`src/net/fetch.rs` (orphaned by no-browser pivot):**
- `struct ResearchModeGuard` + `relax_for_renderer()` + `Drop` impl
  (~20 lines).
- `pub fn fetch_url` (URL dispatcher).
- `pub fn fetch_http` (~50 lines).
- `pub fn fetch_post_url` (URL dispatcher).
- `pub fn fetch_post_http` (~40 lines).
- The "HTTPS note (STUMP #94)" paragraph at lines 18-23 documenting
  the now-deleted renderer-relax behavior.

**`src/ui/shell.rs`:**
- `"tls-mode"` dispatch arm.
- `fn cmd_tls_mode` (~25 lines).

**`src/net/tls.rs`:**
- The chain-fail fallback block (~62 lines, currently lines
  993–1054): `leaf_info_with_host` extraction, the
  `tls_pinning::check_cert` cascade, the `match
  tls_pinning::current_mode()` arms that conditionally accept on
  `Research`/`Open`, and all comments referencing pin-fallback or
  `STRICT_MODE`.

## What gets changed (not deleted)

1. **`src/net/x509.rs`** — add a `pub fn as_static_str(&self) ->
   &'static str` method on `VerifyError`. Each of the 9 variants
   returns a debug-friendly static string of the form
   `"TLS: chain validation failed: <reason>"`:

   ```text
   Parse              -> "TLS: chain validation failed: certificate parse error"
   EmptyChain         -> "TLS: chain validation failed: empty chain"
   UnsupportedSigAlg  -> "TLS: chain validation failed: unsupported signature algorithm"
   HostnameMismatch   -> "TLS: chain validation failed: hostname mismatch"
   NotYetValid        -> "TLS: chain validation failed: certificate not yet valid"
   Expired            -> "TLS: chain validation failed: expired certificate"
   BadSignature       -> "TLS: chain validation failed: bad signature"
   UntrustedRoot      -> "TLS: chain validation failed: untrusted root"
   ChainIncomplete    -> "TLS: chain validation failed: chain incomplete"
   ```

   Strings are specific enough to debug, generic enough to not leak
   peer-side internals. ~15 lines added.

2. **`src/net/tls.rs`** — chain-fail branch collapses from ~62 lines
   to 3:

   ```rust
   crate::net::x509::VerifyOutcome::Err(e) => {
       return Err(e.as_static_str());
   }
   ```

   No fallback paths. Failure means the handshake aborts with the
   verifier's specific reason on the wire as `&'static str`.

3. **`src/net/mod.rs`** — boot-status block (~lines 60-80)
   simplifies. Drop `tls_pinning::PINS.len()` reference and the
   "Populate src/net/x509.rs::TRUST_STORE OR src/net/tls_pinning"
   hint. Replace with one line:
   ```text
   [tls] trust store: 5 CA roots, chain-only auth, hybrid PQ on
   ```

4. **`src/ui/wm.rs`** — TLS pill (lines 499-505): replace the
   `match crate::net::tls_pinning::current_mode()` with a constant
   label. Try `"LOCK/PQ"` first; if it pushes adjacent segments off
   the screen at 1280px width, drop to `"LD/PQ"`. Color stays CYAN.
   No atomic load, no match, no toggle.

5. **`src/ui/wm.rs`** — comment at line 331 ("Status bar 28px: 5
   live-state segments (ENCRYPTED · NET · TLS …)") gets updated to
   note the TLS segment is now a constant indicator, not live state.

6. **`src/net/fetch.rs`** module-level comment block — replace the
   STUMP #94 paragraph with:
   ```text
   All HTTPS goes through fetch_https / fetch_post_https. Strict
   chain validation against TRUST_STORE; hybrid PQ on; no fallback
   trust paths. See DESIGN_TLS_HARDENING.md.
   ```

7. **`src/ui/shell.rs`** — add `cmd_x509_selftest` and a `"x509-selftest"
   dispatch arm. The selftest body is described in the Testing
   section below. ~25 lines added including the dispatch arm.

## Default behavior

After this lands:
- `TLS_HYBRID_ENABLED_FLAG = true` at boot (unchanged — the flag
  already initialized to `true`; the renderer-relax was the only
  thing that flipped it false).
- Chain validation always runs. `verify_chain` is the only path to
  a trusted handshake.
- Any `verify_chain` error path aborts the handshake with
  `Err(e.as_static_str())`.
- No mode atomic. No pin store. No `tls-mode` command. No mode
  toggle in the WM.

## Approximate diff

- ~250-300 lines deleted across 7 modified files (`tls.rs`, `x509.rs`,
  `fetch.rs`, `mod.rs`, `wm.rs`, `shell.rs`, `qemu_boot_smoke.py`)
  plus 1 file deleted whole (`tls_pinning.rs`). Total touched: 8.
- ~50 lines added: `as_static_str` impl on `VerifyError` (~15),
  simplified chain-fail branch (~3), new boot-status line (~1),
  constant WM pill (~3), refreshed `fetch.rs` comment (~5),
  `cmd_x509_selftest` + dispatch (~25).

## Testing & verification

**Layer 1 — build verification (mandatory between phases):**
```bash
cargo check --target aarch64-unknown-none --release
cargo build --release --target aarch64-unknown-none --features gicv3
```
Both must pass at every phase commit. Warning count should not rise.

**Layer 2 — boot smoke (existing harness, with one addition):**
- `scripts/qemu_boot_smoke.py` PASSES.
- New required marker: the boot-status line `[tls] trust store: 5
  CA roots, chain-only auth, hybrid PQ on` appears in serial output.

Zombie-reference checks (e.g. lingering `tls_pinning::` calls,
orphaned `cmd_tls_mode`) belong in the static grep acceptance check
below, not the boot smoke — those are source-code symbols, not serial
output, and the smoke can't see them.

**Layer 3 — new x509 selftest (`cmd_x509_selftest`, dispatched from
shell):**

A kernel-side selftest exercising the new error-mapping path. Two
sub-tests:

1. **Hostname mismatch** — call `verify_chain(root_der, &[],
   b"wrong-host.example")` where `root_der` is one of the 5
   `TRUST_STORE` entries (a real X.509 cert with valid DER and a
   subject that won't match the hostname). Expect
   `VerifyOutcome::Err(VerifyError::HostnameMismatch)`. Verify
   `e.as_static_str()` contains `"hostname mismatch"`.
2. **Bad bytes** — feed truncated DER (`&root_der[..root_der.len() -
   5]`) into `verify_chain`. Expect `VerifyOutcome::Err(VerifyError::
   Parse)`. Verify `e.as_static_str()` contains `"parse error"`.

Selftest prints `[x509-selftest] PASS: <case>` or `FAIL: <case>` per
sub-test to UART. Wired via `"x509-selftest" => cmd_x509_selftest()`
in the shell dispatch.

The selftest validates deterministic verifier errors and
`as_static_str` mapping. The `tls.rs` hard-abort branch (`return
Err(e.as_static_str())` with no fallback) is verified by code
review + grep — no contortion to mock a TLS handshake from inside the
shell. Untrusted-root case skipped (would require a runtime-overrideable
trust store, which contorts the code for one test).

## Acceptance criteria

- ✅ All deletions from "What gets deleted" land cleanly.
- ✅ All changes from "What gets changed" land cleanly.
- ✅ `cargo build --release --target aarch64-unknown-none --features
  gicv3` produces a kernel binary; warning count does not rise from
  the post-no-browser baseline.
- ✅ `scripts/qemu_boot_smoke.py` PASSES with the additional markers.
- ✅ `cmd_x509_selftest` prints PASS for both cases and no FAIL lines
  when run inside booted Sphragis shell.
- ✅ The grep
  ```bash
  rg 'tls_pinning|cmd_tls_mode|fetch_url|fetch_http\b|fetch_post_url|fetch_post_http\b|ResearchModeGuard' src
  ```
  returns empty. (Single-quoted so the shell preserves `\b`.)

## Out of scope

- Adding new HTTPS callers (none exist today; future M2M /
  audit-shipping / license-check work spawns its own threads).
- Populating `TRUST_STORE` with additional CA roots beyond the 5
  already embedded.
- OCSP / CRL revocation.
- A pinned-HTTPS API. If real internal endpoints make the case for
  pinning, design that as a separate explicit module (`tls_pinned.rs`
  or similar) and a separate API (`fetch_pinned_https` or similar).
  Don't reintroduce the mode-toggle pattern.
- Investigating the hybrid PQ handshake bug against major public
  servers. With M2M as the primary use case (endpoints we control),
  the bug doesn't block anything; if it surfaces against a real
  caller's endpoint later, that's its own thread.

## Reversibility

The current pin/mode machinery has been in tree for several
audit-fix iterations and is well-traveled in git history. If
chain-only-strict turns out to be wrong for a use case we haven't
anticipated, reverting the deletion commits is mechanical (one PR
revert per phase). Tag `pre-tls-hardening-2026-05-07` will be
applied at the pre-deletion commit before any code changes land,
matching the no-browser pivot's reversibility convention.

## Implementation plan

A separate plan doc handles the actual phasing (which files in what
order, with `cargo check` between phases) — this design doc is the
*why*, the plan is the *how*.

🦇
