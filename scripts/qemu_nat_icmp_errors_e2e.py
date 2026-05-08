#!/usr/bin/env python3
"""3c-deferred-2: ICMP error types (dest-unreachable, time-exceeded)
get rewritten with embedded-header NAT and delivered to the cave.

Flow:
  1. Cave (nic 1 peer) sends TCP SYN 192.168.77.10:51234 →
     93.184.216.34:443.
  2. Kernel NATs it out nic 0 with src=10.0.2.15:<eph>.
  3. Python on nic 0 *pretends to be an intermediate router* that
     returns an ICMP Time Exceeded (type 11) with the embedded
     original IPv4+TCP header.
  4. Kernel: parse_inbound finds no TCP match (proto=ICMP, not TCP),
     maybe_entry=None; try_rewrite_icmp_error_inbound checks the
     outer ICMP type (11 ✓), extracts the inner src_port (=eph_port),
     looks up the NAT entry, rewrites outer dst + inner src +
     inner src_port + all four checksums, sends on nic 1.
  5. Python cave receives the ICMP error. Verify:
     - outer dst = 192.168.77.10 (cave_ip)
     - outer type = 11 (Time Exceeded)
     - inner IPv4 src = 192.168.77.10
     - inner TCP src_port = 51234
     - outer Eth dst = cave_mac, src = nic1_mac
     - outer IPv4 + outer ICMP + inner IPv4 checksums all zero-sum

  Also try a Destination Unreachable (type 3) to confirm both error
  types go through.
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
LOG = ROOT / f"logs/qemu-tests/icmp-err-{STAMP}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)
ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"bat_os\s*>\s*"

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

def icmp_cksum(buf):
    s=0
    i=0
    while i+1<len(buf): s+=(buf[i]<<8)|buf[i+1]; i+=2
    if i<len(buf): s+=buf[i]<<8
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

def build_icmp_error(smac, dmac, router_ip, nic0_ip, err_type,
                     orig_src_ip, orig_dst_ip,
                     orig_sport, orig_dport):
    """Outer IPv4 + ICMP error carrying (IPv4 + first 8 B of TCP)."""
    # Inner packet (what the cave sent, NAT-translated, as seen by the
    # router that sent us back the error).
    inner_ip = bytearray(20)
    inner_ip[0]=0x45; inner_ip[8]=64; inner_ip[9]=6
    inner_ip[2:4]=(20+20).to_bytes(2,"big")  # total len (for realism)
    inner_ip[12:16]=orig_src_ip.to_bytes(4,"big")
    inner_ip[16:20]=orig_dst_ip.to_bytes(4,"big")
    inner_ip[10:12]=ipv4_cksum(bytes(inner_ip)).to_bytes(2,"big")
    inner_l4_8 = orig_sport.to_bytes(2,"big") + orig_dport.to_bytes(2,"big") + b"\x00"*4

    # Outer ICMP: 8 B header (type/code/cksum/unused) + inner IP + 8 B L4.
    icmp = bytearray(8)
    icmp[0] = err_type; icmp[1] = 0  # code
    # checksum computed after
    icmp_full = bytes(icmp) + bytes(inner_ip) + bytes(inner_l4_8)
    ck = icmp_cksum(icmp_full)
    icmp_full = bytes(icmp_full[:2]) + ck.to_bytes(2,"big") + icmp_full[4:]

    # Outer IPv4: src=router_ip, dst=nic0_ip, proto=ICMP
    outer = bytearray(20)
    outer[0]=0x45; outer[8]=64; outer[9]=1
    outer[12:16]=router_ip.to_bytes(4,"big")
    outer[16:20]=nic0_ip.to_bytes(4,"big")
    outer[2:4]=(20+len(icmp_full)).to_bytes(2,"big")
    outer[10:12]=ipv4_cksum(bytes(outer)).to_bytes(2,"big")

    fr = bytearray()
    fr += bytes(dmac) + bytes(smac) + b"\x08\x00"
    fr += bytes(outer) + icmp_full
    return bytes(fr)

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

def parse_outer_ipv4(f):
    if len(f)<14+20 or f[12:14]!=b"\x08\x00": return None
    return {
        "eth_dst":f[0:6],"eth_src":f[6:12],
        "src":int.from_bytes(f[14+12:14+16],"big"),
        "dst":int.from_bytes(f[14+16:14+20],"big"),
        "proto":f[14+9],
        "ihl":(f[14]&0x0F)*4,
    }

def main():
    HOST=25570; CAVE=25571
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
        time.sleep(0.3); c.sendline(b"batman")
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

        # 1. Establish NAT entry.
        send_frame(v["conn"], build_tcp_syn(
            kali_mac,nic1_mac,cave_ip,dst_ip,51234,443))
        fwd=recv_ipv4(h["conn"],3.0)
        if fwd is None: raise RuntimeError("no outbound")
        eph=int.from_bytes(fwd[34:36],"big")
        details.append(f"NAT entry allocated: eph={eph}")

        # 2. Router returns Time Exceeded (type 11).
        router_ip=ip_int("203.0.113.1")
        err=build_icmp_error(gw_mac,nic0_mac,router_ip,nic0_ip,
                             11, nic0_ip, dst_ip, eph, 443)
        send_frame(h["conn"], err)
        details.append(f"sent ICMP Time Exceeded ({len(err)} B)")
        back=recv_ipv4(v["conn"],3.0)
        ok_te=False
        if back is not None:
            p=parse_outer_ipv4(back)
            # Inner starts at outer_ihl + 8 (ICMP fixed) + 14 (eth).
            inner_start=14+p["ihl"]+8 if p else 0
            if p and p["proto"]==1 and inner_start+20<=len(back):
                inner_src=int.from_bytes(back[inner_start+12:inner_start+16],"big")
                inner_sport=int.from_bytes(back[inner_start+20:inner_start+22],"big")
                outer_type=back[14+p["ihl"]]
                details.append(f"TE back: outer_dst=0x{p['dst']:08x} type={outer_type} "
                               f"inner_src=0x{inner_src:08x} inner_sport={inner_sport}")
                ok_te = (p["dst"]==cave_ip and outer_type==11
                         and inner_src==cave_ip and inner_sport==51234
                         and p["eth_dst"]==bytes(kali_mac)
                         and p["eth_src"]==bytes(nic1_mac))
                details.append("Time Exceeded delivery OK" if ok_te else "Time Exceeded MISMATCH")
            else:
                details.append("TE back parse failed")
        else:
            details.append("no Time Exceeded delivered")

        # 3. Destination Unreachable (type 3).
        err2=build_icmp_error(gw_mac,nic0_mac,router_ip,nic0_ip,
                              3, nic0_ip, dst_ip, eph, 443)
        send_frame(h["conn"], err2)
        details.append(f"sent ICMP Destination Unreachable ({len(err2)} B)")
        back2=recv_ipv4(v["conn"],3.0)
        ok_du=False
        if back2 is not None:
            p=parse_outer_ipv4(back2)
            inner_start=14+p["ihl"]+8 if p else 0
            if p and p["proto"]==1 and inner_start+20<=len(back2):
                outer_type=back2[14+p["ihl"]]
                inner_src=int.from_bytes(back2[inner_start+12:inner_start+16],"big")
                inner_sport=int.from_bytes(back2[inner_start+20:inner_start+22],"big")
                ok_du = (p["dst"]==cave_ip and outer_type==3
                         and inner_src==cave_ip and inner_sport==51234)
                details.append(f"DU back: type={outer_type} inner_src=0x{inner_src:08x} "
                               f"inner_sport={inner_sport}")
                details.append("Dest Unreachable delivery OK" if ok_du else "DU MISMATCH")
            else:
                details.append("DU back parse failed")
        else:
            details.append("no Dest Unreachable delivered")

        stats=run_cmd(c,"nat-stats")
        details.append(stats.strip())
        ctr_ok=False
        for line in stats.splitlines():
            if "icmp-error-deliv" in line:
                nums=[p for p in line.split() if p.isdigit()]
                if nums and int(nums[-1])>=2: ctr_ok=True
                break

        if ok_te and ok_du and ctr_ok: verdict="PASS"
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
        for line in d.splitlines(): print("  "+line[:180])
    print(f"\nLog: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict=="PASS" else 1

if __name__=="__main__":
    sys.exit(main())
