# M4 kernelcache RE artifacts

Index + handoff notes for Path A of the M4 chicken hunt (see
`docs/M4_CHICKEN_HUNT.md`). The kernelcache itself (~75 MB) is not
committed — redownload it with the commands below. What IS committed
here is the symbol / string index (decompiled function names from
kext strings sections — stable and small), which is enough to know
WHERE to point a disassembler next session.

## Source IPSW

  - Device: `iPad16,3` (iPad Pro M4 11", same H16/T8132 CPU family
    as MacBook Pro M4; smaller IPSW → smaller kernelcache)
  - Build: `23E254`, Version `26.4.1`, kernel `xnu-12377.102.10~3`
  - Chip: `RELEASE_ARM64_T8132`

## Redownload commands

```bash
# Install ipsw (blacktop/ipsw, v3.1.672 used for this RE)
curl -sL -o /tmp/ipsw.tar.gz \
  https://github.com/blacktop/ipsw/releases/download/v3.1.672/ipsw_3.1.672_linux_x86_64.tar.gz
tar -xzf /tmp/ipsw.tar.gz -C /tmp/ ipsw

# Pull just the kernelcache (not the whole 7 GB IPSW)
mkdir -p /tmp/m4_ipsw && cd /tmp/m4_ipsw
/tmp/ipsw download ipsw --device iPad16,3 --latest --kernel --confirm
# Drops kernelcache.release.iPad16,3_4_5_6 (~75 MB) next to each build dir.

# Verify
/tmp/ipsw kernel version \
  23E254__iPad16,3/kernelcache.release.iPad16,3_4_5_6
# -> Darwin Kernel Version 25.4.0 ... RELEASE_ARM64_T8132
```

## Kernelcache layout

`LC_FILESET` Mach-O with 303 fileset entries. Key entries for the
chicken / APSC hunt:

| fileset entry                              | why we care                                                                 |
|--------------------------------------------|-----------------------------------------------------------------------------|
| `com.apple.kernel`                         | xnu core; has early `_start` trampoline at 0xfffffe000b078000 but chicken init is NOT there (__TEXT_BOOT_EXEC has only 12 mrs/msr — none to Apple HID regs) |
| `com.apple.driver.AppleT8132PMGR`          | T8132-specific PMGR subclass; strings confirm `AppleT8132PMGR::enableAPSC(VoltageRail,bool)` and `_waitAPSCPending(PerfDomainID)` exist |
| `com.apple.driver.ApplePMGR`               | Generic PMGR base class; strings include `enableCPUCluster`, `enableCPUComplex`, `cpu-apsc`, `soc-apsc`, `apsc-snooze`, `apsc-sleep-soc` |
| `com.apple.driver.AppleT8132CLPC`          | CLPC (Closed-Loop Performance Controller). Has raw `mrs cpu_cnt0..4`, `core_nrg_acc_dat`, `pmc0..9`, `cpu_cnt_ctl` — perf counter regs, not chicken bits |
| `com.apple.driver.AppleT8132SOCTuner`      | SoC-level tuning (MCC, DCS, audio LLT) — NOT cpu chicken bits; irrelevant |

## Disassembly commands

```bash
KC=/tmp/m4_ipsw/23E254__iPad16,3/kernelcache.release.iPad16,3_4_5_6

# Whole kernel __TEXT_BOOT_EXEC (32 KB, 5325 instructions)
/tmp/ipsw macho disass "$KC" --fileset-entry com.apple.kernel \
    -x __TEXT_BOOT_EXEC.__bootcode --force --quiet \
    > /tmp/kc_bootcode.txt

# T8132PMGR (the APSC enable entry point)
/tmp/ipsw macho disass "$KC" --fileset-entry com.apple.driver.AppleT8132PMGR \
    -x __TEXT_EXEC.__text --force --quiet \
    > /tmp/kc_t8132pmgr.txt

# ApplePMGR base class (where enableCPUCluster likely lives)
/tmp/ipsw macho disass "$KC" --fileset-entry com.apple.driver.ApplePMGR \
    -x __TEXT_EXEC.__text --force --quiet \
    > /tmp/kc_applepmgr.txt
```

## What Path A found (2026-04-20 18:00 session)

  - __TEXT_BOOT_EXEC has no HID-register writes. Chicken init moved
    off the early boot path on H16. Candidate locations: SPTM (has
    its own `__DATA_SPTM` segment in kernelcache and a separate
    signed blob we haven't extracted), or the kernel's IOKit
    PMGR driver graph executed after IOService matching.
  - `AppleT8132PMGR::enableAPSC(VoltageRail, bool)` is the concrete
    M4-specific APSC entry. Its actual body is reached via PAC-signed
    IOKit vtable dispatch; the disassembly around the string reference
    (offset `0xe9c` in T8132PMGR __TEXT at vaddr 0xfffffe0007692e9c)
    lives in a dense cluster of assertion / logging stubs starting
    at approximately 0xfffffe0009532228.
  - `ApplePMGR::enableCPUCluster(unsigned int)` is the generic
    base-class implementation. That is the function to read for the
    actual MMIO sequence (register offsets, bit masks, poll loops).
  - The strings `cpu-apsc` and `soc-apsc` appear in ApplePMGR's
    __cstring section — they are the exact Apple Device Tree
    property names that gate APSC enable. Stock m1n1 already matches
    on `cpu-apsc` via `pmgr_get_feature()` in `cpufreq.c`.

## Next steps (from here)

  1. In `/tmp/kc_applepmgr.txt`, find `ApplePMGR::enableCPUCluster`
     body. Strategy: the method string is at `0xfffffe000758ce06`,
     so grep for `add x11, x11, #0xe06` in the disasm (already done —
     line 91031, vaddr `0xfffffe00090c48b8`). That address is an
     assertion stub. Follow up:
       - Read the `__DATA_CONST.__auth_got` section for vtable
         entries that point into `0xfffffe00090c4000`-ish range.
       - Or find where `ApplePMGR::enableCPUCluster` is CALLED
         externally — look for its vtable slot in
         `__DATA_CONST.__const` and trace the blraa to the real
         function body.
  2. Alternative faster route: compile an iOS-debug-kernel in
     ghidra/binja with IOKit vtable analysis and jump directly to
     the `enableCPUCluster` / `enableAPSC` function bodies.
  3. Once the MMIO sequence is recovered, port to m1n1's
     `cpufreq.c` T8132 path (currently `set64(cluster->base +
     0x200f8, BIT(40))` which SErrors on real M4 — see `docs/
     SESSION_JOURNAL.md` 2026-04-20 18:00 entry).

## Committed artifacts

  - `AppleT8132PMGR.strings.txt`  — 134-line cstring dump
  - `AppleT8132PMGR.apsc_strings.txt` — apsc/chicken grep subset
  - `ApplePMGR.strings.txt`       — 1743-line cstring dump
  - `ApplePMGR.apsc_strings.txt`  — apsc/chicken grep subset
