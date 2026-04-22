#!/usr/bin/env python3
"""v13: like v11 but NO pre-RUN OB_CTRL reset + fast pre-INBOX poll.

v11 = stage + bootargs + reset-OB + RUN + send64 + poll → FW queued msg
      but no response. (CS=0x48, IB=0x100101, OB=0x20001.)
v12 = stage + bootargs + reset-OB + RUN + NO send → Mac crashes ~50ms.

v13: drop the pre-RUN OB_CTRL reset (0x20001 write). FW may be about
to emit Hello, and writing to OB_CTRL might be clearing it. After
RUN=1, fast-poll OUTBOX for 500ms to catch an autonomous Hello.
Only if nothing arrives, THEN send SetIOPPower via write64.

If autonomous Hello arrives, we found the right recipe.
If not, SetIOPPower fallback + watch for response.
"""
import os, pathlib, struct, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port
from m1n1 import asm

AOP_BLOB    = ROOT / "firmware/aop/aopfw-mac16gaop_l4.RELEASE.im4p"
M1N1_MACHO  = ROOT / "external/m1n1/build/m1n1.macho"
AOP = 0x390600000

CPU_CONTROL  = 0x0044
CPU_STATUS   = 0x0048
INBOX_CTRL   = 0x8110
OUTBOX_CTRL  = 0x8114
INBOX0       = 0x8800
INBOX1       = 0x8808
OUTBOX0      = 0x8830
OUTBOX1      = 0x8838
EMPTY_BIT    = 1 << 17


def log(m): print(f"[v14] {m}", flush=True)


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


def try_recv(p):
    oc = p.read32(AOP + OUTBOX_CTRL)
    if oc & EMPTY_BIT:
        return None, None, oc
    m0 = p.read64(AOP + OUTBOX0)
    m1 = p.read64(AOP + OUTBOX1)
    return m0, m1, oc


def main():
    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    log("chainloading...")
    chainload(iface, p, u)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    try:
        p.dapf_init("/arm-io/dart-aop")
    except Exception as e:
        log(f"dapf: {e!r}")

    # Stage FW
    log("staging...")
    aop_node = u.adt["/arm-io/aop"]
    sr = getattr(aop_node, "segment-ranges", None)
    names_raw = getattr(aop_node, "segment-names", b"")
    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_by = dict(zip(names, parse_adt_segments(sr)))
    macho = AOP_BLOB.read_bytes()
    # .im4p wrapper: strip prefix before arm64 mach-o magic
    idx = macho.find(b"\xcf\xfa\xed\xfe")
    if idx > 0:
        log(f"stripping im4p prefix ({idx} bytes)")
        macho = macho[idx:]
    mc_by = {seg[0]: seg for seg in macho_segs_fn(macho)}
    skip = set()
    for nm in ("__TEXT", "__ETEXT"):
        if nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        ok = True
        for off in (0x0, 0x100, 0x200):
            if off >= m[4]: break
            if iface.readmem(a["phys"]+off, 16) != macho[m[3]+off:m[3]+off+16]:
                ok = False; break
        if ok: skip.add(nm)
    for nm in names:
        if nm in skip or nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0 or m[4] > a["size"]: continue
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], macho[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], macho[m[3]:m[3]+m[4]])

    try:
        from m1n1.fw.aop.base import AOPBase
        AOPBase(u).update_bootargs({'p0CE': 0x20000, 'laCn': 0,
                                    'tPOA': 1, 'gila': 0x80})
        log("bootargs OK")
    except Exception as e:
        log(f"bootargs: {e!r}")

    # Read the pre-RUN state (no OB_CTRL reset!)
    cc = p.read32(AOP + CPU_CONTROL)
    cs = p.read32(AOP + CPU_STATUS)
    ic = p.read32(AOP + INBOX_CTRL)
    oc = p.read32(AOP + OUTBOX_CTRL)
    log(f"pre-RUN (no OB reset): CC={cc:#x} CS={cs:#x} IB={ic:#x} OB={oc:#x}")

    log("\nCC.RUN=1")
    p.write32(AOP + CPU_CONTROL, 0x10)

    # Fast poll 500ms for autonomous Hello — NO sleep, tight loop
    log("fast-poll 500ms for autonomous Hello...")
    msgs = []
    deadline = time.time() + 0.5
    n_reads = 0
    while time.time() < deadline:
        try:
            m0, m1, oc = try_recv(p)
            n_reads += 1
            if m0 is not None:
                log(f"  *** AUTO-HELLO m0={m0:#x} m1={m1:#x} "
                    f"TYPE={(m0>>52)&0xff:#x} EP={m1&0xff:#x} ***")
                msgs.append((m0, m1))
        except Exception as e:
            log(f"proxy err: {e!r}")
            break
    log(f"  {n_reads} reads, {len(msgs)} msgs in 500ms")

    if msgs:
        log("*** FW self-Hello'd! Continuing with handshake ***")
    else:
        # Fallback: send SetIOPPower
        try:
            cc = p.read32(AOP + CPU_CONTROL)
            cs = p.read32(AOP + CPU_STATUS)
            ic = p.read32(AOP + INBOX_CTRL)
            oc = p.read32(AOP + OUTBOX_CTRL)
            log(f"\npre-fallback: CC={cc:#x} CS={cs:#x} IB={ic:#x} OB={oc:#x}")
        except Exception as e:
            log(f"proxy err before fallback: {e!r}")
            os._exit(3)

        log("\nfallback: SetIOPPower(0x220) via write64...")
        try:
            p.write64(AOP + INBOX0, (6 << 52) | 0x220)
            p.write64(AOP + INBOX1, 0)
        except Exception as e:
            log(f"send err: {e!r}")
            os._exit(3)

        # Then poll 10s with tight read loop
        deadline = time.time() + 10.0
        last = None
        while time.time() < deadline:
            try:
                cs = p.read32(AOP + CPU_STATUS)
                oc = p.read32(AOP + OUTBOX_CTRL)
                ic = p.read32(AOP + INBOX_CTRL)
            except Exception as e:
                log(f"proxy err: {e!r}")
                break
            state = (cs, oc, ic)
            if state != last:
                t = 10 - (deadline - time.time())
                log(f"  t={t:5.2f}s CS={cs:#x} OB={oc:#x} IB={ic:#x}")
                last = state
            if not (oc & EMPTY_BIT):
                m0 = p.read64(AOP + OUTBOX0)
                m1 = p.read64(AOP + OUTBOX1)
                log(f"  *** OUT m0={m0:#x} m1={m1:#x} "
                    f"TYPE={(m0>>52)&0xff:#x} EP={m1&0xff:#x} ***")
                msgs.append((m0, m1))
            time.sleep(0.02)

    log(f"\n*** total msgs: {len(msgs)} ***")
    for m0, m1 in msgs:
        log(f"  m0={m0:#x} m1={m1:#x}")
    os._exit(0 if msgs else 2)


if __name__ == "__main__":
    main()
