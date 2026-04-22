#!/usr/bin/env python3
"""AOP doorbell-ring boot — v3.

v1/v2 findings:
  - reg[3] of /arm-io/aop at 0x3882A8000 is NOT a PMGR device-enable
    slot — it's a 24 MHz always-on timer/counter register (value
    increments monotonically ~24M/s across reads). Previous
    "PMGR write" attempts had no effect because the reg ignores
    writes; reads are just capturing the timer.
  - AOP's ASC regs at 0x390600000 ARE accessible immediately (no
    power gate to kick). What we were missing:
      * dapf_init("/arm-io/dart-aop") — fabric protection config
      * doorbell ring after INBOX write (+0x1004=0x10, +0x1014=1)
    These are documented via disasm in scripts/hv/boot_aop_doorbell.py.
  - update_bootargs works fine via AOPClient.

This script:
  1. chainload patched m1n1 (WDT fix in C)
  2. dapf_init for dart-aop
  3. stage AOP firmware
  4. update_bootargs
  5. CC.RUN=1
  6. send SetIOPPower(0x220) via INBOX0/1 at +0x8800/+0x8808
  7. RING DOORBELL (+0x1004=0x10 then +0x1014=1)
  8. poll OUTBOX +0x8830/+0x8838 for Hello response
  9. if Hello: chain MTP boot (dartmap flow)

NO SMC panic-scratch writes.
"""
import os, pathlib, struct, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

AOP_BLOB    = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
MTP_BLOB    = ROOT / "firmware/mtp/J604_MtpFirmware.bin"
M1N1_MACHO  = ROOT / "external/m1n1/build/m1n1.macho"

AOP = 0x390600000
MTP = 0x394600000

# ASCWrapV6 layout (verified via kext disasm + m1n1 hw/asc.py)
CPU_CONTROL  = 0x0044  # RUN bit 4
IRQ_EN       = 0x0048
FIQ_NMI_CFG  = 0x1004
FIQ_NMI_ARM  = 0x1014
INBOX_CTRL   = 0x8110  # A2I_CTRL
OUTBOX_CTRL  = 0x8114  # I2A_CTRL
INBOX0       = 0x8800
INBOX1       = 0x8808
OUTBOX0      = 0x8830
OUTBOX1      = 0x8838
EMPTY_BIT    = 1 << 17

# Mgmt endpoint msg types
MGMT_SET_IOP_POWER = 6  # TYPE in bits [59:52]


def log(m): print(f"[v3] {m}", flush=True)


def snap(p, base, tag):
    cc = p.read32(base + CPU_CONTROL)
    ie = p.read32(base + IRQ_EN)
    ic = p.read32(base + INBOX_CTRL)
    oc = p.read32(base + OUTBOX_CTRL)
    ob0 = p.read32(base + OUTBOX0) | (p.read32(base + OUTBOX0 + 4) << 32)
    ob1 = p.read32(base + OUTBOX1) | (p.read32(base + OUTBOX1 + 4) << 32)
    log(f"  [{tag}] CC={cc:#x} IE={ie:#x} IB={ic:#x} OB={oc:#x} "
        f"OB0={ob0:#x} OB1={ob1:#x}")


def ring_doorbell(p, base):
    """ASCWrapV6 doorbell: wakes FW's FIQ handler so it drains INBOX."""
    p.write32(base + FIQ_NMI_CFG, 0x10)
    p.write32(base + FIQ_NMI_ARM, 0x1)


def send_inbox(p, base, msg0, msg1):
    """128-bit INBOX msg. Apple does atomic stp; we do 4x 32-bit writes
    which can be partial but in practice is fine since FW reads only
    after doorbell."""
    p.write32(base + INBOX0 + 0, msg0 & 0xffffffff)
    p.write32(base + INBOX0 + 4, (msg0 >> 32) & 0xffffffff)
    p.write32(base + INBOX1 + 0, msg1 & 0xffffffff)
    p.write32(base + INBOX1 + 4, (msg1 >> 32) & 0xffffffff)


def recv_outbox(p, base):
    """Read pending OUTBOX msg. Returns (msg0, msg1) or (None, None)."""
    oc = p.read32(base + OUTBOX_CTRL)
    if oc & EMPTY_BIT:
        return None, None
    m0 = p.read32(base + OUTBOX0) | (p.read32(base + OUTBOX0 + 4) << 32)
    m1 = p.read32(base + OUTBOX1) | (p.read32(base + OUTBOX1 + 4) << 32)
    return m0, m1


def chainload(iface, p, u):
    from m1n1.macho import MachO
    from m1n1.tgtypes import BootArgs_r1, BootArgs_r2, BootArgs_r3
    from m1n1 import asm
    from m1n1.utils import align
    new_base = u.base
    data = M1N1_MACHO.read_bytes()
    macho = MachO(data)
    image = macho.prepare_image() + b"\x00\x00\x00\x00"
    entry = macho.entry - macho.vmin + new_base
    sepfw_start, sepfw_length = u.adt["chosen"]["memory-map"].SEPFW
    preoslog_start, preoslog_size = 0, 0
    if hasattr(u.adt["chosen"]["memory-map"], "preoslog"):
        preoslog_start, preoslog_size = u.adt["chosen"]["memory-map"].preoslog
    image_size = align(len(image))
    sepfw_off = image_size; image_size += align(sepfw_length)
    preoslog_off = image_size; image_size += align(preoslog_size)
    bootargs_off = image_size; image_size += 0x4000
    image_addr = u.malloc(image_size)
    u.compressed_writemem(image_addr, image, True)
    p.dc_cvau(image_addr, len(image))
    p.memcpy8(image_addr + sepfw_off, sepfw_start, sepfw_length)
    u.adt["chosen"]["memory-map"].SEPFW = (new_base + sepfw_off, sepfw_length)
    u.adt["chosen"]["memory-map"].BootArgs = (new_base + bootargs_off, 0x4000)
    if preoslog_size:
        p.memcpy8(image_addr + preoslog_off, preoslog_start, preoslog_size)
        u.adt["chosen"]["memory-map"].preoslog = (new_base + preoslog_off, preoslog_size)
    u.push_adt()
    tba = u.ba.copy()
    tba.top_of_kernel_data = new_base + image_size
    if tba.revision <= 1:
        iface.writemem(image_addr + bootargs_off, BootArgs_r1.build(tba))
    elif tba.revision == 2:
        iface.writemem(image_addr + bootargs_off, BootArgs_r2.build(tba))
    else:
        iface.writemem(image_addr + bootargs_off, BootArgs_r3.build(tba))
    stub = asm.ARMAsm(f"""
1:      ldp x4, x5, [x1], #16
        stp x4, x5, [x2]
        dc cvau, x2
        ic ivau, x2
        add x2, x2, #16
        sub x3, x3, #16
        cbnz x3, 1b
        ldr x1, ={entry}
        br x1
""", image_addr + image_size)
    iface.writemem(stub.addr, stub.data)
    p.dc_cvau(stub.addr, stub.len)
    p.ic_ivau(stub.addr, stub.len)
    p.reload(stub.addr, new_base + bootargs_off, image_addr, new_base, image_size)
    iface.nop()


def macho_segs_fn(mb):
    ncmds = struct.unpack("<I", mb[16:20])[0]
    cur, segs = 32, []
    for _ in range(ncmds):
        cmd, sz = struct.unpack("<II", mb[cur:cur+8])
        if cmd == 0x19:
            name = mb[cur+8:cur+24].rstrip(b"\x00").decode()
            vm, vmsz, fo, fs = struct.unpack("<QQQQ", mb[cur+24:cur+56])
            segs.append((name, vm, vmsz, fo, fs))
        cur += sz
    return segs


def parse_adt_segments(raw):
    out = []
    for i in range(len(raw) // 32):
        s = raw[i*32:(i+1)*32]
        phys, iova, remap, size = struct.unpack("<QQQI4x", s)
        out.append({"phys": phys, "iova": iova, "size": size})
    return out


def stage_fw(iface, u, node_path, blob_path, tag):
    node = u.adt[node_path]
    sr = getattr(node, "segment-ranges", None)
    if sr is None: return {}, []
    names_raw = getattr(node, "segment-names", b"")
    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_by = dict(zip(names, parse_adt_segments(sr)))
    raw = blob_path.read_bytes()
    idx = raw.find(b"\xcf\xfa\xed\xfe")
    if idx > 0:
        raw = raw[idx:]
    mc_by = {seg[0]: seg for seg in macho_segs_fn(raw)}
    skip = set()
    for nm in ("__TEXT", "__ETEXT"):
        if nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        ok = True
        for off in (0x0, 0x100, 0x200):
            if off >= m[4]: break
            exp = raw[m[3]+off:m[3]+off+16]
            got = iface.readmem(a["phys"]+off, 16)
            if got != exp: ok = False; break
        if ok:
            skip.add(nm)
            log(f"  {tag} {nm}: iBoot-staged (skip)")
    for nm in names:
        if nm in skip or nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0 or m[4] > a["size"]: continue
        t0 = time.time()
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], raw[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], raw[m[3]:m[3]+m[4]])
        log(f"  {tag} {nm}: {m[4]}B -> {a['phys']:#x} ({(time.time()-t0)*1000:.0f}ms)")
    return adt_by, names


def poll_outbox(p, base, timeout, tag="OB"):
    """Poll for any OUTBOX message within `timeout` seconds. Returns list."""
    msgs = []
    deadline = time.time() + timeout
    last_state = None
    while time.time() < deadline:
        oc = p.read32(base + OUTBOX_CTRL)
        ie = p.read32(base + IRQ_EN)
        state = (oc, ie)
        if state != last_state:
            t = timeout - (deadline - time.time())
            log(f"  {tag} t={t:5.2f}s IE={ie:#x} OB_CTRL={oc:#x}")
            last_state = state
        if not (oc & EMPTY_BIT):
            m0, m1 = recv_outbox(p, base)
            log(f"  *** {tag} MSG m0={m0:#x} m1={m1:#x} "
                f"(TYPE={(m0>>52)&0xff:#x}) ***")
            msgs.append((m0, m1))
            # Re-check CTRL to advance queue
            time.sleep(0.005)
        time.sleep(0.02)
    return msgs


def boot_mtp(p, u, iface):
    """Chain MTP boot after AOP is up."""
    log("\n--- MTP boot chain ---")

    # SMC start (dep)
    log("  SMC start...")
    from m1n1.fw.smc import SMCClient
    smc = SMCClient(u, u.adt["arm-io/smc"].get_reg(0)[0])
    smc.start()
    smc.start_ep(0x20)
    log(f"    SMC up: iop={smc.mgmt.iop_power_state:#x} "
        f"ap={smc.mgmt.ap_power_state:#x}")

    # Stage MTP firmware
    mtp_adt_by, mtp_names = stage_fw(iface, u, "/arm-io/mtp", MTP_BLOB, "mtp")

    # DART-MTP
    from m1n1.hw.dart import DART
    log("  DART-MTP setup...")
    dart_mtp = DART.from_adt(u, "/arm-io/dart-mtp",
                             iova_range=(0x8000, 0x100000))
    dart_mtp.dart.regs.TCR[1].set(BYPASS_DAPF=1, BYPASS_DART=0,
                                   TRANSLATE_ENABLE=1)
    dart_mtp.initialize()
    PAGE_SIZE = 0x4000
    def align_up(x, a): return (x + a - 1) & ~(a - 1)
    for nm in mtp_names:
        if nm not in mtp_adt_by: continue
        a = mtp_adt_by[nm]
        phys_a = a["phys"] & ~(PAGE_SIZE - 1)
        iova_a = a["iova"] & ~(PAGE_SIZE - 1)
        size = align_up(a["size"], PAGE_SIZE)
        try:
            dart_mtp.dart.iomap_at(1, iova_a, phys_a, size)
            log(f"    mapped {nm} stream=1 iova={iova_a:#x}")
        except Exception as e:
            log(f"    {nm}: {type(e).__name__}: {e}")

    # DockChannel
    from m1n1.hw.dockchannel import DockChannel
    irq_base  = u.adt["/arm-io/dockchannel-mtp"].get_reg(1)[0]
    fifo_base = u.adt["/arm-io/dockchannel-mtp"].get_reg(2)[0]
    dc = DockChannel(u, irq_base, fifo_base, 1)
    while dc.rx_count:
        dc.read(dc.rx_count)
    log(f"  DockChannel ready")

    # MTP ASC - try our doorbell-based flow instead of StandardASC.start()
    snap(p, MTP, "mtp pre")
    p.write32(MTP + OUTBOX_CTRL, 0x20001)
    p.write32(MTP + CPU_CONTROL, 0x10)
    time.sleep(0.1)
    snap(p, MTP, "mtp post-RUN")
    # SetIOPPower
    send_inbox(p, MTP, (MGMT_SET_IOP_POWER << 52) | 0x220, 0)
    ring_doorbell(p, MTP)
    snap(p, MTP, "mtp post-ring")
    msgs = poll_outbox(p, MTP, 8, "MTP-OB")
    if msgs:
        log(f"  *** MTP RESPONDED: {len(msgs)} msgs ***")
    else:
        log("  MTP silent")


def main():
    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    if os.environ.get("BATOS_SKIP_BOOTSTRAP", "0") != "1":
        log("chainloading patched m1n1...")
        chainload(iface, p, u)
        u = ProxyUtils(p, heap_size=128 * 1024 * 1024)
        log("  patched m1n1 up")

    # === STEP 1: DAPF init ===
    log("\n=== STEP 1: dapf_init(/arm-io/dart-aop) ===")
    try:
        rc = p.dapf_init("/arm-io/dart-aop")
        log(f"  rc={rc}")
    except Exception as e:
        log(f"  FAIL {type(e).__name__}: {e}")

    # === STEP 2: Stage AOP firmware ===
    log("\n=== STEP 2: stage AOP firmware ===")
    aop_adt_by, aop_names = stage_fw(iface, u, "/arm-io/aop", AOP_BLOB, "aop")

    # === STEP 3: bootargs update ===
    log("\n=== STEP 3: update_bootargs ===")
    try:
        from m1n1.hw.dart import DART
        dart_node = u.adt["/arm-io/dart-aop"]
        vm_base = getattr(dart_node, "vm-base", None) or 0x8000
        dart_aop = DART.from_adt(u, "/arm-io/dart-aop",
                                 iova_range=(vm_base, 0x1000000000))
        dart_aop.initialize()
        from m1n1.fw.aop.client import AOPClient
        aop = AOPClient(u, "/arm-io/aop", dart_aop)
        aop.verbose = 0
        keys = list(aop.read_bootargs().keys())
        log(f"  bootargs keys: {keys[:10]}")
        aop.update_bootargs({'p0CE': 0x20000, 'laCn': 0x0,
                             'tPOA': 0x1, 'gila': 0x80})
        log("  updated")
    except Exception as e:
        log(f"  FAIL: {type(e).__name__}: {e}")

    # === STEP 4: reset OUTBOX + snap ===
    p.write32(AOP + OUTBOX_CTRL, 0x20001)
    snap(p, AOP, "pre-RUN")

    # === STEP 5: CC.RUN=1 ===
    log("\n=== STEP 5: CC.RUN=1 ===")
    p.write32(AOP + CPU_CONTROL, 0x10)
    time.sleep(0.2)
    snap(p, AOP, "post-RUN")

    # Quick poll — maybe FW Hellos on its own after RUN
    log("  pre-doorbell poll 0.5s...")
    msgs = poll_outbox(p, AOP, 0.5, "pre-door")
    if msgs:
        log(f"  *** {len(msgs)} msgs BEFORE doorbell — FW self-started ***")

    # === STEP 6: send SetIOPPower + ring doorbell ===
    log("\n=== STEP 6: SetIOPPower(0x220) + doorbell ring ===")
    msg0 = (MGMT_SET_IOP_POWER << 52) | 0x220
    msg1 = 0
    log(f"  INBOX msg0={msg0:#x} msg1={msg1:#x}")
    send_inbox(p, AOP, msg0, msg1)
    snap(p, AOP, "post-send")

    ring_doorbell(p, AOP)
    log("  doorbell rung (+0x1004=0x10, +0x1014=1)")
    snap(p, AOP, "post-doorbell")

    # === STEP 7: Poll 8 s ===
    log("\n=== STEP 7: OUTBOX poll 8 s ===")
    msgs = poll_outbox(p, AOP, 8.0, "AOP-OB")
    snap(p, AOP, "final")

    aop_hellod = len(msgs) > 0
    if aop_hellod:
        log(f"\n*** AOP BOOTED: {len(msgs)} msgs received ***")
        for m0, m1 in msgs:
            log(f"    msg0={m0:#x} msg1={m1:#x} TYPE={(m0>>52)&0xff:#x}")
    else:
        log("\nAOP silent after 8s — no OUTBOX activity")

    # === STEP 8: If AOP up, try MTP ===
    if aop_hellod:
        try:
            boot_mtp(p, u, iface)
        except Exception as e:
            log(f"\nMTP chain FAIL: {type(e).__name__}: {e}")

    os._exit(0 if aop_hellod else 2)


if __name__ == "__main__":
    main()
