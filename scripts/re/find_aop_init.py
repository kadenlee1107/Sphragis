#!/usr/bin/env python3
"""Analyze a macOS kext (or the kernelcache) to find AOP init sequence.

Strategy:
  1. Load the Mach-O via LIEF.
  2. Find C++ class names (from __TEXT.__cstring) related to AOP/ASC/ISP.
  3. Find functions that reference ADT compatibles 'iop,ascwrap-v6',
     'apple,j604', 'rtbuddy-v2', 'dart,t8110', 'aic,3'.
  4. For those functions, look for MMIO read/write patterns (mov/ldr/str
     with likely MMIO-range addresses).
  5. Dump findings — function addresses and disassembly.
"""
import sys
import pathlib
import argparse
try:
    import lief
except ImportError:
    print("pip3 install --user --break-system-packages lief", file=sys.stderr)
    sys.exit(1)
try:
    import capstone
except ImportError:
    print("pip3 install --user --break-system-packages capstone", file=sys.stderr)
    sys.exit(1)

# Strings we expect to find in the AOP driver
INTEREST_STRINGS = [
    b"iop,ascwrap-v6",
    b"iop-nub,rtbuddy-v2",
    b"dart,t8110",
    b"aic,3",
    b"apple,j604",
    b"iop,ascwrap",
    b"ascwrap",
    b"mbox",
    b"mailbox",
    b"IOP",
    b"AOPMailbox",
    b"AOPASCWrap",
    b"RtBuddy",
    b"rtbuddy",
    b"asc-wrap",
    b"IOPMgmtEP",
    b"IOPMgmt",
    b"SetIOPPower",
    b"HelloAck",
    b"mac16gaop",
]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("binary", help="path to kext/binary/kernelcache")
    ap.add_argument("--maxstrlen", type=int, default=128)
    ap.add_argument("--search", help="only print findings matching this substring")
    args = ap.parse_args()

    print(f"[*] Loading {args.binary}...", flush=True)
    path = pathlib.Path(args.binary).resolve()
    binary = lief.parse(str(path))
    if binary is None:
        print(f"[!] LIEF couldn't parse {path}", file=sys.stderr)
        return 1

    # Handle FatBinary (multi-arch)
    if isinstance(binary, lief.MachO.FatBinary):
        print(f"[*] FAT binary with {binary.size} archs; selecting arm64")
        arm = None
        for b in binary:
            if b.header.cpu_type == lief.MachO.Header.CPU_TYPE.ARM64:
                arm = b
                break
        if not arm:
            print("[!] no arm64 slice", file=sys.stderr); return 1
        binary = arm

    print(f"[*] arch={binary.header.cpu_type}  type={binary.header.file_type}")
    print(f"[*] entrypoint: {binary.entrypoint:#x}")
    print()

    # Search for interesting strings
    text = None
    cstring = None
    for sec in binary.sections:
        if sec.name == "__text":
            text = sec
        elif sec.name == "__cstring":
            cstring = sec
    if cstring is None:
        # Try __const or __string
        for sec in binary.sections:
            if "string" in sec.name.lower() or "const" in sec.name.lower():
                print(f"[*] candidate string section: {sec.name} @ {sec.virtual_address:#x}")

    found_strings = {}  # offset_va → bytes
    for sec in binary.sections:
        if sec.size == 0: continue
        data = bytes(sec.content)
        for needle in INTEREST_STRINGS:
            off = 0
            while True:
                idx = data.find(needle, off)
                if idx < 0: break
                va = sec.virtual_address + idx
                # Extract null-terminated string at that offset
                end = idx
                while end < len(data) and end - idx < args.maxstrlen and data[end] != 0:
                    end += 1
                s = data[idx:end]
                found_strings[va] = s
                off = idx + len(needle)
                print(f"[FOUND] {sec.name} +{idx:#x} (va {va:#x}) : {s!r}")

    print()
    if not found_strings:
        print("[*] no AOP/ASC-related strings found. Maybe this is not the right binary.")
        return 0

    # For each found string, look for code that references its VA
    # (ARM64 adrp/add pattern to load string addresses)
    print(f"[*] Found {len(found_strings)} interest strings. Searching for code xrefs...")

    # Build disassembler
    md = capstone.Cs(capstone.CS_ARCH_ARM64, capstone.CS_MODE_ARM)
    md.detail = True

    if text is None:
        print("[!] no __text section — cannot disassemble")
        return 0

    text_data = bytes(text.content)
    text_va = text.virtual_address
    print(f"[*] __text: va={text_va:#x} size={len(text_data)}  disassembling...", flush=True)

    # Walk text, find adrp+add pairs that target string VAs
    instructions = list(md.disasm(text_data, text_va))
    print(f"[*] {len(instructions)} instructions")

    # Build fast index by mnemonic
    adrp_matches = []
    for i, ins in enumerate(instructions):
        if ins.mnemonic != "adrp":
            continue
        # Parse operand: adrp x0, #page
        try:
            ops = ins.op_str.split(", ")
            reg = ops[0]
            page = int(ops[1].replace("#", ""), 16 if "0x" in ops[1] else 10)
        except:
            continue
        # Look for immediately following add that resolves the address
        for j in range(i+1, min(i+8, len(instructions))):
            if instructions[j].mnemonic == "add":
                # add xN, xN, #imm
                addops = instructions[j].op_str.split(", ")
                if len(addops) < 3 or addops[0] != reg:
                    continue
                try:
                    imm = int(addops[-1].replace("#", ""), 16 if "0x" in addops[-1] else 10)
                except:
                    continue
                addr = page + imm
                if addr in found_strings:
                    adrp_matches.append((ins.address, addr, found_strings[addr]))
                break

    if adrp_matches:
        print(f"\n[*] Found {len(adrp_matches)} code references to interest strings:")
        for pc, target_va, s in adrp_matches:
            print(f"  code {pc:#x} -> string {target_va:#x} : {s!r}")
    else:
        print("[*] No adrp+add xrefs to found strings.")

    # Dump symbols if available
    print("\n[*] Symbols matching AOP/ASC/mbox/rtbuddy/iop:")
    for sym in binary.symbols:
        nm = str(sym.name)
        if any(s in nm.lower() for s in ("aop", "asc", "mbox", "mailbox", "rtbuddy", "iop")):
            if args.search and args.search.lower() not in nm.lower():
                continue
            print(f"  {sym.value:#x}  {nm}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
