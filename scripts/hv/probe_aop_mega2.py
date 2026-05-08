#!/usr/bin/env python3
"""Mega-probe v2: focus on AOP reg[4] + __DATA seed + post-stall MMIO.

Skip SMC enumeration (caused SYNC fault last time). Keep tight.
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

CPU_CONTROL  = 0x0044
CPU_STATUS   = 0x0048
INBOX_CTRL   = 0x8110
OUTBOX_CTRL  = 0x8114
INBOX0       = 0x8800
INBOX1       = 0x8808
OUTBOX0      = 0x8830
OUTBOX1      = 0x8838
EMPTY_BIT    = 1 << 17


def log(m): print(f"[m2] {m}", flush=True)


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

    # ========== Probe 1: AOP reg[4] dump (pre-boot) ==========
    log("\n=== P1: AOP reg[4] pre-boot dump (0x390c62000 + 256 bytes) ===")
    try:
        dump = iface.readmem(0x390c62000, 512)
        for i in range(0, min(256, len(dump)), 32):
            log(f"  +{i:#04x}: {dump[i:i+32].hex()}")
        # Look for non-zero entries
        nz = [(i, b) for i, b in enumerate(dump) if b != 0]
        log(f"  {len(nz)} non-zero bytes in first 512")
    except Exception as e:
        log(f"  reg[4] dump err: {e!r}")

    # ========== Probe 2: AOP reg[0]+0x1000..0x2000 scan ==========
    # The doorbell area; likely has more config regs
    log("\n=== P2: AOP reg[0]+0x1000..0x1200 (MMIO scan 128 regs) ===")
    interesting = []
    for off in range(0x1000, 0x1200, 4):
        try:
            v = p.read32(AOP + off)
            if v != 0 and v != 0xffffffff:
                interesting.append((off, v))
        except Exception as e:
            log(f"  scan err at +{off:#x}: {e!r}")
            break
    log(f"  found {len(interesting)} non-trivial values in +0x1000..0x1200")
    for off, v in interesting[:30]:
        log(f"    +{off:#x} = {v:#x}")

    # ========== Probe 3: Scan AOP reg[0]+0x100..0x1000 ==========
    log("\n=== P3: AOP reg[0]+0x100..0x1000 scan ===")
    interesting = []
    for off in range(0x100, 0x1000, 4):
        try:
            v = p.read32(AOP + off)
            if v != 0 and v != 0xffffffff:
                interesting.append((off, v))
        except Exception:
            break
    log(f"  found {len(interesting)} non-trivial values in +0x100..0x1000")
    for off, v in interesting[:30]:
        log(f"    +{off:#x} = {v:#x}")

    # ========== Stage AOP FW ==========
    log("\n=== Staging AOP FW ===")
    try:
        p.dapf_init("/arm-io/dart-aop")
    except: pass
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
    log("  staged")

    # ========== Probe 4: AOP __DATA+0x498 (PAC seed dump) ==========
    log("\n=== P4: AOP __DATA+0x498 (PAC seed + 128 bytes) ===")
    data_phys = adt_by['__DATA']['phys']
    seed_phys = data_phys + 0x498
    try:
        seed = iface.readmem(seed_phys, 128)
        for i in range(0, 128, 16):
            u64_lo = struct.unpack('<Q', seed[i:i+8])[0] if i+8 <= len(seed) else 0
            u64_hi = struct.unpack('<Q', seed[i+8:i+16])[0] if i+16 <= len(seed) else 0
            log(f"  +{i:#04x}: {seed[i:i+16].hex()}  ({u64_lo:#018x} {u64_hi:#018x})")
    except Exception as e:
        log(f"  seed err: {e!r}")

    # ========== Probe 5: Dump __DATA+0x380 (table area seen in disasm) ==========
    log("\n=== P5: AOP __DATA+0x370..0x400 (dispatch table area) ===")
    # From disasm: adrp x1, #0x1119000; add #0x390 — table at 0x1119390
    # Also 0x1118000 + 0x3e5, 0x3e1, 0x494, 0x504, 0x514 used
    for off in [0x117-0x000, 0x117-0x010, 0x118390-0x10cd000, 0x118494-0x10cd000]:
        pass  # computed below
    # Cleaner:
    for va_off in [0x1118494, 0x11183e5, 0x11183e1, 0x1118504, 0x1118514,
                   0x1119378, 0x1119388, 0x1119390, 0x111b020,
                   0x111a000]:
        phys = data_phys + (va_off - 0x10cd000)
        try:
            dump = iface.readmem(phys, 32)
            log(f"  VA={va_off:#x} phys={phys:#x}: {dump.hex()}")
        except Exception:
            log(f"  VA={va_off:#x}: err")

    # ========== update bootargs + RUN ==========
    try:
        from m1n1.fw.aop.base import AOPBase
        AOPBase(u).update_bootargs({'p0CE': 0x20000, 'laCn': 0,
                                    'tPOA': 1, 'gila': 0x80})
    except Exception as e:
        log(f"bootargs: {e!r}")

    log("\n=== RUN sequence ===")
    cc0 = p.read32(AOP + CPU_CONTROL); cs0 = p.read32(AOP + CPU_STATUS)
    ic0 = p.read32(AOP + INBOX_CTRL);  oc0 = p.read32(AOP + OUTBOX_CTRL)
    log(f"  pre-RUN: CC={cc0:#x} CS={cs0:#x} IB={ic0:#x} OB={oc0:#x}")
    p.write32(AOP + CPU_CONTROL, 0x10)
    time.sleep(0.2)
    cc1 = p.read32(AOP + CPU_CONTROL); cs1 = p.read32(AOP + CPU_STATUS)
    ic1 = p.read32(AOP + INBOX_CTRL);  oc1 = p.read32(AOP + OUTBOX_CTRL)
    log(f"  post-RUN: CC={cc1:#x} CS={cs1:#x} IB={ic1:#x} OB={oc1:#x}")

    # Send INBOX
    p.write64(AOP + INBOX0, (6 << 52) | 0x220)
    p.write64(AOP + INBOX1, 0)
    time.sleep(0.2)
    cc2 = p.read32(AOP + CPU_CONTROL); cs2 = p.read32(AOP + CPU_STATUS)
    ic2 = p.read32(AOP + INBOX_CTRL);  oc2 = p.read32(AOP + OUTBOX_CTRL)
    log(f"  post-INBOX: CC={cc2:#x} CS={cs2:#x} IB={ic2:#x} OB={oc2:#x}")

    # ========== Probe 6: Scan AOP MMIO AGAIN post-stall ==========
    log("\n=== P6: AOP MMIO scan +0x100..0x2000 POST-STALL (compare pre-boot) ===")
    interesting_post = []
    for off in range(0x100, 0x2000, 4):
        try:
            v = p.read32(AOP + off)
            if v != 0 and v != 0xffffffff:
                interesting_post.append((off, v))
        except: break
    log(f"  found {len(interesting_post)} non-trivial post-stall (was {len(interesting)} pre-boot +0x1000..0x1200)")
    # Print just first 40
    for off, v in interesting_post[:40]:
        log(f"    +{off:#x} = {v:#x}")

    # ========== Probe 7: Dump __DATA after stall — any changes? ==========
    log("\n=== P7: AOP __DATA+0x498 POST-STALL (compare to pre-RUN) ===")
    try:
        seed2 = iface.readmem(seed_phys, 64)
        log("  post-RUN __DATA+0x498:")
        for i in range(0, 64, 16):
            log(f"    +{i:#04x}: {seed2[i:i+16].hex()}")
    except Exception as e:
        log(f"  err: {e!r}")

    # ========== Probe 8: Dump __OS_LOG — FW might have logged something ==========
    log("\n=== P8: AOP __OS_LOG post-stall (first 2 KB, look for NEW text) ===")
    os_log_phys = adt_by['__OS_LOG']['phys']
    try:
        dump = iface.readmem(os_log_phys, 2048)
        # Look for printable strings not in the original blob
        original_os_log = macho[mc_by['__OS_LOG'][3]:mc_by['__OS_LOG'][3]+2048]
        diff_regions = []
        for i in range(len(dump)):
            if i >= len(original_os_log): break
            if dump[i] != original_os_log[i]:
                diff_regions.append(i)
        log(f"  OS_LOG bytes differ from blob at {len(diff_regions)} offsets")
        if diff_regions:
            # Show first diff region
            first = diff_regions[0]
            log(f"  First diff at offset {first:#x}")
            log(f"  live: {dump[first:first+64].hex()}")
            log(f"  blob: {original_os_log[first:first+64].hex()}")
    except Exception as e:
        log(f"  OS_LOG err: {e!r}")

    os._exit(0)


if __name__ == "__main__":
    main()
