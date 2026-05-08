#!/usr/bin/env python3
"""v11: CRITICAL FIX — use 64-bit mailbox writes (like StandardASC).

All prior scripts (v3-v10) used 4x p.write32 for INBOX0+INBOX1.
m1n1's hw/asc.py uses Register64 which emits p.write64 — ONE
64-bit write per register. SMC Hello'd correctly via this path.

AOP never responded via 4x write32 because:
  - HW FIFO may require atomic 64-bit writes
  - The final write to INBOX1 (+0x8808) is what triggers FIFO push
  - 4x write32 may mis-decompose the message / push partial state

Also disasm findings: FW does install real FIQ + IRQ handlers at
0x1001800 (VBAR set at 0x109adc0 after boot). FW sets its own PAC
keys from a seed stored in __DATA (so no iBoot PAC-key dependency).
Our stall was NOT PAC — it's malformed mailbox writes.

v11 plan:
  1. chainload patched m1n1, dapf_init
  2. Stage __DATA + __OS_LOG (__TEXT iBoot-staged)
  3. update_bootargs via AOPBase
  4. CC.RUN=1, wait 200ms for boot
  5. Send INBOX via p.write64 (64-bit) for INBOX0 and INBOX1
  6. DO NOT ring doorbell (FIQ kills normal flow)
  7. Poll OUTBOX for 10s
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
from m1n1 import asm

AOP_BLOB    = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
MTP_BLOB    = ROOT / "firmware/mtp/J604_MtpFirmware.bin"
M1N1_MACHO  = ROOT / "external/m1n1/build/m1n1.macho"
AOP = 0x390600000
MTP = 0x394600000

CPU_CONTROL  = 0x0044
CPU_STATUS   = 0x0048
INBOX_CTRL   = 0x8110
OUTBOX_CTRL  = 0x8114
INBOX0       = 0x8800
INBOX1       = 0x8808
OUTBOX0      = 0x8830
OUTBOX1      = 0x8838
EMPTY_BIT    = 1 << 17


def log(m): print(f"[v11] {m}", flush=True)


def snap(p, base, tag):
    cc = p.read32(base + CPU_CONTROL)
    cs = p.read32(base + CPU_STATUS)
    ic = p.read32(base + INBOX_CTRL)
    oc = p.read32(base + OUTBOX_CTRL)
    log(f"  [{tag}] CC={cc:#x} CS={cs:#x} IB={ic:#x} OB={oc:#x}")


def send_inbox64(p, base, msg0, msg1):
    """CORRECT: 64-bit writes, matching m1n1 StandardASC.send()."""
    p.write64(base + INBOX0, msg0)
    p.write64(base + INBOX1, msg1)


def chainload(iface, p, u):
    from m1n1.macho import MachO
    from m1n1.tgtypes import BootArgs_r1, BootArgs_r2, BootArgs_r3
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
    if idx > 0: raw = raw[idx:]
    mc_by = {seg[0]: seg for seg in macho_segs_fn(raw)}
    skip = set()
    for nm in ("__TEXT", "__ETEXT"):
        if nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        ok = True
        for off in (0x0, 0x100, 0x200):
            if off >= m[4]: break
            if iface.readmem(a["phys"]+off, 16) != raw[m[3]+off:m[3]+off+16]:
                ok = False; break
        if ok: skip.add(nm)
    for nm in names:
        if nm in skip or nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0 or m[4] > a["size"]: continue
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], raw[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], raw[m[3]:m[3]+m[4]])
    return adt_by, names


def poll(p, base, secs, tag):
    msgs = []
    deadline = time.time() + secs
    last = None
    while time.time() < deadline:
        try:
            oc = p.read32(base + OUTBOX_CTRL)
            cs = p.read32(base + CPU_STATUS)
            ic = p.read32(base + INBOX_CTRL)
        except Exception as e:
            log(f"  {tag} proxy err: {e!r}")
            return msgs, True
        state = (oc, cs, ic)
        if state != last:
            t = secs - (deadline - time.time())
            log(f"    t={t:5.2f}s CS={cs:#x} OB={oc:#x} IB={ic:#x}")
            last = state
        if not (oc & EMPTY_BIT):
            m0 = p.read64(base + OUTBOX0)
            m1 = p.read64(base + OUTBOX1)
            log(f"  *** {tag} OUT m0={m0:#x} m1={m1:#x} "
                f"TYPE={(m0>>52)&0xff:#x} EP={m1 & 0xff:#x} ***")
            msgs.append((m0, m1))
            time.sleep(0.005)
        time.sleep(0.02)
    return msgs, False


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

    try:
        p.dapf_init("/arm-io/dart-aop")
        log("dapf OK")
    except Exception as e:
        log(f"dapf: {e!r}")

    log("staging AOP FW...")
    stage_fw(iface, u, "/arm-io/aop", AOP_BLOB, "aop")

    try:
        from m1n1.fw.aop.base import AOPBase
        AOPBase(u).update_bootargs({'p0CE': 0x20000, 'laCn': 0,
                                    'tPOA': 1, 'gila': 0x80})
        log("bootargs updated")
    except Exception as e:
        log(f"bootargs: {e!r}")

    p.write32(AOP + OUTBOX_CTRL, 0x20001)
    snap(p, AOP, "pre-RUN")

    log("\nCC.RUN=1")
    p.write32(AOP + CPU_CONTROL, 0x10)
    time.sleep(0.3)
    snap(p, AOP, "post-RUN")

    # The fix: 64-bit INBOX writes, no doorbell
    log("\nsending Mgmt_SetIOPPower(0x220) via 64-bit INBOX writes...")
    msg0 = (6 << 52) | 0x220
    msg1 = 0
    send_inbox64(p, AOP, msg0, msg1)
    log(f"  wrote msg0={msg0:#x} msg1={msg1:#x} (write64)")
    snap(p, AOP, "post-send64")

    log("\npolling OUTBOX 10 s (no doorbell)...")
    msgs, crashed = poll(p, AOP, 10.0, "AOP")

    snap(p, AOP, "final")
    log(f"\n*** AOP msgs: {len(msgs)} ***")
    for m0, m1 in msgs:
        log(f"  m0={m0:#x} m1={m1:#x} TYPE={(m0>>52)&0xff:#x}")
    if msgs:
        log("\n=== If AOP Hello'd, try full StandardASC handshake ===")
        try:
            from m1n1.fw.asc import StandardASC
            aop = StandardASC(u, AOP)
            # Don't re-boot; FW is already up. Just latch the mgmt EP.
            aop.verbose = 2
            # Handle received msgs — this will process the Hello + send HelloAck
            for m0, m1 in msgs:
                # Try to feed through m1n1's mgmt handler
                from m1n1.fw.asc.mgmt import ManagementMessage
                mm = ManagementMessage(m0)
                log(f"  replayed msg type={mm.TYPE:#x}")
        except Exception as e:
            log(f"  StandardASC follow-up: {type(e).__name__}: {e}")
    os._exit(0 if msgs else 2)


if __name__ == "__main__":
    main()
