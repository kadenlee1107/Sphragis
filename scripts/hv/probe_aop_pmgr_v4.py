#!/usr/bin/env python3
"""AOP boot — v4: preserve iBoot's DART, use AOPBase for bootargs.

v3 crash cause (hypothesis): dart_aop.initialize() blew away iBoot's
DART mappings for AOP FW segments. When AOP FW ran its FIQ handler
and tried to DMA into __DATA/__OS_LOG, the DART fault cascaded into a
SoC reset. boot_aop_doorbell.py already documented skipping DART
initialize — v3 ignored that.

v4 fix:
  - NO dart.initialize() call anywhere
  - use AOPBase (no DART needed) for bootargs read/update
  - keep dapf_init for fabric protection
  - keep doorbell sequence (+0x1004=0x10, +0x1014=1)

IRQ_EN / CPU_STATUS clarification from v3:
  - +0x48 is CPU_STATUS (per m1n1 hw/asc.py), not IRQ_EN
    (boot_aop_doorbell.py comment is wrong on that label)
  - R_CPU_STATUS bits: RUNNING=0 STOPPED=1 IRQ_NOT_PEND=2
    FIQ_NOT_PEND=3 IDLE=5  (bit 6 meaning unknown)
  - v3 saw the transitions:
      0x6a pre-RUN → 0x68 post-RUN  (STOPPED cleared)
      0x68 → 0x48                    (IDLE cleared — FW running)
      0x48 → 0x40                    (FIQ_NOT_PEND cleared — FIQ taken)
    So FW IS running and FIQ was taken. Reset happens while FW handler
    runs → DART miss was the likely cause.
"""
import os
import pathlib
import struct
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

AOP_BLOB    = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
MTP_BLOB    = ROOT / "firmware/mtp/J604_MtpFirmware.bin"
M1N1_MACHO  = ROOT / "external/m1n1/build/m1n1.macho"

AOP = 0x390600000
MTP = 0x394600000

# ASCWrapV6 reg layout (verified via boot_aop_doorbell.py kext disasm
# + m1n1 hw/asc.py)
CPU_CONTROL  = 0x0044
CPU_STATUS   = 0x0048
FIQ_NMI_CFG  = 0x1004
FIQ_NMI_ARM  = 0x1014
INBOX_CTRL   = 0x8110
OUTBOX_CTRL  = 0x8114
INBOX0       = 0x8800
INBOX1       = 0x8808
OUTBOX0      = 0x8830
OUTBOX1      = 0x8838
EMPTY_BIT    = 1 << 17

MGMT_SET_IOP_POWER = 6


def log(m): print(f"[v4] {m}", flush=True)


def snap(p, base, tag):
    cc = p.read32(base + CPU_CONTROL)
    cs = p.read32(base + CPU_STATUS)
    ic = p.read32(base + INBOX_CTRL)
    oc = p.read32(base + OUTBOX_CTRL)
    ob0 = p.read32(base + OUTBOX0) | (p.read32(base + OUTBOX0 + 4) << 32)
    ob1 = p.read32(base + OUTBOX1) | (p.read32(base + OUTBOX1 + 4) << 32)
    log(f"  [{tag}] CC={cc:#x} CS={cs:#x}(run={cs&1} stop={(cs>>1)&1} "
        f"fiqP={(cs>>3)&1^1} idle={(cs>>5)&1}) "
        f"IB={ic:#x} OB={oc:#x} OB0={ob0:#x} OB1={ob1:#x}")


def ring_doorbell(p, base):
    p.write32(base + FIQ_NMI_CFG, 0x10)
    p.write32(base + FIQ_NMI_ARM, 0x1)


def send_inbox(p, base, msg0, msg1):
    p.write32(base + INBOX0 + 0, msg0 & 0xffffffff)
    p.write32(base + INBOX0 + 4, (msg0 >> 32) & 0xffffffff)
    p.write32(base + INBOX1 + 0, msg1 & 0xffffffff)
    p.write32(base + INBOX1 + 4, (msg1 >> 32) & 0xffffffff)


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
        log(f"  {tag} {nm}: {m[4]}B -> {a['phys']:#x} "
            f"({(time.time()-t0)*1000:.0f}ms)")
    return adt_by, names


def poll_outbox(p, base, timeout, tag="OB"):
    msgs = []
    deadline = time.time() + timeout
    last_state = None
    while time.time() < deadline:
        try:
            oc = p.read32(base + OUTBOX_CTRL)
            cs = p.read32(base + CPU_STATUS)
        except Exception as e:
            log(f"  {tag}: proxy error — Mac may have reset: {type(e).__name__}")
            return msgs
        state = (oc, cs)
        if state != last_state:
            t = timeout - (deadline - time.time())
            log(f"  {tag} t={t:5.2f}s CS={cs:#x} OB_CTRL={oc:#x}")
            last_state = state
        if not (oc & EMPTY_BIT):
            m0 = p.read32(base + OUTBOX0) | (p.read32(base + OUTBOX0 + 4) << 32)
            m1 = p.read32(base + OUTBOX1) | (p.read32(base + OUTBOX1 + 4) << 32)
            type_ = (m0 >> 52) & 0xff
            log(f"  *** {tag} MSG m0={m0:#x} m1={m1:#x} "
                f"(TYPE={type_:#x}  ep={m1 & 0xff}) ***")
            msgs.append((m0, m1))
            time.sleep(0.005)
        time.sleep(0.02)
    return msgs


def main():
    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    if os.environ.get("SPHRAGIS_SKIP_BOOTSTRAP", "0") != "1":
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
        log(f"  FAIL: {type(e).__name__}: {e}")

    # === STEP 2: Stage AOP firmware ===
    log("\n=== STEP 2: stage AOP firmware ===")
    aop_adt_by, aop_names = stage_fw(iface, u, "/arm-io/aop", AOP_BLOB, "aop")

    # === STEP 3: bootargs update via AOPBase (no DART) ===
    log("\n=== STEP 3: update_bootargs via AOPBase ===")
    try:
        from m1n1.fw.aop.base import AOPBase
        aopb = AOPBase(u)
        aopb.update_bootargs({'p0CE': 0x20000, 'laCn': 0x0,
                              'tPOA': 0x1, 'gila': 0x80})
        log("  update_bootargs OK")
    except Exception as e:
        log(f"  update_bootargs FAIL: {type(e).__name__}: {e}")

    # === STEP 4: Pre-RUN snap + reset OUTBOX ===
    snap(p, AOP, "pre-everything")
    p.write32(AOP + OUTBOX_CTRL, 0x20001)
    snap(p, AOP, "post-ob-reset")

    # === STEP 5: CC.RUN=1 ===
    log("\n=== STEP 5: CC.RUN=1 ===")
    p.write32(AOP + CPU_CONTROL, 0x10)
    time.sleep(0.2)
    snap(p, AOP, "post-RUN")

    # Quick poll before doorbell — FW might Hello on its own
    log("  pre-doorbell poll 0.5s (maybe FW Hellos by itself)...")
    msgs = poll_outbox(p, AOP, 0.5, "pre-door")
    if msgs:
        log(f"  *** FW SELF-HELLO: {len(msgs)} msgs ***")

    # === STEP 6: SetIOPPower + doorbell ===
    log("\n=== STEP 6: SetIOPPower(0x220) + doorbell ===")
    msg0 = (MGMT_SET_IOP_POWER << 52) | 0x220
    msg1 = 0
    log(f"  INBOX msg0={msg0:#x} msg1={msg1:#x}")
    send_inbox(p, AOP, msg0, msg1)
    snap(p, AOP, "post-send")
    ring_doorbell(p, AOP)
    log("  doorbell (+0x1004=0x10, +0x1014=1)")
    snap(p, AOP, "post-doorbell")

    # === STEP 7: Poll 10 s ===
    log("\n=== STEP 7: poll OUTBOX 10 s ===")
    msgs = poll_outbox(p, AOP, 10.0, "AOP-OB")

    try:
        snap(p, AOP, "final")
    except Exception as e:
        log(f"  final snap failed: {e} (Mac may be down)")

    aop_hellod = len(msgs) > 0
    log(f"\nTotal OUTBOX msgs: {len(msgs)}")
    if aop_hellod:
        log("*** AOP RESPONDED ***")
        for m0, m1 in msgs:
            log(f"    msg0={m0:#x} msg1={m1:#x} TYPE={(m0>>52)&0xff:#x}")
    else:
        log("AOP silent — FW crashed or needs different startup")

    os._exit(0 if aop_hellod else 2)


if __name__ == "__main__":
    main()
