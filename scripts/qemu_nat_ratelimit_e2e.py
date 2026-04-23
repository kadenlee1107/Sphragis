#!/usr/bin/env python3
"""Second-layer defense: per-cave token-bucket rate limit.

Scenario:
  Cave has cave_policy allow for example.com:443. Attacker (stuck
  inside the cave) uses that single authorized destination as cover
  for an exfil burst — blasting 80 TCP SYNs to 93.184.216.34:443.

Bat_OS defense:
  1. cave_policy says Allow for each one (destination is in list).
  2. cave_shaper (pps=5, burst=10) says OverLimit after the bucket
     drains. First ~10 packets go through (burst), remaining get
     DropRate.

Expected counters:
  allow       ~= 10    (burst)
  drop-rate   ~= 70    (rest)
  drop-policy  = 0     (not a policy denial)

PASS iff drop-rate >= 60 and allow <= 20.
"""
import pexpect, re, socket, struct, subprocess, sys, threading, time
from pathlib import Path
from datetime import datetime

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
STAMP = datetime.now().strftime('%Y%m%d-%H%M%S')
LOG = ROOT / f"logs/qemu-tests/ratelimit-{STAMP}.log"
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

def build_tcp(smac,dmac,sip,dip,sport,dport,flags=0x02):
    fr=bytearray(); fr+=bytes(dmac)+bytes(smac)+b"\x08\x00"
    ip=bytearray(20); ip[0]=0x45; ip[8]=64; ip[9]=6
    ip[12:16]=sip.to_bytes(4,"big"); ip[16:20]=dip.to_bytes(4,"big")
    tcp=bytearray(20)
    tcp[0:2]=sport.to_bytes(2,"big"); tcp[2:4]=dport.to_bytes(2,"big")
    tcp[12]=5<<4; tcp[13]=flags; tcp[14:16]=(8192).to_bytes(2,"big")
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

def parse_counter(stats, key):
    for line in stats.splitlines():
        if key in line:
            nums=[p for p in line.split() if p.isdigit()]
            if nums: return int(nums[-1])
    return 0

def main():
    HOST=25602; CAVE=25603
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
        # Install the shaper BEFORE the attack.
        run_cmd(c,"cpol-rate kali 5 10")
        details.append("shaper: pps=5, burst=10")

        kali_mac=[0x02,0xAA,0,0,0,0x10]
        nic1_mac=[0x52,0x54,0,0x12,0x34,0x57]
        cave_ip=ip_int("192.168.77.10")

        # Flood: 80 SYNs as fast as we can send.
        FLOOD = 80
        for i in range(FLOOD):
            send_frame(v["conn"], build_tcp(
                kali_mac, nic1_mac, cave_ip, ip_int("93.184.216.34"),
                60000 + i, 443, flags=0x02))
        details.append(f"flood: sent {FLOOD} SYNs to allowed destination")
        time.sleep(1.2)

        stats = run_cmd(c, "nat-stats")
        allow       = parse_counter(stats, "allow:")
        drop_policy = parse_counter(stats, "drop-policy")
        drop_rate   = parse_counter(stats, "drop-rate")
        details.append(f"kernel counters: allow={allow} drop-policy={drop_policy} drop-rate={drop_rate}")

        # Expected: burst (~10) allowed plus a few refills during the flood.
        # Rest (70+) should be drop-rate. drop-policy must stay 0.
        ok = (drop_rate >= 60
              and allow <= 25
              and drop_policy == 0
              and allow + drop_rate + drop_policy >= FLOOD - 2)
        details.append(f"shaper enforced correctly: {ok}")
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
