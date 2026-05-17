# Wave 3 — Pre-flight Kernel API Resolutions

Resolves the 5 kernel API gaps from `docs/superpowers/specs/2026-05-14-caves-manager-design.md` §"Kernel API gaps". Each gap is marked **EXISTS**, **STUB IN TASK 2**, or **DEFER TO WAVE 4+**.

---

## Gap 1: NET MODE per cave

**Resolution:** STUB IN TASK 2

**Investigation:**

- Grep (`grep -nE 'net_mode|NetMode|set_net|cave_net' src/caves/cave.rs src/net/mod.rs`): zero results.
- `src/net/mod.rs` line 67: `pub fn is_isolated() -> bool { true } // Wave 2 stub` — global boolean, not per-cave.
- `src/net/cave_policy.rs` exists and has per-cave egress-rule tables (`CavePolicy`, `set_policy`, `check`), but this is a whitelist/firewall concept, not a mode enum (Isolated / Filtered / Open).
- `Cave` struct (lines 83–160 of `cave.rs`) has no `net_mode` field.

**Decision rationale:** The global `is_isolated()` stub was always meant as temporary. Per-cave net mode is the right shape — we already have per-cave egress policy in `cave_policy`, so adding a coarse mode enum alongside it is low-risk. A `NetMode { Isolated, PolicyFiltered, Open }` enum lets the TUI show a meaningful per-cave indicator.

**Implications for Task 2:** Add to `src/caves/cave.rs`:
```rust
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum NetMode { Isolated = 0, PolicyFiltered = 1, Open = 2 }
```
Add `pub net_mode: NetMode` to `Cave` struct. Initialize to `NetMode::Isolated` in `Cave::empty()` and in `create()`. Add:
```rust
pub fn set_net_mode_by_name(name: &str, mode: NetMode) -> Result<(), &'static str>
pub fn net_mode_of(cave_id: u16) -> NetMode
```
Both follow the same pattern as `set_sensitivity_by_name` / `sensitivity_of`.

**Implications for Tasks 10–12:**
- Task 10 (detail-view paint): read `cave.net_mode` directly from the `&Cave` reference passed by `cave::list`.
- Task 12 (Configure form apply): call `cave::set_net_mode_by_name(name, mode)`. The form presents `[I]solated / [F]iltered / [O]pen` with a cycle key.

---

## Gap 2: MOUNT editing

**Resolution:** DEFER TO WAVE 4+  *(mount field is read-only in the Configure form)*

**Investigation:**

- Grep (`grep -nE 'set_mount|mount_override|cave_mount' src/caves/cave.rs`): zero results.
- `Cave` struct has no `mount_override` field.
- The mount concept in `cave.rs` is the **namespace prefix** used by SealFS: `active_mount_prefix()` (line 983) concatenates the cave name with `:` to scope per-cave file keys. It is derived from the cave name at access time — there is no configurable override and no stored mount path field.
- `create()` (line 1287) sets `name`, `sensitivity`, `integrity`, memory quota, `fs_key` — no mount path is written.

**Decision rationale:** Mount scoping is implicit-by-name, not a stored editable field. Adding `mount_override` would require threaded changes in `active_mount_prefix()` and SealFS key derivation, which is out of scope for Wave 3. The Configure form should show MOUNT as `<cave-name>:` (derived display) with no edit widget.

**Implications for Task 2:** Nothing to add to the kernel. Task 12 (Configure form apply): skip the MOUNT field entirely — mark it visually as read-only/derived in the form design.

**Implications for Tasks 10–12:**
- Task 10 (detail-view paint): display `<name>:` as the mount prefix (compute inline from `cave.name_str()`).
- Task 12: no setter call for mount.

---

## Gap 3: TAINT setter

**Resolution:** EXISTS

**Investigation:**

- `src/caves/taint.rs` does **not exist** as a standalone file; taint lives inline in `src/caves/cave.rs` starting at line 747.
- Exact functions found (lines 776–822):

| Function | Signature | Notes |
|----------|-----------|-------|
| `taint_of` | `pub fn taint_of(cave_id: u16) -> u32` | reads from `CAVE_TAINT[i]` side-table |
| `add_taint` | `pub fn add_taint(cave_id: u16, bits: u32)` | OR-accumulates, idempotent |
| `set_taint` | `pub fn set_taint(cave_id: u16, bits: u32)` | clobbers; admin-only path |
| `active_taint` | `pub fn active_taint() -> u32` | current cave |
| `active_add_taint` | `pub fn active_add_taint(bits: u32)` | called from `sealfs::ns_read` |
| `clear_all_taints` | `pub fn clear_all_taints()` | selftest + admin reset-all |

The spec mentions `taint::stamp(cave_id, value)` — this does **not match** any actual function name. The correct setter is `cave::set_taint(cave_id: u16, bits: u32)` (no return value, `-> ()`). Tasks 11 and 12 must call `cave::set_taint`, not `taint::stamp`.

Taint is stored in a **side-table** `static CAVE_TAINT: [AtomicU32; MAX_CAVES]` (line 772), not as a `Cave` struct field. Task 10's paint code must call `cave::taint_of(cave_id)` with the cave's array index cast to `u16`, not read a struct field.

**Implications for Task 2:** Nothing to stub — everything needed exists.

**Implications for Tasks 10–12:**
- Task 10 (detail-view paint): call `cave::taint_of(cave_id as u16)` — requires knowing the cave's index. The index is the loop variable inside `cave::list`, which yields `&Cave` but not the index. **Fix**: the paint loop must use an enumeration wrapper or call `cave::find_id(name)` to get the index after iterating. Simplest approach: `find_id(cave.name_str()).unwrap_or(usize::MAX) as u16` called once per cave in the list callback.
- Task 12 (Configure form apply): call `cave::set_taint(id as u16, value)` for a full clobber, or `cave::add_taint(id as u16, bits)` for additive semantics.
- Note: `set_taint` is the admin clobber path; the form should use it (not `add_taint`) when the user explicitly sets a taint value.

---

## Gap 4: Cave rename

**Resolution:** DEFER TO WAVE 4+  *(NAME is read-only in the Configure form)*

**Investigation:**

- Grep (`grep -nE 'pub fn rename|set_name|cave_rename' src/caves/cave.rs`): zero results.
- No rename API exists anywhere in `src/caves/`.
- `find_id`, `find_mut`, and all internal lookups key on `name_str()` equality; a rename would require a table walk + SealFS manifest re-write + re-key of `fs_key`.

**Decision rationale:** No API exists; adding one is non-trivial (SealFS persistence, audit record, potential active-cave guard). Spec already anticipated this — "Rename cave (depends on kernel API)" is listed under §"What's NOT in v1". No change.

**Implications for Task 2:** Nothing to add.

**Implications for Tasks 10–12:** Task 12 (Configure form): NAME field is display-only. Render it without an edit cursor; add a `(read-only)` annotation or grey it out per the Wave-2 palette.

---

## Gap 5: `cave::list` signature

**Resolution:** EXISTS

**Investigation:**

- Line 2034: `pub fn list<F: FnMut(&Cave)>(mut callback: F)`
- Implementation iterates `CAVES[0..MAX_CAVES]`, skips `CaveState::Free` slots, and calls `callback(&CAVES[i])` for each active cave.

The callback receives `&Cave` — a read-only borrow of the struct. The loop index `i` is **not** passed to the callback, which is the taint-of blocker documented in Gap 3 above.

**Access notes for Tasks 10–12:** Paint code iterating via `cave::list` gets `&Cave` with these directly accessible fields:

| Field | Type | Access |
|-------|------|--------|
| `state` | `CaveState` | `pub` direct; compare to `CaveState::Running` / `CaveState::Stopped` |
| `cave_type` | `CaveType` | `pub` direct |
| `name` | `[u8; MAX_NAME]` | `pub` direct; use `cave.name_str()` (method) for `&str` |
| `name_len` | `usize` | `pub` direct |
| `sensitivity` | `u8` | `pub` direct; wrap with `Sensitivity::from_u8(cave.sensitivity)` for enum |
| `integrity` | `u8` | `pub` direct; wrap with `Integrity::from_u8(cave.integrity)` for enum |
| `mem_quota_pages` | `u32` | `pub` direct |
| `mem_used_pages` | `AtomicU32` | `pub`; read via `.load(Ordering::Relaxed)` |
| `cpu_ticks` | `AtomicU64` | `pub`; read via `.load(Ordering::Relaxed)` |
| `net_tx_bytes` | `AtomicU64` | `pub`; read via `.load(Ordering::Relaxed)` |
| `net_rx_bytes` | `AtomicU64` | `pub`; read via `.load(Ordering::Relaxed)` |
| `net_mode` | `NetMode` | **not yet** — added by Task 2 stub |
| `taint` | not a field | side-table; call `cave::taint_of(find_id(cave.name_str()).unwrap() as u16)` |

There is **no `is_running()` method**. State checks are done inline: `cave.state == CaveState::Running`.

**Paint-frame call frequency:** `cave::list` iterates MAX_CAVES slots (currently 16 based on usage patterns). Safe to call on every frame — no heap allocation, O(MAX_CAVES) scan. No caching needed for Wave 3.

---

## Cave struct field reference

Complete fields of `pub struct Cave` (lines 83–160 of `src/caves/cave.rs`), for Tasks 10–12:

| Field | Type | Access | Notes |
|-------|------|--------|-------|
| `state` | `CaveState` | pub direct | `Free / Stopped / Running / Destroyed`; no `is_running()` method |
| `cave_type` | `CaveType` | pub direct | `Persistent / Ephemeral` |
| `name` | `[u8; MAX_NAME]` | pub direct | use `.name_str()` for `&str` |
| `name_len` | `usize` | pub direct | |
| `tools` | `[CaveTool; MAX_TOOLS]` | pub direct | not needed by caves_mgr TUI |
| `tool_count` | `usize` | pub direct | |
| `caps` | `[CaveCap; MAX_CAPS]` | pub direct | not needed by caves_mgr TUI |
| `cap_count` | `usize` | pub direct | |
| `fs_key` | `[u8; 32]` | pub direct | never display; used by SealFS only |
| `display_x/y/w/h` | `u32` | pub direct | display sandbox allocation |
| `backing` | `CaveBacking` | pub direct | `Native / Docker` |
| `image` | `[u8; MAX_IMAGE]` | pub direct | use `.image_str()` for `&str` |
| `image_len` | `usize` | pub direct | |
| `cave_l1_phys` | `usize` | pub direct | MMU; not needed by TUI |
| `cave_l1_slot` | `usize` | pub direct | MMU; not needed by TUI |
| `mem_quota_pages` | `u32` | pub direct | quota in 4 KiB pages |
| `mem_used_pages` | `AtomicU32` | pub direct | `.load(Relaxed)` |
| `cpu_ticks` | `AtomicU64` | pub direct | `.load(Relaxed)` |
| `net_tx_bytes` | `AtomicU64` | pub direct | `.load(Relaxed)` |
| `net_rx_bytes` | `AtomicU64` | pub direct | `.load(Relaxed)` |
| `sensitivity` | `u8` | pub direct | `Sensitivity::from_u8(cave.sensitivity)` for enum display |
| `integrity` | `u8` | pub direct | `Integrity::from_u8(cave.integrity)` for enum display |
| `net_mode` | `NetMode` | pub direct | **added by Task 2**; read direct |
| `taint` | — | **not a struct field** | call `cave::taint_of(id as u16)`; id from `find_id()` |

Helper functions in `cave.rs` available to Tasks 10–12:

| Function | Signature | Use |
|----------|-----------|-----|
| `cave::list` | `pub fn list<F: FnMut(&Cave)>(callback: F)` | primary enumeration |
| `cave::find_id` | `pub fn find_id(name: &str) -> Option<usize>` | get array index for taint_of |
| `cave::taint_of` | `pub fn taint_of(cave_id: u16) -> u32` | read taint |
| `cave::set_taint` | `pub fn set_taint(cave_id: u16, bits: u32)` | write taint (admin clobber) |
| `cave::sensitivity_of` | `pub fn sensitivity_of(cave_id: u16) -> Sensitivity` | convenience wrapper |
| `cave::integrity_of` | `pub fn integrity_of(cave_id: u16) -> Integrity` | convenience wrapper |
| `cave::set_sensitivity_by_name` | `pub fn set_sensitivity_by_name(name: &str, level: Sensitivity) -> Result<(), &'static str>` | Configure apply |
| `cave::set_integrity_by_name` | `pub fn set_integrity_by_name(name: &str, level: Integrity) -> Result<(), &'static str>` | Configure apply |
| `cave::set_net_mode_by_name` | TBD — added by Task 2 | Configure apply |
| `cave::name_of` | `pub fn name_of(cave_id: u16) -> &'static str` | reverse lookup |
| `cave::count` | `pub fn count() -> usize` | cheap active-cave count |

---

## Summary of resolutions

| Gap | Resolution | One-liner |
|-----|------------|-----------|
| 1 NET MODE | STUB IN TASK 2 | No per-cave NetMode exists; add enum + field + two setters |
| 2 MOUNT editing | DEFER TO WAVE 4+ | Mount is implicit-by-name (SealFS prefix); no editable field; form shows read-only derived value |
| 3 TAINT setter | EXISTS | `cave::set_taint(cave_id: u16, bits: u32)` at line 794; spec name `taint::stamp` is wrong |
| 4 Cave rename | DEFER TO WAVE 4+ | No rename API; NAME is read-only in Configure form |
| 5 `cave::list` | EXISTS | Line 2034 `pub fn list<F: FnMut(&Cave)>(mut callback: F)`; call on every frame |
