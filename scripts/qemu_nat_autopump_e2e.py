#!/usr/bin/env python3
"""Followup 3c-autopump: main-loop runs NAT without explicit shell ticks.

Same cave→internet→cave round-trip as qemu_nat_full_pipeline_e2e.py
but NO `nat-forward` / `nat-reply` shell commands between sends.
The desktop's idle loop is expected to call `nat::tick()` on every
iteration, catching + forwarding each frame within a few ms.
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
LOG = ROOT / f"logs/qemu-tests/nat-autopump-{STAMP}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"sphragis\s*>\s*"

def ipv4_checksum(hdr: bytes) -> int:
    total = 0
    for i in range(0, len(hdr), 2):
        if i == 10: w = 0
        elif i + 1 < len(hdr): w = (hdr[i] << 8) | hdr[i+1]
        else: w = hdr[i] << 8
        total += w
    while total >> 16: total = (total & 0xFFFF) + (total >> 16)
    return (~total) & 0xFFFF

def l4_checksum(src_ip, dst_ip, proto, l4: bytes) -> int:
    s = (src_ip >> 16) + (src_ip & 0xFFFF) + (dst_ip >> 16) + (dst_ip & 0xFFFF)
    s += proto + len(l4)
    i = 0
    while i + 1 < len(l4):
        s += (l4[i] << 8) | l4[i+1]; i += 2
    if i < len(l4): s += l4[i] << 8
    while s >> 16: s = (s & 0xFFFF) + (s >> 16)
    return (~s) & 0xFFFF

def build_tcp(src_mac, dst_mac, src_ip, dst_ip, src_port, dst_port,
              flags=0x02, seq=0, ack=0):
    frame = bytearray()
    frame += bytes(dst_mac) + bytes(src_mac) + b"\x08\x00"
    ip = bytearray(20)
    ip[0] = 0x45
    ip[8] = 64; ip[9] = 6
    ip[12:16] = src_ip.to_bytes(4, "big")
    ip[16:20] = dst_ip.to_bytes(4, "big")
    tcp = bytearray(20)
    tcp[0:2] = src_port.to_bytes(2, "big")
    tcp[2:4] = dst_port.to_bytes(2, "big")
    tcp[4:8] = seq.to_bytes(4, "big")
    tcp[8:12] = ack.to_bytes(4, "big")
    tcp[12] = 5 << 4
    tcp[13] = flags
    tcp[14:16] = (8192).to_bytes(2, "big")
    total_len = 20 + len(tcp)
    ip[2:4] = total_len.to_bytes(2, "big")
    ip[10:12] = ipv4_checksum(bytes(ip)).to_bytes(2, "big")
    tcp[16:18] = l4_checksum(src_ip, dst_ip, 6, bytes(tcp)).to_bytes(2, "big")
    return bytes(frame + ip + tcp)

def ip_int(s): a,b,c,d = [int(p) for p in s.split(".")]; return (a<<24)|(b<<16)|(c<<8)|d

def send_frame(conn, f): conn.sendall(struct.pack(">I", len(f)) + f)

def recv_frame(conn, timeout):
    conn.settimeout(timeout)
    buf = b""
    while len(buf) < 4:
        c = conn.recv(4 - len(buf))
        if not c: return None
        buf += c
    n = struct.unpack(">I", buf)[0]
    if n > 65536: return None
    data = b""
    while len(data) < n:
        c = conn.recv(n - len(data))
        if not c: return None
        data += c
    return data

def recv_ipv4(conn, timeout):
    deadline = time.time() + timeout
    while time.time() < deadline:
        left = max(0.1, deadline - time.time())
        f = recv_frame(conn, left)
        if f is None: return None
        if len(f) >= 14 and f[12:14] == b"\x08\x00": return f

def listener(port):
    srv = socket.socket(); srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("127.0.0.1", port)); srv.listen(1)
    state = {"conn": None, "srv": srv}
    def loop(): conn, _ = srv.accept(); state["conn"] = conn
    threading.Thread(target=loop, daemon=True).start()
    return state

def run_cmd(c, cmd, timeout=10):
    c.sendline(cmd.encode())
    c.expect(PROMPT, timeout=timeout)
    return ANSI.sub(b"", c.before or b"").decode("utf-8", "replace")

def main():
    HOST = 25560; CAVE = 25561
    h = listener(HOST); v = listener(CAVE)
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
        "-netdev", f"socket,id=hostnet,connect=127.0.0.1:{HOST}",
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
        time.sleep(0.3); c.sendline(b"batman")
        c.expect(PROMPT, timeout=60)
        for _ in range(60):
            if h["conn"] and v["conn"]: break
            time.sleep(0.2)
        if not (h["conn"] and v["conn"]): raise RuntimeError("QEMU sockets")

        run_cmd(c, "nat-reset")
        run_cmd(c, "nat-bind 192.168.77.10 kali")
        run_cmd(c, "cpol-add kali 93.184.216.34 443 tcp")

        kali_mac = [0x02, 0xAA, 0, 0, 0, 0x10]
        nic1_mac = [0x52, 0x54, 0, 0x12, 0x34, 0x57]
        nic0_mac = [0x52, 0x54, 0, 0x12, 0x34, 0x56]
        gw_mac   = [0x52, 0x55, 0x0A, 0x00, 0x02, 0x02]
        cave_ip = ip_int("192.168.77.10")
        dst_ip  = ip_int("93.184.216.34")

        # NO explicit nat-forward! Just send and wait.
        out_frame = build_tcp(kali_mac, nic1_mac, cave_ip, dst_ip, 51234, 443, flags=0x02)
        send_frame(v["conn"], out_frame)
        details.append(f"cave→nic1 SYN sent ({len(out_frame)} B)")

        fwd = recv_ipv4(h["conn"], timeout=5.0)
        if fwd is None:
            details.append("no IPv4 frame forwarded to nic 0 — autopump may be stuck")
        else:
            sport = int.from_bytes(fwd[34:36], "big")
            dport = int.from_bytes(fwd[36:38], "big")
            src_ip = int.from_bytes(fwd[14+12:14+16], "big")
            details.append(f"autopump forwarded: src=0x{src_ip:08x}:{sport} dport={dport}")
            out_ok = (src_ip == ip_int("10.0.2.15") and dport == 443 and sport >= 50000
                      and fwd[6:12] == bytes(nic0_mac) and fwd[0:6] == bytes(gw_mac))
            details.append("outbound OK" if out_ok else "outbound MISMATCH")
            if out_ok:
                # Send the reply on the host side — desktop tick will reverse-NAT it.
                reply = build_tcp(gw_mac, nic0_mac, dst_ip, ip_int("10.0.2.15"),
                                  443, sport, flags=0x12, seq=0xAA, ack=1)
                send_frame(h["conn"], reply)
                details.append(f"internet→nic0 SYN/ACK sent ({len(reply)} B)")
                back = recv_ipv4(v["conn"], timeout=5.0)
                if back is None:
                    details.append("no IPv4 frame delivered back on nic 1")
                else:
                    d2 = int.from_bytes(back[14+16:14+20], "big")
                    dp2 = int.from_bytes(back[36:38], "big")
                    in_ok = (d2 == cave_ip and dp2 == 51234
                             and back[0:6] == bytes(kali_mac))
                    details.append(f"autopump delivered: dst=0x{d2:08x}:{dp2}")
                    details.append("inbound OK" if in_ok else "inbound MISMATCH")
                    if out_ok and in_ok: verdict = "PASS"

        run_cmd(c, "nat-stats")
    except (pexpect.TIMEOUT, RuntimeError) as e:
        details.append(f"error: {e}")
    finally:
        try: c.terminate(force=True)
        except Exception: pass
        fp.close()
        for s in (h, v):
            try: s["srv"].close()
            except Exception: pass
            if s["conn"]:
                try: s["conn"].close()
                except Exception: pass
        daemon.terminate()
        try: daemon.wait(timeout=3)
        except subprocess.TimeoutExpired: daemon.kill()

    print("--- details ---")
    for d in details: print("  " + d)
    print(f"\nLog: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict == "PASS" else 1

if __name__ == "__main__":
    sys.exit(main())
