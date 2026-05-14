# STUMP #153 — TTBR1_EL1 split (kernel upper-half VAs)

**Status: SHELVED.** Plan locked, code not landed. Pull this off the
shelf when ANY of {SMP bring-up, PAN enforcement, KASLR, ASID-tagged
TLB} gets scheduled — those force the change. Until then, ROI is
poor (~12-15 dev-days for an architectural cleanup that buys nothing
the OS doesn't already have).

## Background

Sphragis today uses ONLY `TTBR0_EL1`. Both kernel and user mappings
live in the same page-table tree. STUMP #145 added per-cave L1s, but
each cave's L1 must include kernel identity mappings (so the kernel
can run when a cave is active).

The proper architecture is the `TTBR0/TTBR1` split:
- **TTBR1**: kernel mappings, upper half of VA space (`0xFFFF_xxxx`)
- **TTBR0**: user mappings, lower half (`0x0000_xxxx`)
- Cave switch swaps TTBR0 only; kernel TTBR1 stays put

What this unlocks:
1. Cleaner cave switches (no kernel identity in every cave L1)
2. PAN (Privileged Access Never): kernel can't accidentally read
   user memory without explicit opt-in
3. ASIDs become straightforward when SMP lands
4. Matches Linux/Darwin/Windows conventions

## Honest scope verdict

| Estimate | Days | Note |
|---|---|---|
| Best case | 8 | Boot+linker+TTBR1+VA helpers land first try |
| Realistic | 12-15 | One full debug cycle on T1SZ / pre-MMU stack |
| Worst case | 20+ | M4 hardware diverges, or hidden `phys==virt` invariant |

For comparison: STUMP #145 was 5-7 days for per-cave L1s with
identity already in place. #153 touches the same plumbing but adds
a coordinate-system change that ripples through every kernel
pointer originating from the frame allocator. Roughly 740 raw-
pointer call sites exist across the tree — a meaningful share
assume PA==KVA today.

## Architectural changes

| Area | Today | After #153 |
|---|---|---|
| Kernel link addr | `0x40200000` (PA=VA) | `0xFFFF_8000_4020_0000` (KVA), PA still `0x40200000` |
| MMU bring-up timing | OFF until first Linux cave calls `setup_and_enable` | ON within first ~30 instructions of `boot.s` |
| Stack | `__stack_start` referenced via absolute load | Pre-MMU temp stack at PA, then SP reloaded to KVA stack right after `msr sctlr_el1` + `isb` |
| TTBR0 (lower half) | Holds *everything* | User mappings only (per-cave L1) |
| TTBR1 (upper half) | unused | Kernel `.text/.rodata/.data/.bss/.stack` + a linear PA→KVA window |
| Frame allocator | Returns PA, callers use as VA | Returns PA; new `__va(pa)` / `__pa(kva)` helpers in `kernel/mm/`; `frame::alloc()` callers must wrap |
| Cave L1s | L1[1..=4] kernel identity entries | Drop them — kernel reachable via TTBR1 |
| VBAR_EL1 | `exception_vectors` PA via `adrp+lo12` (PC-relative) | Same `adrp` form, resolves to KVA after relink |
| TCR_EL1 | T0SZ=25, T1SZ unset | T0SZ=25 (39-bit user), T1SZ=16 (48-bit kernel) |

## Half-day chunks

| # | Chunk | Files | Verifies | Break risk |
|---|---|---|---|---|
| 1 | Add `__va`/`__pa`/`PHYS_OFFSET` skeleton, KVA constants. NO behavior change. | `kernel/mm/mod.rs`, new `kernel/mm/kva.rs` | Compiles, helpers inlinable | none |
| 2 | Audit `frame::alloc` callers; wrap reads/writes in `__va()`. Still PA==KVA, identity. | `kernel/mm/frame.rs` + every `phys as *mut` site | Boot still green in QEMU | A site missed → hangs only after split |
| 3 | Rewrite `boot.s`: pre-MMU temp stack at PA, build minimal TTBR0+TTBR1 in static .bss tables, set TCR/MAIR, enable MMU, reload SP from KVA, `br` to kernel_main_kva | `arch/aarch64/boot.s`, new `kernel/mm/early_mmu.rs` | UART "boot:mmu-on" message in upper-half code path | T1SZ wrong → walker faults instantly |
| 4 | Flip `linker.ld` to `0xFFFF_8000_4020_0000`. Verify symbols resolve. | `linker.ld` | `nm` shows kernel symbols at upper-half addrs; QEMU still boots | Boot stub absolute loads need the relink to be coherent before MMU-on |
| 5 | Move `setup_and_enable`'s register-programming bits into a kernel-owned `kernel/mm/mmu.rs`; have cave path call into it for shared TCR/MAIR. Cave keeps only TTBR0 swap. | `batcave/linux/mmu.rs`, `kernel/mm/mmu.rs` (new) | Native caves still enter (post-#145 path unchanged) | Sharing TCR between kernel boot and cave switch is fine — they program identical fields |
| 6 | Drop kernel identity from `setup_native_cave_l1` (L1[1..=4] entries). | `batcave/linux/mmu.rs` | Native cave executes; kernel reentry on exception still works because TTBR1 holds kernel | This is *the* cave-regression risk |
| 7 | Linux ELF cave path — confirm it only programs TTBR0 and never writes TTBR1. | `batcave/linux/mmu.rs::setup_and_enable` | Chromium content_shell loads | If a Linux cave wrote TTBR1, kernel disappears |
| 8 | M4-hardware bring-up bake. C-bit on, atomic test, exception path test. | n/a (test only) | M4 reaches desktop loop | STUMP #59 atomic regression on TTBR1 |

**Chunks 3-4 are atomic** — neither can ship alone. Treat as one
"burn the boats" commit.

## Sharp edges

- **TCR_EL1.T1SZ vs link address** — `0xFFFF_8000_xxxx` requires
  T1SZ = 16 (48-bit, top 16 = sign-extension). Today's code only
  programs T0SZ=25; setting T1SZ wrong = walker faults on first
  kernel access after MMU-on, **before any UART output is reachable**.
  Mitigation: keep MAIR+UART identity-mapped in TTBR0 *only for boot
  chunk 3* until first `puts` lands, then drop.

- **Pre-MMU stack** — `boot.s` today does `ldr x0, =__stack_start`
  which after relink resolves to a KVA. That load runs with MMU
  off, returns the upper-half address, SP gets garbage. Fix in
  chunk 3: load `__stack_start_phys` (a new linker symbol equal to
  `__stack_start - KERNEL_VA_BASE`) for the temporary stack, then
  reload after MMU-on.

- **Self-mapping** — Optional. Linux's recursive index-511 trick is
  nice for editing tables in place but adds a slot to every L1.
  Recommendation: skip in v1, walk tables explicitly through
  `__va(pte_phys)`. Add later if needed.

- **STUMP #59 atomics on M4** — C-bit dependency is orthogonal to
  TTBR1, but a relink that subtly changes which page atomics sit
  on can re-trigger it. Run the M4 atomic stress harness in
  chunk 8.

- **boot.s ~50-instruction budget** — current stub is ~30 insns.
  Adding TTBR setup needs maybe 25 more (load 2 table addrs, MAIR,
  TCR, TTBR0, TTBR1, SCTLR, ISB, SP reload, branch). Tight but
  doable. The L1/L2 tables themselves should be **compile-time**
  static `.bss`-resident arrays the boot stub only fills 4 PTEs
  in, not allocates.

- **Cave's `setup_and_enable` reentry** — V2-NEW-026 path (line
  1090 of `mmu.rs`) bails when SCTLR.M==1. Post-#153, MMU is
  *always* on at cave entry. Cave path becomes pure TTBR0 swap;
  the entire `setup_and_enable` register-programming block becomes
  dead code for the kernel-init use case. Don't delete it yet —
  leave the cave-time TTBR0 swap helper using the same TCR/MAIR
  values for safety symmetry.

- **Early printk** — `uart::puts` uses an absolute MMIO PA. After
  MMU-on, that PA must be mapped in TTBR1 (kernel side) too. Pre-
  #153 the cave path identity-maps it via `setup_and_enable`'s 4
  GiB widen. Post-#153, kernel TTBR1 explicitly maps the UART
  range as device memory.

## Test plan

| Test | How | Pass criterion |
|---|---|---|
| Kernel boots through `kernel_main` | QEMU virt, observe UART | "[sphragis] desktop loop entered", no halt |
| Native cave still enters (post-#145) | Run native cave self-test | Cave exits cleanly, kernel resumes |
| Linux ELF cave loads | Chromium `content_shell` | Process reaches event loop |
| Kernel reachable from cave context | Trigger syscall from inside cave; kernel reads user buffer via `__va(user_pa)` | Returns expected bytes; no walk fault |
| PAN-readiness | `mrs x0, id_aa64mmfr1_el1`; verify `SPAN` field present | Bit accessible (don't enable yet) |
| M4 hardware boot | Ubuntu Claude harness on bare M4 | Same UART trail as QEMU |
| Atomic regression (STUMP #59) | LL/SC stress in cave | No deadlock, no spurious failure |
| TLB hygiene on cave switch | Switch caves rapidly 1000x | No stale TTBR0 mappings |

## Stop point — ROI verdict

**Don't ship right now.** Math:

- **Cost**: 12-15 dev-days realistic, plus a debug tail that every
  downstream STUMP inherits if a `phys as *mut` site got missed.
- **Benefit at this moment**: cleaner cave switches (cosmetic —
  #145 already works), PAN (not enabled yet anyway), ASIDs (no SMP
  yet), industry-standard layout (no external pressure).
- **None of those benefits unblock anything currently planned.**

Forcing function: schedule #153 the half-day BEFORE the first of
{SMP, PAN-enforcement, KASLR, ASID-tagged TLB} starts. That work
cannot be done sanely without it.

**Minimum responsible prep work** (high-leverage, zero behavioral
change): ship chunks 1-2 (KVA helpers + `__va` audit) opportunistically.
That makes the eventual flip a 4-day job instead of 12-15. Buys
optionality at low cost.

## Files involved

| File | Role |
|---|---|
| `src/arch/aarch64/boot.s` | Boot stub. Currently MMU off; needs early MMU enable with TTBR0+TTBR1 split. |
| `src/main.rs` | `kernel_main`. Reads `BootArgsRaw` etc. via PA, then later does work that assumes pointers are valid. |
| `linker.ld` | Kernel link address. Currently `0x40200000`. Becomes `0xFFFF_8000_4020_0000`. |
| `src/batcave/linux/mmu.rs` | Current MMU machinery. Mostly cave-side; the kernel-owned MMU bring-up needs a NEW module. |
| `src/kernel/arch/mod.rs` | Exception vectors (VBAR_EL1). Need upper-half VAs (already PC-relative — relink works). |
| `src/kernel/mm/page_table.rs` | Exists but largely unused; could be the foundation for the kernel's own L1. |
| `src/kernel/mm/frame.rs` | Frame allocator — every `phys as *mut` caller. |
| ~740 raw-pointer call sites | Need audit + `__va()` wrap. |

---

When you pick this up: read `STUMP #145` for the per-cave L1
context first, then start with chunk 1 (KVA helpers — pure refactor)
to get a feel for the codebase before committing to chunks 3-4 (the
atomic boot-stub + linker flip).
