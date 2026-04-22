#!/usr/bin/env python3
"""Mega-probe: PMP boot + AOP + MMIO/DATA introspection. One cycle.

Priority ordered for information-density:
  1. chainload patched m1n1 (WDT-immortal)
  2. Enumerate live pre-RUN state of all ASCWrapV6 IOPs
  3. Try PMP boot without any staging (iBoot might have it ready)
  4. If PMP Hellos: capture + use it
  5. Dump AOP reg[4] (0x190c62000 size 0x3c008) — unknown AOP-only region
  6. Dump AOP __DATA+0x498 (PAC seed + nearby config)
  7. Try AOP boot and capture INBOX-CTRL bit-level state
  8. If AOP stalls, send multiple EP messages (0, 1, 2, 8, 0x20)
"""
import os, pathlib, struct, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port
from m1n1 import asm

AOP_BLOB    = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
MTP_BLOB    = ROOT / "firmware/mtp/J604_MtpFirmware.bin"
M1N1_MACHO  = ROOT / "external/m1n1/build/m1n1.macho"

AOP = 0x390600000
PMP = None  # discover
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


def log(m): print(f"[mega] {m}", flush=True)


def snap(p, base, tag):
    cc = p.read32(base + CPU_CONTROL)
    cs = p.read32(base + CPU_STATUS)
    ic = p.read32(base + INBOX_CTRL)
    oc = p.read32(base + OUTBOX_CTRL)
    log(f"  [{tag}] @{base:#x} CC={cc:#x} CS={cs:#x} IB={ic:#x} OB={oc:#x}")


def send64(p, base, msg0, msg1):
    p.write64(base + INBOX0, msg0)
    p.write64(base + INBOX1, msg1)


def poll_msgs(p, base, secs, tag):
    msgs = []
    deadline = time.time() + secs
    last = None
    while time.time() < deadline:
        try:
            oc = p.read32(base + OUTBOX_CTRL)
            cs = p.read32(base + CPU_STATUS)
        except Exception as e:
            log(f"  {tag} proxy err: {e!r}")
            return msgs, True
        if (oc, cs) != last:
            t = secs - (deadline - time.time())
            log(f"    {tag} t={t:5.2f}s CS={cs:#x} OB={oc:#x}")
            last = (oc, cs)
        if not (oc & EMPTY_BIT):
            m0 = p.read64(base + OUTBOX0)
            m1 = p.read64(base + OUTBOX1)
            log(f"    *** {tag} MSG m0={m0:#x} m1={m1:#x} "
                f"TYPE={(m0>>52)&0xff:#x} EP={m1&0xff:#x} ***")
            msgs.append((m0, m1))
            time.sleep(0.005)
        time.sleep(0.02)
    return msgs, False


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

    # ========== STEP 1: Enumerate IOPs ==========
    log("\n=== STEP 1: enumerate ASCWrapV6 IOPs + pre-RUN state ===")
    iops = {}
    for path in ['/arm-io/aop', '/arm-io/pmp', '/arm-io/mtp', '/arm-io/smc',
                 '/arm-io/sep', '/arm-io/sio', '/arm-io/dcp']:
        try:
            n = u.adt[path]
            reg0 = n.get_reg(0)
            base = reg0[0]
            cc = p.read32(base + CPU_CONTROL)
            cs = p.read32(base + CPU_STATUS)
            ic = p.read32(base + INBOX_CTRL)
            oc = p.read32(base + OUTBOX_CTRL)
            # FW segments?
            segs = getattr(n, 'segment-names', b'')
            if isinstance(segs, bytes):
                segs = segs.decode('ascii', errors='replace').strip(chr(0))
            n_reg = 0
            for i in range(10):
                try: n.get_reg(i); n_reg += 1
                except: break
            running = "RUN" if (cc & 0x10) else "halt"
            log(f"  {path:<15s} base={base:#x} n_reg={n_reg} {running} "
                f"CC={cc:#x} CS={cs:#x} IB={ic:#x} OB={oc:#x} segs='{segs}'")
            iops[path] = base
        except Exception as e:
            log(f"  {path:<15s} MISSING: {type(e).__name__}")

    # ========== STEP 2: Try PMP boot (use iBoot-staged state) ==========
    if '/arm-io/pmp' in iops:
        log("\n=== STEP 2: PMP boot (no stage, use iBoot state) ===")
        PMP_base = iops['/arm-io/pmp']
        snap(p, PMP_base, "PMP pre-RUN")
        # If already running (iBoot started it), try sending msg directly
        cc = p.read32(PMP_base + CPU_CONTROL)
        if not (cc & 0x10):
            log("  PMP was halted; setting RUN=1")
            p.write32(PMP_base + CPU_CONTROL, 0x10)
            time.sleep(0.2)
            snap(p, PMP_base, "PMP post-RUN")
        else:
            log("  PMP was already running")
        # Send SetIOPPower
        log("  sending Mgmt_SetIOPPower(0x220)...")
        send64(p, PMP_base, (6 << 52) | 0x220, 0)
        snap(p, PMP_base, "PMP post-send")
        pmp_msgs, crashed = poll_msgs(p, PMP_base, 3.0, "PMP")
        log(f"  PMP msgs: {len(pmp_msgs)}, crashed={crashed}")
        if pmp_msgs:
            log("  *** PMP HELLO'd! PMP was the missing prereq?")

    # ========== STEP 3: Dump AOP reg[4] (240 KB unknown region) ==========
    log("\n=== STEP 3: Dump AOP reg[4] first 256 bytes (0x390c62000) ===")
    aop_reg4_base = 0x390c62000
    try:
        dump = iface.readmem(aop_reg4_base, 256)
        log(f"  reg[4]@{aop_reg4_base:#x}:")
        for i in range(0, 256, 32):
            log(f"    +{i:#04x}: {dump[i:i+32].hex()}")
    except Exception as e:
        log(f"  dump err: {e!r}")

    # ========== STEP 4: Check AOP MMIO +0xb14 (older scripts read this) ==========
    log("\n=== STEP 4: Check AOP MMIO at non-standard offsets ===")
    for off in [0xb14, 0x140, 0x818, 0x1000, 0x1008, 0x1010, 0x1018,
                0x100, 0x104, 0x108, 0x10c, 0x110]:
        try:
            v = p.read32(AOP + off)
            log(f"  AOP+{off:#x} = {v:#x}")
        except Exception as e:
            log(f"  AOP+{off:#x}: {type(e).__name__}")

    # ========== STEP 5: AOP full boot sequence ==========
    log("\n=== STEP 5: AOP boot (stage + bootargs + RUN + INBOX) ===")
    try:
        p.dapf_init("/arm-io/dart-aop")
        log("  dapf OK")
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
    log("  AOP FW staged")

    # Dump AOP __DATA+0x498 (PAC seed)
    data_phys = adt_by['__DATA']['phys']
    seed_phys = data_phys + 0x498
    try:
        seed_data = iface.readmem(seed_phys, 64)
        log(f"  __DATA+0x498 (PAC seed area) @ {seed_phys:#x}:")
        for i in range(0, 64, 16):
            log(f"    +{i:#04x}: {seed_data[i:i+16].hex()}")
    except Exception as e:
        log(f"  seed dump err: {e!r}")

    try:
        from m1n1.fw.aop.base import AOPBase
        AOPBase(u).update_bootargs({'p0CE': 0x20000, 'laCn': 0,
                                    'tPOA': 1, 'gila': 0x80})
        log("  bootargs OK")
    except Exception as e:
        log(f"  bootargs: {e!r}")

    snap(p, AOP, "AOP pre-RUN")
    p.write32(AOP + CPU_CONTROL, 0x10)
    time.sleep(0.2)
    snap(p, AOP, "AOP post-RUN")

    # ========== STEP 6: Try sending to MULTIPLE endpoints ==========
    log("\n=== STEP 6: Send to multiple EPs ===")
    for ep, name in [(0, "mgmt"), (1, "crashlog"), (2, "syslog"),
                     (4, "ioreporting"), (8, "oslog"), (0x20, "custom")]:
        log(f"  sending msg to EP={ep:#x} ({name})")
        # Try msg = TYPE=6 STATE=0x220 with different EPs
        msg0 = (6 << 52) | 0x220
        msg1 = ep & 0xff
        try:
            send64(p, AOP, msg0, msg1)
            time.sleep(0.3)
            cc = p.read32(AOP + CPU_CONTROL)
            cs = p.read32(AOP + CPU_STATUS)
            ib = p.read32(AOP + INBOX_CTRL)
            oc = p.read32(AOP + OUTBOX_CTRL)
            log(f"    post EP={ep:#x}: CC={cc:#x} CS={cs:#x} "
                f"IB={ib:#x} OB={oc:#x}")
            if not (oc & EMPTY_BIT):
                m0 = p.read64(AOP + OUTBOX0)
                m1 = p.read64(AOP + OUTBOX1)
                log(f"    *** OUT m0={m0:#x} m1={m1:#x} ***")
        except Exception as e:
            log(f"    err: {e!r}")

    # ========== STEP 7: Try various Mgmt msg TYPES ==========
    log("\n=== STEP 7: Try different Mgmt TYPE values ===")
    for mtype, mname in [(1, "Hello"), (2, "HelloAck"), (3, "Ping"),
                          (5, "StartEP"), (6, "SetIOPPower-0x20"),
                          (0xb, "SetAPPower")]:
        if mtype == 6:
            msg0 = (mtype << 52) | 0x20  # try 0x20 instead of 0x220
        elif mtype == 2:
            msg0 = (mtype << 52) | (12 << 16) | 12  # HelloAck with ver 12
        elif mtype == 5:
            msg0 = (mtype << 52) | (0 << 32) | 2  # StartEP 0 flag 2
        elif mtype == 0xb:
            msg0 = (mtype << 52) | 0x20
        else:
            msg0 = (mtype << 52)
        log(f"  send TYPE={mtype} ({mname}) msg0={msg0:#x}")
        send64(p, AOP, msg0, 0)
        time.sleep(0.2)
        oc = p.read32(AOP + OUTBOX_CTRL)
        ib = p.read32(AOP + INBOX_CTRL)
        log(f"    IB={ib:#x} OB={oc:#x}")
        if not (oc & EMPTY_BIT):
            m0 = p.read64(AOP + OUTBOX0)
            m1 = p.read64(AOP + OUTBOX1)
            log(f"    *** OUT m0={m0:#x} m1={m1:#x} ***")
            # Break — got a response, handshake
            break

    snap(p, AOP, "AOP final")

    # ========== STEP 8: Final state dump ==========
    log("\n=== STEP 8: Final AOP state dump ===")
    for off in [0x44, 0x48, 0x4c, 0x50, 0x54, 0x8110, 0x8114, 0x8118,
                0x8800, 0x8808, 0x8830, 0x8838]:
        try:
            v = p.read32(AOP + off)
            log(f"  AOP+{off:#x} = {v:#x}")
        except: pass

    os._exit(0)


if __name__ == "__main__":
    main()
