#!/usr/bin/env python3
"""Time-series probe: watch AOP __DATA mutations over 20s after RUN.

mega2 showed FW populates __DATA+0x498 with a pointer table (writes
0x3_90c9_xxxx-stride entries). So FW IS running. This script times
those writes to see if FW progresses (→ ultimately sends Hello),
or stops writing at a specific point (→ that's the stall).

Also track reg[0]+0x818 — changed 0x40001 → 0x40000 in mega2. And
+0x444, +0xb00, +0xb10 for completeness.
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
M1N1_MACHO  = ROOT / "external/m1n1/build/m1n1.macho"
AOP = 0x390600000


def log(m): print(f"[t] {m}", flush=True)


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
    log("staged")

    try:
        from m1n1.fw.aop.base import AOPBase
        AOPBase(u).update_bootargs({'p0CE': 0x20000, 'laCn': 0,
                                    'tPOA': 1, 'gila': 0x80})
    except: pass

    data_phys = adt_by['__DATA']['phys']
    seed_phys = data_phys + 0x498

    log("\nRUN=1")
    p.write32(AOP + 0x44, 0x10)
    # Send INBOX immediately (within the watchdog window)
    p.write64(AOP + 0x8800, (6 << 52) | 0x220)
    p.write64(AOP + 0x8808, 0)

    log("time-series: 15s of dumps every 500ms...")
    log("  cols: t, CC, CS, IB, OB, [+0x818], __DATA+0x498 first 16 bytes, __DATA+0x498+0x80 (pointer count)")
    t0 = time.time()
    prev_seed_hash = None
    first_change_t = None
    stable_since = None
    pointer_count_history = []

    while time.time() - t0 < 15.0:
        try:
            cc = p.read32(AOP + 0x44)
            cs = p.read32(AOP + 0x48)
            ib = p.read32(AOP + 0x8110)
            ob = p.read32(AOP + 0x8114)
            r818 = p.read32(AOP + 0x818)
            seed = iface.readmem(seed_phys, 128)
            # Count non-zero u64s
            pointers = struct.unpack('<16Q', seed)
            nptr = sum(1 for x in pointers if x != 0)
            pointer_count_history.append((time.time() - t0, nptr))
            h = hash(seed)
            if h != prev_seed_hash:
                if first_change_t is None:
                    first_change_t = time.time() - t0
                    log(f"  FIRST CHANGE at t={first_change_t:.2f}s")
                first16 = seed[:16].hex()
                log(f"  t={time.time()-t0:5.2f}s CC={cc:#x} CS={cs:#x} IB={ib:#x} "
                    f"OB={ob:#x} r818={r818:#x} nptr={nptr} first={first16}")
                prev_seed_hash = h
                stable_since = None
            else:
                if stable_since is None:
                    stable_since = time.time() - t0
            # OUTBOX check
            if not (ob & (1 << 17)):
                m0 = p.read64(AOP + 0x8830)
                m1 = p.read64(AOP + 0x8838)
                log(f"  *** OB MSG @ t={time.time()-t0:.2f}s m0={m0:#x} m1={m1:#x} ***")
        except Exception as e:
            log(f"  err: {e!r}")
            break
        time.sleep(0.5)

    log(f"\nFinal: first_change_t={first_change_t}, stable_since={stable_since}")
    log("Pointer count history:")
    for t, n in pointer_count_history[:30]:
        log(f"  t={t:.2f}s nptr={n}")
    # Dump final seed region (256 bytes)
    final = iface.readmem(seed_phys, 256)
    log("\nFinal __DATA+0x498 (256 bytes):")
    for i in range(0, 256, 16):
        if any(b != 0 for b in final[i:i+16]):
            log(f"  +{i:#04x}: {final[i:i+16].hex()}")
    os._exit(0)


if __name__ == "__main__":
    main()
