#!/usr/bin/env python3
"""Boot AOP with the ASCWrap-v6 doorbell ring.

Disassembled AppleASCWrapV6::_triggerFiqNmi() from macOS 26.3
kernelcache (J604, mac16j) reveals the doorbell mechanism:

  _triggerFiqNmi(self):
    ldr x8, [x0, #0x100]    # x8 = this->mmio_base (reg[0])
    mov w9, #0x10
    str w9, [x8, #0x1004]   # write 0x10 to MMIO +0x1004
    mov w9, #1
    str w9, [x8, #0x1014]   # write 0x1 to MMIO +0x1014
    ret

Also confirmed layout via _inbox / _outbox methods:
  _inbox(void* msg):  stp x8, x9, [base + 0x8800]  # 128-bit write
  _outbox(void* msg): ldp x8, x9, [base + 0x8830]  # 128-bit read

So mailbox IS at +0x8800 / +0x8830 (classical), AS we've been using.
The missing bit is ringing the doorbell after each INBOX write so
FW's FIQ NMI fires and drains INBOX.

Strategy:
  1. skip dart.initialize() (preserves iBoot's DART config)
  2. dapf_init for /arm-io/dart-aop (no hang)
  3. stage firmware + bootargs
  4. CC.RUN=1
  5. write INBOX (+0x8800/+0x8808) — SetIOPPower(0x220)
  6. ** RING DOORBELL: +0x1004=0x10, +0x1014=1 **
  7. poll +0x8830 for FW Hello / IOPPowerAck
"""
import os, pathlib, struct, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

AOP_BLOB = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
M1N1_MACHO = ROOT / "external/m1n1/build/m1n1.macho"

AOP = 0x390600000  # reg[0] base

# MMIO offsets (verified from ASCWrapV6 disasm)
CPU_CONTROL  = 0x0044  # RUN bit 4
IRQ_EN       = 0x0048  # (we've been calling this "CS")
IRQ_ACK      = 0x004c
FIQ_NMI_CFG  = 0x1004  # write 0x10
FIQ_NMI_ARM  = 0x1014  # write 0x1
A2I_CTRL     = 0x8110
I2A_CTRL     = 0x8114
INBOX0       = 0x8800  # 128-bit INBOX (x0 + x1 paired)
INBOX1       = 0x8808
OUTBOX0      = 0x8830
OUTBOX1      = 0x8838


def log(m): print(f"[db] {m}", flush=True)


def snap(p, tag):
    cc = p.read32(AOP + CPU_CONTROL)
    ie = p.read32(AOP + IRQ_EN)
    a2i = p.read32(AOP + A2I_CTRL)
    i2a = p.read32(AOP + I2A_CTRL)
    ob0 = p.read32(AOP + OUTBOX0) | (p.read32(AOP + OUTBOX0 + 4) << 32)
    ob1 = p.read32(AOP + OUTBOX1) | (p.read32(AOP + OUTBOX1 + 4) << 32)
    log(f"[{tag}] CC={cc:#x} IRQ_EN={ie:#x} A2I={a2i:#x} I2A={i2a:#x} OUT0={ob0:#x} OUT1={ob1:#x}")


def ring_doorbell(p):
    """Apple ASCWrapV6::_triggerFiqNmi path."""
    log("  RING DOORBELL: +0x1004=0x10, +0x1014=1")
    p.write32(AOP + FIQ_NMI_CFG, 0x10)
    p.write32(AOP + FIQ_NMI_ARM, 0x1)


def send_inbox(p, msg0, msg1):
    """Write 128-bit INBOX message. Apple does atomic stp but our
    proxy can only do 32-bit at a time — try in order."""
    log(f"  send INBOX msg0={msg0:#x} msg1={msg1:#x}")
    p.write32(AOP + INBOX0 + 0, msg0 & 0xffffffff)
    p.write32(AOP + INBOX0 + 4, (msg0 >> 32) & 0xffffffff)
    p.write32(AOP + INBOX1 + 0, msg1 & 0xffffffff)
    p.write32(AOP + INBOX1 + 4, (msg1 >> 32) & 0xffffffff)


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

    for addr, val in [(0x3882BC224, 0), (0x3882B8008, 0xffffffff),
                      (0x3882B802C, 0xffffffff), (0x3882B8020, 0xffffffff)]:
        try: p.write32(addr, val)
        except: pass

    # Stage firmware
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
            skip.add(nm); log(f"  {nm}: iBoot-staged")

    for nm in names:
        if nm in skip or nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0 or m[4] > a["size"]: continue
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], macho[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], macho[m[3]:m[3]+m[4]])
        log(f"  staged {nm}: {m[4]}B -> {a['phys']:#x}")

    # Bootargs
    from m1n1.fw.aop.base import AOPBase
    aopb = AOPBase(u)
    try:
        aopb.update_bootargs({'p0CE':0x20000,'laCn':0,'tPOA':1,'gila':0x80})
        log("bootargs updated")
    except Exception as e:
        log(f"bootargs err: {e!r}")

    # dapf_init for dart-aop only (not dart-mtp which hangs)
    try:
        p.dapf_init("/arm-io/dart-aop")
        log("dapf_init OK")
    except Exception as e:
        log(f"dapf err: {e!r}")

    # Reset OUTBOX_CTRL
    p.write32(AOP + I2A_CTRL, 0x20001)

    snap(p, "pre-RUN")

    # Kick RUN=1
    log("CC.RUN=1...")
    p.write32(AOP + CPU_CONTROL, 0x10)
    time.sleep(0.3)
    snap(p, "post-RUN")

    # Send SetIOPPower(STATE=0x220) — TYPE=6 at bits 59:52
    log("sending SetIOPPower(0x220)...")
    msg0 = 0x60000000000220  # TYPE=6, STATE=0x220
    msg1 = 0x0               # EP=0 (mgmt)
    send_inbox(p, msg0, msg1)
    time.sleep(0.05)

    snap(p, "post-send-noring")

    # *** RING DOORBELL ***
    ring_doorbell(p)

    snap(p, "post-doorbell")

    # Poll OUTBOX hard
    log("polling OUTBOX (5s)...")
    deadline = time.time() + 5
    last = None
    got = False
    while time.time() < deadline:
        i2a = p.read32(AOP + I2A_CTRL)
        ob0 = p.read32(AOP + OUTBOX0)
        if (i2a, ob0) != last:
            t = 5 - (deadline - time.time())
            log(f"  t={t:.2f}s I2A={i2a:#x} OB0={ob0:#x}")
            last = (i2a, ob0)
        if not (i2a & (1 << 17)) and ob0 != 0:
            full0 = p.read32(AOP+OUTBOX0) | (p.read32(AOP+OUTBOX0+4) << 32)
            full1 = p.read32(AOP+OUTBOX1) | (p.read32(AOP+OUTBOX1+4) << 32)
            typ = (full0 >> 52) & 0xff
            log(f"  *** OUTBOX: msg0={full0:#x} msg1={full1:#x} TYPE={typ:#x} ***")
            got = True
            break
        time.sleep(0.05)

    snap(p, "final")

    if got:
        log("*** AOP RESPONDED! FW is sending OUTBOX msgs ***")
    else:
        log("still no OUTBOX. Try different doorbell patterns or check FIQ path.")

    os._exit(0)


if __name__ == "__main__":
    main()
