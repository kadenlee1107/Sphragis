#!/usr/bin/env python3
"""
evil_png.py — craft PNGs that exercise src/browser/media/png.rs bounds
checks (ATTACK-FL-037, FL-038, FL-039, FL-040).

Artifacts:
  - evil_chunk_len_wrap.png   — IDAT chunk_len near usize::MAX
  - evil_idat_bomb.png        — 100 IDAT chunks (buffer cap = 64 KB)
  - evil_palette_no_plte.png  — color_type=3 (indexed) with no PLTE chunk
  - evil_maxed_dims.png       — width=512, height=512 (edge of MAX); overruns inflate buf

PNG layout: [\x89PNG\r\n\x1a\n][IHDR][IDAT*][IEND]
All numeric fields big-endian.

Usage:  python3 evil_png.py [outdir]
"""
import struct
import sys
import os
import zlib

SIG = b'\x89PNG\r\n\x1a\n'

def chunk(tag, data):
    crc = zlib.crc32(tag + data) & 0xFFFFFFFF
    return struct.pack('>I', len(data)) + tag + data + struct.pack('>I', crc)

def raw_chunk(length_field, tag, data):
    """Emit a chunk where the length FIELD lies about the actual data length."""
    crc = zlib.crc32(tag + data) & 0xFFFFFFFF
    return struct.pack('>I', length_field) + tag + data + struct.pack('>I', crc)

def ihdr(width, height, bit_depth=8, color_type=6):
    # color_type: 0=gray, 2=RGB, 3=palette, 4=gray+a, 6=RGBA
    return chunk(b'IHDR',
                 struct.pack('>IIBBBBB', width, height, bit_depth, color_type, 0, 0, 0))

def write(path, data):
    with open(path, 'wb') as f:
        f.write(data)
    print(f"wrote {path} ({len(data)} bytes)")

def main():
    outdir = sys.argv[1] if len(sys.argv) > 1 else '.'
    os.makedirs(outdir, exist_ok=True)

    # 1) chunk_len field says 0xFFFFFFF0 but only 8 bytes of data follow.
    #    parser: pos += 12 + 0xFFFFFFF0 → pos jumps ~4 GiB.
    #    the slice computation data.len()-pos-8 underflows if pos > data.len().
    bogus = raw_chunk(0xFFFFFFF0, b'IDAT', b'\x78\x9c\x00\x00\x00\x00\x00\x00')
    blob = SIG + ihdr(16, 16) + bogus + chunk(b'IEND', b'')
    write(os.path.join(outdir, 'evil_chunk_len_wrap.png'), blob)

    # 2) IDAT bomb — lots of small IDATs, parser silently truncates at 64 KiB.
    idat_payload = zlib.compress(b'\x00' + b'\xff\x00\x00\xff' * 16)  # 1-row RGBA
    idats = b''.join(chunk(b'IDAT', idat_payload) for _ in range(200))
    blob = SIG + ihdr(16, 1) + idats + chunk(b'IEND', b'')
    write(os.path.join(outdir, 'evil_idat_bomb.png'), blob)

    # 3) color_type=3 (indexed) but no PLTE chunk. png.rs sets bpp=1 and
    #    pretends the decompressed stream is pixel data; no palette lookup.
    # Pixel data = filter byte 0 + N indices
    raw = b'\x00' + b'\x00' * 16
    compressed = zlib.compress(raw)
    blob = SIG + ihdr(16, 1, color_type=3) + chunk(b'IDAT', compressed) + chunk(b'IEND', b'')
    write(os.path.join(outdir, 'evil_palette_no_plte.png'), blob)

    # 4) Maxed dimensions — 512x512 RGBA. stride=2049; decomp_size=1_049_088.
    # decompressed buffer in png.rs is 131_072. inflate MAY write past the end
    # if gzip.rs doesn't bound its output.
    big_raw = (b'\x00' + b'\x00' * (512 * 4)) * 512   # 1_049_088 bytes
    big_comp = zlib.compress(big_raw)
    blob = SIG + ihdr(512, 512) + chunk(b'IDAT', big_comp) + chunk(b'IEND', b'')
    write(os.path.join(outdir, 'evil_maxed_dims.png'), blob)

if __name__ == '__main__':
    main()
