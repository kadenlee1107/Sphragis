# M4 Chicken Hunt — pick up here

You are continuing the work to lift Bat_OS's per-cycle session
ceiling on M4 from ~60-96 s to multi-minute. The supervisor at
`scripts/hv/run_hv_forever.sh` already makes resets controllable;
this doc is for the deeper fix that removes the reset itself.

Current ceiling cause is fully characterised — see
`docs/SESSION_JOURNAL.md` 2026-04-20 17:15 entry. Short version:

  - Apple's M4 (H16 / T8132) requires CPU chicken-bit init that
    Apple strips from public XNU and Asahi hasn't reverse-
    engineered.
  - PCPU cluster MMIO at 0x211e00000 is in a retention state at
    boot and SErrors on naive accesses to most offsets. Cluster
    wake via PMGR is needed first.
  - Without those, every "enable APSC / set CPU performance
    state" attempt either UNDEFs (chickens missing) or SErrors
    (cluster retained).

Two concrete paths below. Each has real first commands. Pick
either, both are independent. **You are not blocked.** All the
pieces below are tractable from this Ubuntu host and the live M4.

---

## Path A — IPSW kernelcache disassembly for APPLY_TUNABLES

### What you're recovering

The asm `APPLY_TUNABLES x12, x13, x14` macro at `osfmk/arm64/
start.s:784` in Apple's XNU drop. Apple invokes it but doesn't
include the macro body. The body is a per-MIDR `case ... mrs/msr ...`
table that writes the chicken bits we need for M4 P-cores
(MIDR_PART = 0x53, "M4 Donan P-core") and E-cores (0x52).

In the *compiled* kernelcache, APPLY_TUNABLES expanded inline
into a sequence of constant `mrs`/`msr` instructions per CPU
generation. Disassembling the kernelcache around early boot
recovers the exact M4 sequence.

### Concrete first commands

```bash
# 1. Install ipsw (the IPSW manipulation tool — written by blacktop)
#    https://github.com/blacktop/ipsw — Linux build available.
go install github.com/blacktop/ipsw@latest
# or grab the prebuilt:
curl -sL https://api.github.com/repos/blacktop/ipsw/releases/latest \
  | grep browser_download_url | grep linux_amd64 | head -1 \
  | cut -d'"' -f4 | xargs curl -sL -o /tmp/ipsw.tar.gz
tar -xzf /tmp/ipsw.tar.gz -C /tmp/ ipsw

# 2. Pull a small M4 IPSW. iPad Pro M4 (iPad16,3 / iPad16,4) is
#    smaller than the MacBook Pro M4 IPSW and has the same H16
#    CPU and the same APPLY_TUNABLES sequence. Check what's
#    current:
/tmp/ipsw download ipsw --device iPad16,3 --latest --confirm

# 3. Extract the kernelcache:
/tmp/ipsw kernel extract <iPad_iOS_*.ipsw>
# This drops kernelcache.development.ipad14.iso or similar.

# 4. Decompress + DEC the kernelcache:
/tmp/ipsw kernel dec kernelcache.* -o /tmp/kc_m4

# 5. Find APPLY_TUNABLES expansion. The XNU source shows it's
#    called immediately after `mrs x12, MIDR_EL1` in start.s.
#    In the binary, look for a code sequence that compares x12
#    against 0x52 / 0x53 (M4 E/P core MIDR parts). Use ipsw's
#    disassembler (built on Capstone):
/tmp/ipsw disass /tmp/kc_m4 --symbol _start_first_cpu | head -200
# or for a global APPLY_TUNABLES inline-expansion search:
/tmp/ipsw symbols /tmp/kc_m4 | grep -i tunable
```

### What you're looking for

The expansion is something like:

```
mrs   x12, MIDR_EL1
ubfx  x13, x12, #4, #12      // extract part number
cmp   x13, #0x53             // M4 P-core?
b.ne  1f
mrs   x9, S3_0_C15_C_x       // some HIDx
orr   x9, x9, #BIT(...)
msr   S3_0_C15_C_x, x9
... more msr/mrs pairs ...
1:
cmp   x13, #0x52             // M4 E-core?
... more for E-core ...
```

Each `S3_0_C15_C_x_y` encoding maps to an Apple HID register.
The XNU `osfmk/arm64/proc_reg.h` will tell you what the HID is
for each S3_*_C*_C*_* — partial reference is in
`pexpert/pexpert/arm64/apple_arm64_regs.h`.

### What to do with the recovered values

Translate the M4-specific case into m1n1 C:

```c
// external/m1n1/src/chickens_donan.c (NEW FILE)
void init_t8132_donan_pcore(int rev) {
    // mirror each msr/mrs sequence Apple does, using m1n1's
    // reg_set / reg_clr / reg_mask helpers, with SYS_IMP_APL_*
    // names from cpu_regs.h
    reg_set(SYS_IMP_APL_HIDxx, BIT(yy));
    ...
}
void init_t8132_donan_ecore(int rev) { ... }
```

Then wire into `external/m1n1/src/chickens.c`:
```c
{MIDR_PART_T8132_DONAN_ECORE, "M4 Donan (E core)",
 init_t8132_donan_ecore, &features_m4},
{MIDR_PART_T8132_DONAN_PCORE, "M4 Donan (P core)",
 init_t8132_donan_pcore, &features_m4},
```

### Validation

Chainload patched m1n1 — should cleanly boot to its banner. If
banner prints, chickens are landed. Then uncomment
`cpufreq_init()` in `m1n1_main` (already wired up; see
0cafdaf5 commit) and endurance-test against the supervisor.
Expect ceiling to either jump dramatically (chickens fix it) or
not (next bottleneck is PMGR cluster wake — go to Path B).

---

## Path B — PMGR cluster wake before APSC

### Why this might be enough on its own

Live probe (`scripts/hv/probe_apsc_reg.py`) showed:
  - ECPU cluster MMIO at 0x210e00000 reads cleanly at every
    offset we tried.
  - PCPU cluster MMIO at 0x211e00000 SErrors on first access to
    most offsets, even on the boot CPU which IS a P-core.

The most likely interpretation given XNU H16 flags:
  - `HAS_RETENTION_STATE 1` — clusters enter retention where
    MMIO is gated.
  - There's a PMGR register that controls per-cluster MMIO
    access / wake.

If we can wake the cluster with one PMGR write, the APSC enable
write at cluster+0x200f8 might work without needing chickens at
all. APSC is a fairly orthogonal feature — chickens are about
*per-core* tunables, APSC is about *cluster-level* P-state
enable. Could go either way; only experiment tells us.

### Concrete first commands

```bash
# 1. Use the existing dump_pmgr.py against stock m1n1:
sg dialout -c "M1N1DEVICE=/dev/ttyACM1 \
  /usr/bin/python3 \
  /home/kaden-lee/code/Bat_OS/external/m1n1/proxyclient/tools/dump_pmgr.py" \
  > /tmp/pmgr_dump.txt
# Read the output — it lists every PMGR PS (power-state) device.
# Look for entries named "ecpu", "pcpu", "cpu_cluster",
# "cluster0", "cluster1", "acc_ecpu", "acc_pcpu", or similar.

# 2. Cross-check with ADT — every PMGR device has a parent /
#    associated ADT node:
sg dialout -c "M1N1DEVICE=/dev/ttyACM1 /usr/bin/python3 -c \"
import sys, pathlib
sys.path.insert(0, '/home/kaden-lee/code/Bat_OS/external/m1n1/proxyclient')
from m1n1.setup import *
for dev in u.adt['/arm-io/pmgr'].devices:
    if any(k in dev.name.lower() for k in ('cpu','clu','acc','pmp')):
        print(dev.name, 'psreg=', dev.psreg, 'psidx=', dev.psidx,
              'flags=', dev.flags)
\""

# 3. Write a probe that toggles each candidate device's power
#    state then reads PCPU MMIO + observes whether SError stops:
cat > /tmp/probe_pcpu_wake.py <<'PY'
import sys, pathlib, time
sys.path.insert(0,
  '/home/kaden-lee/code/Bat_OS/external/m1n1/proxyclient')
from m1n1.setup import *

PCPU_BASE = 0x211e00000
TEST_OFF = 0x200f8

# Test current state
def pcpu_safe():
    try:
        p.read64(PCPU_BASE + TEST_OFF)
        return True
    except Exception:
        return False

print(f"PCPU readable from start: {pcpu_safe()}")

# Iterate PMGR devices that look CPU-related and enable each
for dev in u.adt['/arm-io/pmgr'].devices:
    name = dev.name.lower()
    if not any(k in name for k in ('pcpu','pcore','pcl','acc_p',
                                    'cluster1','cl1')):
        continue
    print(f"trying pmgr_adt_power_enable for {dev.name} ...")
    try:
        # m1n1 proxy has pmgr_adt_power_enable:
        ret = p.pmgr_adt_power_enable(f"/arm-io/pmgr/{dev.name}")
        print(f"  ret={ret}, PCPU readable now: {pcpu_safe()}")
    except Exception as e:
        print(f"  enable failed: {e}")
PY

sg dialout -c "M1N1DEVICE=/dev/ttyACM1 /usr/bin/python3 /tmp/probe_pcpu_wake.py"
```

### What success looks like

If you find a PMGR enable that flips PCPU MMIO from SError to
readable, you've cracked it. Then in m1n1 main.c:

```c
if (chip_id == T8132) {
    // Wake PCPU cluster MMIO before we touch its APSC reg.
    pmgr_adt_power_enable("/arm-io/pmgr/<the-device-name-found>");
    set64(0x210e00000UL + 0x200f8, BIT(40));  // ECPU APSC
    set64(0x211e00000UL + 0x200f8, BIT(40));  // PCPU APSC
}
```

### Validation

Same as Path A — patched m1n1 should chainload cleanly past banner,
endurance test should show extended session length. If APSC enable
alone (post-wake) extends session length, we're done — chickens
become an optimization for later.

---

## Tooling already in the tree

```
scripts/hv/run_hv_forever.sh         # supervisor — already shipped
scripts/hv/batos_hv_supervisor.py    # the auto-recovery loop
scripts/hv/batos_hv_interactive.py   # one-shot HV session driver
scripts/hv/probe_m4_watchdogs.py     # ADT walk for wdt-like nodes
scripts/hv/probe_m4_wdt_rates.py     # WDT counter-rate probe
scripts/hv/probe_aop_firmware.py     # /arm-io/aop subtree dump
scripts/hv/probe_aop_state.py        # ASC mailbox + CPU control state
scripts/hv/probe_cpu_cluster.py      # CPU cluster MMIO base probe
scripts/hv/probe_apsc_reg.py         # APSC register read/noop-write probe
scripts/hv/probe_pcpu_map.py         # PCPU cluster MMIO accessibility
docs/m4_re/xnu_H16.h.txt             # XNU's M4 capability flags
docs/m4_re/H15_vs_H16.diff.txt       # M3→M4 CPU feature delta
```

The known-good emergency-rollback tag is `hv-96s-baseline` at
`d9a454f0`. If anything you do crashes the chainload pipeline,
`git checkout hv-96s-baseline -- external/m1n1/src/main.rs
external/m1n1/src/cpufreq.c external/m1n1/src/chickens.c
external/m1n1/src/main.c` is your bail-out (rebuild m1n1 + retest
chainload).

---

## Mindset for picking this back up

Path A is the "follow Apple's homework" path — disassemble the
kernel, copy what they wrote. Path B is the "skip homework, find
the prerequisite" path — wake the cluster, write the bit, see
if APSC alone fixes it.

A is more thorough; B is faster to test. Try B first — one PMGR
write attempt can be tested in ten minutes. If it works, ship it
and circle back to A as a polish step. If B doesn't work, A is
the systematic answer.

Both paths are real engineering work with concrete first commands.
Neither is a research dead-end. Just keep going.
