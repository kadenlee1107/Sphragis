#!/usr/bin/env python3
"""
Sphragis — Mach-O Wrapper for iBoot
Creates a minimal Mach-O binary that iBoot can load.

iBoot on Apple Silicon expects a Mach-O ARM64 executable.
We create one with:
- Mach-O header (magic, cputype, etc.)
- LC_SEGMENT_64 pointing to our code
- LC_UNIXTHREAD with the entry point

iBoot loads this into memory, sets up the device tree in x0,
and jumps to our entry point. From there, we own the machine.

Usage: python3 make_macho.py <input.bin> <output.macho>
"""

import struct
import sys
import os

# Mach-O constants
MH_MAGIC_64 = 0xFEEDFACF
MH_EXECUTE = 0x2
CPU_TYPE_ARM64 = 0x0100000C  # CPU_TYPE_ARM | CPU_ARCH_ABI64
CPU_SUBTYPE_ALL = 0x0
MH_NOUNDEFS = 0x1

# Load command types
LC_SEGMENT_64 = 0x19
LC_UNIXTHREAD = 0x5

# ARM64 thread state
ARM_THREAD_STATE64 = 6
ARM_THREAD_STATE64_COUNT = 68  # 33 registers * 8 bytes + 4 bytes CPSR / 4... actually it's the count of uint32s

def make_macho(input_path, output_path, load_addr=0x810000000):
    with open(input_path, 'rb') as f:
        payload = f.read()

    payload_size = len(payload)

    # Page-align payload
    page_size = 0x4000  # 16KB pages on Apple Silicon
    aligned_size = (payload_size + page_size - 1) & ~(page_size - 1)
    payload_padded = payload + b'\0' * (aligned_size - payload_size)

    # Build load commands

    # LC_SEGMENT_64 for __TEXT (contains our code)
    segname = b'__TEXT\0\0\0\0\0\0\0\0\0\0\0'  # 16 bytes
    seg_cmd = struct.pack('<II', LC_SEGMENT_64, 72)  # cmd, cmdsize
    seg_cmd += segname  # segname[16]
    seg_cmd += struct.pack('<QQQQ', load_addr, aligned_size, 0, aligned_size)  # vmaddr, vmsize, fileoff, filesize
    seg_cmd += struct.pack('<II', 0x7, 0x5)  # maxprot (rwx), initprot (rx)
    seg_cmd += struct.pack('<II', 0, 0)  # nsects, flags

    # LC_UNIXTHREAD — sets initial register state
    # ARM64 thread state: x0-x28, fp(x29), lr(x30), sp, pc, cpsr
    # Total: 33 64-bit regs + 1 32-bit cpsr = 272 bytes
    # Count is in uint32s: 272/4 = 68
    thread_state_size = 33 * 8 + 4  # 268 bytes
    thread_state_count = 68

    thread_cmd_size = 16 + thread_state_size  # cmd(4) + cmdsize(4) + flavor(4) + count(4) + state
    thread_cmd = struct.pack('<II', LC_UNIXTHREAD, thread_cmd_size)
    thread_cmd += struct.pack('<II', ARM_THREAD_STATE64, thread_state_count)

    # x0-x28 (all zero — iBoot fills x0 with boot args)
    regs = b'\0' * (29 * 8)
    # fp (x29)
    regs += struct.pack('<Q', 0)
    # lr (x30)
    regs += struct.pack('<Q', 0)
    # sp
    regs += struct.pack('<Q', load_addr + aligned_size - 0x10000)  # Stack at end of segment
    # pc — ENTRY POINT
    regs += struct.pack('<Q', load_addr)
    # cpsr
    regs += struct.pack('<I', 0x3C5)  # EL1h, IRQs masked

    thread_cmd += regs

    # Pad thread command to multiple of 8
    while len(thread_cmd) % 8 != 0:
        thread_cmd += b'\0'

    # Recalculate thread command size
    thread_cmd = struct.pack('<II', LC_UNIXTHREAD, len(thread_cmd)) + thread_cmd[8:]

    # Total load commands
    load_cmds = seg_cmd + thread_cmd
    ncmds = 2
    sizeofcmds = len(load_cmds)

    # Mach-O header (32 bytes)
    header = struct.pack('<IIIIIIII',
        MH_MAGIC_64,       # magic
        CPU_TYPE_ARM64,     # cputype
        CPU_SUBTYPE_ALL,    # cpusubtype
        MH_EXECUTE,         # filetype
        ncmds,              # ncmds
        sizeofcmds,         # sizeofcmds
        MH_NOUNDEFS,        # flags
        0                   # reserved
    )

    # Calculate where payload starts (after header + load commands, page-aligned)
    header_size = len(header) + sizeofcmds
    payload_offset = (header_size + page_size - 1) & ~(page_size - 1)

    # Need to adjust segment fileoff to point to payload
    seg_cmd = struct.pack('<II', LC_SEGMENT_64, 72)
    seg_cmd += segname
    seg_cmd += struct.pack('<QQQQ', load_addr, aligned_size, payload_offset, aligned_size)
    seg_cmd += struct.pack('<II', 0x7, 0x5)
    seg_cmd += struct.pack('<II', 0, 0)

    # Rebuild
    load_cmds = seg_cmd + thread_cmd
    sizeofcmds = len(load_cmds)

    header = struct.pack('<IIIIIIII',
        MH_MAGIC_64, CPU_TYPE_ARM64, CPU_SUBTYPE_ALL,
        MH_EXECUTE, ncmds, sizeofcmds, MH_NOUNDEFS, 0
    )

    # Assemble final binary
    out = header + load_cmds
    # Pad to payload offset
    out += b'\0' * (payload_offset - len(out))
    # Append payload
    out += payload_padded

    with open(output_path, 'wb') as f:
        f.write(out)

    print(f"[*] Sphragis Mach-O created: {output_path}")
    print(f"    Payload: {payload_size} bytes")
    print(f"    Load address: 0x{load_addr:X}")
    print(f"    Entry point: 0x{load_addr:X}")
    print(f"    Total size: {len(out)} bytes")

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <input.bin> <output.macho>")
        sys.exit(1)
    make_macho(sys.argv[1], sys.argv[2])
