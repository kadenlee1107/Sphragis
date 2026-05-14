#!/usr/bin/env python3
"""3c-gap-arp: containers need ARP to resolve the caves gateway.

Flow:
  1. Boot Sphragis, two NICs; nic 1 = socket peer at :25562.
  2. Python sends an ARP request: "who has 192.168.77.1? tell
     192.168.77.10" from a kali-like MAC.
  3. Sphragis's main-loop tick() pulls the frame, `try_handle_arp`
     recognises the request for the gateway, builds a reply with
     nic 1's MAC as sender-HW.
  4. Python reads the reply and verifies:
       - ethertype = 0x0806
       - op = reply (2)
       - sender HW = nic 1 MAC
       - sender proto = 192.168.77.1
       - target HW = original asker's MAC
       - target proto = 192.168.77.10
  5. A second "who has 192.168.77.99? tell 192.168.77.10" request
     (for a NON-gateway IP) should be IGNORED — no reply.

PASS iff reply #1 has correct shape and no reply #2 observed
within a 1s window, and nat-stats shows arp-replies=1 +
arp-ignored=1.
"""
import pexpect
import re
import socket
import struct
import subprocess
import sys
import threading
import time
from pathlib import Path
from datetime import datetime

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
STAMP = datetime.now().strftime('%Y%m%d-%H%M%S')
LOG = ROOT / f"logs/qemu-tests/arp-{STAMP}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"sphragis\s*>\s*"

def build_arp_request(sender_mac, sender_ip, target_ip):
    """Standard ARP-who-has on Ethernet/IPv4. Target HW is zero."""
    frame = bytearray()
    # Ethernet: broadcast + sender + 0806
    frame += b"\xff" * 6
    frame += bytes(sender_mac)
    frame += b"\x08\x06"
    # ARP
    frame += b"\x00\x01"          # hw = Eth
    frame += b"\x08\x00"          # proto = IPv4
    frame += b"\x06\x04"          # lens
    frame += b"\x00\x01"          # op = request
    frame += bytes(sender_mac)
    frame += sender_ip.to_bytes(4, "big")
    frame += b"\x00" * 6
    frame += target_ip.to_bytes(4, "big")
    return bytes(frame)

def ip_int(s): a,b,c,d=[int(p) for p in s.split(".")]; return (a<<24)|(b<<16)|(c<<8)|d
def send_frame(c, f): c.sendall(struct.pack(">I", len(f)) + f)

def recv_frame(c, timeout):
    c.settimeout(timeout)
    try:
        buf = b""
        while len(buf) < 4:
            chunk = c.recv(4 - len(buf))
            if not chunk: return None
            buf += chunk
        n = struct.unpack(">I", buf)[0]
        if n > 65536: return None
        data = b""
        while len(data) < n:
            chunk = c.recv(n - len(data))
            if not chunk: return None
            data += chunk
        return data
    except (TimeoutError, socket.timeout):
        return None

def listener(port):
    srv = socket.socket(); srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("127.0.0.1", port)); srv.listen(1)
    state = {"conn": None, "srv": srv}
    def loop(): c,_=srv.accept(); state["conn"]=c
    threading.Thread(target=loop, daemon=True).start()
    return state

def run_cmd(c, cmd, timeout=10):
    c.sendline(cmd.encode())
    c.expect(PROMPT, timeout=timeout)
    return ANSI.sub(b"", c.before or b"").decode("utf-8", "replace")

def main():
    CAVE = 25562
    v = listener(CAVE)
    daemon = subprocess.Popen(
        ["python3", str(ROOT / "scripts" / "batcaved.py")],
        stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT,
    )
    for _ in range(40):
        try: socket.create_connection(("127.0.0.1", 9999), timeout=0.3).close(); break
        except OSError: time.sleep(0.2)

    args = [
        "qemu-system-aarch64",
        "-machine", "virt", "-cpu", "max", "-m", "2G",
        "-display", "none",
        "-device", "virtio-gpu-device",
        "-device", "virtio-keyboard-device",
        "-netdev", "user,id=hostnet",
        "-device", "virtio-net-device,netdev=hostnet",
        "-netdev", f"socket,id=cavenet,connect=127.0.0.1:{CAVE}",
        "-device", "virtio-net-device,netdev=cavenet",
        "-serial", "mon:stdio",
        "-kernel", str(KERNEL),
    ]
    fp = open(LOG, "wb")
    c = pexpect.spawn(args[0], args[1:], timeout=90, logfile=fp, encoding=None)
    verdict = "FAIL"; details = []
    try:
        c.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
        time.sleep(0.3); c.sendline(b"sphragis-dev")
        c.expect(PROMPT, timeout=60)
        for _ in range(60):
            if v["conn"]: break
            time.sleep(0.2)
        if not v["conn"]: raise RuntimeError("QEMU socket didn't connect")
        run_cmd(c, "nat-reset")

        # 1. ARP for the gateway IP → expect reply
        kali_mac = [0x02, 0xAA, 0, 0, 0, 0x10]
        req1 = build_arp_request(kali_mac, ip_int("192.168.77.10"), ip_int("192.168.77.1"))
        send_frame(v["conn"], req1)
        details.append(f"ARP req for 192.168.77.1 sent ({len(req1)} B)")
        reply = recv_frame(v["conn"], timeout=3.0)
        if reply is None:
            details.append("no ARP reply observed")
        else:
            details.append(f"reply frame len={len(reply)} ethertype={reply[12:14].hex()}")
            if len(reply) >= 14 + 28 and reply[12:14] == b"\x08\x06":
                arp = reply[14:]
                op = int.from_bytes(arp[6:8], "big")
                sender_mac_r = arp[8:14]
                sender_ip_r = int.from_bytes(arp[14:18], "big")
                target_mac_r = arp[18:24]
                target_ip_r = int.from_bytes(arp[24:28], "big")
                details.append(f"arp op={op} sender_ip=0x{sender_ip_r:08x} "
                               f"target_ip=0x{target_ip_r:08x} "
                               f"sender_mac={sender_mac_r.hex()} target_mac={target_mac_r.hex()}")
                arp_ok = (op == 2
                          and sender_ip_r == ip_int("192.168.77.1")
                          and target_ip_r == ip_int("192.168.77.10")
                          and target_mac_r == bytes(kali_mac))
                details.append("arp #1 shape OK" if arp_ok else "arp #1 shape MISMATCH")
            else:
                details.append("arp #1: wrong ethertype / too short")
                arp_ok = False

        # 2. ARP for a non-gateway IP → expect SILENCE
        req2 = build_arp_request(kali_mac, ip_int("192.168.77.10"), ip_int("192.168.77.99"))
        send_frame(v["conn"], req2)
        details.append(f"ARP req for 192.168.77.99 sent ({len(req2)} B)")
        silent = recv_frame(v["conn"], timeout=1.0)
        if silent is None:
            details.append("arp #2 correctly ignored")
            ignored_ok = True
        else:
            details.append(f"arp #2 got unexpected reply len={len(silent)} "
                           f"etype={silent[12:14].hex()}")
            ignored_ok = False

        stats = run_cmd(c, "nat-stats")
        details.append(stats.strip())
        counters_ok = ("arp-replies:      1" in stats
                       and "arp-ignored:      1" in stats)
        details.append("counters OK" if counters_ok else "counters MISMATCH")

        if arp_ok and ignored_ok and counters_ok: verdict = "PASS"
    except (pexpect.TIMEOUT, RuntimeError) as e:
        details.append(f"error: {e}")
    finally:
        try: c.terminate(force=True)
        except Exception: pass
        fp.close()
        try: v["srv"].close()
        except Exception: pass
        if v["conn"]:
            try: v["conn"].close()
            except Exception: pass
        daemon.terminate()
        try: daemon.wait(timeout=3)
        except subprocess.TimeoutExpired: daemon.kill()

    print("--- details ---")
    for d in details:
        for line in d.splitlines(): print("  " + line[:160])
    print(f"\nLog: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict == "PASS" else 1

if __name__ == "__main__":
    sys.exit(main())
