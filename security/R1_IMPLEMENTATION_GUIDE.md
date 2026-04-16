# ROOT-1 Implementation Guide — Step by Step

**Prerequisites:** `src/batcave/linux/mmu.rs::setup_cave_pagetable_at(slot, phys_base, virt_base)` already exists (commit `b927af9f`). This guide is the 1-2 day playbook to finish R1.

## Overview

Goal: every cave runs on its own page table with user VA at `0x10000000`,
isolated from kernel RAM except via the L2_high identity map used by the
exception handler.

## Step 1 — Add `load_elf_rebased` to `src/batcave/linux/loader.rs`

New API:
```rust
pub struct LoadedElf {
    pub virt_entry: u64,     // entry point in cave VA space
    pub phys_base: usize,    // physical base (for setup_cave_pagetable_at)
    pub total_size: usize,   // bytes allocated
    pub virt_base: u64,      // what we rebased to
}

pub fn load_elf_rebased(data: &[u8], virt_base: u64) -> Result<LoadedElf, &'static str>;
```

Implementation: copy `load_elf`'s body but replace the single `reloc_offset`
with two offsets:
```rust
// Where to WRITE during loading (physical, identity-mapped during kernel load).
let patch_offset: i64 = phys_base as i64 - min_addr as i64;

// What VALUE to write in relocations (the VA the binary will see at runtime).
let value_offset: i64 = virt_base as i64 - min_addr as i64;
```

- Segment copy + BSS zero: use `patch_offset` (as before)
- Relocation patch address: `r_offset + patch_offset`
- Relocation value: `r_addend + value_offset`  ← the key change
- `virt_entry`: `entry + value_offset`
- `phys_entry` (unused in cave path): `entry + patch_offset`

Keep existing `load_elf` unchanged — it's the legacy identity-map path.

## Step 2 — Cave slot allocator

Add to `src/batcave/linux/mmu.rs`:
```rust
static CAVE_SLOT_USED: [AtomicBool; MAX_CAVE_PAGETABLES] = ...;

pub fn alloc_cave_slot() -> Option<usize> { ... }
pub fn free_cave_slot(slot: usize) { ... }
```

## Step 3 — Modify `runner::run_chromium`

```rust
pub fn run_chromium(url: &str, argv: &[&str]) -> Result<(), &'static str> {
    // ... blob validation unchanged ...

    const CHROMIUM_VIRT_BASE: u64 = 0x10000000;

    let slot = mmu::alloc_cave_slot().ok_or("no free cave slots")?;
    let loaded = loader::load_elf_rebased(blob, CHROMIUM_VIRT_BASE)?;
    let l1 = mmu::setup_cave_pagetable_at(slot, loaded.phys_base, CHROMIUM_VIRT_BASE)?;

    // Opt into threading BEFORE switch_to_cave — threads::init_main_thread
    // touches kernel-only data structures.
    threads::init_main_thread(loaded.virt_entry, 0);

    mmu::switch_to_cave(l1);
    let r = loader::execute_with_args(loaded.virt_entry, argv);
    mmu::switch_to_primary();  // always restore on return
    mmu::free_cave_slot(slot);
    r
}
```

## Step 4 — Verify exception handler still works

The handler runs at EL1. With cave TTBR0 installed:
- Kernel RAM (0x40000000+) routes through cave's L2_high — identical to primary's L2_high, so `SAVED_FRAME`, `SAVED_STACK`, `.bss`, `.data` all work.
- MMIO (0x08M-0x0AM) routes through cave's L2_low — now present as `BLOCK_DEVICE` after `setup_cave_pagetable_at` was updated to include MMIO identity maps.

No handler changes needed. Verify by booting and running Chromium — if UART stops logging mid-syscall, check the MMIO mappings.

## Step 5 — Backward compat for existing runners

Leave `run_busybox_cmd`, `run_test`, etc. using the legacy `load_elf` +
primary TTBR0 path. Those binaries are all < 20 MB and don't collide
with MMIO. Migrating them is a separate task.

## Step 6 — Test matrix

| Test | Expected |
|---|---|
| `./run.sh` boots to shell | ✅ (no regression) |
| `shell> blink` runs Blink tokenizer test | ✅ (legacy path unchanged) |
| `shell> v8` runs V8 bytecode test | ✅ (legacy path unchanged) |
| `shell> csstok` runs CSS tokenizer | ✅ (legacy path unchanged) |
| `shell> chromium https://example.com` (once Docker build finishes) | should load and begin rendering; sandboxed in its own page table |

## Step 7 — Apply R2 copy_from_user tightening

Once R1 is live, `is_user_ptr` in syscall.rs can be upgraded from a
coarse `0x1000..0x40000000` check to an exact walk of the caller's
cave L2_low. Pseudo-code:
```rust
pub fn is_user_ptr_exact(cave_slot: usize, p: usize, size: usize) -> bool {
    let l2_low = mmu::cave_l2_low(cave_slot);
    // Walk each 2 MB block between [p, p+size) and verify it's mapped
    // in L2_low (i.e. user-owned) and NOT a device mapping.
    ...
}
```

## Estimated effort

- Step 1 (loader rebase): 3-4 hours
- Step 2 (slot allocator): 30 min
- Step 3 (runner wiring): 1 hour
- Step 4 (test/debug): 4-8 hours (this is where surprises land)
- Step 5 (compat verification): 2 hours
- Step 7 (R2 upgrade): 2 hours

Realistic: 1-2 dedicated sessions.

## Risks

- PIE binaries may embed assumptions about VA = 0 (unlikely — PIE is
  position-independent by definition, but worth checking Chromium's
  own early init).
- Stack pointer setup in `execute_with_args` — the stack buffer comes
  from the frame allocator (physical addrs). In cave VA space it must
  be mapped. Either map stack frames into the cave page table at a
  user-space VA, or keep the user stack in the same physical/virtual
  mapping as the ELF binary (i.e. allocate stack pages contiguous with
  the ELF's physical range, so they fall inside the cave's user window).
- Exception return after `switch_to_cave` uses `elr_el1` which points
  into cave VA — confirm that `elr_el1` is a VA (it is, per ARM64).

## Rollback

If anything breaks, revert these commits and existing binaries still
run on the primary page table. The legacy identity-map path is
untouched.
