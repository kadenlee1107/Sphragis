# ROOT-1 Sandbox Wiring — Design Notes

## Status
**Not yet landed.** Requires a design decision before implementation.

## Problem
`setup_cave_pagetable` + `switch_to_cave` in `src/batcave/linux/mmu.rs` are
defined but never called. Every cave runs on the kernel's primary TTBR0_EL1.

Wiring them in isn't a one-liner because of an MMIO layout conflict:
- Our MMIO lives at virtual addresses `0x08000000 .. 0x0A5FFFFF` (blocks
  64, 72, 80, 81, 82 of the user L2)
- The cave's user-code window maps blocks 0..99 (0..200 MB)
- These ranges overlap. If we add MMIO to the cave's L2 (for handler
  access), we punch 5 × 2 MB holes through user code — a 150 MB
  Chromium binary at blocks 64/72/80 would land pages on MMIO.
- If we DON'T add MMIO to the cave's L2, the EL1 exception handler
  faults when it writes to UART (`uart::puts`) while the cave's TTBR0
  is active.

## Three possible fixes

### Option A — Switch TTBR0 on exception entry
On EL0→EL1 transition, save the cave's TTBR0 and install primary.
On eret back, restore cave's TTBR0.

- Pros: minimal code change, MMIO stays where it is
- Cons: ~50 ns TLB flush cost on every syscall; exception vector
  must be carefully ordered (TTBR0 swap + `dsb + isb` before any
  kernel code that touches MMIO/kernel RAM)
- Implementation: modify `src/kernel/arch/aarch64/exceptions.s` and
  the Rust exception handler entry in `src/kernel/arch/mod.rs`

### Option B — Relocate cave user window above MMIO
Map cave user blocks starting at virtual `0x10000000` (block 128+)
so MMIO at blocks 64/72/80/81/82 is below, no overlap.

- Pros: clean separation; per-VA protections natural; no TTBR0 swap
  on every syscall
- Cons: PIE binaries typically expect to start at VA 0; loader must
  rebase. Chromium PIE entry ends up at `0x10000000 + entry_offset`.
  `min_addr` logic in loader already computes offsets relative to
  the lowest PT_LOAD vaddr — this should actually "just work" if
  we pass the desired virtual base in.
- Implementation: `loader::load_elf` grows a `virt_base` parameter;
  `setup_cave_pagetable` places user blocks at a configurable L2
  index; MMIO stays identity-mapped at the original L2 indices.

### Option C — L3 (4 KB) granularity for MMIO blocks
Replace the 5 × 2 MB MMIO blocks in the cave L2 with L3 tables that
map only the specific 4 KB MMIO pages as BLOCK_DEVICE and the
remaining 4 KB pages as BLOCK_USER_RW_EXEC.

- Pros: no VA changes, cave keeps its 0-based layout
- Cons: extra 5 L3 tables per cave (20 KB of metadata); more page
  table walks; complexity
- Implementation: `setup_cave_pagetable` hybrid 2 MB / 4 KB mapping
  at 5 specific indices

## Recommendation
**Option B.** Cleanest separation, minimal runtime cost, sets us up
for future per-segment permissions (cave .text RO+X, cave .data
RW+NX) via L3 granularity later.

Concrete plan:
1. Add a `virt_base: u64` arg to `loader::load_elf` (default 0 for
   legacy, `0x10000000` for Chromium)
2. Pass `virt_base` to `setup_cave_pagetable` so the cave's L2 low
   starts mapping user blocks at the right L2 index
3. Ensure the reloc offset is computed as
   `phys_base - min_addr + virt_base` so PIE binaries land correctly
4. After load, call `switch_to_cave(l1)` just before `execute_with_args`
5. Handler doesn't need changes — MMIO stays at its original VA,
   cave's L2_high kernel map remains identical to primary's

## What retires once this lands
- ATTACK-ESC-002, 003, 004, 011, 012 (direct cross-cave / kernel peek)
- ATTACK-KM-007, 008 (cave PT escape)
- Much of ATTACK-SYS-001, 002, 044 (arbitrary kernel R/W via syscall
  pointer args — still need copy_from_user on top, but cave isolation
  is the first gate)

## Effort
1-2 days for Option B, assuming no unexpected ABI issues with PIE
entry points at non-zero VAs.
