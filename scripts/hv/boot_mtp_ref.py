#!/usr/bin/env python3
"""MTP boot matching external/m1n1/proxyclient/experiments/mtp.py reference.

Differences from boot_mtp_full.py:
  - Call p.dapf_init_all() BEFORE DART (patched m1n1 may fix M4 hang)
  - BYPASS_DAPF=0 (reference uses 0; we were using 1)
  - Plain mtp.boot() — no mgmt.start() SetIOPPower kick
  - Chainload patched m1n1 at startup

Goal: replicate the M1/M2-working sequence as closely as possible
and see if the ascwrap-v6 FW boots under those conditions.
"""
import os, pathlib, struct, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

MTP_BLOB = ROOT / "firmware/mtp/J604_MtpFirmware.bin"
M1N1_MACHO = ROOT / "external/m1n1/build/m1n1.macho"


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


def parse_rkosftab(data):
    cursor = data.find(b"rkosftab") + 16
    while True:
        tag = data[cursor:cursor+4]
        if tag == b"A5PH":
            o, s = struct.unpack("<II", data[cursor+4:cursor+12])
            return data[o:o+s]
        cursor += 16


def macho_segs_fn(macho):
    ncmds = struct.unpack("<I", macho[16:20])[0]
    cur, segs = 32, []
    for _ in range(ncmds):
        cmd, sz = struct.unpack("<II", macho[cur:cur+8])
        if cmd == 0x19:
            name = macho[cur+8:cur+24].rstrip(b"\x00").decode()
            vm, vmsz, fo, fs = struct.unpack("<QQQQ", macho[cur+24:cur+56])
            segs.append((name, vm, vmsz, fo, fs))
        cur += sz
    return segs


def log(m):
    print(f"[mtp] {m}", flush=True)


def main():
    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=64 * 1024 * 1024)

    if os.environ.get("BATOS_SKIP_BOOTSTRAP", "0") != "1":
        log("chainloading patched m1n1...")
        chainload(iface, p, u)
        u = ProxyUtils(p, heap_size=64 * 1024 * 1024)
        log("  patched m1n1 up")

    # 1. SMC (like reference experiments/mtp.py)
    from m1n1.fw.smc import SMCClient
    smc_addr = u.adt["arm-io/smc"].get_reg(0)[0]
    smc = SMCClient(u, smc_addr)
    try:
        smc.start(); smc.start_ep(0x20); smc.verbose = 0
        log("SMC up")
    except Exception as e:
        log(f"SMC err (may be already running): {e!r}")

    # 2. DAPF init — reference calls this. Our journal said it hangs
    # on M4 with stock m1n1. Patched m1n1 may have fixed it.
    skip_dapf = os.environ.get("BATOS_SKIP_DAPF", "0") == "1"
    if not skip_dapf:
        log("calling p.dapf_init_all() (60s timeout)...")
        saved = iface.dev.timeout
        iface.dev.timeout = 60
        t0 = time.time()
        try:
            p.dapf_init_all()
            dt = time.time() - t0
            log(f"  dapf_init_all OK in {dt*1000:.0f}ms")
            dapf_ok = True
        except Exception as e:
            dt = time.time() - t0
            log(f"  dapf_init_all FAILED after {dt*1000:.0f}ms: {type(e).__name__}: {e}")
            dapf_ok = False
        finally:
            iface.dev.timeout = saved
    else:
        log("BATOS_SKIP_DAPF=1 — skipping dapf_init_all")
        dapf_ok = False

    # 3. DART — reference uses BYPASS_DAPF=0 if dapf_init_all ran OK
    from m1n1.hw.dart import DART
    dart = DART.from_adt(u, "/arm-io/dart-mtp", iova_range=(0x8000, 0x100000))
    bypass_dapf = 0 if dapf_ok else 1
    dart.dart.regs.TCR[1].set(BYPASS_DAPF=bypass_dapf, BYPASS_DART=0,
                              TRANSLATE_ENABLE=1)
    log(f"DART configured: BYPASS_DAPF={bypass_dapf} TRANSLATE_ENABLE=1")

    # 4. DockChannel
    from m1n1.hw.dockchannel import DockChannel
    irq_base = u.adt["/arm-io/dockchannel-mtp"].get_reg(1)[0]
    fifo_base = u.adt["/arm-io/dockchannel-mtp"].get_reg(2)[0]
    dc = DockChannel(u, irq_base, fifo_base, 1)
    while dc.rx_count:
        dc.read(dc.rx_count)
    log(f"dockchannel ready (irq={irq_base:#x} fifo={fifo_base:#x})")

    # 5. Stage firmware (skip __TEXT, stage __DATA + __OS_LOG)
    blob = MTP_BLOB.read_bytes()
    macho = parse_rkosftab(blob)
    mtp_node = u.adt["/arm-io/mtp"]
    sr = getattr(mtp_node, "segment-ranges")
    names_raw = getattr(mtp_node, "segment-names")
    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_segs = []
    for i in range(len(sr) // 32):
        s = sr[i*32:(i+1)*32]
        phys, iova, remap, size = struct.unpack("<QQQI4x", s)
        adt_segs.append((phys, iova, size))
    adt_by = dict(zip(names, adt_segs))
    mc_by = {m[0]: m for m in macho_segs_fn(macho)}
    for nm in ("__DATA", "__OS_LOG"):
        if nm in adt_by and nm in mc_by:
            seg = mc_by[nm]
            iface.writemem(adt_by[nm][0], macho[seg[3]:seg[3]+seg[4]])
            log(f"staged {nm}: {seg[4]}B -> {adt_by[nm][0]:#x}")

    # 6. Create StandardASC + call plain mtp.boot() (match reference)
    from m1n1.fw.asc import StandardASC
    from m1n1.fw.asc.base import ASCTimeout
    mtp_base = mtp_node.get_reg(0)[0]
    mtp = StandardASC(u, mtp_base, dart, stream=1)
    mtp.verbose = 3
    mtp.allow_phys = True

    log("calling mtp.boot() (plain, no mgmt.start kick)...")
    t0 = time.time()
    try:
        # mtp.boot() = super().boot() + mgmt.wait_boot(1)
        # Use a longer timeout — do a 15s wait instead.
        mtp.asc.CPU_CONTROL.set(RUN=1)
        deadline = time.time() + 15
        last_snap = time.time()
        while time.time() < deadline:
            if (mtp.mgmt.iop_power_state == 0x20 and
                    mtp.mgmt.ap_power_state == 0x20):
                break
            mtp.work()
            dc_rx = dc.rx_count
            if dc_rx:
                data = dc.read(dc_rx)
                log(f"  DC RX ({dc_rx}B): {data.hex()}")
            if time.time() - last_snap > 0.5:
                cc = p.read32(mtp_base + 0x44)
                cs = p.read32(mtp_base + 0x48)
                oc = p.read32(mtp_base + 0x8114)
                b14 = p.read32(mtp_base + 0x0b14)
                log(f"  t={int((time.time()-t0)*1000):4d}ms "
                    f"CC={cc:#x} CS={cs:#x} OB={oc:#x} +b14={b14:#x} "
                    f"iop={mtp.mgmt.iop_power_state:#x} "
                    f"ap={mtp.mgmt.ap_power_state:#x}")
                last_snap = time.time()
        else:
            log("BOOT TIMEOUT — no Hello in 15s")
            os._exit(2)
        log(f"BOOT OK in {(time.time()-t0)*1000:.0f}ms — Hello received!")
    except ASCTimeout as e:
        log(f"ASCTimeout: {e}")
        os._exit(2)
    except Exception as e:
        log(f"boot err: {type(e).__name__}: {e}")
        import traceback; traceback.print_exc()
        os._exit(2)

    log("Attempting keyboard init...")
    from m1n1.fw.mtp import MTPProtocol
    node = u.adt["/arm-io/dockchannel-mtp/mtp-transport"]
    mp = MTPProtocol(u, node, mtp, dc, smc)
    try:
        mp.wait_init("keyboard")
        log("KEYBOARD INITIALIZED!")
    except Exception as e:
        log(f"wait_init err: {type(e).__name__}: {e}")
    os._exit(0)


if __name__ == "__main__":
    main()
