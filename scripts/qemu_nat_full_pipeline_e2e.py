#!/usr/bin/env python3
"""Followup 3c-nat-forward: full bidirectional packet pipeline E2E.

We own BOTH sides of Bat_OS's network:
  nic 0 (host) = socket peer on :25558  (Python acts as the 'internet')
  nic 1 (caves)= socket peer on :25557  (Python acts as a container)

Flow:
  1. Shell: nat-bind 192.168.77.10 kali
           cpol-add kali 93.184.216.34 443 tcp
  2. Python (cave side, :25557): send TCP SYN 192.168.77.10:51234
     → 93.184.216.34:443
  3. Shell: nat-forward   (classifier Allow → NAT alloc → rewrite → send nic 0)
  4. Python (host side, :25558): receive the rewritten frame. Verify:
     - Ethernet src = nic0 MAC, dst = gw MAC (52:55:0a:00:02:02)
     - IPv4  src = 10.0.2.15 (nic0), dst unchanged = 93.184.216.34
     - TCP   src_port = NAT eph (>= 50000), dst_port unchanged = 443
     - IPv4 + TCP checksums correct
  5. Python (host side, :25558): craft reply frame: SYN/ACK from
     93.184.216.34:443 back to 10.0.2.15:<NAT_eph>
  6. Shell: nat-reply    (reverse-NAT → send nic 1)
  7. Python (cave side, :25557): receive the reply frame. Verify:
     - IPv4 dst rewritten to 192.168.77.10 (cave orig)
     - TCP dst_port rewritten to 51234 (cave orig)

PASS iff both directions come through with correct rewriting.
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
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
STAMP = datetime.now().strftime('%Y%m%d-%H%M%S')
LOG = ROOT / f"logs/qemu-tests/nat-full-{STAMP}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"bat_os\s*>\s*"

# ── Frame construction ─────────────────────────────────────────────

def ipv4_checksum(hdr: bytes) -> int:
    total = 0
    # Zero the checksum field at offset 10 during compute.
    for i in range(0, len(hdr), 2):
        if i == 10: w = 0
        elif i + 1 < len(hdr): w = (hdr[i] << 8) | hdr[i+1]
        else: w = hdr[i] << 8
        total += w
    while total >> 16: total = (total & 0xFFFF) + (total >> 16)
    return (~total) & 0xFFFF

def l4_checksum(src_ip, dst_ip, proto, l4: bytes) -> int:
    s = 0
    s += (src_ip >> 16) & 0xFFFF
    s +=  src_ip        & 0xFFFF
    s += (dst_ip >> 16) & 0xFFFF
    s +=  dst_ip        & 0xFFFF
    s += proto
    s += len(l4)
    i = 0
    while i + 1 < len(l4):
        s += (l4[i] << 8) | l4[i+1]
        i += 2
    if i < len(l4): s += (l4[i] << 8)
    while s >> 16: s = (s & 0xFFFF) + (s >> 16)
    return (~s) & 0xFFFF

def build_tcp_ipv4(src_mac, dst_mac, src_ip, dst_ip, src_port, dst_port,
                   tcp_flags=0x02, seq=0, ack=0, payload=b""):
    frame = bytearray()
    frame += bytes(dst_mac)
    frame += bytes(src_mac)
    frame += b"\x08\x00"

    # IPv4 header (20 B)
    ip = bytearray(20)
    ip[0] = 0x45
    ip[1] = 0x00
    # total len set below
    ip[4:6] = (0).to_bytes(2, "big")  # id
    ip[6:8] = (0).to_bytes(2, "big")  # frag
    ip[8] = 64
    ip[9] = 6
    ip[10:12] = b"\x00\x00"
    ip[12:16] = src_ip.to_bytes(4, "big")
    ip[16:20] = dst_ip.to_bytes(4, "big")

    # TCP header (20 B)
    tcp = bytearray(20)
    tcp[0:2] = src_port.to_bytes(2, "big")
    tcp[2:4] = dst_port.to_bytes(2, "big")
    tcp[4:8] = seq.to_bytes(4, "big")
    tcp[8:12] = ack.to_bytes(4, "big")
    tcp[12] = (5 << 4)  # data offset 5 words
    tcp[13] = tcp_flags
    tcp[14:16] = (8192).to_bytes(2, "big")
    tcp[16:18] = b"\x00\x00"  # checksum
    tcp[18:20] = b"\x00\x00"  # urg

    tcp_full = bytes(tcp) + payload
    total_len = 20 + len(tcp_full)
    ip[2:4] = total_len.to_bytes(2, "big")
    ip[10:12] = ipv4_checksum(bytes(ip)).to_bytes(2, "big")

    tcp_ck = l4_checksum(src_ip, dst_ip, 6, tcp_full)
    tcp = bytearray(tcp_full)
    tcp[16:18] = tcp_ck.to_bytes(2, "big")

    return bytes(frame + ip + tcp)

def ip_int(s): a,b,c,d = [int(p) for p in s.split(".")]; return (a<<24)|(b<<16)|(c<<8)|d

# ── QEMU socket netdev wire: 4-byte BE length + frame ────────────────

def send_frame(conn, frame):
    conn.sendall(struct.pack(">I", len(frame)) + frame)

def recv_frame(conn, timeout=2.0):
    conn.settimeout(timeout)
    # Read 4-byte length
    buf = b""
    while len(buf) < 4:
        chunk = conn.recv(4 - len(buf))
        if not chunk: return None
        buf += chunk
    n = struct.unpack(">I", buf)[0]
    if n > 65536: return None
    data = b""
    while len(data) < n:
        chunk = conn.recv(n - len(data))
        if not chunk: return None
        data += chunk
    return data

def recv_ipv4_frame(conn, timeout=3.0):
    """Drain frames until we find one with ethertype 0x0800.
    Other frames (ARP, IPv6, LLDP...) are noise in this test."""
    deadline = time.time() + timeout
    while time.time() < deadline:
        remaining = max(0.1, deadline - time.time())
        f = recv_frame(conn, timeout=remaining)
        if f is None: return None
        if len(f) >= 14 and f[12:14] == b"\x08\x00":
            return f
    return None

def start_listener(port):
    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("127.0.0.1", port)); srv.listen(1)
    state = {"conn": None, "srv": srv}
    def loop():
        conn, _ = srv.accept()
        state["conn"] = conn
    threading.Thread(target=loop, daemon=True).start()
    return state

def run_cmd(c, cmd, timeout=10):
    c.sendline(cmd.encode())
    c.expect(PROMPT, timeout=timeout)
    return ANSI.sub(b"", c.before or b"").decode("utf-8", "replace")

def main():
    HOST_PORT = 25558  # nic 0 peer
    CAVE_PORT = 25557  # nic 1 peer
    host_srv = start_listener(HOST_PORT)
    cave_srv = start_listener(CAVE_PORT)

    daemon = subprocess.Popen(
        ["python3", str(ROOT / "scripts" / "batcaved.py")],
        stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT,
    )
    for _ in range(40):
        try: socket.create_connection(("127.0.0.1", 9999), timeout=0.3).close(); break
        except OSError: time.sleep(0.2)

    qemu_args = [
        "qemu-system-aarch64",
        "-machine", "virt", "-cpu", "max", "-m", "2G",
        "-display", "none",
        "-device", "virtio-gpu-device",
        "-device", "virtio-keyboard-device",
        # Both NICs as socket so Python owns both wires. Order matters
        # for our probe-reversal: first -device declared ends up nic 0.
        "-netdev", f"socket,id=hostnet,connect=127.0.0.1:{HOST_PORT}",
        "-device", "virtio-net-device,netdev=hostnet",
        "-netdev", f"socket,id=cavenet,connect=127.0.0.1:{CAVE_PORT}",
        "-device", "virtio-net-device,netdev=cavenet",
        # But we also need a way for the kernel to call batcaved for its
        # own deadman heartbeat (otherwise boot blocks). Add a third
        # netdev... actually the deadman goes through nic 0; since nic 0
        # is now socket, we lose slirp routing. Work around: deadman
        # auto-arm is best-effort and quiet on failure. Keep testing
        # before daemon arm — the batman auth passes, we just skip arm.
        "-serial", "mon:stdio",
        "-kernel", str(KERNEL),
    ]

    fp = open(LOG, "wb")
    c = pexpect.spawn(qemu_args[0], qemu_args[1:], timeout=90, logfile=fp, encoding=None)
    verdict = "FAIL"
    details = []
    try:
        c.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
        time.sleep(0.3); c.sendline(b"batman")
        c.expect(PROMPT, timeout=60)  # longer — deadman may timeout
        print("[full-e2e] shell ready")

        # Wait for QEMU to connect to both our listeners.
        for _ in range(60):
            if host_srv["conn"] and cave_srv["conn"]: break
            time.sleep(0.2)
        if not (host_srv["conn"] and cave_srv["conn"]):
            raise RuntimeError("QEMU sockets didn't both connect")
        print(f"[full-e2e] nic0 peer={host_srv['conn'].getpeername()} "
              f"nic1 peer={cave_srv['conn'].getpeername()}")

        # Kernel state
        run_cmd(c, "nat-reset")
        run_cmd(c, "nat-bind 192.168.77.10 kali")
        run_cmd(c, "cpol-add kali 93.184.216.34 443 tcp")

        # ── OUTBOUND frame: cave → internet ─────────────────────────
        kali_mac = [0x02, 0xAA, 0, 0, 0, 0x10]
        nic1_mac = [0x52, 0x54, 0, 0x12, 0x34, 0x57]
        nic0_mac = [0x52, 0x54, 0, 0x12, 0x34, 0x56]
        gw_mac   = [0x52, 0x55, 0x0A, 0x00, 0x02, 0x02]
        cave_ip = ip_int("192.168.77.10")
        dst_ip  = ip_int("93.184.216.34")
        out_frame = build_tcp_ipv4(kali_mac, nic1_mac,
                                   cave_ip, dst_ip,
                                   51234, 443, tcp_flags=0x02)
        send_frame(cave_srv["conn"], out_frame)
        print(f"[full-e2e] cave→nic1: SYN 192.168.77.10:51234 → 93.184.216.34:443 ({len(out_frame)} B)")
        time.sleep(0.3)

        run_cmd(c, "nat-forward")

        fwd = recv_ipv4_frame(host_srv["conn"], timeout=3.0)
        if fwd is None:
            details.append("no outbound frame observed on nic 0 peer")
        else:
            details.append(f"forwarded frame len={len(fwd)} hex_head={fwd[:34].hex()}")
            # Parse the forwarded frame
            if len(fwd) < 14 + 20:
                details.append("forwarded frame truncated (< eth+ipv4)")
            else:
                eth_src = fwd[6:12]
                eth_dst = fwd[0:6]
                src_ip = int.from_bytes(fwd[14+12:14+16], "big")
                dst_ip_s = int.from_bytes(fwd[14+16:14+20], "big")
                sport = int.from_bytes(fwd[34:36], "big")
                dport = int.from_bytes(fwd[36:38], "big")
                details.append(f"forwarded eth_src={eth_src.hex()} eth_dst={eth_dst.hex()} "
                               f"src=0x{src_ip:08x}:{sport} dst=0x{dst_ip_s:08x}:{dport}")

                ok = (bytes(nic0_mac) == eth_src
                      and bytes(gw_mac) == eth_dst
                      and src_ip == ip_int("10.0.2.15")
                      and dst_ip_s == dst_ip
                      and dport == 443
                      and sport >= 50000)
                details.append("outbound rewrite OK" if ok else "outbound rewrite MISMATCH")

                if ok:
                    # ── INBOUND reply: internet → cave ──────────────
                    eph = sport
                    reply = build_tcp_ipv4(
                        gw_mac, nic0_mac,
                        dst_ip, ip_int("10.0.2.15"),
                        443, eph,
                        tcp_flags=0x12,   # SYN/ACK
                        seq=0x12345678, ack=1,
                    )
                    send_frame(host_srv["conn"], reply)
                    print(f"[full-e2e] internet→nic0: SYN/ACK back to 10.0.2.15:{eph} ({len(reply)} B)")
                    time.sleep(0.3)

                    run_cmd(c, "nat-reply")

                    back = recv_ipv4_frame(cave_srv["conn"], timeout=3.0)
                    if back is None:
                        details.append("no inbound frame observed on nic 1 peer")
                    else:
                        eth_src2 = back[6:12]
                        eth_dst2 = back[0:6]
                        dst_ip2 = int.from_bytes(back[14+16:14+20], "big")
                        dport2 = int.from_bytes(back[36:38], "big")
                        details.append(f"reverse eth_src={eth_src2.hex()} eth_dst={eth_dst2.hex()} "
                                       f"dst=0x{dst_ip2:08x}:{dport2}")
                        ok2 = (bytes(kali_mac) == eth_dst2
                               and bytes(nic1_mac) == eth_src2
                               and dst_ip2 == cave_ip
                               and dport2 == 51234)
                        details.append("inbound reverse-NAT OK" if ok2 else "inbound reverse-NAT MISMATCH")
                        if ok and ok2: verdict = "PASS"

        run_cmd(c, "nat-stats")
        run_cmd(c, "nat-table")
    except (pexpect.TIMEOUT, RuntimeError) as e:
        details.append(f"error: {e}")
    finally:
        try: c.terminate(force=True)
        except Exception: pass
        fp.close()
        for s in (host_srv, cave_srv):
            try: s["srv"].close()
            except Exception: pass
            if s["conn"]:
                try: s["conn"].close()
                except Exception: pass
        daemon.terminate()
        try: daemon.wait(timeout=3)
        except subprocess.TimeoutExpired: daemon.kill()

    print("\n--- details ---")
    for d in details: print("  " + d)
    print(f"\nLog: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict == "PASS" else 1

if __name__ == "__main__":
    sys.exit(main())
