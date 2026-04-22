#!/usr/bin/env python3
"""MTP uses ascwrap-v6 too. Apply doorbell + skip-dart-init fix to MTP.
MTP may boot more fully than AOP since journal said it at least consumed
1 INBOX message. Simpler endpoint set likely needs fewer init calls.
"""
import os, pathlib, struct, sys, time
ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))
from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

MTP_BLOB = ROOT / "firmware/mtp/J604_MtpFirmware.bin"
M1N1_MACHO = ROOT / "external/m1n1/build/m1n1.macho"

MTP = 0x394600000  # MTP reg[0]


def log(m): print(f"[mtp-db] {m}", flush=True)


def snap(p, tag):
    c = p.read32(MTP+0x44)
    ie = p.read32(MTP+0x48)
    a2i = p.read32(MTP+0x8110)
    i2a = p.read32(MTP+0x8114)
    out0 = p.read32(MTP+0x8830) | (p.read32(MTP+0x8834) << 32)
    b14 = p.read32(MTP+0xb14)
    log(f"[{tag}] CC={c:#x} IE={ie:#x} A2I={a2i:#x} I2A={i2a:#x} OUT0={out0:#x} b14={b14:#x}")


def ring(p):
    p.write32(MTP + 0x1004, 0x10)
    p.write32(MTP + 0x1014, 0x1)


def send_inbox(p, msg0, msg1=0):
    p.write32(MTP + 0x8800, msg0 & 0xffffffff)
    p.write32(MTP + 0x8804, (msg0 >> 32) & 0xffffffff)
    p.write32(MTP + 0x8808, msg1 & 0xffffffff)
    p.write32(MTP + 0x880c, (msg1 >> 32) & 0xffffffff)


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


def parse_rkosftab(blob):
    """MTP firmware is rkosftab format. Find the A5PH section which is the Mach-O."""
    pos = 0
    while pos + 16 < len(blob):
        magic = blob[pos:pos+4]
        size = int.from_bytes(blob[pos+4:pos+8], "little")
        if magic == b"A5PH":
            macho_start = pos + 16
            return blob[macho_start:macho_start + size - 16]
        pos += size if size > 0 else 16
    return None


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

    # Parse MTP firmware (rkosftab → Mach-O)
    raw = MTP_BLOB.read_bytes()
    if raw[:4] == b"A5PH":
        macho = raw[16:]  # strip rkosftab header if present
    elif b"A5PH" in raw[:256]:
        macho = parse_rkosftab(raw)
        if macho is None:
            log("couldn't find A5PH section")
            macho = raw
    else:
        macho = raw
    # Verify Mach-O magic
    if macho[:4] != b"\xcf\xfa\xed\xfe":
        log(f"not Mach-O, magic={macho[:4].hex()}")
        # Try offset at 0x10 (skip rkosftab header)
        macho = raw[0x10:]
        if macho[:4] != b"\xcf\xfa\xed\xfe":
            # Search for magic
            idx = raw.find(b"\xcf\xfa\xed\xfe")
            if idx >= 0:
                log(f"found Mach-O at offset {idx:#x}")
                macho = raw[idx:]
            else:
                log("no Mach-O found; aborting")
                os._exit(1)
    log(f"MTP Mach-O: {len(macho)} bytes")

    # Stage MTP firmware segments
    mtp_node = u.adt["/arm-io/mtp"]
    sr = getattr(mtp_node, "segment-ranges", None)
    names_raw = getattr(mtp_node, "segment-names", b"")
    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_by = dict(zip(names, parse_adt_segments(sr)))
    mc_by = {seg[0]: seg for seg in macho_segs_fn(macho)}
    log(f"MTP ADT segs: {list(adt_by.keys())}")
    log(f"MTP Mach-O segs: {list(mc_by.keys())}")

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

    # dapf_init for dart-mtp — KNOWN TO HANG on M4. Skip.
    # Instead use BYPASS_DAPF if available. For now, skip dapf.
    log("skipping dapf_init for dart-mtp (known to hang on M4)")
    log("skipping dart.initialize() (clobbers iBoot setup)")

    # Reset OUTBOX_CTRL
    p.write32(MTP + 0x8114, 0x20001)

    snap(p, "pre-RUN")

    # RUN=1
    log("CC.RUN=1...")
    p.write32(MTP + 0x44, 0x10)
    time.sleep(0.3)
    snap(p, "post-RUN")

    # Wait 3s passively for FW to Hello
    log("passive wait 3s for FW Hello...")
    deadline = time.time() + 3
    while time.time() < deadline:
        i2a = p.read32(MTP + 0x8114)
        out0 = p.read32(MTP + 0x8830)
        if not (i2a & (1 << 17)) and out0 != 0:
            out0_full = out0 | (p.read32(MTP + 0x8834) << 32)
            out1_full = p.read32(MTP + 0x8838) | (p.read32(MTP + 0x883c) << 32)
            log(f"*** HELLO: msg0={out0_full:#x} msg1={out1_full:#x} ***")
            typ = (out0_full >> 52) & 0xff
            log(f"*** TYPE={typ:#x} ***")
            if typ == 1:
                log("*** FW sent HELLO! ***")
                # Send HelloAck back
                min_ver = (out0_full >> 0) & 0xffff
                max_ver = (out0_full >> 16) & 0xffff
                log(f"  MIN_VER={min_ver} MAX_VER={max_ver}")
                want_ver = min(11, max_ver)
                reply = (2 << 52) | (want_ver << 16) | want_ver
                log(f"  sending HelloAck: {reply:#x}")
                send_inbox(p, reply, 0)
                ring(p)
                time.sleep(1.0)
                snap(p, "post-helloack")
                # Poll for EPMap
                for i in range(10):
                    time.sleep(0.5)
                    i2a = p.read32(MTP + 0x8114)
                    if not (i2a & (1 << 17)):
                        out0f = p.read32(MTP+0x8830) | (p.read32(MTP+0x8834) << 32)
                        out1f = p.read32(MTP+0x8838) | (p.read32(MTP+0x883c) << 32)
                        typ2 = (out0f >> 52) & 0xff
                        log(f"  got msg TYPE={typ2:#x}  msg0={out0f:#x} msg1={out1f:#x}")
            break
        time.sleep(0.05)
    else:
        log("no Hello in 3s — ring doorbell and wait more")

    # Ring doorbell, maybe FW needs kick
    ring(p)
    time.sleep(3)
    snap(p, "post-doorbell")

    # Send Hello (TYPE=1) in case FW expects AP initiates
    log("sending Mgmt_Hello (TYPE=1)...")
    msg0 = (1 << 52) | (0xff << 16) | 0x1
    send_inbox(p, msg0, 0)
    ring(p)
    time.sleep(3)
    snap(p, "post-hello-ring")

    # Try SetIOPPower variant
    log("sending SetIOPPower(0x220)...")
    msg0 = (6 << 52) | 0x220
    send_inbox(p, msg0, 0)
    ring(p)
    time.sleep(3)
    snap(p, "post-siop")

    # Extended poll
    log("polling 10s for ANY OUTBOX activity...")
    deadline = time.time() + 10
    last = None
    while time.time() < deadline:
        i2a = p.read32(MTP + 0x8114)
        out0 = p.read32(MTP + 0x8830)
        b14 = p.read32(MTP + 0xb14)
        state = (i2a, out0, b14)
        if state != last:
            t = time.time() - (deadline - 10)
            log(f"  t={t:.2f}s I2A={i2a:#x} OUT0={out0:#x} b14={b14:#x}")
            last = state
        if not (i2a & (1 << 17)) and out0 != 0:
            log(f"  *** OUTBOX: {out0:#x} ***")
            break
        time.sleep(0.1)

    snap(p, "final")
    os._exit(0)


if __name__ == "__main__":
    main()
