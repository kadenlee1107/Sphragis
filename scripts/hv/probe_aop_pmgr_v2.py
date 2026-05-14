#!/usr/bin/env python3
"""AOP PMGR + DAPF + OS_LOG observation — v2 experiment.

v1 findings (logs/aop-pmgr-probe-*.log):
  - RMW TARGET=0xf at 0x3882A8000 reaches PS_ACTUAL=0xf (HW cycles thru
    sticky→partial→active within ~150 ms)
  - bootargs READABLE for first time (GKTS p0CE p0DE laCn Hlca Epan
    Hsid gila tPOA Idrb)
  - update_bootargs succeeds (DRAM writes land)
  - CC.RUN=1 set; but aop.start() times out (OB EMPTY bit stays 1)
  - PMGR drifts back to ACT~0xa with DEV_DISABLE re-set during ASC ops

v2 additions:
  - dapf_init("/arm-io/dart-aop") before ASC work (fabric protection)
  - OS_LOG region dump post-RUN (check if FW is logging)
  - Periodic PMGR pin (rewrite TARGET=0xf if ACT drifts)
  - Direct inbox poke (mgmt SetIOPPower) instead of relying on
    aop.start() wrapper
  - Longer observation window (10 s)
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

AOP_BLOB    = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
MTP_BLOB    = ROOT / "firmware/mtp/J604_MtpFirmware.bin"
M1N1_MACHO  = ROOT / "external/m1n1/build/m1n1.macho"

AOP      = 0x390600000
MTP      = 0x394600000
AOP_PMGR = 0x3882A8000

# StandardASC + mgmt layout
ASC_CC    = 0x044      # CPU_CONTROL
ASC_CS    = 0x048      # CPU_STATUS
IB_CTRL   = 0x8110
OB_CTRL   = 0x8114
IB_MSG0   = 0x8800     # inbox message data (low u32, +4 = high)
IB_MSG1   = 0x8808     # inbox message data2
OB_MSG0   = 0x8830
OB_MSG1   = 0x8838
OB_EMPTY  = 1 << 17


def log(m): print(f"[v2] {m}", flush=True)


def pmgr_snap(p, tag):
    v = p.read32(AOP_PMGR + 0)
    v4 = p.read32(AOP_PMGR + 4)
    log(f"  [{tag}] PMGR={v:#010x} TGT={v&0xf:x} ACT={(v>>4)&0xf:x} "
        f"DD={(v>>10)&1} PO={(v>>11)&1} WC={(v>>9)&1} WP={(v>>8)&1} "
        f"+4={v4:#x}")
    return v


def pmgr_pin(p):
    """If ACT drifted off 0xf, RMW TARGET=0xf. Return current value."""
    cur = p.read32(AOP_PMGR + 0)
    if ((cur >> 4) & 0xf) == 0xf:
        return cur
    new = (cur & ~0xf) | 0xf
    p.write32(AOP_PMGR + 0, new)
    time.sleep(0.05)
    return p.read32(AOP_PMGR + 0)


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

    if os.environ.get("SPHRAGIS_SKIP_BOOTSTRAP", "0") != "1":
        log("chainloading patched m1n1...")
        chainload(iface, p, u)
        u = ProxyUtils(p, heap_size=128 * 1024 * 1024)
        log("  patched m1n1 up")

    # === STEP 1: PMGR enable ===
    log("\n=== STEP 1: PMGR RMW TARGET=0xf ===")
    pmgr_snap(p, "pre-pmgr")
    cur = p.read32(AOP_PMGR + 0)
    p.write32(AOP_PMGR + 0, (cur & ~0xf) | 0xf)
    t = time.time() + 0.5
    while time.time() < t:
        v = p.read32(AOP_PMGR + 0)
        if ((v >> 4) & 0xf) == 0xf:
            break
        time.sleep(0.02)
    pmgr_snap(p, "post-pmgr")

    # === STEP 2: DAPF fabric init for dart-aop ===
    log("\n=== STEP 2: DAPF init for /arm-io/dart-aop ===")
    try:
        rc = p.dapf_init("/arm-io/dart-aop")
        log(f"  dapf_init(dart-aop): rc={rc}")
    except Exception as e:
        log(f"  dapf_init err: {type(e).__name__}: {e}")
    pmgr_pin(p)
    pmgr_snap(p, "post-dapf")

    # === STEP 3: Stage AOP firmware ===
    log("\n=== STEP 3: Stage AOP firmware ===")
    aop_node = u.adt["/arm-io/aop"]
    sr = getattr(aop_node, "segment-ranges", None)
    names_raw = getattr(aop_node, "segment-names", b"")
    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_by = dict(zip(names, parse_adt_segments(sr)))
    macho = AOP_BLOB.read_bytes()
    mc_by = {seg[0]: seg for seg in macho_segs_fn(macho)}
    os_log_seg = adt_by.get("__OS_LOG")
    os_log_phys = os_log_seg["phys"] if os_log_seg else None
    os_log_size = os_log_seg["size"] if os_log_seg else 0
    log(f"  __OS_LOG at phys={os_log_phys:#x} size={os_log_size:#x}"
        if os_log_phys else "  no __OS_LOG segment")

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
            log(f"  aop {nm}: iBoot-staged (skip)")
    for nm in names:
        if nm in skip or nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0 or m[4] > a["size"]: continue
        t0 = time.time()
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], macho[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], macho[m[3]:m[3]+m[4]])
        log(f"  aop {nm}: {m[4]}B -> {a['phys']:#x} "
            f"({(time.time()-t0)*1000:.0f}ms)")
    pmgr_snap(p, "post-stage")

    # === STEP 4: update_bootargs via AOPClient ===
    log("\n=== STEP 4: bootargs update ===")
    try:
        from m1n1.hw.dart import DART
        dart_node = u.adt["/arm-io/dart-aop"]
        vm_base = getattr(dart_node, "vm-base", None) or 0x8000
        dart_aop = DART.from_adt(u, "/arm-io/dart-aop",
                                 iova_range=(vm_base, 0x1000000000))
        dart_aop.initialize()
        pmgr_pin(p)

        from m1n1.fw.aop.client import AOPClient
        aop = AOPClient(u, "/arm-io/aop", dart_aop)
        aop.verbose = 1
        args = aop.read_bootargs()
        log(f"  bootargs keys: {list(args.keys())[:10]}")
        aop.update_bootargs({'p0CE': 0x20000, 'laCn': 0x0,
                             'tPOA': 0x1, 'gila': 0x80})
        log("  update_bootargs done")
    except Exception as e:
        log(f"  bootargs FAIL: {type(e).__name__}: {e}")
        aop = None

    pmgr_pin(p)
    pmgr_snap(p, "post-bootargs")

    # Clear OUTBOX_CTRL per m1n1 convention
    p.write32(AOP + OB_CTRL, 0x20001)

    # === STEP 5: RUN + observe ===
    log("\n=== STEP 5: CC.RUN=1 + 10 s observation ===")
    pre_cc = p.read32(AOP + ASC_CC)
    pre_cs = p.read32(AOP + ASC_CS)
    log(f"  pre: CC={pre_cc:#x} CS={pre_cs:#x}")
    # Set RUN bit (bit 4)
    p.write32(AOP + ASC_CC, pre_cc | 0x10)
    log(f"  wrote CC={pre_cc | 0x10:#x}")

    last_state = None
    deadline = time.time() + 10.0
    iters = 0
    while time.time() < deadline:
        iters += 1
        pmgr_pin(p)   # re-assert TARGET=0xf if drifted
        cc = p.read32(AOP + ASC_CC)
        cs = p.read32(AOP + ASC_CS)
        ib = p.read32(AOP + IB_CTRL)
        ob = p.read32(AOP + OB_CTRL)
        pmgr = p.read32(AOP_PMGR + 0)
        state = (cc, cs, ib, ob, pmgr)
        if state != last_state:
            t = time.time() - (deadline - 10)
            log(f"  t={t:5.2f}s CC={cc:#x} CS={cs:#x} IB={ib:#x} OB={ob:#x} "
                f"PMGR={pmgr:#x}")
            last_state = state
        # OUTBOX non-empty?
        if not (ob & OB_EMPTY):
            m0 = p.read32(AOP + OB_MSG0) | (p.read32(AOP + OB_MSG0 + 4) << 32)
            m1 = p.read32(AOP + OB_MSG1) | (p.read32(AOP + OB_MSG1 + 4) << 32)
            log(f"  *** OUTBOX MSG m0={m0:#x} m1={m1:#x} ***")
            # Ack by rewriting CTRL
            p.write32(AOP + OB_CTRL, ob)
        time.sleep(0.05)
    log(f"  observation done ({iters} iters)")

    # === STEP 6: Dump OS_LOG ===
    log("\n=== STEP 6: OS_LOG dump (first 2 KB) ===")
    if os_log_phys:
        data = iface.readmem(os_log_phys, 2048)
        # Look for ASCII
        ascii_runs = []
        run = b""
        for b in data:
            if 32 <= b < 127 or b in (9, 10):
                run += bytes([b])
            else:
                if len(run) >= 4:
                    ascii_runs.append(run)
                run = b""
        if len(run) >= 4:
            ascii_runs.append(run)
        for r in ascii_runs[:20]:
            log(f"  OS_LOG: {r[:100]!r}")
        if not ascii_runs:
            nonzero = sum(1 for b in data if b != 0)
            log(f"  OS_LOG: no printable strings, {nonzero}/{len(data)} "
                f"non-zero bytes")
            log(f"  head: {data[:64].hex()}")

    # === STEP 7: Final state ===
    log("\n=== STEP 7: Final state ===")
    pmgr_snap(p, "final")
    log(f"  CC={p.read32(AOP + ASC_CC):#x}")
    log(f"  CS={p.read32(AOP + ASC_CS):#x}")
    log(f"  IB_CTRL={p.read32(AOP + IB_CTRL):#x}")
    log(f"  OB_CTRL={p.read32(AOP + OB_CTRL):#x}")
    # Also dump a few 32-bit words around the mailbox region for context
    for off in (0x50, 0x60, 0x70, 0x80, 0x90, 0xa0):
        try:
            v = p.read32(AOP + off)
            log(f"  AOP+{off:#x} = {v:#x}")
        except Exception as e:
            log(f"  AOP+{off:#x}: {type(e).__name__}")

    os._exit(0)


if __name__ == "__main__":
    main()
