#!/usr/bin/env python3
"""MTP boot with EXPLICIT DART segment mapping.

Prior attempts: dart.initialize() created blank page tables but never
mapped MTP firmware segments. MTP FW does DMA via DART to its __DATA
iova — without a page-table entry it silently faults.

AppleA7IOP::_mapFirmware and AppleA7IOP::_dartMapiBootFirmware in the
extracted kext do exactly this (symbols seen). We replicate via
dart.iomap_at(stream=1, iova, phys, size) for each segment.

Steps:
 1. (optional) chainload patched m1n1 (safe WDT fix variant)
 2. SMC boot via SMCClient (dependency)
 3. Stage MTP __DATA + __OS_LOG
 4. DART.initialize() for dart-mtp stream 1
 5. *** NEW: dart.iomap_at for each MTP segment ***
 6. DockChannel ready
 7. MTP StandardASC.boot + mgmt.start with 15s wait
 8. If boot completes, attach keyboard
"""
import os
import pathlib
import struct
import sys
import time

ROOT = pathlib.Path("/home/kaden-lee/code/Bat_OS")
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))
from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

MTP_BLOB = ROOT / "firmware/mtp/J604_MtpFirmware.bin"
M1N1_MACHO = ROOT / "external/m1n1/build/m1n1.macho"
MTP = 0x394600000


def log(m): print(f"[dartmap] {m}", flush=True)


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

    if os.environ.get("MTP_CHAINLOAD", "0") == "1":
        log("chainloading patched m1n1 (safe WDT variant)...")
        chainload(iface, p, u)
        u = ProxyUtils(p, heap_size=128 * 1024 * 1024)
    else:
        log("using stock m1n1 (no chainload — SMC is picky)")

    # SMC boot (must come before DART init because SMC uses DRAM pool)
    log("starting SMC (dependency)...")
    from m1n1.fw.smc import SMCClient
    smc = SMCClient(u, u.adt["arm-io/smc"].get_reg(0)[0])
    smc.start()
    smc.start_ep(0x20)
    smc.verbose = 0
    log(f"  SMC up: iop={smc.mgmt.iop_power_state:#x} ap={smc.mgmt.ap_power_state:#x}")

    # Stage MTP firmware
    raw = MTP_BLOB.read_bytes()
    idx = raw.find(b"\xcf\xfa\xed\xfe")
    macho = raw[idx:]
    mtp_node = u.adt["/arm-io/mtp"]
    sr = getattr(mtp_node, "segment-ranges", None)
    names_raw = getattr(mtp_node, "segment-names", b"")
    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_by = dict(zip(names, parse_adt_segments(sr)))
    mc_by = {seg[0]: seg for seg in macho_segs_fn(macho)}

    log("staging MTP firmware segments...")
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
            skip.add(nm)
            log(f"  {nm}: iBoot-staged (skip)")
    for nm in names:
        if nm in skip or nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0 or m[4] > a["size"]: continue
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], macho[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], macho[m[3]:m[3]+m[4]])
        log(f"  staged {nm}: {m[4]}B -> phys {a['phys']:#x}, iova {a['iova']:#x}")

    # DART setup
    log("setting up DART-MTP stream 1...")
    from m1n1.hw.dart import DART
    dart = DART.from_adt(u, "/arm-io/dart-mtp", iova_range=(0x8000, 0x100000))
    dart.dart.regs.TCR[1].set(BYPASS_DAPF=1, BYPASS_DART=0, TRANSLATE_ENABLE=1)
    dart.initialize()
    log("  DART initialized (blank page tables installed)")

    # *** THE MISSING PIECE ***: map MTP firmware segments into DART
    log("mapping MTP firmware segments into DART (iomap_at)...")
    PAGE_SIZE = 0x4000  # Apple's DART page size
    def align_up(x, a): return (x + a - 1) & ~(a - 1)

    for nm in names:
        if nm not in adt_by: continue
        a = adt_by[nm]
        phys = a["phys"]
        iova = a["iova"]
        size = align_up(a["size"], PAGE_SIZE)
        # Align phys down to PAGE_SIZE
        phys_aligned = phys & ~(PAGE_SIZE - 1)
        iova_aligned = iova & ~(PAGE_SIZE - 1)
        if phys != phys_aligned or iova != iova_aligned:
            log(f"  {nm}: phys={phys:#x}->{phys_aligned:#x} iova={iova:#x}->{iova_aligned:#x}")
        try:
            dart.dart.iomap_at(1, iova_aligned, phys_aligned, size)
            log(f"  {nm}: mapped stream=1 iova={iova_aligned:#x} phys={phys_aligned:#x} size={size:#x}")
        except Exception as e:
            log(f"  {nm}: iomap_at FAILED: {type(e).__name__}: {e}")

    # Also map stream 0 (some FW uses stream 0 for mailbox/control)
    log("also trying stream 0...")
    dart.dart.regs.TCR[0].set(BYPASS_DAPF=1, BYPASS_DART=0, TRANSLATE_ENABLE=1)
    for nm in names:
        if nm not in adt_by: continue
        a = adt_by[nm]
        phys_aligned = a["phys"] & ~(PAGE_SIZE - 1)
        iova_aligned = a["iova"] & ~(PAGE_SIZE - 1)
        size = align_up(a["size"], PAGE_SIZE)
        try:
            dart.dart.iomap_at(0, iova_aligned, phys_aligned, size)
            log(f"  stream 0: {nm} iova={iova_aligned:#x} mapped")
        except Exception as e:
            log(f"  stream 0: {nm}: {type(e).__name__}: {e}")

    # DockChannel
    log("setting up DockChannel-MTP...")
    from m1n1.hw.dockchannel import DockChannel
    irq_base = u.adt["/arm-io/dockchannel-mtp"].get_reg(1)[0]
    fifo_base = u.adt["/arm-io/dockchannel-mtp"].get_reg(2)[0]
    dc = DockChannel(u, irq_base, fifo_base, 1)
    while dc.rx_count:
        dc.read(dc.rx_count)
    log(f"  dockchannel irq={irq_base:#x} fifo={fifo_base:#x}")

    # MTP ASC boot
    log("creating MTP StandardASC + booting...")
    from m1n1.fw.asc import StandardASC
    mtp = StandardASC(u, MTP, dart, stream=1)
    mtp.verbose = 1
    mtp.allow_phys = True

    cc = p.read32(MTP + 0x44); cs = p.read32(MTP + 0x48)
    log(f"pre-boot: CC={cc:#x} CS={cs:#x}")
    try:
        mtp.asc.CPU_CONTROL.set(RUN=1)
        mtp.mgmt.start()
        mtp.mgmt.wait_boot(15)
        log(f"*** MTP BOOT OK *** iop={mtp.mgmt.iop_power_state:#x} ap={mtp.mgmt.ap_power_state:#x}")
        log(f"  endpoints: {list(mtp.epmap.keys())}")

        # If we got here, try keyboard
        log("starting MTPProtocol for keyboard...")
        from m1n1.fw.mtp import MTPProtocol
        node = u.adt["/arm-io/dockchannel-mtp/mtp-transport"]
        mp = MTPProtocol(u, node, mtp, dc, smc)
        mp.wait_init("keyboard")
        log("*** KEYBOARD INITIALIZED — type on Mac keyboard for HID events ***")

        # Poll briefly
        for i in range(30):
            mp.work_pending()
            mtp.work()
            time.sleep(0.1)
        log("30s done — keyboard was active")

    except Exception as e:
        log(f"MTP boot FAIL: {type(e).__name__}: {e}")
        log(f"final: CC={p.read32(MTP+0x44):#x} CS={p.read32(MTP+0x48):#x} "
            f"A2I={p.read32(MTP+0x8110):#x} I2A={p.read32(MTP+0x8114):#x} "
            f"OUT0={p.read32(MTP+0x8830):#x} b14={p.read32(MTP+0xb14):#x}")

    os._exit(0)


if __name__ == "__main__":
    main()
