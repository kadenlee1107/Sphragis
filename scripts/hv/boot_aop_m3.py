#!/usr/bin/env python3
"""Boot AOP via the M3-style mailbox protocol (Asahi apple,m3-mailbox-v2).

Per Asahi Linux drivers/soc/apple/mailbox.c:
  M3 mailbox layout (offsets from ASC base):
    +0x44 CPU_CONTROL  (RUN bit 4)
    +0x48 IRQ_ENABLE
    +0x4c IRQ_ACK
    +0x50 A2I_CTRL
    +0x60 A2I_SEND0
    +0x68 A2I_SEND1
    +0x70 A2I_RECV0 (FW-side)
    +0x78 A2I_RECV1
    +0x80 I2A_CTRL
    +0x90 I2A_SEND0 (FW writes here)
    +0x98 I2A_SEND1
    +0xa0 I2A_RECV0  <- AP reads OUTBOX here!
    +0xa8 I2A_RECV1
  CTRL bits: bit 16 FULL, bit 17 EMPTY
  IRQ bits: 0 A2I_EMPTY, 1 A2I_NOT_EMPTY, 2 I2A_EMPTY, 3 I2A_NOT_EMPTY

Problem found: reg[0]+0x50..+0xa8 are unwritable on M4 until AOP is
fully powered. reg[3] at 0x3882a8000 is a PMGR-style device reg:
current pre-boot = 0x4e744f7f (TARGET=0xf, ACTUAL=0x7 — partial).
Need AUTO_ENABLE or specific sequence to reach ACTUAL=0xf.

This script tries:
  1. Chainload patched m1n1.
  2. Disable WDT.
  3. Power up AOP via reg[3]: write AUTO_ENABLE bit, poll for ACTUAL=0xf.
  4. Stage __DATA + __OS_LOG.
  5. Update bootargs.
  6. dapf_init for /arm-io/dart-aop.
  7. Verify reg[0]+0x50..+0xa8 becomes writable.
  8. Kick CPU_CONTROL.RUN=1.
  9. Send SetIOPPower via M3 INBOX (reg[0]+0x60).
 10. Poll reg[0]+0xa0 for I2A response (Hello etc).
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

AOP_BLOB = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
M1N1_MACHO = ROOT / "external/m1n1/build/m1n1.macho"

AOP = 0x390600000  # reg[0] — ASC + mailbox
AOP_PMGR = 0x3882a8000  # reg[3] — PMGR dev reg (8 bytes)

# M3 mailbox offsets (from reg[0] base)
M3_CPU_CTRL = 0x44
M3_IRQ_EN   = 0x48
M3_IRQ_ACK  = 0x4c
M3_A2I_CTRL = 0x50
M3_A2I_SEND0 = 0x60
M3_A2I_SEND1 = 0x68
M3_I2A_CTRL = 0x80
M3_I2A_RECV0 = 0xa0
M3_I2A_RECV1 = 0xa8

CPU_RUN = 1 << 4
PMGR_AUTO_ENABLE = 1 << 28
PMGR_PS_ACTIVE = 0xf


def log(m): print(f"[m3] {m}", flush=True)


def snap(p, tag):
    cc = p.read32(AOP + M3_CPU_CTRL)
    ie = p.read32(AOP + M3_IRQ_EN)
    a2i = p.read32(AOP + M3_A2I_CTRL)
    i2a = p.read32(AOP + M3_I2A_CTRL)
    pmgr = p.read32(AOP_PMGR + 0)
    log(f"[{tag}] CC={cc:#x} IRQ_EN={ie:#x} A2I={a2i:#x} I2A={i2a:#x} "
        f"PMGR={pmgr:#x} (T={pmgr&0xf:x} A={(pmgr>>4)&0xf:x})")


def power_up_aop(p):
    """Try to push AOP's PMGR device reg to full ACTIVE (ACTUAL=0xf)."""
    cur = p.read32(AOP_PMGR + 0)
    target = cur & 0xf
    actual = (cur >> 4) & 0xf
    log(f"AOP PMGR pre: {cur:#x}  TARGET={target:#x} ACTUAL={actual:#x}")

    # Already fully powered?
    if actual == 0xf:
        log("  AOP already fully powered.")
        return True

    # Try: set AUTO_ENABLE bit to let hardware auto-manage power
    tries = [
        ("AUTO_ENABLE", cur | PMGR_AUTO_ENABLE),
        ("TARGET=f + AUTO", (cur & ~0xf) | PMGR_PS_ACTIVE | PMGR_AUTO_ENABLE),
        ("clear low nibble, then set", ((cur & ~0xff) | PMGR_PS_ACTIVE)),
    ]
    for name, val in tries:
        log(f"  try '{name}': write {val:#x}")
        p.write32(AOP_PMGR + 0, val)
        # Poll 200ms for ACTUAL=0xf
        deadline = time.time() + 0.2
        while time.time() < deadline:
            r = p.read32(AOP_PMGR + 0)
            if ((r >> 4) & 0xf) == 0xf:
                log(f"    SUCCESS: {r:#x}  ACTUAL=0xf!")
                return True
            time.sleep(0.01)
        log(f"    ACTUAL stuck at {(p.read32(AOP_PMGR+0) >> 4) & 0xf:#x}")

    final = p.read32(AOP_PMGR + 0)
    log(f"AOP PMGR post: {final:#x}  TARGET={final&0xf:x} ACTUAL={(final>>4)&0xf:x}")
    return ((final >> 4) & 0xf) == 0xf


def test_mbox_writable(p):
    """Try writing to A2I_SEND0 and reading back. Return True if writable."""
    p.write32(AOP + M3_A2I_SEND0, 0xdeadbeef)
    v = p.read32(AOP + M3_A2I_SEND0)
    writable = (v == 0xdeadbeef)
    p.write32(AOP + M3_A2I_SEND0, 0)  # clear
    log(f"  A2I_SEND0 write test: {'WRITABLE' if writable else 'UNWRITABLE (still gated)'}")
    return writable


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
    sepfw_off = image_size
    image_size += align(sepfw_length)
    preoslog_off = image_size
    image_size += align(preoslog_size)
    bootargs_off = image_size
    image_size += 0x4000
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

    for addr, val in [(0x3882BC224, 0), (0x3882B8008, 0xffffffff),
                      (0x3882B802C, 0xffffffff), (0x3882B8020, 0xffffffff)]:
        try: p.write32(addr, val)
        except: pass

    snap(p, "initial")

    # === STEP 1: Power up AOP ===
    log("\n=== Power up AOP via reg[3] PMGR ===")
    power_up_aop(p)
    snap(p, "post-power")

    # === STEP 2: Verify mailbox writable ===
    log("\n=== Test mailbox writability ===")
    if not test_mbox_writable(p):
        log("MAILBOX STILL NOT WRITABLE — power sequence didn't open it.")
        log("Try: manually force ACTUAL=0xf by writing to reg[3]+0x4 or neighboring PMGR regs.")

    # === STEP 3: Stage firmware ===
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
        if ok:
            skip.add(nm)
            log(f"  {nm}: iBoot-staged (skip)")

    for nm in names:
        if nm in skip: continue
        if nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0: continue
        if m[4] > a["size"]: log(f"  {nm}: OVERFLOW"); continue
        t = time.time()
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], macho[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], macho[m[3]:m[3]+m[4]])
        log(f"  {nm}: {m[4]}B ({(time.time()-t)*1000:.0f}ms)")

    # === STEP 4: Bootargs ===
    from m1n1.fw.aop.base import AOPBase
    aopb = AOPBase(u)
    try:
        aopb.update_bootargs({'p0CE':0x20000,'laCn':0,'tPOA':1,'gila':0x80})
        log("bootargs updated")
    except Exception as e:
        log(f"bootargs err: {e!r}")

    # === STEP 5: dapf_init ===
    try:
        p.dapf_init("/arm-io/dart-aop")
        log("dapf_init OK")
    except Exception as e:
        log(f"dapf_init err: {e!r}")

    # === STEP 6: RUN ===
    snap(p, "pre-RUN")
    log("CC.RUN=1...")
    p.write32(AOP + M3_CPU_CTRL, CPU_RUN)
    time.sleep(0.2)
    snap(p, "post-RUN")

    # === STEP 7: Send Mgmt_SetIOPPower via M3 INBOX ===
    log("sending SetIOPPower(0x220) via M3 A2I_SEND (+0x60)...")
    msg0 = 0x60000000000220
    msg1 = 0x0
    # Check FULL bit first
    a2i_ctrl = p.read32(AOP + M3_A2I_CTRL)
    log(f"  A2I_CTRL pre={a2i_ctrl:#x}")
    p.write32(AOP + M3_A2I_SEND0 + 0, msg0 & 0xffffffff)
    p.write32(AOP + M3_A2I_SEND0 + 4, (msg0 >> 32) & 0xffffffff)
    p.write32(AOP + M3_A2I_SEND1 + 0, msg1 & 0xffffffff)
    p.write32(AOP + M3_A2I_SEND1 + 4, (msg1 >> 32) & 0xffffffff)
    time.sleep(0.1)
    snap(p, "post-send")

    # === STEP 8: Poll I2A ===
    log("polling I2A for 5s...")
    deadline = time.time() + 5
    last_i2a = None
    while time.time() < deadline:
        i2a = p.read32(AOP + M3_I2A_CTRL)
        if i2a != last_i2a:
            t = 5 - (deadline - time.time())
            log(f"  t={t:.2f}s I2A_CTRL={i2a:#x}")
            last_i2a = i2a
        # EMPTY bit is 17. If NOT empty: read RECV0/RECV1
        if i2a != 0 and not (i2a & (1 << 17)):
            r0 = p.read32(AOP + M3_I2A_RECV0) | (p.read32(AOP + M3_I2A_RECV0 + 4) << 32)
            r1 = p.read32(AOP + M3_I2A_RECV1) | (p.read32(AOP + M3_I2A_RECV1 + 4) << 32)
            log(f"  *** I2A msg: msg0={r0:#x} msg1={r1:#x} ***")
            break
        time.sleep(0.05)

    snap(p, "final")
    os._exit(0)


if __name__ == "__main__":
    main()
