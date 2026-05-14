#!/usr/bin/env python3
"""3c-deferred-5: inbound IPv4 fragment reassembly on nic 0.

Symmetric to the outbound reassembler: if a reply arrives fragmented
from the internet, buffer until complete, then reverse-NAT the
reassembled datagram and deliver on nic 1. Before this fix, fragment
replies were dropped (parse_inbound rejected MF/offset frames).

Flow:
  1. Cave sends TCP SYN → NAT entry with eph_port.
  2. Kernel forwards the outbound SYN out nic 0.
  3. Python internet peer replies with a FRAGMENTED SYN/ACK
     (400-byte payload, split at offset 200 so we get two
     fragments: offset=0 MF=1, offset=200 MF=0).
  4. Kernel reassembles on nic 0, reverse-NATs, delivers to cave
     as one unfragmented frame.
  5. Python cave receives the complete SYN/ACK.

PASS iff:
  - Cave-side peer receives exactly one IPv4 frame (not two).
  - Reassembled frame total_len = 440 (20 + 20 + 400).
  - MF=0, offset=0 (reassembled).
  - dst=192.168.77.10 (cave), dst_port=51234 (cave orig).
  - frag-reassembled counter ≥ 2 (one frag accepted, one completed).
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
LOG = ROOT / f"logs/qemu-tests/frag-in-{STAMP}.log"
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

def build_tcp_syn(smac,dmac,sip,dip,sport,dport):
    fr=bytearray(); fr+=bytes(dmac)+bytes(smac)+b"\x08\x00"
    ip=bytearray(20); ip[0]=0x45; ip[8]=64; ip[9]=6
    ip[12:16]=sip.to_bytes(4,"big"); ip[16:20]=dip.to_bytes(4,"big")
    tcp=bytearray(20)
    tcp[0:2]=sport.to_bytes(2,"big"); tcp[2:4]=dport.to_bytes(2,"big")
    tcp[12]=5<<4; tcp[13]=0x02; tcp[14:16]=(8192).to_bytes(2,"big")
    ip[2:4]=(20+len(tcp)).to_bytes(2,"big")
    ip[10:12]=ipv4_cksum(bytes(ip)).to_bytes(2,"big")
    tcp[16:18]=l4_cksum(sip,dip,6,bytes(tcp)).to_bytes(2,"big")
    return bytes(fr+ip+tcp)

def build_fragment(src_mac, dst_mac, src_ip, dst_ip, ip_id,
                   frag_offset_bytes, more_fragments, payload_bytes,
                   proto=6):
    """Build one Ethernet+IPv4 fragment carrying `payload_bytes`."""
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
    ip[8]=64; ip[9]=proto
    ip[12:16]=src_ip.to_bytes(4,"big"); ip[16:20]=dst_ip.to_bytes(4,"big")
    ip[10:12]=ipv4_cksum(bytes(ip)).to_bytes(2,"big")
    fr += bytes(ip) + bytes(payload_bytes)
    return bytes(fr)

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

def main():
    HOST=25574; CAVE=25575
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
        nic0_ip=ip_int("10.0.2.15")

        # 1. Cave SYN → NAT entry allocated.
        out=build_tcp_syn(kali_mac,nic1_mac,cave_ip,dst_ip,51234,443)
        send_frame(v["conn"], out)
        fwd=recv_ipv4(h["conn"],3.0)
        if fwd is None: raise RuntimeError("no outbound")
        eph=int.from_bytes(fwd[34:36],"big")
        details.append(f"NAT entry allocated: eph={eph}")

        # 2. Build a fragmented SYN/ACK reply: TCP header (20 B) +
        #    400 bytes of data. Payload total = 420 B.
        tcp_hdr=bytearray(20)
        tcp_hdr[0:2]=(443).to_bytes(2,"big")
        tcp_hdr[2:4]=eph.to_bytes(2,"big")
        tcp_hdr[4:8]=(0x12345678).to_bytes(4,"big")
        tcp_hdr[8:12]=(1).to_bytes(4,"big")
        tcp_hdr[12]=5<<4; tcp_hdr[13]=0x12  # SYN/ACK
        tcp_hdr[14:16]=(65535).to_bytes(2,"big")
        app_data=bytes([0x42]*400)
        full_l4=bytes(tcp_hdr)+app_data
        ck=l4_cksum(dst_ip, nic0_ip, 6, full_l4)
        full_l4=bytes(full_l4[:16])+ck.to_bytes(2,"big")+full_l4[18:]

        split=208  # must be multiple of 8; carries TCP hdr + 188 B data
        part1=full_l4[:split]
        part2=full_l4[split:]
        assert split%8==0

        f1=build_fragment(gw_mac, nic0_mac, dst_ip, nic0_ip,
                          ip_id=0xCAFE, frag_offset_bytes=0,
                          more_fragments=True, payload_bytes=part1, proto=6)
        f2=build_fragment(gw_mac, nic0_mac, dst_ip, nic0_ip,
                          ip_id=0xCAFE, frag_offset_bytes=split,
                          more_fragments=False, payload_bytes=part2, proto=6)
        details.append(f"frag1={len(f1)} B (offset 0 MF=1); frag2={len(f2)} B (offset {split} MF=0)")

        send_frame(h["conn"], f1)
        time.sleep(0.3)
        send_frame(h["conn"], f2)
        time.sleep(0.5)

        back=recv_ipv4(v["conn"],3.0)
        if back is None:
            details.append("no frame delivered back on nic 1")
        else:
            ihl=(back[14]&0x0F)*4
            total_len=int.from_bytes(back[14+2:14+4],"big")
            frag_field=int.from_bytes(back[14+6:14+8],"big")
            mf=(frag_field&0x2000)!=0
            offset=(frag_field&0x1FFF)*8
            dst_out=int.from_bytes(back[14+16:14+20],"big")
            dport=int.from_bytes(back[14+ihl+2:14+ihl+4],"big")
            details.append(f"back: len={len(back)} total_len={total_len} MF={mf} "
                           f"offset={offset} dst=0x{dst_out:08x} dport={dport}")
            ok = (total_len == 440  # 20 + 20 + 400
                  and not mf and offset == 0
                  and dst_out == cave_ip
                  and dport == 51234
                  and back[0:6] == bytes(kali_mac))
            details.append("reassembled + reverse-NAT'd OK" if ok else "MISMATCH")

            stats=run_cmd(c,"nat-stats")
            details.append(stats.strip())
            frag_ok=False
            for line in stats.splitlines():
                if "frag-reassembled" in line:
                    nums=[p for p in line.split() if p.isdigit()]
                    if nums and int(nums[-1])>=1: frag_ok=True
                    break
            if ok and frag_ok: verdict="PASS"
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
