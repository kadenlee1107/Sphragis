#!/usr/bin/env python3
"""Wire-level SNI enforcement E2E.

Cave has: cpol-add-sni kali 93.184.216.34 443 example.com
  (allow TCP to that IP:port but ONLY with TLS SNI=example.com)

Then two attack probes + one legit probe:
  A. SYN to 93.184.216.34:443 (no payload) — allowed; ClientHello
     hasn't arrived yet so we give handshake a chance.
  B. TCP segment with ClientHello(SNI=attacker.com) to the same dst.
     → DropSni (NOT DropPolicy — destination matched, SNI wrong).
  C. TCP segment with ClientHello(SNI=example.com) → Allow.

PASS iff:
  drop-sni   >= 1  (from attack B)
  allow      >= 2  (from A and C)
  drop-policy == 0 (no off-allowlist traffic)
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
LOG = ROOT / f"logs/qemu-tests/sni-e2e-{STAMP}.log"
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

def ip_int(s): a,b,c,d=[int(p) for p in s.split(".")]; return (a<<24)|(b<<16)|(c<<8)|d
def send_frame(c,f): c.sendall(struct.pack(">I",len(f))+f)

def build_tls_client_hello(sni_host: str) -> bytes:
    sni = sni_host.encode("ascii")
    # server_name extension body
    list_len = 1 + 2 + len(sni)
    sn_ext = (bytes([(list_len>>8)&0xFF, list_len&0xFF])
              + b"\x00"
              + bytes([(len(sni)>>8)&0xFF, len(sni)&0xFF])
              + sni)
    ext = (b"\x00\x00"
           + bytes([(len(sn_ext)>>8)&0xFF, len(sn_ext)&0xFF])
           + sn_ext)

    ch = (b"\x03\x03"                        # version
          + b"\x00"*32                       # random
          + b"\x00"                          # session_id_len
          + b"\x00\x02\xc0\x2c"              # cs_len=2 + one suite
          + b"\x01\x00"                      # cm_len=1 + null
          + bytes([(len(ext)>>8)&0xFF, len(ext)&0xFF])
          + ext)
    hs = (b"\x01"                            # ClientHello
          + bytes([(len(ch)>>16)&0xFF, (len(ch)>>8)&0xFF, len(ch)&0xFF])
          + ch)
    record = (b"\x16\x03\x01"
              + bytes([(len(hs)>>8)&0xFF, len(hs)&0xFF])
              + hs)
    return record

def build_tcp(smac,dmac,sip,dip,sport,dport,flags=0x02,seq=0,ack=0,payload=b""):
    fr=bytearray(); fr+=bytes(dmac)+bytes(smac)+b"\x08\x00"
    ip=bytearray(20); ip[0]=0x45; ip[8]=64; ip[9]=6
    ip[12:16]=sip.to_bytes(4,"big"); ip[16:20]=dip.to_bytes(4,"big")
    tcp=bytearray(20)
    tcp[0:2]=sport.to_bytes(2,"big"); tcp[2:4]=dport.to_bytes(2,"big")
    tcp[4:8]=seq.to_bytes(4,"big"); tcp[8:12]=ack.to_bytes(4,"big")
    tcp[12]=5<<4; tcp[13]=flags; tcp[14:16]=(8192).to_bytes(2,"big")
    tcp_full=bytes(tcp)+payload
    ip[2:4]=(20+len(tcp_full)).to_bytes(2,"big")
    ip[10:12]=ipv4_cksum(bytes(ip)).to_bytes(2,"big")
    tcp_full=bytes(tcp_full[:16])+l4_cksum(sip,dip,6,tcp_full).to_bytes(2,"big")+tcp_full[18:]
    return bytes(fr+ip+tcp_full)

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

def parse_counter(stats, key):
    for line in stats.splitlines():
        if key in line:
            nums=[p for p in line.split() if p.isdigit()]
            if nums: return int(nums[-1])
    return 0

def main():
    HOST=25604; CAVE=25605
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
        run_cmd(c,"cpol-add-sni kali 93.184.216.34 443 example.com")
        details.append("rule: allow 93.184.216.34:443/tcp pinned SNI=example.com")

        kali_mac=[0x02,0xAA,0,0,0,0x10]
        nic1_mac=[0x52,0x54,0,0x12,0x34,0x57]
        cave_ip=ip_int("192.168.77.10")
        dst_ip=ip_int("93.184.216.34")

        # A. SYN — no payload, handshake phase.
        send_frame(v["conn"], build_tcp(
            kali_mac, nic1_mac, cave_ip, dst_ip,
            51234, 443, flags=0x02))
        time.sleep(0.3)

        # B. ClientHello with attacker SNI.
        ch_bad = build_tls_client_hello("attacker.com")
        send_frame(v["conn"], build_tcp(
            kali_mac, nic1_mac, cave_ip, dst_ip,
            51234, 443, flags=0x18, seq=1, ack=0x12345679,
            payload=ch_bad))
        time.sleep(0.3)

        # C. ClientHello with legit SNI.
        ch_good = build_tls_client_hello("example.com")
        send_frame(v["conn"], build_tcp(
            kali_mac, nic1_mac, cave_ip, dst_ip,
            51235, 443, flags=0x18, seq=1, ack=0x12345679,
            payload=ch_good))
        time.sleep(0.5)

        stats=run_cmd(c,"nat-stats")
        details.append(stats.strip())
        allow       = parse_counter(stats, "allow:")
        drop_policy = parse_counter(stats, "drop-policy")
        drop_sni    = parse_counter(stats, "drop-sni")
        details.append(f"allow={allow} drop-policy={drop_policy} drop-sni={drop_sni}")

        ok = (drop_sni >= 1
              and allow >= 2           # SYN + good ClientHello
              and drop_policy == 0)
        if ok: verdict = "PASS"
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
