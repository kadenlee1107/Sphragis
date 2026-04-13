#!/usr/bin/env python3
"""
Create a minimal ARM64 Linux ELF binary that prints "Hello from BatCave!"
Hand-encoded ARM64 instructions — no cross compiler needed.
"""

import struct

# ARM64 machine code for our hello program
code = bytearray()

# Use x1 = PC-relative address via adr
# Instructions are 4 bytes each, msg is after 9 instructions + 1 nop = 40 bytes

# _start (offset 0):
# mov x8, #64        (write syscall number)
code += struct.pack('<I', 0xd2800808)  # mov x8, #64
# mov x0, #1         (fd = stdout)
code += struct.pack('<I', 0xd2800020)  # mov x0, #1
# adr x1, . + 32     (msg is 8 instructions ahead = 32 bytes from here)
# adr encoding: immhi (bits 23:5), immlo (bits 30:29), Rd (bits 4:0)
# offset = 32, immlo = 32 & 3 = 0, immhi = 32 >> 2 = 8
code += struct.pack('<I', 0x10000101)  # adr x1, #+32 (from this instruction)
# mov x2, #20        (count)
code += struct.pack('<I', 0xd2800282)  # mov x2, #20
# svc #0
code += struct.pack('<I', 0xd4000001)  # svc #0

# mov x8, #93        (exit syscall)
code += struct.pack('<I', 0xd2800ba8)  # mov x8, #93
# mov x0, #0         (exit code)
code += struct.pack('<I', 0xd2800000)  # mov x0, #0
# svc #0
code += struct.pack('<I', 0xd4000001)  # svc #0

# msg at offset 32 from adr instruction = offset 40 from start:
# But adr is at offset 8, so msg at 8+32=40. Let's add 2 nops to reach 40.
# We have 8 instructions = 32 bytes. Need 8 more bytes to reach 40.
code += struct.pack('<I', 0xd503201f)  # nop
code += struct.pack('<I', 0xd503201f)  # nop

# msg at offset 40:
msg = b"Hello from BatCave!\n"
code += msg
# Pad to 4-byte alignment
while len(code) % 4 != 0:
    code += b'\0'

code_size = len(code)

# Build minimal ELF64 for ARM64 Linux
LOAD_ADDR = 0x400000

# ELF Header (64 bytes)
elf = bytearray()
elf += b'\x7fELF'       # e_ident[EI_MAG]
elf += bytes([2])        # EI_CLASS: 64-bit
elf += bytes([1])        # EI_DATA: little-endian
elf += bytes([1])        # EI_VERSION: current
elf += bytes([0])        # EI_OSABI: ELFOSABI_NONE
elf += bytes([0]*8)      # EI_ABIVERSION + padding

elf += struct.pack('<H', 2)      # e_type: ET_EXEC
elf += struct.pack('<H', 183)    # e_machine: EM_AARCH64
elf += struct.pack('<I', 1)      # e_version
elf += struct.pack('<Q', LOAD_ADDR + 120)  # e_entry (code starts after headers)
elf += struct.pack('<Q', 64)     # e_phoff (program headers right after ELF header)
elf += struct.pack('<Q', 0)      # e_shoff (no section headers)
elf += struct.pack('<I', 0)      # e_flags
elf += struct.pack('<H', 64)     # e_ehsize
elf += struct.pack('<H', 56)     # e_phentsize
elf += struct.pack('<H', 1)      # e_phnum (1 program header)
elf += struct.pack('<H', 0)      # e_shentsize
elf += struct.pack('<H', 0)      # e_shnum
elf += struct.pack('<H', 0)      # e_shstrndx

# Program Header (56 bytes) — single PT_LOAD segment
elf += struct.pack('<I', 1)      # p_type: PT_LOAD
elf += struct.pack('<I', 5)      # p_flags: PF_R | PF_X
elf += struct.pack('<Q', 0)      # p_offset: start of file
elf += struct.pack('<Q', LOAD_ADDR)  # p_vaddr
elf += struct.pack('<Q', LOAD_ADDR)  # p_paddr
file_size = 120 + code_size
elf += struct.pack('<Q', file_size)  # p_filesz
elf += struct.pack('<Q', file_size)  # p_memsz
elf += struct.pack('<Q', 0x1000)  # p_align

# Code starts at offset 120 (64 header + 56 phdr)
assert len(elf) == 120
elf += code

with open('test_binaries/hello_batcave.elf', 'wb') as f:
    f.write(elf)

print(f"Created hello_batcave.elf ({len(elf)} bytes)")
print(f"  Load address: 0x{LOAD_ADDR:X}")
print(f"  Entry point:  0x{LOAD_ADDR + 120:X}")
print(f"  Code size:    {code_size} bytes")
print(f"  Message:      'Hello from BatCave!'")
