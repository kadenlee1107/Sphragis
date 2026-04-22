#!/usr/bin/env python3
"""Unstick probe: after FW reaches steady state with our msg queued,
try poking various control regs to see if anything drains INBOX.

Previous findings:
  - FW reaches steady state within 500ms
  - __DATA+0x498 populates with 16+ page-pointer table entries
  - reg[0]+0x818 goes 0x40001 → 0x40000 (post-RUN) → 0x40003 (post-init)
  - INBOX stays FIFOCNT=1 indefinitely
  - Dispatch table at 0x1117938 has non-null entry

v18: after normal boot, toggle/write these registers in sequence
and watch for INBOX drain:
  - AOP+0x48 (CPU_STATUS writes? usually RO but try)
  - AOP+0x818 various values
  - AOP+0xb00/+0xb10 unknown regs
  - AOP+0x810/+0x814 (near 0x818)
  - AOP+0x1008/+0x100c/+0x1010 (doorbell neighborhood besides 0x1004/0x1014)
"""
import os, pathlib, struct, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port
from m1n1 import asm

AOP_BLOB    = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
M1N1_MACHO  = ROOT / "external/m1n1/build/m1n1.macho"
AOP = 0x390600000


def log(m): print(f"[u] {m}", flush=True)


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


def snap(p, tag):
    cc = p.read32(AOP + 0x44)
    cs = p.read32(AOP + 0x48)
    ib = p.read32(AOP + 0x8110)
    ob = p.read32(AOP + 0x8114)
    r818 = p.read32(AOP + 0x818)
    r810 = p.read32(AOP + 0x810)
    r814 = p.read32(AOP + 0x814)
    log(f"  [{tag}] CC={cc:#x} CS={cs:#x} IB={ib:#x} OB={ob:#x} "
        f"r810={r810:#x} r814={r814:#x} r818={r818:#x}")
    return (cc, cs, ib, ob, r818)


def check_ob(p, tag):
    ob = p.read32(AOP + 0x8114)
    if not (ob & (1 << 17)):
        m0 = p.read64(AOP + 0x8830)
        m1 = p.read64(AOP + 0x8838)
        log(f"  *** {tag} OUTBOX MSG m0={m0:#x} m1={m1:#x} TYPE={(m0>>52)&0xff:#x} ***")
        return True
    return False


def main():
    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    log("chainloading...")
    chainload(iface, p, u)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    try: p.dapf_init("/arm-io/dart-aop")
    except: pass

    # Stage
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
    except: pass

    # RUN + INBOX
    p.write32(AOP + 0x44, 0x10)
    p.write64(AOP + 0x8800, (6 << 52) | 0x220)
    p.write64(AOP + 0x8808, 0)
    time.sleep(1.0)  # let FW init stabilize
    snap(p, "stable")

    # === Unstick attempts ===

    # Attempt 1: Re-write INBOX to trigger push
    log("\nA1: rewrite INBOX in case trigger was missed")
    p.write64(AOP + 0x8800, (6 << 52) | 0x220)
    p.write64(AOP + 0x8808, 0)
    time.sleep(0.3)
    snap(p, "A1")
    if check_ob(p, "A1"): os._exit(0)

    # Attempt 2: Toggle +0x818 bit 0 (maybe triggers polling)
    log("\nA2: toggle +0x818 bit 0 off/on")
    cur_818 = p.read32(AOP + 0x818)
    log(f"  cur +0x818 = {cur_818:#x}")
    p.write32(AOP + 0x818, cur_818 & ~1)
    time.sleep(0.1)
    p.write32(AOP + 0x818, cur_818)
    time.sleep(0.3)
    snap(p, "A2")
    if check_ob(p, "A2"): os._exit(0)

    # Attempt 3: Set/clear bits in CPU_CONTROL (+0x44) beyond RUN
    log("\nA3: try CPU_CONTROL additional bits")
    p.write32(AOP + 0x44, 0x10 | 0x1)  # add bit 0
    time.sleep(0.2)
    snap(p, "A3a bit0")
    if check_ob(p, "A3a"): os._exit(0)
    p.write32(AOP + 0x44, 0x10 | 0x100)  # bit 8
    time.sleep(0.2)
    snap(p, "A3b bit8")
    if check_ob(p, "A3b"): os._exit(0)
    p.write32(AOP + 0x44, 0x10)  # restore
    time.sleep(0.1)

    # Attempt 4: doorbell regs BESIDES 0x1004/0x1014
    log("\nA4: doorbell neighbors")
    for off in [0x1000, 0x1008, 0x100c, 0x1010, 0x1018, 0x101c, 0x1020]:
        try:
            before = p.read32(AOP + off)
            p.write32(AOP + off, 1)
            time.sleep(0.1)
            after = p.read32(AOP + off)
            log(f"  +{off:#x}: before={before:#x} after=1 → read={after:#x}")
            if check_ob(p, f"A4+{off:#x}"): os._exit(0)
        except Exception as e:
            log(f"  +{off:#x}: {type(e).__name__}")

    # Attempt 5: INBOX_CTRL bits (maybe trigger drain)
    log("\nA5: try writing to INBOX_CTRL / OUTBOX_CTRL")
    orig = p.read32(AOP + 0x8110)
    log(f"  orig IB_CTRL = {orig:#x}")
    # Try writing same value back (sometimes wakes HW)
    p.write32(AOP + 0x8110, orig)
    time.sleep(0.1)
    snap(p, "A5 rewrite")
    if check_ob(p, "A5"): os._exit(0)
    # Try clearing ENABLE bit then setting
    p.write32(AOP + 0x8110, orig & ~1)
    time.sleep(0.1)
    p.write32(AOP + 0x8110, orig | 1)
    time.sleep(0.3)
    snap(p, "A5 re-enable")
    if check_ob(p, "A5-re"): os._exit(0)

    # Attempt 6: Scan for WRITABLE regs that aren't readable
    log("\nA6: try +0xb00/+0xb10/+0x444 writes")
    for off in [0x444, 0xb00, 0xb10]:
        before = p.read32(AOP + off)
        try:
            # Try writing 1 (low bit)
            p.write32(AOP + off, 1)
            time.sleep(0.1)
            after = p.read32(AOP + off)
            log(f"  +{off:#x}: before={before:#x} write=1 after={after:#x}")
            if check_ob(p, f"A6+{off:#x}"): os._exit(0)
            # Restore
            p.write32(AOP + off, before)
        except Exception as e:
            log(f"  +{off:#x}: {type(e).__name__}")

    # Attempt 7: wait long
    log("\nA7: long wait 10s...")
    t0 = time.time()
    while time.time() - t0 < 10:
        if check_ob(p, "A7-wait"):
            os._exit(0)
        time.sleep(0.5)

    snap(p, "FINAL")
    log("\nno unstick attempt worked")
    os._exit(2)


if __name__ == "__main__":
    main()
