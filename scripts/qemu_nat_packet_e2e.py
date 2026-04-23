#!/usr/bin/env python3
"""Followup 3c-packet-e2e: real frames into nic 1 → Bat_OS classifier.

Launches QEMU with:
  nic 0 = -netdev user                (slirp, for daemon control)
  nic 1 = -netdev socket,connect=127.0.0.1:NNNN
Our Python listener on NNNN is the peer of nic 1. We send raw
Ethernet frames (QEMU length-prefix protocol: 4-byte BE length)
simulating containers on the caves segment. Bat_OS's kernel pulls
frames off nic 1 via `nat-pump`, classifies each one against
`cave_policy`, and updates counters. We read counters back through
the shell and verify.

Expected:
  1. nat-reset                       (counters zero)
  2. nat-bind 192.168.77.10 kali     (register caves source IP)
  3. cpol-add kali 8.8.8.8 53 udp
     cpol-add kali 93.184.216.34 443 tcp
  4. Python sends 4 frames into nic 1:
       a) kali → 8.8.8.8:53/udp           (Allow)
       b) kali → 93.184.216.34:443/tcp    (Allow)
       c) kali → 203.0.113.42:443/tcp     (DropPolicy)
       d) unknown src 10.0.0.77 → any     (DropUnknownSrc)
  5. nat-pump drains nic 1 and reports:
       drained 4, allow=2, drop-policy=1, drop-unk-src=1, drop-parse=0
"""
import pexpect, re, socket, struct, subprocess, sys, threading, time
from pathlib import Path
from datetime import datetime

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
STAMP = datetime.now().strftime('%Y%m%d-%H%M%S')
LOG = ROOT / f"logs/qemu-tests/nat-e2e-qemu-{STAMP}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"bat_os\s*>\s*"

def build_eth_ipv4(src_mac, dst_mac, src_ip, dst_ip,
                   src_port, dst_port, proto=6):
    # src_ip / dst_ip are 32-bit host-order ints (high byte = first octet).
    ver_ihl = 0x45
    tos = 0
    total_len_slot = None
    frame = bytearray()
    # Ethernet
    frame += bytes(dst_mac)
    frame += bytes(src_mac)
    frame += b"\x08\x00"          # IPv4
    # IPv4
    frame += bytes([ver_ihl, tos])
    total_len_slot = len(frame)
    frame += b"\x00\x00"          # total length (fill later)
    frame += b"\x00\x00\x00\x00"  # id + flags/frag
    frame += bytes([64, proto])   # TTL + proto
    frame += b"\x00\x00"          # header checksum (skipped)
    frame += struct.pack(">I", src_ip)
    frame += struct.pack(">I", dst_ip)
    # TCP/UDP (minimal)
    frame += struct.pack(">HH", src_port, dst_port)
    # pad L4 to 20 B so parser has >= 4 bytes
    frame += b"\x00" * 16
    # Back-fill IPv4 total length
    total_len = len(frame) - 14
    frame[total_len_slot:total_len_slot+2] = struct.pack(">H", total_len)
    return bytes(frame)

def ip_int(s):
    a, b, c, d = [int(p) for p in s.split(".")]
    return (a<<24) | (b<<16) | (c<<8) | d

def start_qemu_socket_server(port):
    """QEMU will connect to us; we accept one connection and keep
    it ready for write+read. Returns a state dict with 'conn' once
    QEMU has connected."""
    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("127.0.0.1", port))
    srv.listen(1)
    state = {"conn": None, "srv": srv}
    def accept_once():
        conn, _ = srv.accept()
        conn.setblocking(True)
        state["conn"] = conn
    threading.Thread(target=accept_once, daemon=True).start()
    return state

def send_frame(conn, frame: bytes):
    conn.sendall(struct.pack(">I", len(frame)) + frame)

def run_cmd(c, cmd, timeout=10):
    c.sendline(cmd.encode())
    c.expect(PROMPT, timeout=timeout)
    raw = ANSI.sub(b"", c.before or b"").decode("utf-8", "replace")
    return "\n".join(l.rstrip() for l in raw.splitlines())

def main():
    port = 25557
    srv_state = start_qemu_socket_server(port)

    # Deadman likes a reachable daemon at boot.
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
        "-netdev", "user,id=hostnet",
        "-device", "virtio-net-device,netdev=hostnet",
        "-netdev", f"socket,id=cavenet,connect=127.0.0.1:{port}",
        "-device", "virtio-net-device,netdev=cavenet",
        "-serial", "mon:stdio",
        "-kernel", str(KERNEL),
    ]

    fp = open(LOG, "wb")
    c = pexpect.spawn(qemu_args[0], qemu_args[1:], timeout=90, logfile=fp, encoding=None)
    verdict = "FAIL"
    try:
        c.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
        time.sleep(0.3); c.sendline(b"batman")
        c.expect(PROMPT, timeout=30)
        print("[e2e] shell ready")

        # Wait for QEMU's socket netdev to connect to us.
        for _ in range(40):
            if srv_state["conn"] is not None: break
            time.sleep(0.2)
        if srv_state["conn"] is None:
            raise RuntimeError("QEMU socket netdev never connected")
        conn = srv_state["conn"]
        print("[e2e] nic-1 socket peer connected")

        # Wire kernel state.
        print(run_cmd(c, "nat-reset"))
        print(run_cmd(c, "nat-bind 192.168.77.10 kali"))
        print(run_cmd(c, "cpol-add kali 8.8.8.8 53 udp"))
        print(run_cmd(c, "cpol-add kali 93.184.216.34 443 tcp"))

        # Craft + send 4 frames.
        kali_mac   = [0x02, 0xAA, 0, 0, 0, 0x10]
        unknown_mac= [0x02, 0xCC, 0, 0, 0, 0x01]
        gw_mac     = [0x02, 0xBB, 0, 0, 0, 0x01]
        frames = [
            # Allow: kali → 8.8.8.8:53/udp
            build_eth_ipv4(kali_mac, gw_mac,
                           ip_int("192.168.77.10"), ip_int("8.8.8.8"),
                           40000, 53, proto=17),
            # Allow: kali → 93.184.216.34:443/tcp
            build_eth_ipv4(kali_mac, gw_mac,
                           ip_int("192.168.77.10"), ip_int("93.184.216.34"),
                           52000, 443, proto=6),
            # DropPolicy: kali → 203.0.113.42:443/tcp (off-list)
            build_eth_ipv4(kali_mac, gw_mac,
                           ip_int("192.168.77.10"), ip_int("203.0.113.42"),
                           52001, 443, proto=6),
            # DropUnknownSrc: unknown 10.0.0.77 → anywhere
            build_eth_ipv4(unknown_mac, gw_mac,
                           ip_int("10.0.0.77"), ip_int("8.8.8.8"),
                           40000, 53, proto=17),
        ]
        for i, f in enumerate(frames):
            send_frame(conn, f)
            print(f"[e2e] sent frame {i} ({len(f)} B)")
        # Give virtio a moment to DMA.
        time.sleep(0.5)

        # Drain + report.
        out = run_cmd(c, "nat-pump", timeout=10)
        print(out)
        pass_checks = (
            "drained 4" in out
            and "allow=2" in out
            and "drop-policy=1" in out
            and "drop-unk-src=1" in out
            and "drop-parse=0" in out
        )
        verdict = "PASS" if pass_checks else "FAIL"
    except pexpect.TIMEOUT:
        print("[e2e] TIMEOUT")
    finally:
        try: c.terminate(force=True)
        except Exception: pass
        fp.close()
        try: srv_state["srv"].close()
        except Exception: pass
        if srv_state["conn"] is not None:
            try: srv_state["conn"].close()
            except Exception: pass
        daemon.terminate()
        try: daemon.wait(timeout=3)
        except subprocess.TimeoutExpired: daemon.kill()

    print(f"Log: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict == "PASS" else 1

if __name__ == "__main__":
    sys.exit(main())
