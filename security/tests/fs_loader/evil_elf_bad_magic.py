#!/usr/bin/env python3
"""
evil_elf_bad_magic.py — ELFs that should be rejected by header validation
but currently aren't (ATTACK-FL-009, ATTACK-FL-010).

Artifacts:
  - evil_elfclass32.elf     — claims ELFCLASS32 (32-bit) but loader assumes 64-bit
  - evil_et_core.elf        — e_type = ET_CORE (4), not ET_EXEC/ET_DYN
  - evil_wrong_machine.elf  — e_machine = EM_X86_64 (62)
  - evil_big_endian.elf     — ELFDATA2MSB (2), loader is LE-only

Usage:  python3 evil_elf_bad_magic.py [outdir]
"""
import struct
import sys
import os

def write(path, data):
    with open(path, 'wb') as f:
        f.write(data)
    print(f"wrote {path} ({len(data)} bytes)")

def base_elf64_le(ei_class=2, ei_data=1, e_type=2, e_machine=183):
    ident = bytes([0x7f, ord('E'), ord('L'), ord('F'),
                   ei_class, ei_data, 1, 0] + [0]*8)
    return (ident
            + struct.pack('<HHIQQQIHHHHHH',
                          e_type, e_machine, 1,
                          0x400000, 64, 0, 0,
                          64, 56, 1, 0, 0, 0))

def dummy_phdr():
    return struct.pack('<IIQQQQQQ', 1, 5, 0, 0x400000, 0x400000, 0, 0, 0x1000)

def main():
    outdir = sys.argv[1] if len(sys.argv) > 1 else '.'
    os.makedirs(outdir, exist_ok=True)

    write(os.path.join(outdir, 'evil_elfclass32.elf'),
          base_elf64_le(ei_class=1) + dummy_phdr())
    write(os.path.join(outdir, 'evil_et_core.elf'),
          base_elf64_le(e_type=4) + dummy_phdr())
    write(os.path.join(outdir, 'evil_wrong_machine.elf'),
          base_elf64_le(e_machine=62) + dummy_phdr())
    # Big-endian variant: byte-swap the header fields. The loader uses
    # from_le_bytes everywhere so this will read garbage.
    ident = bytes([0x7f, ord('E'), ord('L'), ord('F'),
                   2, 2, 1, 0] + [0]*8)
    be_hdr = (ident + struct.pack('>HHIQQQIHHHHHH',
                                   2, 183, 1,
                                   0x400000, 64, 0, 0,
                                   64, 56, 1, 0, 0, 0))
    write(os.path.join(outdir, 'evil_big_endian.elf'), be_hdr + dummy_phdr())

if __name__ == '__main__':
    main()
