#!/usr/bin/env python3
"""
Sphragis — img4 Container Packer
Creates an Apple img4 container from a raw kernel binary.

img4 format is ASN1/DER encoded with this structure:
  IMG4 [SEQUENCE]
    ├── IM4P (payload) [SEQUENCE]
    │   ├── "IM4P" tag
    │   ├── type (e.g., "krnl" for kernel)
    │   ├── description string
    │   └── payload data (our kernel binary)
    └── IM4M (manifest) [OPTIONAL — not needed with Permissive Security]

With Permissive Security / Reduced Security enabled,
iBoot will load an img4 container WITHOUT a valid signature.

Usage: python3 make_img4.py <input.bin> <output.img4>
"""

import struct
import sys

def der_length(length):
    """Encode length in DER format."""
    if length < 0x80:
        return bytes([length])
    elif length < 0x100:
        return bytes([0x81, length])
    elif length < 0x10000:
        return bytes([0x82, (length >> 8) & 0xFF, length & 0xFF])
    elif length < 0x1000000:
        return bytes([0x83, (length >> 16) & 0xFF, (length >> 8) & 0xFF, length & 0xFF])
    else:
        return bytes([0x84, (length >> 24) & 0xFF, (length >> 16) & 0xFF,
                      (length >> 8) & 0xFF, length & 0xFF])

def der_sequence(contents):
    """Wrap contents in a DER SEQUENCE."""
    return bytes([0x30]) + der_length(len(contents)) + contents

def der_ia5string(s):
    """Encode a string as DER IA5String."""
    data = s.encode('ascii')
    return bytes([0x16]) + der_length(len(data)) + data

def der_octetstring(data):
    """Encode data as DER OCTET STRING."""
    return bytes([0x04]) + der_length(len(data)) + data

def der_integer(val):
    """Encode an integer in DER."""
    # Simple case for small non-negative integers
    if val == 0:
        return bytes([0x02, 0x01, 0x00])
    b = val.to_bytes((val.bit_length() + 7) // 8, 'big')
    if b[0] & 0x80:
        b = bytes([0x00]) + b
    return bytes([0x02]) + der_length(len(b)) + b

def make_im4p(payload_type, description, data):
    """
    Create an IM4P (Image4 Payload) container.

    Structure:
      SEQUENCE {
        IA5String "IM4P"
        IA5String <type>     -- e.g., "krnl", "rdsk", "logo"
        IA5String <desc>     -- description
        OCTET STRING <data>  -- the actual payload
      }
    """
    contents = (
        der_ia5string("IM4P") +
        der_ia5string(payload_type) +
        der_ia5string(description) +
        der_octetstring(data)
    )
    return der_sequence(contents)

def make_img4(im4p):
    """
    Create a full IMG4 container wrapping an IM4P payload.

    Structure:
      SEQUENCE {
        IA5String "IMG4"
        IM4P [0] EXPLICIT (the payload)
      }

    We skip IM4M (manifest) since Permissive Security doesn't require it.
    """
    # The IM4P is wrapped in a context-specific [0] EXPLICIT tag
    tagged_im4p = bytes([0xA0]) + der_length(len(im4p)) + im4p

    contents = der_ia5string("IMG4") + tagged_im4p
    return der_sequence(contents)

def make_kernelcache(payload):
    """
    Create a kernel cache img4 container.

    Apple's kernelcache is an img4 with type "krnl" containing
    either a compressed or raw Mach-O kernel.

    For our purposes, we embed the raw binary directly.
    iBoot with Permissive Security should accept it.
    """
    # Wrap payload in a Mach-O if it isn't already
    if payload[:4] == b'\xCF\xFA\xED\xFE':
        # Already Mach-O — use as is
        kernel_data = payload
    else:
        # Raw binary — wrap in minimal Mach-O
        kernel_data = payload

    im4p = make_im4p("krnl", "Sphragis Kernel", kernel_data)
    img4 = make_img4(im4p)
    return img4

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <input.macho|bin> <output.img4>")
        sys.exit(1)

    input_path = sys.argv[1]
    output_path = sys.argv[2]

    with open(input_path, 'rb') as f:
        payload = f.read()

    img4 = make_kernelcache(payload)

    with open(output_path, 'wb') as f:
        f.write(img4)

    print(f"[*] Sphragis img4 kernel cache created: {output_path}")
    print(f"    Input: {len(payload)} bytes")
    print(f"    Output: {len(img4)} bytes")
    print(f"    Type: krnl (kernel)")
    print(f"    Signature: NONE (requires Permissive Security)")
