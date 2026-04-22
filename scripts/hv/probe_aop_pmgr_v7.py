#!/usr/bin/env python3
"""v7: 3-phase message probe — Ping vs SetIOPPower vs empty msg.

One boot cycle, 3 phases:
  A: Mgmt_Ping (TYPE=3) + doorbell; poll 3 s for Pong (TYPE=4).
     If Pong: handler is alive, just SetIOPPower specifically stalls.
  B: Zero INBOX + doorbell (malformed msg); poll 3 s.
     If handler replies with an error or skips cleanly, FIQ/error
     paths are OK. If it stalls, dispatch code is dying.
  C: SetIOPPower(0x220) + doorbell (reproduces v4); poll 3 s.
     Baseline for comparison — should stall.

Between phases: re-read INBOX state to see if prior msg was drained.
"""
import os, pathlib, struct, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

AOP_BLOB    = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
M1N1_MACHO  = ROOT / "external/m1n1/build/m1n1.macho"
AOP = 0x390600000

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


def log(m): print(f"[v7] {m}", flush=True)


def snap(p, tag):
    cc = p.read32(AOP + CPU_CONTROL)
    cs = p.read32(AOP + CPU_STATUS)
    ic = p.read32(AOP + INBOX_CTRL)
    oc = p.read32(AOP + OUTBOX_CTRL)
    log(f"  [{tag}] CC={cc:#x} CS={cs:#x} IB={ic:#x} OB={oc:#x}")
    return cc, cs, ic, oc


def ring(p):
    p.write32(AOP + FIQ_NMI_CFG, 0x10)
    p.write32(AOP + FIQ_NMI_ARM, 0x1)


def send(p, msg0, msg1):
    p.write32(AOP + INBOX0 + 0, msg0 & 0xffffffff)
    p.write32(AOP + INBOX0 + 4, (msg0 >> 32) & 0xffffffff)
    p.write32(AOP + INBOX1 + 0, msg1 & 0xffffffff)
    p.write32(AOP + INBOX1 + 4, (msg1 >> 32) & 0xffffffff)


def poll(p, secs, tag):
    msgs = []
    deadline = time.time() + secs
    last = None
    while time.time() < deadline:
        try:
            oc = p.read32(AOP + OUTBOX_CTRL)
            cs = p.read32(AOP + CPU_STATUS)
            ic = p.read32(AOP + INBOX_CTRL)
        except Exception as e:
            log(f"  {tag} proxy-err: {e!r}")
            return msgs, True  # crashed
        state = (oc, cs, ic)
        if state != last:
            t = secs - (deadline - time.time())
            log(f"    t={t:5.2f}s CS={cs:#x} OB={oc:#x} IB={ic:#x}")
            last = state
        if not (oc & EMPTY_BIT):
            m0 = p.read32(AOP + OUTBOX0) | (p.read32(AOP + OUTBOX0 + 4) << 32)
            m1 = p.read32(AOP + OUTBOX1) | (p.read32(AOP + OUTBOX1 + 4) << 32)
            log(f"    *** {tag} OUT m0={m0:#x} m1={m1:#x} TYPE={(m0>>52)&0xff:#x} ***")
            msgs.append((m0, m1))
            time.sleep(0.005)
        time.sleep(0.02)
    return msgs, False


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
    except Exception as e:
        log(f"dapf: {e!r}")

    # Stage FW
    aop_node = u.adt["/arm-io/aop"]
    sr = getattr(aop_node, "segment-ranges", None)
    names_raw = getattr(aop_node, "segment-names", b"")
    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_by = dict(zip(names, parse_adt_segments(sr)))
    macho = AOP_BLOB.read_bytes()
    mc_by = {seg[0]: seg for seg in macho_segs_fn(macho)}
    skip = set()
    for nm in ("__TEXT", "__ETEXT"):
        if nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        ok = True
        for off in (0x0, 0x100, 0x200):
            if off >= m[4]: break
            exp = macho[m[3]+off:m[3]+off+16]
            got = iface.readmem(a["phys"]+off, 16)
            if got != exp: ok = False; break
        if ok: skip.add(nm)
    for nm in names:
        if nm in skip or nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0 or m[4] > a["size"]: continue
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], macho[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], macho[m[3]:m[3]+m[4]])
    log("FW staged")

    try:
        from m1n1.fw.aop.base import AOPBase
        AOPBase(u).update_bootargs({'p0CE': 0x20000, 'laCn': 0,
                                    'tPOA': 1, 'gila': 0x80})
        log("bootargs updated")
    except Exception as e:
        log(f"bootargs: {e!r}")

    p.write32(AOP + OUTBOX_CTRL, 0x20001)
    snap(p, "pre-RUN")
    p.write32(AOP + CPU_CONTROL, 0x10)
    time.sleep(0.2)
    snap(p, "post-RUN")

    # ------------------ PHASE A: Mgmt_Ping (TYPE=3) ------------------
    log("\n=== PHASE A: Mgmt_Ping (TYPE=3) + doorbell ===")
    ping_msg0 = (3 << 52)   # TYPE=3, no payload
    ping_msg1 = 0           # EP=0 (mgmt)
    send(p, ping_msg0, ping_msg1)
    snap(p, "A post-send")
    ring(p)
    snap(p, "A post-ring")
    msgs_a, crashed_a = poll(p, 3.0, "A")
    log(f"  PHASE A: {len(msgs_a)} msgs, crashed={crashed_a}")
    if crashed_a:
        os._exit(3)

    # ------------------ PHASE B: empty msg + doorbell ------------------
    log("\n=== PHASE B: empty INBOX (zeros) + doorbell ===")
    send(p, 0, 0)
    snap(p, "B post-send")
    ring(p)
    snap(p, "B post-ring")
    msgs_b, crashed_b = poll(p, 3.0, "B")
    log(f"  PHASE B: {len(msgs_b)} msgs, crashed={crashed_b}")
    if crashed_b:
        os._exit(3)

    # ------------------ PHASE C: SetIOPPower (baseline) ------------------
    log("\n=== PHASE C: SetIOPPower(0x220) + doorbell (baseline) ===")
    sip_msg0 = (6 << 52) | 0x220
    send(p, sip_msg0, 0)
    snap(p, "C post-send")
    ring(p)
    snap(p, "C post-ring")
    msgs_c, crashed_c = poll(p, 3.0, "C")
    log(f"  PHASE C: {len(msgs_c)} msgs, crashed={crashed_c}")

    log("\n=== SUMMARY ===")
    log(f"  A (Ping):        {len(msgs_a)} msgs")
    log(f"  B (zero msg):    {len(msgs_b)} msgs")
    log(f"  C (SetIOPPower): {len(msgs_c)} msgs")
    for tag, msgs in [("A", msgs_a), ("B", msgs_b), ("C", msgs_c)]:
        for m0, m1 in msgs:
            log(f"    {tag}: m0={m0:#x} m1={m1:#x} TYPE={(m0>>52)&0xff:#x}")

    total = len(msgs_a) + len(msgs_b) + len(msgs_c)
    os._exit(0 if total else 2)


if __name__ == "__main__":
    main()
