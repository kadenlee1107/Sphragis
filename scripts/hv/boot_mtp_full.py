#!/usr/bin/env python3
"""Full MTP ASC bring-up: stage firmware + init deps + boot.

Sequence (matches batos_hv_interactive.py's _mtp_kbd_probe, with
firmware staging prepended — test_mtp_data_write.py confirmed we
can stage __DATA/__OS_LOG even though __TEXT is iBoot-locked):

  1. SMC client start + start_ep(0x20)      [MTP may query SMC early]
  2. DART /arm-io/dart-mtp: BYPASS_DAPF=1, TRANSLATE_ENABLE=1
  3. DockChannel dockchannel-mtp           [MTP protocol transport]
  4. Stage __DATA + __OS_LOG (verify __TEXT iBoot's copy)
  5. StandardASC(u, mtp_base, dart, stream=1)
  6. mtp.boot()                             [CPU_CONTROL.RUN=1 + wait_boot]

If boot succeeds, MTP sends Hello on mgmt EP and we can attach
MTPProtocol for keyboard events.

Usage:
  /usr/bin/python3 scripts/hv/boot_mtp_full.py [--wait-kbd]

Flags:
  --wait-kbd   After boot, attach MTPProtocol and wait for keyboard
               init message. Press keys on Mac keyboard to see HID
               reports.
"""
import argparse
import os
import pathlib
import struct
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "external/m1n1/proxyclient"))

from m1n1.proxy import M1N1Proxy, UartInterface
from m1n1.proxyutils import ProxyUtils, bootstrap_port

MTP_BLOB = ROOT / "firmware/mtp/J604_MtpFirmware.bin"
M1N1_MACHO = ROOT / "external/m1n1/build/m1n1.macho"
MH_MAGIC_64 = 0xfeedfacf
LC_SEGMENT_64 = 0x19


def chainload_patched_m1n1(iface, p, u):
    """Chainload our patched m1n1.macho over the existing proxy session.
    Mirrors batos_hv_interactive.chainload_inline. The patched m1n1 has
    the 118s-watchdog-disable fix (commit 72c606f4) which may matter if
    macOS's watchdog is triggering on MTP ASC boot attempts. After this,
    the same iface/p are valid against the new m1n1.
    """
    from m1n1.macho import MachO
    from m1n1.tgtypes import BootArgs_r1, BootArgs_r2, BootArgs_r3
    from m1n1 import asm
    from m1n1.utils import align

    if not M1N1_MACHO.exists():
        print(f"[chainload] ERROR: {M1N1_MACHO} missing — build it first")
        return False

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
    bootargs_size = 0x4000
    image_size += bootargs_size

    print(f"[chainload] total region size 0x{image_size:x}")
    image_addr = u.malloc(image_size)

    print(f"[chainload] loading kernel image (0x{len(image):x} bytes)...")
    u.compressed_writemem(image_addr, image, True)
    p.dc_cvau(image_addr, len(image))

    print(f"[chainload] copying SEPFW (0x{sepfw_length:x} bytes)...")
    p.memcpy8(image_addr + sepfw_off, sepfw_start, sepfw_length)
    u.adt["chosen"]["memory-map"].SEPFW = (new_base + sepfw_off, sepfw_length)
    u.adt["chosen"]["memory-map"].BootArgs = (new_base + bootargs_off, bootargs_size)
    if preoslog_size:
        p.memcpy8(image_addr + preoslog_off, preoslog_start, preoslog_size)
        u.adt["chosen"]["memory-map"].preoslog = (new_base + preoslog_off, preoslog_size)

    print("[chainload] skipping secondary CPU RVBARs (M4 workaround)")
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
1:
        ldp x4, x5, [x1], #16
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

    print(f"[chainload] entry=0x{entry:x}  reloading into stub at 0x{stub.addr:x}")
    p.reload(stub.addr, new_base + bootargs_off, image_addr, new_base, image_size)

    iface.nop()
    print("[chainload] proxy is alive on new m1n1 (patched)")
    return True


def parse_rkosftab(data):
    off = data.find(b"rkosftab")
    cursor = off + 16
    while True:
        tag = data[cursor:cursor + 4]
        if tag == b"A5PH":
            secoff, secsize = struct.unpack("<II", data[cursor + 4:cursor + 12])
            return data[secoff:secoff + secsize]
        cursor += 16


def parse_macho_segments(macho):
    ncmds = struct.unpack("<I", macho[16:20])[0]
    cur = 32
    segs = []
    for _ in range(ncmds):
        cmd, cmdsize = struct.unpack("<II", macho[cur:cur + 8])
        if cmd == LC_SEGMENT_64:
            segname = macho[cur + 8:cur + 24].rstrip(b"\x00").decode()
            vmaddr, vmsize, fo, fs = struct.unpack(
                "<QQQQ", macho[cur + 24:cur + 56])
            segs.append((segname, vmaddr, vmsize, fo, fs))
        cur += cmdsize
    return segs


def parse_adt_segments(raw):
    segs = []
    for i in range(len(raw) // 32):
        seg = raw[i * 32:(i + 1) * 32]
        phys, iova, remap, size = struct.unpack("<QQQI4x", seg)
        segs.append({"phys": phys, "iova": iova, "size": size})
    return segs


def log(msg):
    print(f"[mtp] {msg}", flush=True)


def stage_firmware(iface, u):
    """Verify __TEXT (iBoot's copy) + stage __DATA + __OS_LOG."""
    blob = MTP_BLOB.read_bytes()
    macho = parse_rkosftab(blob)
    macho_segs = parse_macho_segments(macho)

    mtp = u.adt["/arm-io/mtp"]
    sr = getattr(mtp, "segment-ranges")
    names_raw = getattr(mtp, "segment-names", b"")
    names = names_raw if isinstance(names_raw, str) else \
        names_raw.decode("ascii", errors="replace").strip("\x00")
    names = names.split(";")
    adt_segs = parse_adt_segments(sr)

    adt_by_name = dict(zip(names, adt_segs))
    macho_by_name = {s[0]: s for s in macho_segs}

    # Verify __TEXT matches iBoot's staging
    text_macho = macho_by_name["__TEXT"]
    text_adt = adt_by_name["__TEXT"]
    for probe_off in (0x0, 0x100, 0x200):
        expected = macho[text_macho[3] + probe_off:
                          text_macho[3] + probe_off + 16]
        actual = iface.readmem(text_adt["phys"] + probe_off, 16)
        if actual != expected:
            log(f"__TEXT[+{probe_off:#x}] MISMATCH — ABORT")
            log(f"  iBoot:  {actual.hex()}")
            log(f"  macho:  {expected.hex()}")
            return False
    log("__TEXT: verified iBoot-staged (3 probes match Mach-O)")

    # Stage __DATA
    data_macho = macho_by_name["__DATA"]
    data_adt = adt_by_name["__DATA"]
    payload = macho[data_macho[3]:data_macho[3] + data_macho[4]]
    iface.writemem(data_adt["phys"], payload)
    log(f"__DATA: {data_macho[4]} bytes -> {data_adt['phys']:#x}")

    # Stage __OS_LOG
    oslog_macho = macho_by_name["__OS_LOG"]
    oslog_adt = adt_by_name["__OS_LOG"]
    payload = macho[oslog_macho[3]:oslog_macho[3] + oslog_macho[4]]
    iface.writemem(oslog_adt["phys"], payload)
    log(f"__OS_LOG: {oslog_macho[4]} bytes -> {oslog_adt['phys']:#x}")

    # Verify a readback
    rb = iface.readmem(oslog_adt["phys"], 16)
    log(f"__OS_LOG[0..16] readback: {rb.hex()}")
    if rb[:4] != b"ST I":
        log("readback mismatch (expected 'ST I...') — proceeding anyway")
    return True


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--wait-kbd", action="store_true",
                    help="Attach MTPProtocol + wait for keyboard init")
    ap.add_argument("--boot-timeout", type=int, default=5,
                    help="mtp.boot() wait_boot timeout seconds (default 5)")
    ap.add_argument("--skip-smc", action="store_true",
                    help="Skip SMC startup (debug)")
    args = ap.parse_args()

    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    # Chainload our patched m1n1 (has 118s watchdog disable + other fixes).
    # The stock kmutil-installed bcee7f2 may have quirks that make MTP
    # ASC init fail. On BATOS_SKIP_BOOTSTRAP=1 skip (e.g., when patched
    # m1n1 is already running from a previous session in same power-cycle).
    if os.environ.get("BATOS_SKIP_BOOTSTRAP", "0") != "1":
        log("chainloading patched m1n1...")
        if chainload_patched_m1n1(iface, p, u):
            log("patched m1n1 running — rebuilding proxy utils")
            # Rebuild ProxyUtils against new m1n1 (new heap base / adt)
            u = ProxyUtils(p, heap_size=128 * 1024 * 1024)
        else:
            log("chainload skipped/failed — continuing on current m1n1")

    # CRITICAL: disable M4 AP watchdog. The patched m1n1 only disables
    # this in hv_init(), not at boot. Without it, the watchdog fires
    # every ~118s and reboots the Mac — which is why m1n1 "keeps
    # booting over and over" during long MTP sessions. Replicate the
    # hv.c:151-169 sequence in Python.
    log("disabling M4 AP watchdog (mirror src/hv.c:137-169)...")
    try:
        r1_pre = p.read32(0x3882BC224)
        r2_pre = p.read32(0x3882B8008)
        r3_pre = p.read32(0x3882B802C)
        r4_pre = p.read32(0x3882B8020)
        p.write32(0x3882BC224, 0)
        p.write32(0x3882B8008, 0xffffffff)
        p.write32(0x3882B802C, 0xffffffff)
        p.write32(0x3882B8020, 0xffffffff)
        r1_post = p.read32(0x3882BC224)
        r2_post = p.read32(0x3882B8008)
        r3_post = p.read32(0x3882B802C)
        r4_post = p.read32(0x3882B8020)
        log(f"  WDT: r1 {r1_pre:#x}->{r1_post:#x}  r2 {r2_pre:#x}->{r2_post:#x}  "
            f"r3 {r3_pre:#x}->{r3_post:#x}  r4 {r4_pre:#x}->{r4_post:#x}")
    except Exception as e:
        log(f"  WDT disable err (continuing): {e!r}")

    # Pre-boot CPU state
    mtp_node = u.adt["/arm-io/mtp"]
    mtp_base = mtp_node.get_reg(0)[0]
    cc = p.read32(mtp_base + 0x44)
    cs = p.read32(mtp_base + 0x48)
    log(f"pre-init: mtp_base={mtp_base:#x} CPU_CONTROL={cc:#x} CPU_STATUS={cs:#x}")

    # If FW is already running, stop it first so our staging lands
    # cleanly and mtp.boot() doesn't see a stale state.
    if cc & 0x10:
        log("CPU_CONTROL.RUN=1 already — clearing for clean boot")
        p.write32(mtp_base + 0x44, cc & ~0x10)
        time.sleep(0.1)
        cs = p.read32(mtp_base + 0x48)
        log(f"  after clear: CPU_STATUS={cs:#x}")

    # 1. SMC
    if not args.skip_smc:
        log("starting SMC...")
        from m1n1.fw.smc import SMCClient
        smc_addr = u.adt["arm-io/smc"].get_reg(0)[0]
        smc = SMCClient(u, smc_addr)
        smc.start()
        smc.start_ep(0x20)
        smc.verbose = 0
        log(f"  SMC up @ {smc_addr:#x}")
    else:
        smc = None
        log("skipping SMC")

    # 2. DART /arm-io/dart-mtp
    log("setting up DART /arm-io/dart-mtp (BYPASS_DAPF=1)...")
    from m1n1.hw.dart import DART
    dart = DART.from_adt(u, "/arm-io/dart-mtp", iova_range=(0x8000, 0x100000))
    dart.dart.regs.TCR[1].set(BYPASS_DAPF=1, BYPASS_DART=0, TRANSLATE_ENABLE=1)
    # ISP does this; _mtp_kbd_probe doesn't. Try it — without a valid
    # TTBR the DART refuses translations even with TRANSLATE_ENABLE=1.
    try:
        dart.initialize()
        log("  DART initialized (page tables installed)")
    except Exception as e:
        log(f"  dart.initialize() err (continuing): {e!r}")

    # 3. DockChannel
    log("setting up dockchannel-mtp...")
    from m1n1.hw.dockchannel import DockChannel
    irq_base = u.adt["/arm-io/dockchannel-mtp"].get_reg(1)[0]
    fifo_base = u.adt["/arm-io/dockchannel-mtp"].get_reg(2)[0]
    dc = DockChannel(u, irq_base, fifo_base, 1)
    # Drain stale bytes
    drained = 0
    while dc.rx_count:
        dc.read(dc.rx_count)
        drained += 1
        if drained > 100:
            break
    log(f"  dockchannel irq={irq_base:#x} fifo={fifo_base:#x} drained={drained}")

    # 4. Stage firmware
    log("staging firmware...")
    if not stage_firmware(iface, u):
        return 1

    # 5. StandardASC + 6. boot
    log("creating StandardASC...")
    from m1n1.fw.asc import StandardASC
    from m1n1.fw.asc.base import ASCTimeout
    mtp = StandardASC(u, mtp_base, dart, stream=1)
    mtp.verbose = 1
    mtp.allow_phys = True

    cc = p.read32(mtp_base + 0x44)
    cs = p.read32(mtp_base + 0x48)
    log(f"pre-boot: CPU_CONTROL={cc:#x} CPU_STATUS={cs:#x}")

    log(f"calling mtp.start()-equivalent sequence with {args.boot_timeout}s wait...")
    t0 = time.time()
    boot_err = None
    try:
        # CRITICAL: StandardASC.start() does the full sequence —
        # super().boot() (RUN=1) + mgmt.start() (sends SetIOPPower
        # "host ready, please boot") + wait_boot. We've been calling
        # bare .boot() which skips mgmt.start(), so the FW never gets
        # the "please power on" kick. SMC "just works" because
        # SMCClient.start() does the full sequence via inheritance.
        mtp.asc.CPU_CONTROL.set(RUN=1)
        mtp.mgmt.start()   # sends Mgmt_SetIOPPower(STATE=0x220)
        # Now wait for Hello + EPMap + power-state-0x20 acks.
        deadline = time.time() + args.boot_timeout
        last_snap = time.time()
        # Poll OUTBOX0/OUTBOX1 DIRECTLY — don't trust OUTBOX_CTRL.EMPTY
        # on ascwrap-v6. Each read of OUTBOX1 advances the fifo pointer,
        # so capture all distinct message values we see.
        out_msgs = []
        last_out0 = None
        last_out1 = None
        while time.time() < deadline:
            if (mtp.mgmt.iop_power_state == 0x20 and
                    mtp.mgmt.ap_power_state == 0x20):
                break
            # Manually read OUTBOX — check OUTBOX_CTRL state first
            # but ALSO do a direct read if bits other than bit17 suggest
            # data is present.
            oc = p.read32(mtp_base + 0x8114)
            # On ascwrap-v6, "fifocnt != 0" or "bit 19" set may indicate
            # data — try reading regardless of EMPTY.
            o0 = p.read64(mtp_base + 0x8830)
            o1 = p.read64(mtp_base + 0x8838)
            if (o0 != 0 or (o1 != 0 and o1 != last_out1)) and (o0, o1) != (last_out0, last_out1):
                out_msgs.append((o0, o1))
                log(f"  OUTBOX READ: msg0={o0:#x}  msg1={o1:#x}  OB_CTRL={oc:#x}")
                last_out0, last_out1 = o0, o1
                # Also feed to mgmt for protocol processing
                try:
                    from m1n1.hw.asc import R_INBOX1
                    msg1 = R_INBOX1(o1)
                    ep = mtp.epmap.get(msg1.EP, None)
                    if ep:
                        ep.handle_msg(o0, msg1)
                    else:
                        log(f"    (no EP handler for EP={msg1.EP:#x})")
                except Exception as e:
                    log(f"    handle err: {e!r}")

            dc_rx = dc.rx_count
            if dc_rx:
                data = dc.read(dc_rx)
                log(f"  DOCKCHANNEL RX ({dc_rx}B): {data.hex()}")
            if time.time() - last_snap > 0.5:
                cc_s = p.read32(mtp_base + 0x44)
                cs_s = p.read32(mtp_base + 0x48)
                log(f"  t={int((time.time()-t0)*1000):4d}ms "
                    f"CC={cc_s:#06x} CS={cs_s:#06x} OB={oc:#010x} "
                    f"DC_RX={dc.rx_count} "
                    f"iop={mtp.mgmt.iop_power_state:#x} "
                    f"ap={mtp.mgmt.ap_power_state:#x} "
                    f"out_msgs={len(out_msgs)}")
                last_snap = time.time()
        else:
            boot_err = "timeout"
        if not boot_err:
            dt = time.time() - t0
            log(f"  BOOT OK in {dt*1000:.0f} ms — Hello received!")
    except ASCTimeout as e:
        boot_err = f"ASCTimeout: {e}"
    except Exception as e:
        boot_err = f"{type(e).__name__}: {e}"
        log(f"  unexpected: {e!r}")

    if boot_err:
        dt = time.time() - t0
        log(f"  BOOT FAILED ({boot_err}) after {dt*1000:.0f} ms")
        cc = p.read32(mtp_base + 0x44)
        cs = p.read32(mtp_base + 0x48)
        ob = p.read32(mtp_base + 0x8114)
        log(f"  post-fail: CPU_CONTROL={cc:#x} CPU_STATUS={cs:#x} OUTBOX_CTRL={ob:#x}")
        impl_nz = []
        for off in range(0x100, 0x800, 4):
            v = p.read32(mtp_base + off)
            if v:
                impl_nz.append((off, v))
        if impl_nz:
            log(f"  IMPL non-zero: {[(hex(o), hex(v)) for o, v in impl_nz]}")
        return 2

    if not args.wait_kbd:
        log("boot complete — exiting (use --wait-kbd to attach keyboard)")
        return 0

    # Attach MTPProtocol + wait for keyboard
    log("attaching MTPProtocol...")
    from m1n1.fw.mtp import MTPProtocol
    node = u.adt["/arm-io/dockchannel-mtp/mtp-transport"]
    mp = MTPProtocol(u, node, mtp, dc, smc)
    log("waiting for keyboard init (30s timeout)...")
    try:
        mp.wait_init("keyboard")
    except Exception as e:
        log(f"wait_init failed: {e!r}")
        return 3
    log("KEYBOARD INITIALIZED — press keys on Mac keyboard to see HID reports")
    log("Ctrl+C to exit")
    try:
        while True:
            mtp.work()
            time.sleep(0.01)
    except KeyboardInterrupt:
        log("exiting")
    return 0


if __name__ == "__main__":
    rc = main()
    # Skip pyserial cleanup — closing the CDC-ACM fd drops DTR and
    # wedges m1n1's proxy loop (see 12:45 and 13:45 journal entries).
    # Same pattern as batos_hv_interactive.py's loop mode.
    sys.stdout.flush()
    sys.stderr.flush()
    os._exit(rc if rc is not None else 0)
