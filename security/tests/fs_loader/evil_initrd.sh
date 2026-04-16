#!/usr/bin/env bash
# evil_initrd.sh — forge BATCHROM blobs that exercise
# src/kernel/mm/initrd.rs (ATTACK-FL-032..FL-035).
#
# Produces four test artifacts in $1 (default: .):
#   blob_oversized_size.bin    — size field claims 2 GiB, actual payload = 16 B
#   blob_crc_valid_payload_evil.bin — CRC32 matches but payload is attacker ELF
#   blob_double_magic.bin      — two BATCHROM magics (parser picks first)
#   blob_wrong_tail.bin        — valid head + size + crc, wrong trailing magic
#
# Layout (from initrd.rs):
#   "BATCHROM" (8) | u64_le size | payload (size bytes) | u32_le crc32 | "CHROMEND" (8)

set -euo pipefail

OUT=${1:-.}
mkdir -p "$OUT"

python3 - "$OUT" <<'PY'
import os, struct, sys, zlib

out = sys.argv[1]

HEAD = b'BATCHROM'
TAIL = b'CHROMEND'

def write_blob(path, head, size_field, payload, crc_field, tail):
    with open(path, 'wb') as f:
        f.write(head)
        f.write(struct.pack('<Q', size_field))
        f.write(payload)
        f.write(struct.pack('<I', crc_field))
        f.write(tail)
    print(f"wrote {path} ({os.path.getsize(path)} bytes)")

# 1) Size field lies — claims 2 GiB, actual payload is 16 B.
#    probe() will try to CRC32 over 2 GiB of mystery memory.
payload = b'\x00' * 16
write_blob(os.path.join(out, 'blob_oversized_size.bin'),
           HEAD, 2 * 1024 * 1024 * 1024, payload,
           0xDEADBEEF, TAIL)

# 2) CRC valid, payload = minimal ELF header that would then hit loader.rs bugs.
minimal_elf = (b'\x7fELF\x02\x01\x01' + b'\x00' * 9
               + struct.pack('<HHIQQQIHHHHHH',
                             3, 183, 1,         # ET_DYN, AArch64
                             0x1000, 64, 0, 0,
                             64, 56, 1, 0, 0, 0)
               + b'\x01\x00\x00\x00\x05\x00\x00\x00'
               + b'\x00' * 48)
size = len(minimal_elf)
crc = zlib.crc32(minimal_elf) & 0xFFFFFFFF
write_blob(os.path.join(out, 'blob_crc_valid_payload_evil.bin'),
           HEAD, size, minimal_elf, crc, TAIL)

# 3) Two BATCHROM magics — parser picks the first one (offset 0).
# We put a fake first, the "legit" second at offset 0x1000.
inner_size = 16
inner_payload = b'\xcc' * inner_size
inner_crc = zlib.crc32(inner_payload) & 0xFFFFFFFF
first = HEAD + struct.pack('<Q', inner_size) + inner_payload + struct.pack('<I', inner_crc) + TAIL
pad = b'\x00' * (0x1000 - len(first))
second = HEAD + struct.pack('<Q', inner_size) + (b'\xaa' * inner_size) + struct.pack('<I', zlib.crc32(b'\xaa' * inner_size) & 0xFFFFFFFF) + TAIL
with open(os.path.join(out, 'blob_double_magic.bin'), 'wb') as f:
    f.write(first); f.write(pad); f.write(second)
print(f"wrote {os.path.join(out, 'blob_double_magic.bin')}")

# 4) Valid head+size+crc but wrong tail magic ("CHROMEXX").
write_blob(os.path.join(out, 'blob_wrong_tail.bin'),
           HEAD, 8, b'\x00' * 8,
           zlib.crc32(b'\x00' * 8) & 0xFFFFFFFF, b'CHROMEXX')
PY
