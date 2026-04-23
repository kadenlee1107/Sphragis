#!/usr/bin/env python3
"""3c-gap-icmp: ICMP Echo Request/Reply through NAT.

Full bidirectional proof:
  1. Python cave sends ICMP Echo Request from 192.168.77.10 id=0x1234
     → 8.8.8.8.
  2. Bat_OS classifier: proto=1, id used as src_port. cave_policy
     allows kali → 8.8.8.8 icmp. NAT alloc'd a translated id (eph
     port starting at 50000 now sharing space; the ICMP id is just
     a 16-bit handle).
  3. Rewritten Echo Request appears on nic 0 peer with src=10.0.2.15
     and a new identifier.
  4. Python internet replies with Echo Reply carrying the translated
     identifier back.
  5. Bat_OS reverse-NATs: identifier back to 0x1234, dst=192.168.77.10.
  6. Python cave sees Echo Reply with original id.
"""
import pexpect, re, socket, struct, subprocess, sys, threading, time
from pathlib import Path
from datetime import datetime

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
STAMP = datetime.now().strftime('%Y%m%d-%H%M%S')
LOG = ROOT / f"logs/qemu-tests/icmp-{STAMP}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)
ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"bat_os\s*>\s*"

def ipv4_cksum(hdr):
    s = 0
    for i in range(0, len(hdr), 2):
        w = 0 if i == 10 else (hdr[i] << 8) | (hdr[i+1] if i+1 < len(hdr) else 0)
        s += w
    while s >> 16: s = (s & 0xFFFF) + (s >> 16)
    return (~s) & 0xFFFF

def icmp_cksum(buf):
    s = 0
    i = 0
    while i + 1 < len(buf):
        s += (buf[i] << 8) | buf[i+1]; i += 2
    if i < len(buf): s += buf[i] << 8
    while s >> 16: s = (s & 0xFFFF) + (s >> 16)
    return (~s) & 0xFFFF

def build_icmp_echo(src_mac, dst_mac, src_ip, dst_ip, ident, seq, typ, payload=b"hello"):
    frame = bytearray()
    frame += bytes(dst_mac) + bytes(src_mac) + b"\x08\x00"
    ip = bytearray(20)
    ip[0] = 0x45
    ip[8] = 64; ip[9] = 1  # proto = ICMP
    ip[12:16] = src_ip.to_bytes(4, "big")
    ip[16:20] = dst_ip.to_bytes(4, "big")
    icmp = bytearray(8)
    icmp[0] = typ   # 8 request / 0 reply
    icmp[4:6] = ident.to_bytes(2, "big")
    icmp[6:8] = seq.to_bytes(2, "big")
    icmp_full = bytes(icmp) + payload
    total_len = 20 + len(icmp_full)
    ip[2:4] = total_len.to_bytes(2, "big")
    ip[10:12] = ipv4_cksum(bytes(ip)).to_bytes(2, "big")
    icmp = bytearray(icmp_full)
    icmp[2:4] = icmp_cksum(bytes(icmp)).to_bytes(2, "big")
    return bytes(frame + ip + icmp)

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

def recv_icmp(c, timeout):
    dl = time.time() + timeout
    while time.time() < dl:
        left = max(0.1, dl - time.time())
        f = recv_frame(c, left)
        if f is None: return None
        if len(f) >= 14 + 20 and f[12:14] == b"\x08\x00" and f[14 + 9] == 1:
            return f

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
    HOST = 25564; CAVE = 25565
    h = listener(HOST); v = listener(CAVE)
    daemon = subprocess.Popen(["python3", str(ROOT / "scripts" / "batcaved.py")],
        stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)
    for _ in range(40):
        try: socket.create_connection(("127.0.0.1", 9999), timeout=0.3).close(); break
        except OSError: time.sleep(0.2)

    args = ["qemu-system-aarch64", "-machine", "virt", "-cpu", "max", "-m", "2G",
            "-display", "none",
            "-device", "virtio-gpu-device", "-device", "virtio-keyboard-device",
            "-netdev", f"socket,id=hostnet,connect=127.0.0.1:{HOST}",
            "-device", "virtio-net-device,netdev=hostnet",
            "-netdev", f"socket,id=cavenet,connect=127.0.0.1:{CAVE}",
            "-device", "virtio-net-device,netdev=cavenet",
            "-serial", "mon:stdio", "-kernel", str(KERNEL)]
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
        if not (h["conn"] and v["conn"]): raise RuntimeError("sockets")

        run_cmd(c, "nat-reset")
        run_cmd(c, "nat-bind 192.168.77.10 kali")
        run_cmd(c, "cpol-add kali 8.8.8.8 0 icmp")

        kali_mac = [0x02, 0xAA, 0, 0, 0, 0x10]
        nic1_mac = [0x52, 0x54, 0, 0x12, 0x34, 0x57]
        nic0_mac = [0x52, 0x54, 0, 0x12, 0x34, 0x56]
        gw_mac   = [0x52, 0x55, 0x0A, 0x00, 0x02, 0x02]
        cave_ip = ip_int("192.168.77.10")
        dst_ip  = ip_int("8.8.8.8")
        orig_id = 0x1234

        out = build_icmp_echo(kali_mac, nic1_mac, cave_ip, dst_ip,
                              orig_id, 1, typ=8)
        send_frame(v["conn"], out)
        details.append(f"cave→nic1 Echo Request id={orig_id:#x} ({len(out)} B)")

        fwd = recv_icmp(h["conn"], timeout=3.0)
        if fwd is None:
            details.append("no ICMP frame forwarded to nic 0")
        else:
            src_ip_fwd = int.from_bytes(fwd[14+12:14+16], "big")
            ip_cksum = int.from_bytes(fwd[14+10:14+12], "big")
            icmp_start = 34
            typ = fwd[icmp_start]
            ident = int.from_bytes(fwd[icmp_start+4:icmp_start+6], "big")
            details.append(f"forwarded: src=0x{src_ip_fwd:08x} type={typ} id={ident:#x}")
            out_ok = (src_ip_fwd == ip_int("10.0.2.15")
                      and typ == 8
                      and ident != orig_id  # NAT should have translated it
                      and fwd[6:12] == bytes(nic0_mac)
                      and fwd[0:6] == bytes(gw_mac))
            details.append("outbound ICMP rewrite OK" if out_ok else "outbound ICMP MISMATCH")

            if out_ok:
                reply = build_icmp_echo(gw_mac, nic0_mac, dst_ip, ip_int("10.0.2.15"),
                                        ident, 1, typ=0)
                send_frame(h["conn"], reply)
                details.append(f"internet→nic0 Echo Reply id={ident:#x}")
                back = recv_icmp(v["conn"], timeout=3.0)
                if back is None:
                    details.append("no ICMP delivered back on nic 1")
                else:
                    dst_ip_back = int.from_bytes(back[14+16:14+20], "big")
                    typ2 = back[icmp_start]
                    id2 = int.from_bytes(back[icmp_start+4:icmp_start+6], "big")
                    details.append(f"reverse: dst=0x{dst_ip_back:08x} type={typ2} id={id2:#x}")
                    in_ok = (dst_ip_back == cave_ip
                             and typ2 == 0
                             and id2 == orig_id
                             and back[0:6] == bytes(kali_mac))
                    details.append("inbound ICMP reverse OK" if in_ok else "inbound ICMP MISMATCH")
                    if out_ok and in_ok: verdict = "PASS"

        stats = run_cmd(c, "nat-stats")
        details.append(stats.strip())
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
    for d in details:
        for line in d.splitlines(): print("  " + line[:160])
    print(f"\nLog: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict == "PASS" else 1

if __name__ == "__main__":
    sys.exit(main())
