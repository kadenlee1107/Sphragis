#!/usr/bin/env python3
"""
Create an ARM64 Linux binary that calls uname() and prints the result.
Tests: write, uname, exit syscalls.
"""
import struct

code = bytearray()

# Allocate 325 bytes on stack for utsname struct (5 * 65 = 325)
# sub sp, sp, #336 (aligned to 16)
code += struct.pack('<I', 0xd1054000 | 31)  # sub sp, sp, #0x150

# uname(sp) — syscall 160
code += struct.pack('<I', 0xd2800808 | (160 << 5))  # mov x8, #160
code += struct.pack('<I', 0x910003e0)  # mov x0, sp
code += struct.pack('<I', 0xd4000001)  # svc #0

# write(1, sp, 6) — print sysname ("Sphragis\0" = 6 bytes visible)
code += struct.pack('<I', 0xd2800808)  # mov x8, #64 (write)
code += struct.pack('<I', 0xd2800020)  # mov x0, #1  (stdout)
code += struct.pack('<I', 0x910003e1)  # mov x1, sp  (buf = sysname)
code += struct.pack('<I', 0xd28000a2)  # mov x2, #5  (len = "Sphragis")
code += struct.pack('<I', 0xd4000001)  # svc #0

# write newline
# Store '\n' on stack at sp+325
code += struct.pack('<I', 0xd2800140)  # mov x0, #0xa ('\n')
code += struct.pack('<I', 0x390147e0)  # strb w0, [sp, #0x51] (random safe offset)
code += struct.pack('<I', 0xd2800808)  # mov x8, #64
code += struct.pack('<I', 0xd2800020)  # mov x0, #1
code += struct.pack('<I', 0x910143e1)  # add x1, sp, #0x50 ... actually let me simplify

# Just print " - " separator then version
# For now just print sysname and exit

# write(1, "\n", 1)
code += struct.pack('<I', 0xd2800808)  # mov x8, #64
code += struct.pack('<I', 0xd2800020)  # mov x0, #1
# Point to newline in our message
msg_offset = len(code) + 7*4 + 1  # approximate
# Actually, let's use a simpler approach - write a newline byte
# Store 0x0a at a known stack location
code += struct.pack('<I', 0xd2800149)  # mov x9, #0xa
code += struct.pack('<I', 0x390003e9)  # strb w9, [sp]  -- temporarily overwrite
code += struct.pack('<I', 0x910003e1)  # mov x1, sp
code += struct.pack('<I', 0xd2800022)  # mov x2, #1
code += struct.pack('<I', 0xd4000001)  # svc #0

# exit(0)
code += struct.pack('<I', 0xd2800ba8)  # mov x8, #93
code += struct.pack('<I', 0xd2800000)  # mov x0, #0
code += struct.pack('<I', 0xd4000001)  # svc #0

# Pad
while len(code) % 4 != 0:
    code += b'\x00'

code_size = len(code)
LOAD_ADDR = 0x400000

# Build ELF
elf = bytearray()
# ELF header
elf += b'\x7fELF'
elf += bytes([2, 1, 1, 0]) + bytes(8)
elf += struct.pack('<HHIQ', 2, 183, 1, LOAD_ADDR + 120)
elf += struct.pack('<QQI', 64, 0, 0)
elf += struct.pack('<HHHHHH', 64, 56, 1, 0, 0, 0)
# Program header
elf += struct.pack('<II', 1, 5)  # PT_LOAD, PF_R|PF_X
elf += struct.pack('<QQQQ', 0, LOAD_ADDR, LOAD_ADDR, 120 + code_size)
elf += struct.pack('<QQ', 120 + code_size, 0x1000)
assert len(elf) == 120
elf += code

with open('test_binaries/uname_test.elf', 'wb') as f:
    f.write(elf)

print(f"Created uname_test.elf ({len(elf)} bytes)")
