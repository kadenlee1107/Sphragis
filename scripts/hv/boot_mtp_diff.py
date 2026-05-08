#!/usr/bin/env python3
"""MTP boot with step-by-step register diffs.

Problem: we've been staring at register state snapshots without
tracking WHAT changes BETWEEN our actions. ascwrap-v6 semantics are
different from m1n1's M1/M2-era decoders. This script snapshots
reg[0] state after each action:

  A. Initial (post-chainload, pre-anything)
  B. After staging __DATA + __OS_LOG
  C. After CPU_CONTROL.RUN=1
  D. After SetIOPPower (TYPE=6 STATE=0x220)
  E. After Ping (TYPE=3)
  F. After Pong (TYPE=4) — in case FW sends Ping first
  G. After StartEP for EPs 1,2,4,8 (we already saw those from SMC)

Prints DIFFS between consecutive snapshots. Any bit that changes
is a hint about what the FW is doing or waiting for.

Includes patched-m1n1 chainload.
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


def macho_segs(macho):
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


def snapshot(p, base):
    """Read the ~200 most-interesting registers (not OUTBOX0/1 which
    advance the FIFO) and return a dict."""
    s = {}
    for off in range(0, 0x1000, 4):
        # Skip OUTBOX0 and OUTBOX1 read locations so we don't pop
        # anything. OUTBOX is at +0x8830-+0x883c.
        s[off] = p.read32(base + off)
    # Plus selected mailbox regs OTHER than OUTBOX data
    for off in (0x8110, 0x8114):  # INBOX_CTRL, OUTBOX_CTRL
        s[off] = p.read32(base + off)
    return s


def diff(a, b, label):
    print(f"\n=== DIFF: {label} ===")
    changes = [(k, a[k], b[k]) for k in a if a.get(k) != b.get(k)]
    if not changes:
        print("  (no changes)")
    for k, va, vb in changes:
        print(f"  [+{k:#06x}] {va:#010x} -> {vb:#010x}")


def read_outbox(p, base):
    """Read OUTBOX0/1 ONCE (does not re-read to avoid fifo advance)."""
    return p.read64(base + 0x8830), p.read64(base + 0x8838)


def send_mgmt(p, base, type_, body=0, ep=0):
    """Send a management message to INBOX."""
    msg0 = (type_ << 52) | body
    msg1 = ep
    p.write64(base + 0x8800, msg0)
    p.write64(base + 0x8808, msg1)


def log(msg):
    print(f"[mtp] {msg}", flush=True)


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

    # Disable M4 AP watchdog (mirror src/hv.c:137-169).
    log("disabling M4 AP watchdog...")
    try:
        p.write32(0x3882BC224, 0)
        p.write32(0x3882B8008, 0xffffffff)
        p.write32(0x3882B802C, 0xffffffff)
        p.write32(0x3882B8020, 0xffffffff)
    except Exception as e:
        log(f"  WDT disable err: {e!r}")

    mtp_base = u.adt["/arm-io/mtp"].get_reg(0)[0]
    log(f"MTP @ {mtp_base:#x}")

    # Set up SMC + DART + dockchannel FIRST (FW deps)
    from m1n1.fw.smc import SMCClient
    from m1n1.hw.dart import DART
    from m1n1.hw.dockchannel import DockChannel

    smc_addr = u.adt["arm-io/smc"].get_reg(0)[0]
    smc = SMCClient(u, smc_addr)
    try:
        smc.start(); smc.start_ep(0x20); smc.verbose = 0
        log("  SMC up")
    except Exception as e:
        log(f"  SMC skip (already running?): {e!r}")

    dart = DART.from_adt(u, "/arm-io/dart-mtp", iova_range=(0x8000, 0x100000))
    dart.dart.regs.TCR[1].set(BYPASS_DAPF=1, BYPASS_DART=0, TRANSLATE_ENABLE=1)
    try:
        dart.initialize()
    except Exception:
        pass
    log("  DART up")

    dc_irq = u.adt["/arm-io/dockchannel-mtp"].get_reg(1)[0]
    dc_fifo = u.adt["/arm-io/dockchannel-mtp"].get_reg(2)[0]
    dc = DockChannel(u, dc_irq, dc_fifo, 1)
    while dc.rx_count:
        dc.read(dc.rx_count)
    log("  dockchannel up")

    # STAGE A: initial snapshot
    log("snapshot A: initial (post-chainload, pre-stage)")
    snap_a = snapshot(p, mtp_base)

    # STAGE firmware
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
    mc_by = {m[0]: m for m in macho_segs(macho)}
    for nm in ("__DATA", "__OS_LOG"):
        if nm in adt_by and nm in mc_by:
            seg = mc_by[nm]
            iface.writemem(adt_by[nm][0], macho[seg[3]:seg[3]+seg[4]])
    log("staged __DATA + __OS_LOG")

    # STAGE B: after staging
    log("snapshot B: after staging")
    snap_b = snapshot(p, mtp_base)
    diff(snap_a, snap_b, "A -> B (staging __DATA + __OS_LOG)")

    # STAGE C: after CPU_CONTROL.RUN=1
    log("setting CPU_CONTROL.RUN=1")
    p.write32(mtp_base + 0x44, 0x10)
    time.sleep(0.2)
    log("snapshot C: after RUN=1")
    snap_c = snapshot(p, mtp_base)
    diff(snap_b, snap_c, "B -> C (RUN=1)")
    o0, o1 = read_outbox(p, mtp_base)
    log(f"  C outbox: msg0={o0:#x} msg1={o1:#x}")

    # STAGE D: after SetIOPPower (TYPE=6, STATE=0x220)
    log("sending Mgmt_SetIOPPower(STATE=0x220) to INBOX")
    send_mgmt(p, mtp_base, 6, 0x220)
    time.sleep(1.0)  # Give FW time to process + reply
    log("snapshot D: after SetIOPPower")
    snap_d = snapshot(p, mtp_base)
    diff(snap_c, snap_d, "C -> D (SetIOPPower)")
    o0, o1 = read_outbox(p, mtp_base)
    log(f"  D outbox: msg0={o0:#x} msg1={o1:#x}")

    # STAGE E: after Ping
    log("sending Mgmt_Ping(TYPE=3)")
    send_mgmt(p, mtp_base, 3, 0)
    time.sleep(1.0)
    log("snapshot E: after Ping")
    snap_e = snapshot(p, mtp_base)
    diff(snap_d, snap_e, "D -> E (Ping)")
    o0, o1 = read_outbox(p, mtp_base)
    log(f"  E outbox: msg0={o0:#x} msg1={o1:#x}")

    # STAGE F: send HelloAck to see if FW was waiting
    log("sending Mgmt_HelloAck(TYPE=2, versions=12..12)")
    # MIN_VER=15..0, MAX_VER=31..16. Body = 12 | (12 << 16)
    send_mgmt(p, mtp_base, 2, 12 | (12 << 16))
    time.sleep(1.0)
    log("snapshot F: after HelloAck")
    snap_f = snapshot(p, mtp_base)
    diff(snap_e, snap_f, "E -> F (HelloAck)")
    o0, o1 = read_outbox(p, mtp_base)
    log(f"  F outbox: msg0={o0:#x} msg1={o1:#x}")

    # Final diff: A (initial) -> F (after everything)
    diff(snap_a, snap_f, "TOTAL A -> F")

    # DockChannel final check
    log(f"DockChannel rx_count at end: {dc.rx_count}")
    if dc.rx_count:
        data = dc.read(dc.rx_count)
        log(f"  DC data: {data.hex()}")

    os._exit(0)


if __name__ == "__main__":
    main()
