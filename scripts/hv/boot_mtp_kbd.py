#!/usr/bin/env python3
"""MTP keyboard boot — correct RTBuddy protocol.

Per Asahi rtkit.c: FW always initiates Hello first. AP waits for it,
replies HelloAck, receives EPMap, sends EPMap_Ack, finally gets
SetAPPower=0x20 trigger and boot is complete.

This script:
  1. Chainloads patched m1n1 (has in-C WDT disable now)
  2. Stages MTP firmware __DATA + __OS_LOG (leave __TEXT iBoot-staged)
  3. dart.initialize for dart-mtp (MTP needs it — unlike AOP)
  4. Reset OUTBOX_CTRL = 0x20001
  5. CC.RUN=1
  6. Wait 3s passively for Hello
  7. If silent, ring doorbell bare (no INBOX write) to kick FW
  8. If still silent, send Hello OURSELVES + ring (try reverse protocol)
  9. Process Hello → HelloAck → EPMap(s) → Ack → SetAPPower → boot complete

If boot completes, launch BATOS_HV_MTP_BRIDGE_TO_VUART path.
"""
import os, pathlib, struct, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))
from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

MTP_BLOB = ROOT / "firmware/mtp/J604_MtpFirmware.bin"
M1N1_MACHO = ROOT / "external/m1n1/build/m1n1.macho"
MTP = 0x394600000


def log(m): print(f"[kbd] {m}", flush=True)


def snap(p, tag):
    cc = p.read32(MTP+0x44)
    ie = p.read32(MTP+0x48)
    a2i = p.read32(MTP+0x8110)
    i2a = p.read32(MTP+0x8114)
    out0 = p.read32(MTP+0x8830) | (p.read32(MTP+0x8834) << 32)
    out1 = p.read32(MTP+0x8838) | (p.read32(MTP+0x883c) << 32)
    b14 = p.read32(MTP+0xb14)
    log(f"[{tag}] CC={cc:#x} IE={ie:#x} A2I={a2i:#x} I2A={i2a:#x} "
        f"OUT0={out0:#x} OUT1={out1:#x} b14={b14:#x}")


def ring(p):
    p.write32(MTP + 0x1004, 0x10)
    p.write32(MTP + 0x1014, 0x1)


def send_inbox(p, msg0, msg1=0):
    p.write32(MTP + 0x8800, msg0 & 0xffffffff)
    p.write32(MTP + 0x8804, (msg0 >> 32) & 0xffffffff)
    p.write32(MTP + 0x8808, msg1 & 0xffffffff)
    p.write32(MTP + 0x880c, (msg1 >> 32) & 0xffffffff)


def recv_outbox(p, timeout):
    """Poll OUTBOX for timeout seconds. Return (msg0, msg1) or None."""
    deadline = time.time() + timeout
    while time.time() < deadline:
        i2a = p.read32(MTP + 0x8114)
        if not (i2a & (1 << 17)):  # NOT empty
            o0 = p.read32(MTP+0x8830) | (p.read32(MTP+0x8834) << 32)
            o1 = p.read32(MTP+0x8838) | (p.read32(MTP+0x883c) << 32)
            if o0 != 0:
                return (o0, o1)
        time.sleep(0.02)
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


def handshake(p):
    """Run the RTBuddy handshake.

    Canonical protocol matches m1n1's StandardASC.mgmt.start() +
    Asahi rtkit.c: AP sends SetIOPPower(STATE=0x220) to kick FW
    into init; FW responds with Hello, etc.
    """
    # Step 0: Kick FW with SetIOPPower(0x220). This is what Apple/m1n1
    # do — even though rtkit.c's Hello handler is reactive, it's FW
    # that FIRST needs to see "AP wants me to start" via SetIOPPower.
    log("handshake: sending SetIOPPower(0x220) + doorbell...")
    send_inbox(p, 0x60000000000220, 0)
    ring(p)

    reply = recv_outbox(p, 5)
    if not reply:
        log("handshake: no reply in 5s; ring again")
        ring(p)
        reply = recv_outbox(p, 3)
    if not reply:
        log("handshake: trying Hello-first fallback (AP-initiates-Hello)")
        send_inbox(p, (1 << 52) | (0xff << 16) | 0x1, 0)
        ring(p)
        reply = recv_outbox(p, 3)
    if not reply:
        log("handshake: total silence from FW")
        return False

    # Process Hello
    msg0, msg1 = reply
    typ = (msg0 >> 52) & 0xff
    log(f"first msg TYPE={typ:#x}  msg0={msg0:#x}  msg1={msg1:#x}")

    iop_power = ap_power = 0
    endpoints = []

    for step in range(30):
        typ = (msg0 >> 52) & 0xff
        if typ == 1:
            min_ver = msg0 & 0xffff
            max_ver = (msg0 >> 16) & 0xffff
            want = min(11, max_ver)
            log(f"  Hello MIN={min_ver} MAX={max_ver}; reply HelloAck want={want}")
            send_inbox(p, (2 << 52) | (want << 16) | want, 0)
            ring(p)
        elif typ == 7:  # IOPPowerAck
            iop_power = msg0 & 0xffff
            log(f"  IOPPowerAck STATE={iop_power:#x}")
        elif typ == 8:  # EPMap
            last = (msg0 >> 51) & 1
            base = (msg0 >> 32) & 7
            bitmap = msg0 & 0xffffffff
            for i in range(32):
                if bitmap & (1 << i):
                    endpoints.append(32 * base + i)
            more = 0 if last else 1
            log(f"  EPMap LAST={last} BASE={base} BITMAP={bitmap:#x} "
                f"(eps so far: {endpoints})")
            ack = (8 << 52) | ((last & 1) << 51) | (base << 32) | more
            send_inbox(p, ack, 0)
            ring(p)
            if last:
                log(f"  endpoints: {endpoints}; sending SetAPPower(0x20)...")
                send_inbox(p, (0xb << 52) | 0x20, 0)
                ring(p)
        elif typ == 0xb:  # SetAPPower ack
            ap_power = msg0 & 0xffff
            log(f"  APPowerAck STATE={ap_power:#x}")
            if iop_power == 0x20 and ap_power == 0x20:
                log(f"*** BOOT COMPLETE — {len(endpoints)} endpoints ***")
                return True
        elif typ == 4:  # Pong
            log("  Pong")
        else:
            log(f"  unknown TYPE={typ:#x} (ignoring)")

        nxt = recv_outbox(p, 3)
        if not nxt:
            log(f"  no msg after 3s (iop={iop_power:#x} ap={ap_power:#x})")
            break
        msg0, msg1 = nxt

    return False


def main():
    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    log("chainloading patched m1n1 (with in-C WDT disable for t8132)...")
    chainload(iface, p, u)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    # Find Mach-O in MTP blob
    raw = MTP_BLOB.read_bytes()
    idx = raw.find(b"\xcf\xfa\xed\xfe")
    if idx < 0:
        log("no Mach-O in MTP blob"); os._exit(1)
    macho = raw[idx:]
    log(f"MTP Mach-O at file offset {idx:#x}, size {len(macho)}")

    # Stage firmware segments
    mtp_node = u.adt["/arm-io/mtp"]
    sr = getattr(mtp_node, "segment-ranges", None)
    names_raw = getattr(mtp_node, "segment-names", b"")
    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_by = dict(zip(names, parse_adt_segments(sr)))
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
        log(f"  staged {nm}: {m[4]}B")

    # MTP DART setup — MTP needs dart.initialize() (unlike AOP)
    from m1n1.hw.dart import DART
    try:
        dart_node = u.adt["/arm-io/dart-mtp"]
        vm_base = getattr(dart_node, "vm-base", None) or 0x8000
        dart = DART.from_adt(u, "/arm-io/dart-mtp",
                             iova_range=(vm_base, 0x1000000000))
        dart.initialize()
        log("dart-mtp initialized")
    except Exception as e:
        log(f"dart-mtp init err: {e!r}")

    # Reset OUTBOX_CTRL
    p.write32(MTP + 0x8114, 0x20001)

    snap(p, "pre-RUN")

    # Clear RUN if previously set
    pre_cc = p.read32(MTP + 0x44)
    if pre_cc & 0x10:
        log(f"MTP was running (CC={pre_cc:#x}) — clearing RUN")
        p.write32(MTP + 0x44, pre_cc & ~0x10)
        time.sleep(0.1)

    # RUN=1
    log("CC.RUN=1...")
    p.write32(MTP + 0x44, 0x10)
    time.sleep(0.5)
    snap(p, "post-RUN")

    # Run the RTBuddy handshake
    if handshake(p):
        log("*** MTP IS UP — endpoints ready for keyboard bridging ***")
    else:
        log("*** MTP boot incomplete ***")

    snap(p, "final")
    os._exit(0)


if __name__ == "__main__":
    main()
