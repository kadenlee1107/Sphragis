#!/usr/bin/env python3
"""3c-deferred-3: IPv4 fragment reassembly + NAT forwarding.

Cave sends two IPv4 fragments of a 100-byte-payload TCP SYN:
  frag 1: offset=0,   MF=1   (IPv4 + TCP header + first 72 B payload)
  frag 2: offset=92,  MF=0   (remaining 28 B payload)

Reassembled datagram = 20 B IPv4 + 20 B TCP + 100 B payload = 140 B.

Kernel path:
  1. pump_and_forward on nic 1 sees fragment 1 (MF=1). frag_accept
     buffers it, returns None. Kernel keeps draining.
  2. Fragment 2 arrives, frag_accept buffers, detects completion,
     returns a fresh Ethernet+IPv4 frame with MF=0 / offset=0 and
     a valid IPv4 checksum.
  3. Kernel classifies that frame (allowed: kali → 93.184.216.34:443),
     allocates a NAT entry, rewrites src/port/MAC/checksums, sends
     on nic 0.

Python nic 0 peer MUST see ONE unfragmented packet with:
  - IPv4 total_len = 140 (20+20+100)
  - MF=0, offset=0
  - src = 10.0.2.15:eph (NAT-rewritten)
  - dst = 93.184.216.34:443 unchanged
  - Valid IPv4 + TCP checksums

Also verifies `nat-stats` frag-reassembled >= 1.
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
LOG = ROOT / f"logs/qemu-tests/frag-reasm-{STAMP}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)
ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"sphragis\s*>\s*"

def ipv4_cksum(hdr):
    s=0
    for i in range(0,len(hdr),2):
        w=0 if i==10 else (hdr[i]<<8)|(hdr[i+1] if i+1<len(hdr) else 0)
        s+=w
    while s>>16: s=(s&0xFFFF)+(s>>16)
    return (~s)&0xFFFF

def l4_cksum(sip,dip,proto,l4):
    s=(sip>>16)+(sip&0xFFFF)+(dip>>16)+(dip&0xFFFF)+proto+len(l4)
    i=0
    while i+1<len(l4): s+=(l4[i]<<8)|l4[i+1]; i+=2
    if i<len(l4): s+=l4[i]<<8
    while s>>16: s=(s&0xFFFF)+(s>>16)
    return (~s)&0xFFFF

def ip_int(s): a,b,c,d=[int(p) for p in s.split(".")]; return (a<<24)|(b<<16)|(c<<8)|d
def send_frame(c,f): c.sendall(struct.pack(">I",len(f))+f)

def recv_frame(c, timeout):
    c.settimeout(timeout)
    try:
        buf=b""
        while len(buf)<4:
            chunk=c.recv(4-len(buf))
            if not chunk: return None
            buf+=chunk
        n=struct.unpack(">I",buf)[0]
        if n>65536: return None
        data=b""
        while len(data)<n:
            chunk=c.recv(n-len(data))
            if not chunk: return None
            data+=chunk
        return data
    except (TimeoutError, socket.timeout): return None

def recv_ipv4(c, timeout):
    dl=time.time()+timeout
    while time.time()<dl:
        left=max(0.1,dl-time.time())
        f=recv_frame(c,left)
        if f is None: return None
        if len(f)>=14+20 and f[12:14]==b"\x08\x00": return f

def listener(port):
    srv=socket.socket(); srv.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1)
    srv.bind(("127.0.0.1",port)); srv.listen(1)
    state={"conn":None,"srv":srv}
    def loop(): c,_=srv.accept(); state["conn"]=c
    threading.Thread(target=loop,daemon=True).start()
    return state

def run_cmd(c,cmd,timeout=10):
    c.sendline(cmd.encode())
    c.expect(PROMPT,timeout=timeout)
    return ANSI.sub(b"",c.before or b"").decode("utf-8","replace")

def build_fragment(src_mac, dst_mac, src_ip, dst_ip, ip_id,
                   frag_offset_bytes, more_fragments, payload_bytes):
    """Build ONE Ethernet+IPv4 fragment carrying `payload_bytes` as
    the IP payload (from offset `frag_offset_bytes` into the original
    datagram). No L4-header special handling — caller passes the
    exact bytes that should be at that offset."""
    fr=bytearray()
    fr+=bytes(dst_mac)+bytes(src_mac)+b"\x08\x00"
    ip=bytearray(20)
    ip[0]=0x45
    total_len = 20 + len(payload_bytes)
    ip[2:4]=total_len.to_bytes(2,"big")
    ip[4:6]=ip_id.to_bytes(2,"big")
    frag_field = ((0x2000 if more_fragments else 0) | (frag_offset_bytes // 8))
    ip[6]=(frag_field >> 8) & 0xFF
    ip[7]=frag_field & 0xFF
    ip[8]=64; ip[9]=6  # proto = TCP
    ip[12:16]=src_ip.to_bytes(4,"big"); ip[16:20]=dst_ip.to_bytes(4,"big")
    ip[10:12]=ipv4_cksum(bytes(ip)).to_bytes(2,"big")
    fr += bytes(ip) + bytes(payload_bytes)
    return bytes(fr)

def main():
    HOST=25572; CAVE=25573
    h=listener(HOST); v=listener(CAVE)
    daemon=subprocess.Popen(["python3",str(ROOT/"scripts"/"batcaved.py")],
        stdout=subprocess.DEVNULL,stderr=subprocess.STDOUT)
    for _ in range(40):
        try: socket.create_connection(("127.0.0.1",9999),timeout=0.3).close(); break
        except OSError: time.sleep(0.2)

    args=["qemu-system-aarch64","-machine","virt","-cpu","max","-m","2G",
          "-display","none",
          "-device","virtio-gpu-device","-device","virtio-keyboard-device",
          "-netdev",f"socket,id=hostnet,connect=127.0.0.1:{HOST}",
          "-device","virtio-net-device,netdev=hostnet",
          "-netdev",f"socket,id=cavenet,connect=127.0.0.1:{CAVE}",
          "-device","virtio-net-device,netdev=cavenet",
          "-serial","mon:stdio","-kernel",str(KERNEL)]
    fp=open(LOG,"wb")
    c=pexpect.spawn(args[0],args[1:],timeout=90,logfile=fp,encoding=None)
    verdict="FAIL"; details=[]
    try:
        c.expect(rb"\[bs\] flush done .+ entering input loop",timeout=60)
        time.sleep(0.3); c.sendline(b"sphragis-dev")
        c.expect(PROMPT,timeout=60)
        for _ in range(60):
            if h["conn"] and v["conn"]: break
            time.sleep(0.2)
        if not (h["conn"] and v["conn"]): raise RuntimeError("sockets")

        run_cmd(c,"nat-reset")
        run_cmd(c,"nat-bind 192.168.77.10 kali")
        run_cmd(c,"cpol-add kali 93.184.216.34 443 tcp")

        kali_mac=[0x02,0xAA,0,0,0,0x10]
        nic1_mac=[0x52,0x54,0,0x12,0x34,0x57]
        nic0_mac=[0x52,0x54,0,0x12,0x34,0x56]
        gw_mac  =[0x52,0x55,0x0A,0x00,0x02,0x02]
        cave_ip=ip_int("192.168.77.10"); dst_ip=ip_int("93.184.216.34")

        # Build the full L4 payload we want reassembled.
        tcp_hdr = bytearray(20)
        tcp_hdr[0:2] = (51234).to_bytes(2,"big")
        tcp_hdr[2:4] = (443).to_bytes(2,"big")
        tcp_hdr[4:8] = (0x1).to_bytes(4,"big")
        tcp_hdr[12]  = 5 << 4
        tcp_hdr[13]  = 0x02  # SYN
        tcp_hdr[14:16] = (8192).to_bytes(2,"big")
        app_data = bytes([0x41] * 100)  # 100 B of "A"
        full_l4 = bytes(tcp_hdr) + app_data
        tcp_ck = l4_cksum(cave_ip, dst_ip, 6, full_l4)
        # Patch TCP checksum
        full_l4 = bytes(full_l4[:16]) + tcp_ck.to_bytes(2,"big") + full_l4[18:]

        # Split at offset 72 (must be multiple of 8). First fragment carries
        # bytes 0..72 of L4 (which includes the 20 B TCP header + 52 B data).
        # Second fragment carries bytes 72..120.
        split = 72
        part1 = full_l4[:split]
        part2 = full_l4[split:]
        assert split % 8 == 0, "frag offset must be 8-aligned"

        f1 = build_fragment(kali_mac, nic1_mac, cave_ip, dst_ip,
                            ip_id=0xBEEF, frag_offset_bytes=0,
                            more_fragments=True, payload_bytes=part1)
        f2 = build_fragment(kali_mac, nic1_mac, cave_ip, dst_ip,
                            ip_id=0xBEEF, frag_offset_bytes=split,
                            more_fragments=False, payload_bytes=part2)
        details.append(f"frag1={len(f1)} B (offset 0, MF=1); frag2={len(f2)} B (offset {split}, MF=0)")

        send_frame(v["conn"], f1)
        time.sleep(0.3)
        send_frame(v["conn"], f2)
        time.sleep(0.5)

        out = recv_ipv4(h["conn"], 3.0)
        if out is None:
            details.append("no outbound IPv4 frame observed")
        else:
            # Validate shape of reassembled+NAT'd packet.
            ihl = (out[14] & 0x0F) * 4
            total_len = int.from_bytes(out[14+2:14+4],"big")
            frag_field = int.from_bytes(out[14+6:14+8],"big")
            mf = (frag_field & 0x2000) != 0
            offset = (frag_field & 0x1FFF) * 8
            src_ip = int.from_bytes(out[14+12:14+16],"big")
            dst_ip_out = int.from_bytes(out[14+16:14+20],"big")
            sport = int.from_bytes(out[14+ihl:14+ihl+2],"big")
            dport = int.from_bytes(out[14+ihl+2:14+ihl+4],"big")
            details.append(f"out: len={len(out)} total_len={total_len} MF={mf} offset={offset} "
                           f"src=0x{src_ip:08x}:{sport} dst=0x{dst_ip_out:08x}:{dport}")
            ok = (total_len == 140  # 20 + 20 + 100
                  and not mf and offset == 0
                  and src_ip == ip_int("10.0.2.15")
                  and dst_ip_out == dst_ip
                  and dport == 443 and sport >= 50000
                  and out[6:12] == bytes(nic0_mac) and out[0:6] == bytes(gw_mac))
            details.append("reassembled + NAT'd OK" if ok else "reassembled NAT MISMATCH")

            stats = run_cmd(c, "nat-stats")
            details.append(stats.strip())
            frag_ok = False
            for line in stats.splitlines():
                if "frag-reassembled" in line:
                    nums = [p for p in line.split() if p.isdigit()]
                    if nums and int(nums[-1]) >= 1: frag_ok = True
                    break
            if ok and frag_ok: verdict = "PASS"
    except (pexpect.TIMEOUT,RuntimeError) as e:
        details.append(f"error: {e}")
    finally:
        try: c.terminate(force=True)
        except Exception: pass
        fp.close()
        for s in (h,v):
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
        for line in d.splitlines(): print("  "+line[:200])
    print(f"\nLog: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict=="PASS" else 1

if __name__=="__main__":
    sys.exit(main())
