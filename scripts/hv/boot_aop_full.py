#!/usr/bin/env python3
"""Combined AOP boot + doorbell + ACK hunt + OUTBOX wait.

Does in one continuous session (no intermediate script runs
that could reset state):
  1. chainload m1n1 + WDT disable + stage FW
  2. update bootargs (4 keys + some new experiments)
  3. dapf_init
  4. OUTBOX_CTRL=0x20001
  5. RUN=1
  6. Poll 2s for FW to self-emit Hello (maybe it DOES hello first!)
  7. If nothing: send SetIOPPower + doorbell
  8. Poll 3s
  9. If nothing: try other TYPEs + different doorbell values
  10. Try writing to +0x8118/+0x811c and +0x8158
"""
import os, pathlib, struct, sys, time
ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))
from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

AOP_BLOB = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
M1N1_MACHO = ROOT / "external/m1n1/build/m1n1.macho"
AOP = 0x390600000


def log(m): print(f"[full] {m}", flush=True)


def snap(p, tag):
    c = p.read32(AOP+0x44)
    ie = p.read32(AOP+0x48)
    a2i = p.read32(AOP+0x8110)
    i2a = p.read32(AOP+0x8114)
    out0 = p.read32(AOP+0x8830) | (p.read32(AOP+0x8834) << 32)
    out1 = p.read32(AOP+0x8838) | (p.read32(AOP+0x883c) << 32)
    db1 = p.read32(AOP+0x100c); db2 = p.read32(AOP+0x101c)
    log(f"[{tag}] CC={c:#x} IE={ie:#x} A2I={a2i:#x} I2A={i2a:#x} "
        f"OUT0={out0:#x} OUT1={out1:#x} DB={db1:#x}/{db2:#x}")


def peek_outbox(p):
    i2a = p.read32(AOP+0x8114)
    if i2a & (1 << 17):  # EMPTY set
        return None
    o0 = p.read32(AOP+0x8830) | (p.read32(AOP+0x8834) << 32)
    o1 = p.read32(AOP+0x8838) | (p.read32(AOP+0x883c) << 32)
    if o0 == 0 and (o1 & 0xffffffff00000000) == 0xa0000 << 32:
        return None  # probably stale
    return (o0, o1)


def ring(p):
    p.write32(AOP + 0x1004, 0x10)
    p.write32(AOP + 0x1014, 0x1)


def send_inbox(p, msg0, msg1=0):
    p.write32(AOP + 0x8800, msg0 & 0xffffffff)
    p.write32(AOP + 0x8804, (msg0 >> 32) & 0xffffffff)
    p.write32(AOP + 0x8808, msg1 & 0xffffffff)
    p.write32(AOP + 0x880c, (msg1 >> 32) & 0xffffffff)


def wait_outbox(p, timeout, tag):
    deadline = time.time() + timeout
    last_i2a = None
    last_out0 = None
    while time.time() < deadline:
        i2a = p.read32(AOP + 0x8114)
        out0 = p.read32(AOP + 0x8830)
        if (i2a, out0) != (last_i2a, last_out0):
            t = time.time() - (deadline - timeout)
            log(f"  t={t:.2f}s I2A={i2a:#x} OUT0={out0:#x}")
            last_i2a, last_out0 = i2a, out0
        if not (i2a & (1 << 17)):
            full0 = p.read32(AOP+0x8830) | (p.read32(AOP+0x8834) << 32)
            full1 = p.read32(AOP+0x8838) | (p.read32(AOP+0x883c) << 32)
            if full0 != 0:
                typ = (full0 >> 52) & 0xff
                log(f"  *** OUTBOX {tag}: msg0={full0:#x} msg1={full1:#x} TYPE={typ:#x} ***")
                return (full0, full1)
        time.sleep(0.05)
    return None


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

    log("chainloading patched m1n1...")
    chainload(iface, p, u)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    for addr, val in [(0x3882BC224, 0), (0x3882B8008, 0xffffffff),
                      (0x3882B802C, 0xffffffff), (0x3882B8020, 0xffffffff)]:
        try: p.write32(addr, val)
        except: pass

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
            if iface.readmem(a["phys"]+off, 16) != macho[m[3]+off:m[3]+off+16]:
                ok = False; break
        if ok: skip.add(nm); log(f"  {nm}: iBoot-staged")
    for nm in names:
        if nm in skip or nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0 or m[4] > a["size"]: continue
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], macho[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], macho[m[3]:m[3]+m[4]])
        log(f"  staged {nm}: {m[4]}B")

    # Bootargs
    from m1n1.fw.aop.base import AOPBase
    aopb = AOPBase(u)
    aopb.update_bootargs({'p0CE':0x20000,'laCn':0,'tPOA':1,'gila':0x80})
    log("bootargs updated")

    # dapf_init
    try:
        p.dapf_init("/arm-io/dart-aop")
        log("dapf_init OK")
    except Exception as e:
        log(f"dapf err: {e!r}")

    # Reset OUTBOX_CTRL
    p.write32(AOP + 0x8114, 0x20001)

    snap(p, "pre-RUN")

    # RUN=1
    log("CC.RUN=1...")
    p.write32(AOP + 0x44, 0x10)
    time.sleep(0.3)
    snap(p, "post-RUN")

    # === STEP 1: Poll 3s for FW to self-emit Hello without us writing anything ===
    log("\n=== STEP 1: wait 3s for FW to spontaneously Hello ===")
    got = wait_outbox(p, 3, "hello-wait")
    if got:
        log(f"FW HELLO: {got}")
        snap(p, "got-hello")
        os._exit(0)

    snap(p, "no-hello-yet")

    # === STEP 2: send various TYPEs with doorbell ===
    # Reference messages from mgmt.py:
    # TYPE=1 Hello,  TYPE=2 HelloAck, TYPE=3 Ping,  TYPE=4 Pong,
    # TYPE=5 StartEP, TYPE=6 SetIOPPower, TYPE=7 IOPPowerAck,
    # TYPE=8 EPMap,  TYPE=0xb SetAPPower
    test_msgs = [
        ("SetIOPPower=0x20", (6 << 52) | 0x20, 0),
        ("SetIOPPower=0x220", (6 << 52) | 0x220, 0),
        ("Hello MIN=0 MAX=0xff", (1 << 52) | (0xff << 16) | 0x0, 0),
        ("HelloAck MIN=0 MAX=0xff", (2 << 52) | (0xff << 16) | 0x0, 0),
        ("Ping", (3 << 52), 0),
        ("SetAPPower=0x20", (0xb << 52) | 0x20, 0),
    ]
    for name, msg0, msg1 in test_msgs:
        log(f"\n--- TX {name}  msg0={msg0:#x} ---")
        send_inbox(p, msg0, msg1)
        ring(p)
        snap(p, "post-send")
        got = wait_outbox(p, 2, name)
        if got:
            log(f"  *** FW replied! ***")
            snap(p, "got-reply")
            os._exit(0)

    # === STEP 3: try alternate doorbell values ===
    log("\n=== STEP 3: alternate doorbell values ===")
    send_inbox(p, (6 << 52) | 0x20, 0)
    for val_cfg, val_arm in ((0x0, 0x1), (0x11, 0x1), (0x12, 0x1),
                              (0x10, 0x2), (0x10, 0x3), (0x10, 0xffffffff),
                              (0xff, 0x1), (0x1, 0x1)):
        log(f"  doorbell: +0x1004={val_cfg:#x}, +0x1014={val_arm:#x}")
        p.write32(AOP + 0x1004, val_cfg)
        p.write32(AOP + 0x1014, val_arm)
        got = wait_outbox(p, 1, f"cfg={val_cfg:#x}")
        if got:
            log("  FOUND IT!")
            os._exit(0)

    # === STEP 4: read other candidate OUTBOX paths ===
    log("\n=== STEP 4: scan for any FW-written regs in reg[0] ===")
    changed = []
    for off in range(0x0, 0x8200, 4):
        try:
            v = p.read32(AOP + off)
            if v != 0 and off not in (0x0, 0x8, 0x40, 0x44, 0x48, 0x4c,
                                        0x444, 0x818, 0x8110, 0x8114,
                                        0x8118, 0x811c, 0x1004, 0x100c,
                                        0x1014, 0x101c, 0x8800, 0x8804,
                                        0x8810, 0x8814, 0x8818, 0x881c,
                                        0x8180, 0x8188, 0x818c):
                changed.append((off, v))
        except:
            break
    log(f"  other non-zero regs: {len(changed)}")
    for off, v in changed[:30]:
        log(f"    +{off:#x} = {v:#x}")

    snap(p, "end")
    os._exit(0)


if __name__ == "__main__":
    main()
