#!/usr/bin/env python3
"""
evil_elf_overflow.py — forge ELF64 files that exercise integer-overflow
bounds checks in src/caves/linux/loader.rs.

Produces three artifacts:
  - evil_pt_offset_overflow.elf   (ATTACK-FL-001): p_offset+filesz wraps past data.len()
  - evil_pt_vaddr_overflow.elf    (ATTACK-FL-002): vaddr+memsz wraps
  - evil_phnum_huge.elf           (ATTACK-FL-006): e_phnum = 0xFFFF

Usage:  python3 evil_elf_overflow.py [outdir]
Default outdir = current directory.

These files are research inputs. Do not run them on a host — they are not
valid executables; they exist only to exercise the loader's bounds checks.
"""
import struct
import sys
import os

ELFCLASS64 = 2
ELFDATA2LSB = 1
EV_CURRENT = 1
ET_EXEC = 2
EM_AARCH64 = 183
PT_LOAD = 1
PT_DYNAMIC = 2

def elf_header(e_entry, e_phoff, e_phnum, e_phentsize=56, e_type=ET_EXEC, e_machine=EM_AARCH64):
    e_ident = bytes([0x7f, ord('E'), ord('L'), ord('F'),
                     ELFCLASS64, ELFDATA2LSB, EV_CURRENT, 0,
                     0, 0, 0, 0, 0, 0, 0, 0])
    return (e_ident
            + struct.pack('<HHIQQQIHHHHHH',
                          e_type, e_machine, EV_CURRENT,
                          e_entry, e_phoff, 0,         # e_shoff=0
                          0,                           # e_flags
                          64,                          # e_ehsize
                          e_phentsize, e_phnum,
                          0, 0, 0))                    # shentsize, shnum, shstrndx

def pt_load(p_offset, p_vaddr, p_filesz, p_memsz, p_flags=5):
    # Elf64_Phdr: type, flags, offset, vaddr, paddr, filesz, memsz, align
    return struct.pack('<IIQQQQQQ', PT_LOAD, p_flags,
                       p_offset & 0xFFFFFFFFFFFFFFFF,
                       p_vaddr & 0xFFFFFFFFFFFFFFFF,
                       p_vaddr & 0xFFFFFFFFFFFFFFFF,
                       p_filesz & 0xFFFFFFFFFFFFFFFF,
                       p_memsz & 0xFFFFFFFFFFFFFFFF,
                       0x1000)

def write(path, data):
    with open(path, 'wb') as f:
        f.write(data)
    print(f"wrote {path} ({len(data)} bytes)")

def main():
    outdir = sys.argv[1] if len(sys.argv) > 1 else '.'
    os.makedirs(outdir, exist_ok=True)

    # 1) p_offset + filesz overflow (ATTACK-FL-001)
    # p_offset = 0xFFFF_FFFF_FFFF_FFF0, filesz = 0x20.
    # Sum wraps to 0x10, which passes the `<= data.len()` check on any file >= 16 bytes.
    phdr = pt_load(p_offset=0xFFFFFFFFFFFFFFF0,
                   p_vaddr=0x400000,
                   p_filesz=0x20,
                   p_memsz=0x20)
    hdr = elf_header(e_entry=0x400000, e_phoff=64, e_phnum=1)
    blob = hdr + phdr + b'\x00' * 64  # padding so the misleading sum still fits
    write(os.path.join(outdir, 'evil_pt_offset_overflow.elf'), blob)

    # 2) vaddr + memsz overflow (ATTACK-FL-002)
    # Two PT_LOADs. The second wraps max_addr.
    ph0 = pt_load(0x1000, 0x400000, 0x1000, 0x1000)
    ph1 = pt_load(0x1000, 0xFFFFFFFFFFFF0000, 0x100, 0x20000)
    hdr = elf_header(e_entry=0x400000, e_phoff=64, e_phnum=2)
    blob = hdr + ph0 + ph1 + b'\x00' * 0x2000
    write(os.path.join(outdir, 'evil_pt_vaddr_overflow.elf'), blob)

    # 3) e_phnum = 0xFFFF (ATTACK-FL-006) — forces the loader to iterate 65535 times.
    # Each phdr is 56 bytes → 3.5 MiB of "program header table".
    fake_ph = pt_load(0, 0, 0, 0) * 0xFFFF
    hdr = elf_header(e_entry=0x400000, e_phoff=64, e_phnum=0xFFFF)
    blob = hdr + fake_ph
    write(os.path.join(outdir, 'evil_phnum_huge.elf'), blob)

if __name__ == '__main__':
    main()
