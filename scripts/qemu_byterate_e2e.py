#!/usr/bin/env python3
"""Byte-rate shaper: catch volume attacks that evade pps limits.

Scenario: attacker has already been pps-rate-limited but uses big
packets to evade it. Policy: 100 pps / burst 200 (generous) AND
4000 Bps / byte_burst 8000 (tight).

Attacker sends 20 large TCP frames (~1500 B each) at the allowed
dst. pps budget is fine (20 < 200) but bytes budget (~30000) is
way over 8000 burst.

Expected: a few get through (whatever fits in the byte burst),
rest get DropRate. drop-policy stays 0.
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
LOG = ROOT / f"logs/qemu-tests/byterate-{STAMP}.log"
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

def build_tcp(smac,dmac,sip,dip,sport,dport,flags=0x18,payload=b""):
    fr=bytearray(); fr+=bytes(dmac)+bytes(smac)+b"\x08\x00"
    ip=bytearray(20); ip[0]=0x45; ip[8]=64; ip[9]=6
    ip[12:16]=sip.to_bytes(4,"big"); ip[16:20]=dip.to_bytes(4,"big")
    tcp=bytearray(20)
    tcp[0:2]=sport.to_bytes(2,"big"); tcp[2:4]=dport.to_bytes(2,"big")
    tcp[4:8]=(1).to_bytes(4,"big")
    tcp[12]=5<<4; tcp[13]=flags; tcp[14:16]=(8192).to_bytes(2,"big")
    tcp_full=bytes(tcp)+payload
    ip[2:4]=(20+len(tcp_full)).to_bytes(2,"big")
    ip[10:12]=ipv4_cksum(bytes(ip)).to_bytes(2,"big")
    tcp_full=bytes(tcp_full[:16])+l4_cksum(sip,dip,6,tcp_full).to_bytes(2,"big")+tcp_full[18:]
    return bytes(fr+ip+tcp_full)

def ip_int(s): a,b,c,d=[int(p) for p in s.split(".")]; return (a<<24)|(b<<16)|(c<<8)|d
def send_frame(c,f): c.sendall(struct.pack(">I",len(f))+f)

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
    HOST=25606; CAVE=25607
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
        # Generous pps; tight bps.
        run_cmd(c,"cpol-rate kali 100 200")
        run_cmd(c,"cpol-byte-rate kali 4000 8000")
        details.append("shaper: pps=100/200 + bps=4000/8000")

        kali_mac=[0x02,0xAA,0,0,0,0x10]
        nic1_mac=[0x52,0x54,0,0x12,0x34,0x57]
        cave_ip=ip_int("192.168.77.10")
        dst_ip=ip_int("93.184.216.34")

        # 20 TCP segments of 1400 B payload each (~1454 B on wire).
        # Total = ~29 KB  >> 8 KB byte burst.
        FRAMES = 20
        for i in range(FRAMES):
            payload = bytes([0x41] * 1400)
            send_frame(v["conn"], build_tcp(
                kali_mac, nic1_mac, cave_ip, dst_ip,
                60000 + i, 443, flags=0x18, payload=payload))
        time.sleep(1.5)

        stats = run_cmd(c,"nat-stats")
        allow       = parse_counter(stats, "allow:")
        drop_policy = parse_counter(stats, "drop-policy")
        drop_rate   = parse_counter(stats, "drop-rate")
        details.append(f"kernel counters: allow={allow} drop-policy={drop_policy} drop-rate={drop_rate}")

        # Expected: allow <= 7 (8192 burst / ~1454B each ≈ 5-6 frames),
        # drop-rate >= 10, drop-policy == 0.
        ok = (drop_rate >= 10 and allow <= 10 and drop_policy == 0
              and allow + drop_rate + drop_policy >= FRAMES - 2)
        details.append(f"byte shaper enforced: {ok}")
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
