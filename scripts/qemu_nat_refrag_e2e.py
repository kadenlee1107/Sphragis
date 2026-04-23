#!/usr/bin/env python3
"""3c-deferred-6: egress re-fragmentation when reassembled > MTU.

Setup:
  Cave sends a fragmented IPv4 TCP SYN whose REASSEMBLED size is
  ~1900 B — bigger than nic 0's 1500 B MTU after rewrite.

  Each cave-side fragment is small (~1000 B, fits the cave wire).
  After reassembly + NAT rewrite, the kernel notices the resulting
  datagram > MTU and re-fragments before transmitting on nic 0.

Expected on nic 0:
  Two fragments with the same IP id, MF=1+0, payload 1480 + ~440.
  Both have valid IPv4 checksums and src=10.0.2.15 (NAT-rewritten).

PASS iff:
  - At least 2 IPv4 frames observed on nic 0
  - First frame: MF=1, offset=0
  - Second frame: MF=0, offset=1480/8=185
  - Both frames' src IP is 10.0.2.15 (NAT)
  - Total payload bytes across all fragments == original 1880 B
  - frag-refragd counter ≥ 1
"""
import pexpect, re, socket, struct, subprocess, sys, threading, time
from pathlib import Path
from datetime import datetime

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
STAMP = datetime.now().strftime('%Y%m%d-%H%M%S')
LOG = ROOT / f"logs/qemu-tests/refrag-{STAMP}.log"
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

def build_fragment(src_mac, dst_mac, src_ip, dst_ip, ip_id,
                   frag_offset_bytes, more_fragments, payload_bytes,
                   proto=6):
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
    HOST=25576; CAVE=25577
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

        # Build a 1900-byte L4 (TCP header + 1880 B data). After
        # reassembly the IPv4 datagram is 1920 B (>1500 MTU).
        tcp_hdr=bytearray(20)
        tcp_hdr[0:2]=(51234).to_bytes(2,"big")
        tcp_hdr[2:4]=(443).to_bytes(2,"big")
        tcp_hdr[4:8]=(0x1).to_bytes(4,"big")
        tcp_hdr[12]=5<<4
        tcp_hdr[13]=0x02
        tcp_hdr[14:16]=(8192).to_bytes(2,"big")
        app_data=bytes([0x43]*1880)
        full_l4=bytes(tcp_hdr)+app_data
        ck=l4_cksum(cave_ip,dst_ip,6,full_l4)
        full_l4=bytes(full_l4[:16])+ck.to_bytes(2,"big")+full_l4[18:]

        # Split at offset 1000 (multiple of 8).
        split=1000
        part1=full_l4[:split]
        part2=full_l4[split:]
        f1=build_fragment(kali_mac, nic1_mac, cave_ip, dst_ip,
                          ip_id=0xDEAD, frag_offset_bytes=0,
                          more_fragments=True, payload_bytes=part1)
        f2=build_fragment(kali_mac, nic1_mac, cave_ip, dst_ip,
                          ip_id=0xDEAD, frag_offset_bytes=split,
                          more_fragments=False, payload_bytes=part2)
        details.append(f"sending frags: {len(f1)} + {len(f2)} = "
                       f"{len(f1)+len(f2)} B (reassembled L4 = {len(full_l4)})")

        send_frame(v["conn"], f1)
        time.sleep(0.3)
        send_frame(v["conn"], f2)
        time.sleep(0.7)

        # Drain ALL frames seen on nic 0.
        seen=[]
        for _ in range(8):
            f=recv_frame(h["conn"], 1.0)
            if f is None: break
            if len(f)>=14+20 and f[12:14]==b"\x08\x00":
                seen.append(f)
        details.append(f"received {len(seen)} IPv4 frame(s) on nic 0")

        # Validate the fragmentation.
        ok=False
        if len(seen)>=2:
            ids=set()
            mf_offsets=[]
            total_payload=0
            srcs=set()
            for f in seen:
                ihl=(f[14]&0x0F)*4
                tl=int.from_bytes(f[14+2:14+4],"big")
                payload=tl-ihl
                ids.add(int.from_bytes(f[14+4:14+6],"big"))
                ff=int.from_bytes(f[14+6:14+8],"big")
                mf=(ff&0x2000)!=0
                offset=(ff&0x1FFF)*8
                mf_offsets.append((mf,offset,payload))
                srcs.add(int.from_bytes(f[14+12:14+16],"big"))
                total_payload += payload
            details.append(f"  ids={ids}  shapes={mf_offsets}  srcs={[hex(s) for s in srcs]}  total_payload={total_payload}")
            # Same id, last has MF=0, all have nic 0 src.
            ok = (len(ids)==1
                  and any(mf for mf,_,_ in mf_offsets)
                  and any(not mf for mf,_,_ in mf_offsets)
                  and srcs == {ip_int("10.0.2.15")}
                  and total_payload == 1900)
            details.append("re-fragmented + NAT'd correctly" if ok else "MISMATCH")

        stats=run_cmd(c,"nat-stats")
        details.append(stats.strip())
        refrag_ok=False
        for line in stats.splitlines():
            if "frag-refragd" in line:
                nums=[p for p in line.split() if p.isdigit()]
                if nums and int(nums[-1])>=1: refrag_ok=True
                break
        if ok and refrag_ok: verdict="PASS"
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
        for line in d.splitlines(): print("  "+line[:240])
    print(f"\nLog: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict=="PASS" else 1

if __name__=="__main__":
    sys.exit(main())
