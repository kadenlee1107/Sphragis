#!/usr/bin/env python3
"""One-shot AOP PMGR enable + boot + MTP chain.

Hypothesis (from docs/SESSION_JOURNAL.md 2026-04-22):
  /arm-io/aop reg[3] (canonical 0x3_8882_A800? see note) is a PMGR
  device-enable MMIO slot that macOS writes directly. Our prior boot
  attempts never touched it — hence AOP never Hello'd.

Live-M4 probe on 2026-04-22:
  reg[0] base       = 0x390600000 (ASC)
  reg[3] = ioreg address 15169388544 = 0x3_882A_8000 (size 8)
  reg[3]+0 pre-boot = 0x43979756
      bit 30 set (unknown)
      bit 28 (AUTO_ENABLE) = 0
      bit 11 (PARENT_OFF)  = 0  ← good, parent is live
      bit 10 (DEV_DISABLE) = 1  ← device gated
      bit 9  (WAS_CLKGATED)= 1  ← sticky
      bit 8  (WAS_PWRGATED)= 1  ← sticky
      PS_ACTUAL [7:4] = 0x5
      PS_TARGET [3:0] = 0x6
  reg[3]+4 pre-boot = 0x00000000

boot_aop_m3.py earlier tried `cur | AUTO_ENABLE` + variants, all
preserving DEV_DISABLE and the WAS_* sticky bits → failed to reach
ACTUAL=0xf.

This script tries clean writes (no sticky bits preserved):
  attempt 1: 0x1000000f  — AUTO_ENABLE + TARGET=0xf
  attempt 2:       0xf   — TARGET=0xf only
  attempt 3: preserve bit 30 + AUTO_ENABLE + TARGET
After each, poll 500 ms for ACTUAL=0xf.

IMPORTANT: does NOT write 0xffffffff to SMC panic-scratch regs
(0x3882B8008 / 802C / 8020). Per wdt.c fix those corrupt SMC.
The patched m1n1 at chainload time zeros 0x3882BC224 already.
"""
import os, pathlib, struct, sys, time

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

# PMGR bit layout (m1n1 src/pmgr.c)
PMGR_RESET          = 1 << 31
PMGR_AUTO_ENABLE    = 1 << 28
PMGR_PARENT_OFF     = 1 << 11
PMGR_DEV_DISABLE    = 1 << 10
PMGR_WAS_CLKGATED   = 1 << 9
PMGR_WAS_PWRGATED   = 1 << 8
PMGR_PS_ACTIVE      = 0xf


def log(m): print(f"[exp] {m}", flush=True)


def decode_pmgr(v):
    return (
        f"TGT=0x{v & 0xf:x} "
        f"ACT=0x{(v >> 4) & 0xf:x} "
        f"DD={'1' if v & PMGR_DEV_DISABLE else '0'} "
        f"PO={'1' if v & PMGR_PARENT_OFF else '0'} "
        f"WC={'1' if v & PMGR_WAS_CLKGATED else '0'} "
        f"WP={'1' if v & PMGR_WAS_PWRGATED else '0'} "
        f"AE={'1' if v & PMGR_AUTO_ENABLE else '0'} "
        f"RST={'1' if v & PMGR_RESET else '0'} "
    )


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


def try_pmgr_write(p, val, tag, timeout=0.5):
    log(f"  [{tag}] write {val:#010x}")
    p.write32(AOP_PMGR + 0, val)
    deadline = time.time() + timeout
    best = None
    while time.time() < deadline:
        r = p.read32(AOP_PMGR + 0)
        if best is None or r != best:
            log(f"    read-back: {r:#010x}  {decode_pmgr(r)}")
            best = r
        if ((r >> 4) & 0xf) == 0xf:
            log(f"    *** ACTUAL=0xf reached ***")
            return True, r
        time.sleep(0.02)
    return False, best


def pmgr_enable_aop(p):
    pre = p.read32(AOP_PMGR + 0)
    log(f"AOP PMGR pre: {pre:#010x}  {decode_pmgr(pre)}")
    log(f"AOP PMGR+4 pre: {p.read32(AOP_PMGR + 4):#010x}")
    if ((pre >> 4) & 0xf) == 0xf:
        log("  already ACTIVE")
        return True

    # Attempt 1: gentlest RMW — set TARGET=0xf, preserve everything else.
    # Matches m1n1's pmgr_set_mode(): mask32(addr, PMGR_PS_TARGET, mode).
    ok, v = try_pmgr_write(p, (pre & ~0xf) | 0xf, "RMW TARGET=0xf only")
    if ok: return True
    cur = p.read32(AOP_PMGR + 0)

    # Attempt 2: RMW TARGET + clear DEV_DISABLE (bit 10 gates the device)
    ok, v = try_pmgr_write(p,
                           (cur & ~(PMGR_DEV_DISABLE | 0xf)) | 0xf,
                           "RMW clear DEV_DISABLE + TARGET=0xf")
    if ok: return True
    cur = p.read32(AOP_PMGR + 0)

    # Attempt 3: + AUTO_ENABLE, keep RMW semantics
    ok, v = try_pmgr_write(p,
                           (cur & ~(PMGR_DEV_DISABLE | 0xf)) | 0xf | PMGR_AUTO_ENABLE,
                           "RMW + AUTO_ENABLE + TARGET=0xf")
    if ok: return True
    cur = p.read32(AOP_PMGR + 0)

    # Attempt 4: aggressive — clear ALL sticky + gating bits [11:8], keep upper
    ok, v = try_pmgr_write(p,
                           (cur & 0xfffff000) | PMGR_AUTO_ENABLE | 0xf,
                           "preserve upper-20 + AUTO + TARGET=0xf")
    if ok: return True
    cur = p.read32(AOP_PMGR + 0)

    # Attempt 5: clean slate (strip everything, set just AUTO+TARGET)
    ok, v = try_pmgr_write(p, PMGR_AUTO_ENABLE | 0xf, "clean 0x1000000f")
    if ok: return True

    # Attempt 6: bare TARGET (no AUTO_ENABLE)
    ok, v = try_pmgr_write(p, 0xf, "bare 0xf")
    if ok: return True

    log(f"  PMGR enable FAILED; stuck at {v:#010x}  {decode_pmgr(v)}")
    return False


def stage_fw_segments(iface, u, node_path, blob_path, log_prefix):
    node = u.adt[node_path]
    sr = getattr(node, "segment-ranges", None)
    if sr is None:
        log(f"{log_prefix}: no segment-ranges"); return {}
    names_raw = getattr(node, "segment-names", b"")
    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_by = dict(zip(names, parse_adt_segments(sr)))
    log(f"{log_prefix} ADT segs: {list(adt_by.keys())}")
    raw = blob_path.read_bytes()
    idx = raw.find(b"\xcf\xfa\xed\xfe")
    if idx > 0:
        raw = raw[idx:]
    mc_by = {seg[0]: seg for seg in macho_segs_fn(raw)}
    skip = set()
    for nm in ("__TEXT", "__ETEXT"):
        if nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        ok = True
        for off in (0x0, 0x100, 0x200):
            if off >= m[4]: break
            exp = raw[m[3]+off:m[3]+off+16]
            got = iface.readmem(a["phys"]+off, 16)
            if got != exp: ok = False; break
        if ok:
            skip.add(nm)
            log(f"  {log_prefix} {nm}: iBoot-staged (skip)")
    for nm in names:
        if nm in skip or nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0 or m[4] > a["size"]: continue
        t = time.time()
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], raw[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], raw[m[3]:m[3]+m[4]])
        log(f"  {log_prefix} {nm}: {m[4]}B -> {a['phys']:#x} ({(time.time()-t)*1000:.0f}ms)")
    return adt_by


def main():
    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    if os.environ.get("BATOS_SKIP_BOOTSTRAP", "0") != "1":
        log("chainloading patched m1n1 (with WDT fix)...")
        chainload(iface, p, u)
        u = ProxyUtils(p, heap_size=128 * 1024 * 1024)
        log("  patched m1n1 up")

    # === STEP 1: PMGR enable ===
    log("\n=== STEP 1: AOP PMGR device enable ===")
    if not pmgr_enable_aop(p):
        log("ABORT: could not drive AOP to PS_ACTIVE")
        os._exit(1)

    # Sanity: is the ASC mailbox area writable now?
    log("\nchecking ASC mailbox writability...")
    test_off = AOP + 0x60
    p.write32(test_off, 0xdeadbeef)
    rb = p.read32(test_off)
    log(f"  [AOP+0x60] write 0xdeadbeef → read {rb:#x}  "
        f"{'WRITABLE' if rb == 0xdeadbeef else 'UNWRITABLE'}")
    p.write32(test_off, 0)

    # === STEP 2: PMGR power on dart-aop (m1n1 helper) ===
    log("\n=== STEP 2: pmgr_adt_power_enable /arm-io/dart-aop ===")
    try:
        rc = p.pmgr_adt_power_enable("/arm-io/dart-aop")
        log(f"  rc={rc}")
    except Exception as e:
        log(f"  {type(e).__name__}: {e}")

    # === STEP 3: Stage AOP firmware ===
    log("\n=== STEP 3: Stage AOP firmware ===")
    stage_fw_segments(iface, u, "/arm-io/aop", AOP_BLOB, "aop")

    # === STEP 4: DART + AOPClient + bootargs + start ===
    log("\n=== STEP 4: AOP DART + AOPClient boot ===")
    from m1n1.hw.dart import DART
    dart_node = u.adt["/arm-io/dart-aop"]
    vm_base = getattr(dart_node, "vm-base", None) or 0x8000
    log(f"  DART vm-base={vm_base:#x}")
    dart_aop = DART.from_adt(u, "/arm-io/dart-aop",
                             iova_range=(vm_base, 0x1000000000))
    dart_aop.initialize()

    from m1n1.fw.aop.client import AOPClient
    aop = AOPClient(u, "/arm-io/aop", dart_aop)
    aop.verbose = 2

    pre_cc = p.read32(AOP + 0x44)
    pre_cs = p.read32(AOP + 0x48)
    log(f"  pre-boot CC={pre_cc:#x} CS={pre_cs:#x}")
    if pre_cc & 0x10:
        log("  AOP.RUN already set — clearing for bootargs re-read")
        p.write32(AOP + 0x44, pre_cc & ~0x10)
        time.sleep(0.1)

    try:
        args = aop.read_bootargs()
        log(f"  bootargs keys: {list(args.keys())[:10]}")
    except Exception as e:
        log(f"  read_bootargs fail: {type(e).__name__}: {e}")
        try:
            aop.asc.CPU_CONTROL.set(RUN=1)
            time.sleep(0.2)
            args = aop.read_bootargs()
            log(f"  after RUN=1, bootargs: {list(args.keys())[:10]}")
        except Exception as e2:
            log(f"  still failing: {e2!r}")
            log(f"  final PMGR: {p.read32(AOP_PMGR+0):#x} {decode_pmgr(p.read32(AOP_PMGR+0))}")
            log(f"  final CC: {p.read32(AOP + 0x44):#x}")
            os._exit(1)

    log("  update_bootargs({p0CE:0x20000,laCn:0,tPOA:1,gila:0x80})")
    try:
        aop.update_bootargs({'p0CE': 0x20000, 'laCn': 0x0,
                             'tPOA': 0x1, 'gila': 0x80})
    except Exception as e:
        log(f"  update_bootargs err: {type(e).__name__}: {e}")

    # Reset OUTBOX_CTRL
    try:
        p.write32(AOP + 0x8114, 0x20001)
    except Exception: pass

    log("  aop.start() — waits up to 3s for Hello...")
    t0 = time.time()
    aop_started = False
    try:
        aop.start()
        log(f"  *** AOP HELLO in {(time.time()-t0)*1000:.0f}ms *** "
            f"iop={aop.mgmt.iop_power_state:#x} "
            f"ap={aop.mgmt.ap_power_state:#x}")
        aop_started = True
        for epno in [0x20, 0x21, 0x22, 0x24, 0x25, 0x26, 0x27, 0x28]:
            try:
                aop.start_ep(epno)
                log(f"    ep {epno:#x}: started")
            except Exception as e:
                log(f"    ep {epno:#x}: {type(e).__name__}")
    except Exception as e:
        log(f"  aop.start() FAIL: {type(e).__name__}: {e}")
        cc = p.read32(AOP + 0x44)
        cs = p.read32(AOP + 0x48)
        log(f"  CC={cc:#x} CS={cs:#x} "
            f"IB={p.read32(AOP + 0x8110):#x} "
            f"OB={p.read32(AOP + 0x8114):#x} "
            f"PMGR={p.read32(AOP_PMGR+0):#x}")

    # === STEP 5: If AOP up, boot MTP ===
    if not aop_started:
        log("\n=== AOP did not Hello → skipping MTP chain ===")
        log(f"final PMGR: {p.read32(AOP_PMGR+0):#x} "
            f"{decode_pmgr(p.read32(AOP_PMGR+0))}")
        os._exit(2)

    log("\n=== STEP 5: MTP boot (chain) ===")
    try:
        # SMC start (dependency for MTP keyboard protocol)
        from m1n1.fw.smc import SMCClient
        smc = SMCClient(u, u.adt["arm-io/smc"].get_reg(0)[0])
        smc.start()
        smc.start_ep(0x20)
        log(f"  SMC up: iop={smc.mgmt.iop_power_state:#x} "
            f"ap={smc.mgmt.ap_power_state:#x}")

        # Stage MTP firmware
        log("staging MTP firmware...")
        mtp_adt_by = stage_fw_segments(iface, u, "/arm-io/mtp",
                                        MTP_BLOB, "mtp")

        # DART-MTP
        log("  DART-MTP setup...")
        dart_mtp = DART.from_adt(u, "/arm-io/dart-mtp",
                                 iova_range=(0x8000, 0x100000))
        dart_mtp.dart.regs.TCR[1].set(BYPASS_DAPF=1, BYPASS_DART=0,
                                       TRANSLATE_ENABLE=1)
        dart_mtp.initialize()

        PAGE_SIZE = 0x4000
        def align_up(x, a): return (x + a - 1) & ~(a - 1)
        for nm, a in mtp_adt_by.items():
            phys_a = a["phys"] & ~(PAGE_SIZE - 1)
            iova_a = a["iova"] & ~(PAGE_SIZE - 1)
            size = align_up(a["size"], PAGE_SIZE)
            try:
                dart_mtp.dart.iomap_at(1, iova_a, phys_a, size)
                log(f"    mapped {nm} stream=1 iova={iova_a:#x}")
            except Exception as e:
                log(f"    {nm} iomap_at: {type(e).__name__}: {e}")

        # DockChannel
        from m1n1.hw.dockchannel import DockChannel
        irq_base  = u.adt["/arm-io/dockchannel-mtp"].get_reg(1)[0]
        fifo_base = u.adt["/arm-io/dockchannel-mtp"].get_reg(2)[0]
        dc = DockChannel(u, irq_base, fifo_base, 1)
        while dc.rx_count:
            dc.read(dc.rx_count)
        log(f"  DockChannel ready (irq={irq_base:#x} fifo={fifo_base:#x})")

        # MTP ASC
        from m1n1.fw.asc import StandardASC
        mtp = StandardASC(u, MTP, dart_mtp, stream=1)
        mtp.verbose = 1
        mtp.allow_phys = True
        log(f"  MTP pre-boot CC={p.read32(MTP + 0x44):#x} "
            f"CS={p.read32(MTP + 0x48):#x}")
        mtp.asc.CPU_CONTROL.set(RUN=1)
        mtp.mgmt.start()
        mtp.mgmt.wait_boot(15)
        log(f"  *** MTP HELLO *** iop={mtp.mgmt.iop_power_state:#x} "
            f"ap={mtp.mgmt.ap_power_state:#x}")

        # Keyboard protocol init (opportunistic)
        try:
            from m1n1.fw.mtp import MTPProtocol
            node = u.adt["/arm-io/dockchannel-mtp/mtp-transport"]
            mp = MTPProtocol(u, node, mtp, dc, smc)
            mp.wait_init("keyboard")
            log("  *** KEYBOARD INITIALIZED ***")
            for _ in range(20):
                mp.work_pending()
                mtp.work()
                time.sleep(0.1)
        except Exception as e:
            log(f"  keyboard init: {type(e).__name__}: {e}")
    except Exception as e:
        log(f"MTP chain FAIL: {type(e).__name__}: {e}")
        log(f"  MTP CC={p.read32(MTP + 0x44):#x} "
            f"CS={p.read32(MTP + 0x48):#x} "
            f"OB={p.read32(MTP + 0x8114):#x}")

    os._exit(0)


if __name__ == "__main__":
    main()
