#!/usr/bin/env python3
"""3c-deferred-1: prove pump_replies doesn't eat kernel control-plane.

Before the fix: once any cave flow populated the NAT table,
`pump_replies` consumed EVERY inbound nic-0 frame — including ones
that should've been dispatched to the kernel's IP stack — and
silently dropped the non-NAT ones.

This test takes full control of BOTH NICs via `-netdev socket` so
it can inject a non-NAT frame on nic 0 and verify the kernel's
host-frames-pass counter went up (i.e. the rescue path fired).

Flow:
  1. Python cave (nic 1) sends TCP SYN → establishes a NAT entry.
  2. Python internet (nic 0) sends an UNRELATED TCP SYN whose ports
     don't match any NAT entry.
  3. Expect: pump_replies pulls the frame, nat_lookup_in returns None,
     falls through to `net::dispatch_host_frame`, counter bumps.
  4. `nat-stats`: host-frames-pass >= 1.
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
LOG = ROOT / f"logs/qemu-tests/host-passthru-{STAMP}.log"
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

def build_tcp(smac,dmac,sip,dip,sport,dport,flags=0x02,seq=0,ack=0):
    fr=bytearray()
    fr+=bytes(dmac)+bytes(smac)+b"\x08\x00"
    ip=bytearray(20); ip[0]=0x45; ip[8]=64; ip[9]=6
    ip[12:16]=sip.to_bytes(4,"big"); ip[16:20]=dip.to_bytes(4,"big")
    tcp=bytearray(20)
    tcp[0:2]=sport.to_bytes(2,"big"); tcp[2:4]=dport.to_bytes(2,"big")
    tcp[4:8]=seq.to_bytes(4,"big"); tcp[8:12]=ack.to_bytes(4,"big")
    tcp[12]=5<<4; tcp[13]=flags
    tcp[14:16]=(8192).to_bytes(2,"big")
    ip[2:4]=(20+len(tcp)).to_bytes(2,"big")
    ip[10:12]=ipv4_cksum(bytes(ip)).to_bytes(2,"big")
    tcp[16:18]=l4_cksum(sip,dip,6,bytes(tcp)).to_bytes(2,"big")
    return bytes(fr+ip+tcp)

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

def main():
    HOST=25568; CAVE=25569
    h=listener(HOST); v=listener(CAVE)
    daemon=subprocess.Popen(["python3",str(ROOT/"scripts"/"batcaved.py")],
        stdout=subprocess.DEVNULL,stderr=subprocess.STDOUT)
    for _ in range(40):
        try: socket.create_connection(("127.0.0.1",9999),timeout=0.3).close(); break
        except OSError: time.sleep(0.2)

    args=["qemu-system-aarch64","-machine","virt","-cpu","max","-m","2G",
          "-display","none",
          "-device","virtio-gpu-device","-device","virtio-keyboard-device",
          # Both NICs as socket peers so Python owns both wires.
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
        if not (h["conn"] and v["conn"]):
            raise RuntimeError("QEMU sockets didn't both connect")

        run_cmd(c,"nat-reset")
        run_cmd(c,"nat-bind 192.168.77.10 kali")
        run_cmd(c,"cpol-add kali 93.184.216.34 443 tcp")

        # 1. Establish a NAT entry via one cave flow.
        kali_mac=[0x02,0xAA,0,0,0,0x10]
        nic1_mac=[0x52,0x54,0,0x12,0x34,0x57]
        out=build_tcp(kali_mac,nic1_mac,
                      ip_int("192.168.77.10"),ip_int("93.184.216.34"),
                      51234,443,flags=0x02)
        send_frame(v["conn"],out)
        time.sleep(0.4)
        table=run_cmd(c,"nat-table")
        details.append(table.strip())

        # 2. Inject a non-NAT frame on nic 0: TCP SYN targeting a port
        #    that's DEFINITELY not in the NAT table (12345 is below our
        #    50000 ephemeral base). pump_replies must NOT eat this.
        nic0_mac=[0x52,0x54,0,0x12,0x34,0x56]
        gw_mac  =[0x52,0x55,0x0A,0x00,0x02,0x02]
        stray=build_tcp(gw_mac,nic0_mac,
                        ip_int("1.2.3.4"),ip_int("10.0.2.15"),
                        9999,12345,flags=0x02)
        send_frame(h["conn"],stray)
        details.append(f"injected stray SYN to :12345 on nic 0 ({len(stray)} B)")
        time.sleep(0.5)

        # 3. Also inject another stray so we have a clean counter delta.
        stray2=build_tcp(gw_mac,nic0_mac,
                         ip_int("1.2.3.4"),ip_int("10.0.2.15"),
                         9999,54321,flags=0x02)
        send_frame(h["conn"],stray2)
        time.sleep(0.5)

        stats=run_cmd(c,"nat-stats")
        details.append(stats.strip())
        host_pass=0
        for line in stats.splitlines():
            if "host-frames-pass" in line:
                parts=[p for p in line.split() if p.isdigit()]
                if parts: host_pass=int(parts[-1])
                break

        table_ok=("active: 1" in table or "active:         1" in table)
        host_ok=(host_pass >= 2)
        details.append(f"table_ok={table_ok}  host-frames-pass={host_pass}  host_ok={host_ok}")
        if table_ok and host_ok: verdict="PASS"
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
