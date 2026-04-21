#!/usr/bin/env python3
"""Stage MTP ASC firmware into physical memory from J604_MtpFirmware.bin.

M4's MTP ASC (running at 0x394600000, +0x88000 wrap) has three
firmware regions declared by the /arm-io/mtp ADT node's
segment-ranges attribute:

  __TEXT   phys=0x394c00000  iova=0x1000000   size=0x5f000
  __DATA   phys=0x394c5f000  iova=0x105f000   size=0x6c000
  __OS_LOG phys=0x10005640000 iova=0x10cb000  size=0x3000

**Updated 2026-04-21 16:xx**: test_mtp_data_write.py confirmed
iBoot stages __TEXT (`0x394c00000`) into write-protected SRAM —
reads match Mach-O byte-for-byte, writes fault. __DATA and __OS_LOG
are NOT staged (live reads zero) but ARE writable. So the loader
only needs to stage __DATA + __OS_LOG; it must NOT try to rewrite
__TEXT (that's what wedged the 15:15 session).

This script parses the Mach-O inside J604_MtpFirmware.bin's "A5PH"
rkosftab section. For __TEXT it VERIFIES the first 16 bytes match
what iBoot staged (sanity). For __DATA and __OS_LOG it copies the
Mach-O content into the matching phys region via the m1n1 proxy's
bulk writemem. After staging, --boot sets CPU_CONTROL.RUN=1 and
watches OUTBOX for mailbox Hello.

Usage:
  # m1n1 already chainloaded (any version is fine — writemem works)
  /usr/bin/python3 scripts/hv/stage_mtp_firmware.py [--boot]

Flags:
  --boot      After staging, write CPU_CONTROL.RUN=1 and poll for
              mailbox Hello (10s timeout).
  --dry-run   Parse + print layout, don't touch memory.
  --force-text  Try to overwrite __TEXT anyway (wedges m1n1; debug).
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

MH_MAGIC_64 = 0xfeedfacf
LC_SEGMENT_64 = 0x19


def parse_rkosftab(data):
    """Find the 'A5PH' Mach-O payload inside an rkosftab blob."""
    magic_off = data.find(b"rkosftab")
    if magic_off < 0:
        raise ValueError("no rkosftab magic")
    cursor = magic_off + 16  # skip magic + count/version
    while cursor + 16 <= len(data):
        tag = data[cursor:cursor + 4]
        if tag == b"\x00\x00\x00\x00":
            break
        if not all(32 <= b < 127 for b in tag):
            break
        off, size = struct.unpack("<II", data[cursor + 4:cursor + 12])
        if tag == b"A5PH":
            return data[off:off + size]
        cursor += 16
    raise ValueError("no A5PH section in rkosftab")


def parse_macho_segments(macho):
    """Walk a 64-bit Mach-O's LC_SEGMENT_64 commands. Returns list of
    (segname, vmaddr, vmsize, fileoff, filesize) tuples."""
    magic = struct.unpack("<I", macho[:4])[0]
    if magic != MH_MAGIC_64:
        raise ValueError(f"not a 64-bit Mach-O (magic {magic:#x})")
    cputype, cpusubtype, filetype, ncmds, sizeofcmds, flags = \
        struct.unpack("<IIIIII", macho[4:28])
    cursor = 32  # header = 32 bytes for MH_MAGIC_64
    segments = []
    for _ in range(ncmds):
        cmd, cmdsize = struct.unpack("<II", macho[cursor:cursor + 8])
        if cmd == LC_SEGMENT_64:
            segname = macho[cursor + 8:cursor + 8 + 16].rstrip(b"\x00") \
                .decode("ascii", errors="replace")
            vmaddr, vmsize, fileoff, filesize = struct.unpack(
                "<QQQQ", macho[cursor + 24:cursor + 56])
            segments.append((segname, vmaddr, vmsize, fileoff, filesize))
        cursor += cmdsize
    return segments


def parse_adt_segments(raw):
    """Decode /arm-io/mtp segment-ranges: 32 bytes per record,
    phys(u64) + iova(u64) + remap(u64) + size(u32) + 4 pad."""
    segs = []
    for i in range(len(raw) // 32):
        seg = raw[i * 32:(i + 1) * 32]
        phys, iova, remap, size = struct.unpack("<QQQI4x", seg)
        segs.append({"phys": phys, "iova": iova, "size": size})
    return segs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--boot", action="store_true",
                    help="After staging, write CPU_CONTROL.RUN=1 and poll for mailbox")
    ap.add_argument("--dry-run", action="store_true",
                    help="Don't touch memory")
    ap.add_argument("--force-text", action="store_true",
                    help="Force __TEXT overwrite (expected to wedge m1n1 — debug only)")
    args = ap.parse_args()

    if not MTP_BLOB.exists():
        sys.stderr.write(f"missing {MTP_BLOB} — run scripts/fw/extract_im4p.py first\n")
        return 1

    blob = MTP_BLOB.read_bytes()
    macho = parse_rkosftab(blob)
    print(f"A5PH Mach-O: {len(macho)} bytes")
    macho_segs = parse_macho_segments(macho)
    print(f"Mach-O segments ({len(macho_segs)}):")
    for name, vm, vmsz, fo, fs in macho_segs:
        print(f"  {name:>12}  vm={vm:#014x} vmsize={vmsz:#x}  "
              f"fileoff={fo:#x} filesize={fs:#x}")

    if args.dry_run:
        # Parse-only mode uses a shipped ADT snapshot so it works
        # without /dev/ttyACM1 / serial access.
        print("\n(--dry-run: ADT values from journal / live-host probe)")
        print("\nADT target regions (3):")
        adt_by_name = {
            "__TEXT":   {"phys": 0x394c00000, "iova": 0x1000000, "size": 0x5f000},
            "__DATA":   {"phys": 0x394c5f000, "iova": 0x105f000, "size": 0x6c000},
            "__OS_LOG": {"phys": 0x10005640000, "iova": 0x10cb000, "size": 0x3000},
        }
        for nm, s in adt_by_name.items():
            print(f"  {nm:>12}  phys={s['phys']:#014x}  "
                  f"iova={s['iova']:#014x}  size={s['size']:#x}")

        macho_by_name = {seg[0]: seg for seg in macho_segs}
        print("\nStaging plan:")
        for nm in adt_by_name:
            if nm in macho_by_name:
                seg = macho_by_name[nm]
                target = adt_by_name[nm]
                fit = "FITS" if seg[4] <= target["size"] else "OVERFLOW"
                print(f"  {nm}: {seg[4]} bytes from Mach-O fileoff "
                      f"{seg[3]:#x} -> phys {target['phys']:#x}  [{fit}]")
            else:
                print(f"  {nm}: no matching Mach-O segment")
        print("\n(dry-run complete — not opening proxy)")
        return 0

    os.environ.setdefault("M1N1DEVICE", "/dev/ttyACM1")
    iface = UartInterface()
    p = M1N1Proxy(iface, debug=False)
    bootstrap_port(iface, p)
    u = ProxyUtils(p, heap_size=128 * 1024 * 1024)

    mtp = u.adt["/arm-io/mtp"]
    sr = getattr(mtp, "segment-ranges")
    names_raw = getattr(mtp, "segment-names", b"")
    if isinstance(names_raw, bytes):
        names = names_raw.decode("ascii", errors="replace").strip("\x00").split(";")
    else:
        names = names_raw.split(";")
    adt_segs = parse_adt_segments(sr)
    assert len(adt_segs) == len(names)
    print(f"\nADT target regions ({len(adt_segs)}):")
    adt_by_name = {}
    for nm, s in zip(names, adt_segs):
        print(f"  {nm:>12}  phys={s['phys']:#014x}  iova={s['iova']:#014x}  "
              f"size={s['size']:#x}")
        adt_by_name[nm] = s

    mtp_base = mtp.get_reg(0)[0]
    cc = p.read32(mtp_base + 0x44)
    cs = p.read32(mtp_base + 0x48)
    print(f"\nMTP ASC pre-stage: CPU_CONTROL={cc:#010x} CPU_STATUS={cs:#010x}")

    macho_by_name = {}
    for seg in macho_segs:
        name = seg[0]
        if name in adt_by_name:
            macho_by_name[name] = seg

    # __TEXT is staged by iBoot into write-protected SRAM. We cannot
    # write it (confirmed: wedges m1n1). Instead, verify iBoot's copy
    # matches the Mach-O first 16 bytes. Skip during staging.
    print("\nStaging plan:")
    for nm in names:
        if nm not in macho_by_name:
            print(f"  {nm}: no matching Mach-O segment — leaving as-is")
            continue
        seg = macho_by_name[nm]
        target = adt_by_name[nm]
        fit = "FITS" if seg[4] <= target["size"] else "OVERFLOW"
        if nm == "__TEXT" and not args.force_text:
            print(f"  {nm}: VERIFY-ONLY (iBoot owns write-protected SRAM)  "
                  f"[{fit}]")
        else:
            print(f"  {nm}: copy {seg[4]} bytes from Mach-O fileoff {seg[3]:#x} "
                  f"-> phys {target['phys']:#x}  [{fit}]")

    # Pre-check __TEXT: does iBoot's staged copy match what we'd write?
    # If not, something's wrong (wrong firmware variant / incomplete
    # iBoot handoff / unsupported state) and we should abort before
    # kicking the CPU.
    if "__TEXT" in macho_by_name and "__TEXT" in adt_by_name:
        text_seg = macho_by_name["__TEXT"]
        text_tgt = adt_by_name["__TEXT"]
        text_fo = text_seg[3]
        # Compare 64 bytes at offset 0 AND at offset 0x100 (exception
        # vector prologue — more distinctive than the reset stub).
        for probe_off in (0x0, 0x100, 0x200):
            expected = macho[text_fo + probe_off:text_fo + probe_off + 16]
            actual = iface.readmem(text_tgt["phys"] + probe_off, 16)
            status = "MATCH" if actual == expected else "MISMATCH"
            print(f"  __TEXT[+{probe_off:#x}] iBoot={actual.hex()} "
                  f"macho={expected.hex()} [{status}]")
            if actual != expected and not args.force_text:
                print(f"  ABORT: iBoot didn't stage the expected __TEXT. "
                      f"Re-run with --force-text to try overwriting anyway.")
                return 1

    print("\nStaging...")
    for nm in names:
        if nm not in macho_by_name:
            continue
        seg = macho_by_name[nm]
        target = adt_by_name[nm]
        fo, fs = seg[3], seg[4]
        if fs == 0:
            print(f"  {nm}: filesize=0, skip")
            continue
        if fs > target["size"]:
            print(f"  {nm}: {fs} > {target['size']}, ABORT")
            return 1
        if nm == "__TEXT" and not args.force_text:
            print(f"  {nm}: skip (iBoot owns — verified above)")
            continue
        payload = macho[fo:fo + fs]
        # Direct bulk writemem — test_mtp_data_write.py confirmed
        # __DATA and __OS_LOG accept host writes. writemem is simpler
        # and more predictable than compressed_writemem+gzdec (which
        # hangs the proxy when it tries to write __TEXT byte-wise).
        iface.writemem(target["phys"], payload)
        print(f"  {nm}: {fs} bytes -> {target['phys']:#x} OK")

    # Verify first 16 bytes of each region matches
    print("\nPost-stage verification:")
    for nm in names:
        if nm not in macho_by_name:
            continue
        seg = macho_by_name[nm]
        target = adt_by_name[nm]
        readback = iface.readmem(target["phys"], 16)
        expected = macho[seg[3]:seg[3] + 16]
        ok = "OK" if readback == expected else "MISMATCH"
        tag = " (iBoot)" if (nm == "__TEXT" and not args.force_text) else ""
        print(f"  {nm}{tag} @ {target['phys']:#x}: {readback.hex()} ({ok})")

    cc = p.read32(mtp_base + 0x44)
    cs = p.read32(mtp_base + 0x48)
    print(f"\nMTP ASC post-stage: CPU_CONTROL={cc:#010x} CPU_STATUS={cs:#010x}")

    if not args.boot:
        print("\n(--boot not specified — not starting ASC)")
        return 0

    print("\nSetting CPU_CONTROL.RUN=1 (bit 4)...")
    p.write32(mtp_base + 0x44, cc | (1 << 4))
    time.sleep(0.2)
    cc = p.read32(mtp_base + 0x44)
    cs = p.read32(mtp_base + 0x48)
    print(f"  immediately after: CPU_CONTROL={cc:#010x} CPU_STATUS={cs:#010x}")

    # Poll for 10 s looking for mailbox activity.
    print("\nPolling OUTBOX_CTRL for Hello (10s)...")
    for i in range(100):
        time.sleep(0.1)
        ctrl = p.read32(mtp_base + 0x8114)
        cs = p.read32(mtp_base + 0x48)
        empty = bool(ctrl & (1 << 17))
        if not empty:
            msg0 = p.read64(mtp_base + 0x8830)
            msg1 = p.read64(mtp_base + 0x8838)
            print(f"  t={i*100}ms: mailbox NON-EMPTY  msg0={msg0:#x}  msg1={msg1:#x}")
            return 0
        if i % 10 == 0:
            print(f"  t={i*100}ms: OUTBOX_CTRL={ctrl:#010x} (empty) "
                  f"CPU_STATUS={cs:#010x}")

    print("\nTimed out waiting for mailbox Hello.")
    return 2


if __name__ == "__main__":
    sys.exit(main())
