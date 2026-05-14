#!/usr/bin/env python3
"""Inject a sequence of raw keystrokes into the M4 HV guest's
dockchannel vuart (/dev/ttyACM2) from a shell that isn't the
one running sphragis_hv_interactive.py.

Background: sphragis_hv_interactive.py holds /dev/ttyACM2 in raw mode.
When you try `printf '\t' > /dev/ttyACM2` from another shell, Linux
opens a fresh fd in cooked/canonical mode, which buffers and
line-discipline-mangles the byte before it reaches the USB endpoint.
Bytes often just never arrive.

This script opens the device in the SAME raw mode that the
interactive script uses (non-exclusive — both processes can coexist
on CDC-ACM), writes each byte with a small delay so the guest's
polling loop has time to consume, and exits.

Usage:
    sg dialout -c "/usr/bin/python3 scripts/hv/inject_keys.py KEYS..."

Key tokens (case-insensitive):
    tab    -> 0x09
    enter  -> 0x0d
    esc    -> 0x1b
    bs     -> 0x08
    space  -> 0x20
    ctrl+X -> x & 0x1f  (e.g. ctrl+q = 0x11)
    0xNN   -> raw hex byte
    any single character: sent as its ASCII byte
    "quoted string": sent literally, char by char

Examples:
    inject_keys.py tab tab tab enter
    inject_keys.py "batman" enter
    inject_keys.py ctrl+q                  # Ctrl+Q (close pane)
    inject_keys.py tab tab tab tab tab tab tab tab tab enter
"""
import sys
import os
import time
import termios

DEV = os.environ.get("SPHRAGIS_VUART", "/dev/ttyACM2")
DELAY_S = float(os.environ.get("INJECT_DELAY", "0.08"))


def parse_token(tok: str) -> bytes:
    lo = tok.lower()
    if lo == "tab":   return b"\x09"
    if lo == "enter": return b"\x0d"
    if lo == "esc":   return b"\x1b"
    if lo == "bs":    return b"\x08"
    if lo == "space": return b"\x20"
    if lo.startswith("ctrl+") and len(lo) == 6:
        c = lo[5]
        if "a" <= c <= "z":
            return bytes([ord(c) - ord("a") + 1])
    if lo.startswith("0x"):
        return bytes([int(lo, 16) & 0xff])
    if len(tok) == 1:
        return tok.encode()
    # quoted / multi-char string — send literal bytes
    return tok.encode()


def main():
    if len(sys.argv) < 2 or sys.argv[1] in ("-h", "--help"):
        print(__doc__)
        return 0 if len(sys.argv) >= 2 else 1

    # Open without DTR/RTS ioctls (USB CDC ACM often returns EPROTO on
    # those), and set raw termios. pyserial is convenient but its
    # dtr/rts setters explode on CDC — so do it by hand.
    fd = os.open(DEV, os.O_RDWR | os.O_NOCTTY | os.O_NONBLOCK)
    try:
        a = termios.tcgetattr(fd)
        a[0] &= ~(termios.BRKINT | termios.ICRNL | termios.INPCK
                  | termios.ISTRIP | termios.IXON)
        a[1] &= ~termios.OPOST
        a[2] = (a[2] & ~(termios.CSIZE | termios.PARENB)) | termios.CS8
        a[3] &= ~(termios.ECHO | termios.ECHONL | termios.ICANON
                  | termios.ISIG | termios.IEXTEN)
        termios.tcsetattr(fd, termios.TCSANOW, a)

        for tok in sys.argv[1:]:
            data = parse_token(tok)
            sys.stderr.write(f"  inject {tok!r} -> {data.hex()}\n")
            for b in data:
                os.write(fd, bytes([b]))
                # give the tight poll loop on the guest time to see it
                time.sleep(DELAY_S)
    finally:
        os.close(fd)
    return 0


if __name__ == "__main__":
    sys.exit(main())
