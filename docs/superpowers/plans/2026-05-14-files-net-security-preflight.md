# Wave 4 — kernel API gap pre-flight resolutions

Resolves the 8 kernel API gaps from the spec before implementation
begins. Tasks 1a-1e land any stubs the investigation surfaced; later
tasks assume those stubs exist.

Run on `feat/files-net-security`, parent `main` at `c4fc1c3b`
(plan commit). Baseline build + clippy clean before edits.

## Resolutions

| # | Gap | Resolution | Notes |
|---|-----|-----------|-------|
| 1 | `net::set_isolation` | **STUB IN TASK 1c** | Only `is_isolated() -> bool { true }` Wave-2 stub exists at `src/net/mod.rs:67`. Task 1c adds storage + `set_isolation(bool)`. |
| 2 | net counters (rx/tx/peak/uptime/clear) | **STUB IN TASK 1c** | None exist in `src/net/mod.rs` or `src/net/*.rs`. Task 1c adds full counter set. |
| 3 | net activity ring | **STUB IN TASK 1d** | No `Activity*` types or `activity` module in net. Task 1d adds `src/net/activity.rs` with 256-entry ring. The one hit (`src/net/nat.rs:330` comment "total activity count") is unrelated. |
| 4 | audit chain enumeration | **EXISTS** | `audit::count() -> usize` (`src/security/audit.rs:270`), `audit::recent(buf) -> usize` (`audit.rs:137`), `audit_chain::chain_head() -> [u8; 32]` (`audit_chain.rs:105`), `audit_chain::verify_chain() -> VerifyOutcome` (`audit_chain.rs:252`). Entry struct: `Entry { ts: u64, cat: u8, mlen: u8, msg: [u8; MSG_LEN] }` at `audit.rs:84`. |
| 5 | taint enumeration | **EXISTS** | `cave::taint_of(cave_id: u16) -> u32` at `src/caves/cave.rs:838`. Iterate via `cave::list<F: FnMut(&Cave)>(callback)` at `cave.rs:2106`. SECURITY app builds the system-OR by walking all live caves. |
| 6 | integrity deny counts | **STUB IN TASK 1e** | No `deny_count`/`blp_denies`/`biba_denies`/`te_deny` exist anywhere in `src/caves/` or `src/security/`. Task 1e adds `src/security/integrity_counts.rs` with rolling 24h counters. |
| 7 | deadman re-arm + read | **EXISTS** | All needed APIs present in `src/security/deadman.rs`: `arm(hours)` (line 36), `disarm()` (line 84), `check()` (line 91), `remaining()` (line 109), `is_armed()` (line 122), `seconds_remaining()` (line 128, alias for `remaining()`). Task 1b is verification-only — no code change needed. |
| 8 | scaled bitmap font | **EXISTS, but name differs** | `font::draw_str_scaled(fb, screen_w, x, y, s, fg, bg, scale)` at `src/ui/font.rs:324`. Plan called it `draw_str_scale` — use the actual `draw_str_scaled` name. Task 1a is verification-only — no code change needed. |

## Task-list impact

- **Task 1a (font::draw_str_scale)**: collapses to a verification step.
  No new code; widget Task 2c calls `font::draw_str_scaled` directly.
- **Task 1b (deadman::arm)**: collapses to a verification step. No new
  code; SECURITY app calls `deadman::arm(48)` to re-arm.
- **Task 1c (net counters + set_isolation)**: full implementation.
- **Task 1d (net activity ring)**: full implementation.
- **Task 1e (integrity deny counters)**: full implementation.
- Tasks 2-7: unchanged.

## Other observations

- `cave::list<F: FnMut(&Cave)>` exposes a callback iterator over live
  caves. The widget/SECURITY app reads `taint_of(cave.id)` for each
  visited cave to build the per-cave taint listing and the system-OR.
- Audit chain `Entry` carries `ts: u64` (wall-clock-ish ticks from the
  same clock topbar uses). The widget will render `HH:MM:SS` from
  `ts`, decoded the same way `topbar::Badge::Clock` decodes time.
- `Cave` struct exposes `id: u16`, `name: [u8; MAX_NAME]`,
  `name_len: u8`. Already used in `caves_mgr.rs`. Widget can reuse
  the same conversion.
