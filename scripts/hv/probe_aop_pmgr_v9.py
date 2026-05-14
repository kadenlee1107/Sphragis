#!/usr/bin/env python3
"""v9: Patch FW entry to disable PAC enforcement before boot runs.

Disasm findings (firmware/aop/aopfw-mac16gaop.RELEASE.bin):
  - 7544 PAC instructions in __TEXT (pacibsp, retab, blraa, etc.)
  - FW sleep/resume path at 0x1002480 LOADS PAC keys from a per-CPU
    context buffer, but FIRST-BOOT prologue (0x1000244 → 0x10002b4)
    NEVER sets PAC keys. FW expects iBoot to pre-load the keys in
    APIAKey/APIBKey before RUN=1.
  - Our proxy can't set AOP's EL1 key regs (wrong CPU, wrong EL).
  - HV-trace v8 hit the same wall from the AP side (APIA key).

Patch approach: overwrite first 8 insns of __TEXT (at 0x1000000)
with code that clears SCTLR_EL1.EnIA/EnIB/EnDA/EnDB before running
the original entry at 0x1000244. This makes PAC instructions
pass-through (no auth). TBI=1 on Apple Silicon means PAC bits in
pointer top-byte get ignored by TLB during address translation.

The original 0x1000000 slot is just `b #0x1000244` + 31 UDFs
(padding for Current EL SP0 SYNC vector). We overwrite 8 slots
(32 bytes) with our disable + branch. Remaining UDFs untouched.

Risk: if FW intentionally uses PAC-GA for explicit signature
comparison (not just CFI), clearing EnIA/EnIB won't matter — FW
reads PAC reg values directly via pacga instruction. But the
first-boot path doesn't go through such a check (verified by
disasm).
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
from m1n1 import asm

AOP_BLOB    = ROOT / "firmware/aop/aopfw-mac16gaop.RELEASE.bin"
M1N1_MACHO  = ROOT / "external/m1n1/build/m1n1.macho"
AOP = 0x390600000

CPU_CONTROL  = 0x0044
CPU_STATUS   = 0x0048
FIQ_NMI_CFG  = 0x1004
FIQ_NMI_ARM  = 0x1014
INBOX_CTRL   = 0x8110
OUTBOX_CTRL  = 0x8114
INBOX0       = 0x8800
INBOX1       = 0x8808
OUTBOX0      = 0x8830
OUTBOX1      = 0x8838
EMPTY_BIT    = 1 << 17


def log(m): print(f"[v9] {m}", flush=True)


def snap(p, tag):
    cc = p.read32(AOP + CPU_CONTROL)
    cs = p.read32(AOP + CPU_STATUS)
    ic = p.read32(AOP + INBOX_CTRL)
    oc = p.read32(AOP + OUTBOX_CTRL)
    log(f"  [{tag}] CC={cc:#x} CS={cs:#x} IB={ic:#x} OB={oc:#x}")


def ring(p):
    p.write32(AOP + FIQ_NMI_CFG, 0x10)
    p.write32(AOP + FIQ_NMI_ARM, 0x1)


def send(p, msg0, msg1):
    p.write32(AOP + INBOX0 + 0, msg0 & 0xffffffff)
    p.write32(AOP + INBOX0 + 4, (msg0 >> 32) & 0xffffffff)
    p.write32(AOP + INBOX1 + 0, msg1 & 0xffffffff)
    p.write32(AOP + INBOX1 + 4, (msg1 >> 32) & 0xffffffff)


def chainload(iface, p, u):
    from m1n1.macho import MachO
    from m1n1.tgtypes import BootArgs_r1, BootArgs_r2, BootArgs_r3
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


def poll(p, secs, tag="AOP"):
    msgs = []
    deadline = time.time() + secs
    last = None
    while time.time() < deadline:
        try:
            oc = p.read32(AOP + OUTBOX_CTRL)
            cs = p.read32(AOP + CPU_STATUS)
            ic = p.read32(AOP + INBOX_CTRL)
        except Exception as e:
            log(f"  {tag} proxy-err: {e!r}")
            return msgs, True
        state = (oc, cs, ic)
        if state != last:
            t = secs - (deadline - time.time())
            log(f"    t={t:5.2f}s CS={cs:#x} OB={oc:#x} IB={ic:#x}")
            last = state
        if not (oc & EMPTY_BIT):
            m0 = p.read32(AOP + OUTBOX0) | (p.read32(AOP + OUTBOX0 + 4) << 32)
            m1 = p.read32(AOP + OUTBOX1) | (p.read32(AOP + OUTBOX1 + 4) << 32)
            log(f"    *** {tag} OUT m0={m0:#x} m1={m1:#x} "
                f"TYPE={(m0>>52)&0xff:#x} ***")
            msgs.append((m0, m1))
            time.sleep(0.005)
        time.sleep(0.02)
    return msgs, False


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

    # NOTE: dapf_init is DEFERRED until AFTER our patch — calling it
    # early locks AOP __TEXT as read-only (SError on write).

    # Stage AOP firmware (full — same as v7/v8)
    aop_node = u.adt["/arm-io/aop"]
    sr = getattr(aop_node, "segment-ranges", None)
    names_raw = getattr(aop_node, "segment-names", b"")
    if isinstance(names_raw, bytes):
        names_raw = names_raw.decode("ascii", errors="replace")
    names = names_raw.strip("\x00").split(";")
    adt_by = dict(zip(names, parse_adt_segments(sr)))
    macho = AOP_BLOB.read_bytes()
    mc_by = {seg[0]: seg for seg in macho_segs_fn(macho)}
    text_phys = adt_by['__TEXT']['phys']
    log(f"AOP __TEXT at phys {text_phys:#x}")
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
        if ok: skip.add(nm)
    for nm in names:
        if nm in skip or nm not in mc_by or nm not in adt_by: continue
        m = mc_by[nm]; a = adt_by[nm]
        if m[4] == 0 or m[4] > a["size"]: continue
        if m[4] >= 64 * 1024:
            u.compressed_writemem(a["phys"], macho[m[3]:m[3]+m[4]], True)
        else:
            iface.writemem(a["phys"], macho[m[3]:m[3]+m[4]])
    log("FW staged (iBoot __TEXT preserved)")

    # === PAC-DISABLE PATCH ===
    # Overwrite first 8 insns of __TEXT (0x1000000..0x100001f).
    # Originally: `b #0x1000244` then 31 UDFs (dead vector padding).
    # Replace with: clear SCTLR.EnIA/EnIB/EnDA/EnDB + branch to 0x1000244.
    log("\n=== PATCH: disable PAC enforcement before FW boot ===")
    # The asm branch offset math is tricky; easier to compute the bytes manually.
    # Each insn is 4 bytes. 8 insns at 0x1000000..0x100001f. The `b` is at 0x100001c.
    # Target = 0x1000244. Offset in bytes = 0x228 = 552. As imm26 = 0x228 >> 2 = 0x8A.
    # b encoding: 0x14000000 | (imm26 & 0x3FFFFFF)
    import struct
    patch_insns = [
        0xd5381001,   # mrs x1, sctlr_el1
        0x927b0021,   # bic x1, x1, #0x80000000  (= and x1, x1, #0x7fff_ffff_ffff_ffff)
        0x927a0021,   # bic x1, x1, #0x40000000  (= and x1, x1, #0xbfff_ffff_ffff_ffff)
        0x9279c021,   # bic x1, x1, #0x08000000  (= and x1, x1, #0xf7ff_ffff_ffff_ffff)
        0x92734021,   # bic x1, x1, #0x00002000  (= and x1, x1, #0xffff_ffff_ffff_dfff)
        0xd5181001,   # msr sctlr_el1, x1
        0xd5033fdf,   # isb
        0x1400008a,   # b #0x228 (from 0x100001c → 0x1000244)
    ]
    # Verify with m1n1's asm module
    try:
        asm_test = asm.ARMAsm("""
        mrs  x1, sctlr_el1
        and  x1, x1, #0x7fffffffffffffff
        and  x1, x1, #0xbfffffffffffffff
        and  x1, x1, #0xf7ffffffffffffff
        and  x1, x1, #0xffffffffffffdfff
        msr  sctlr_el1, x1
        isb
        """, 0x1000000)
        log(f"  asm check: {len(asm_test.data)} bytes for 7 insns")
        # Prepend the mrs + 4 ands + msr + isb, then append the b
        patched_bytes = asm_test.data + struct.pack("<I", 0x1400008a)
    except Exception as e:
        log(f"  asm err: {e!r}, using hand-encoded")
        patched_bytes = b''.join(struct.pack("<I", x) for x in patch_insns)
    log(f"  patch size: {len(patched_bytes)} bytes")

    # Disasm the patch bytes to verify
    try:
        from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM
        md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
        log("  patch disasm:")
        for i in md.disasm(patched_bytes, 0x1000000):
            log(f"    {i.address:#x}: {i.mnemonic:<10s} {i.op_str}")
    except ImportError:
        pass

    # Verify original __TEXT at phys shows expected entry byte pattern
    orig_bytes = iface.readmem(text_phys, len(patched_bytes))
    log(f"  orig __TEXT[0..{len(patched_bytes):#x}]: {orig_bytes.hex()}")

    # Apply patch
    iface.writemem(text_phys, patched_bytes)
    back = iface.readmem(text_phys, len(patched_bytes))
    log(f"  post-patch: {back.hex()}")
    if back == patched_bytes:
        log("  *** patch applied successfully ***")
    else:
        log("  patch MISMATCH after write!")

    # NOW dapf_init (AFTER patch is in place; it will lock things)
    log("\ndapf_init (deferred until after patch)...")
    try:
        p.dapf_init("/arm-io/dart-aop")
        log("  dapf OK")
    except Exception as e:
        log(f"  dapf: {e!r}")

    # update_bootargs via AOPBase
    try:
        from m1n1.fw.aop.base import AOPBase
        AOPBase(u).update_bootargs({'p0CE': 0x20000, 'laCn': 0,
                                    'tPOA': 1, 'gila': 0x80})
    except Exception as e:
        log(f"bootargs: {e!r}")

    p.write32(AOP + OUTBOX_CTRL, 0x20001)
    snap(p, "pre-RUN")

    # RUN
    log("\nCC.RUN=1")
    p.write32(AOP + CPU_CONTROL, 0x10)
    time.sleep(0.2)
    snap(p, "post-RUN")

    # Pre-doorbell poll — maybe FW self-Hellos now with PAC off
    log("\npre-doorbell poll 2 s...")
    msgs, crashed = poll(p, 2.0, "pre-door")
    if crashed:
        log("Mac crashed post-RUN — patch may have caused PAC fault")
        os._exit(3)
    if msgs:
        log(f"  *** FW self-Helloed! {len(msgs)} msgs ***")

    # Send SetIOPPower + doorbell (reproduce v4 with PAC disabled)
    log("\nsend SetIOPPower + ring...")
    msg0 = (6 << 52) | 0x220
    send(p, msg0, 0)
    snap(p, "post-send")
    ring(p)
    snap(p, "post-ring")

    log("\npoll OUTBOX 10 s...")
    msgs2, crashed = poll(p, 10.0, "post-ring")
    all_msgs = msgs + msgs2

    snap(p, "final")
    log(f"\ntotal msgs: {len(all_msgs)}")
    for m0, m1 in all_msgs:
        log(f"  m0={m0:#x} m1={m1:#x} TYPE={(m0>>52)&0xff:#x}")
    os._exit(0 if all_msgs else 2)


if __name__ == "__main__":
    main()
