#!/usr/bin/env python3
"""Drive cpol-flow-rate-selftest inside QEMU."""
import pexpect
import socket
import subprocess
import sys
import time
from pathlib import Path
from datetime import datetime
ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/sphragis"
LOG = ROOT / f"logs/qemu-tests/flowrate-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)
PROMPT = rb"sphragis\s*>\s*"

def main():
    daemon=subprocess.Popen(["python3",str(ROOT/"scripts"/"batcaved.py")],
        stdout=subprocess.DEVNULL,stderr=subprocess.STDOUT)
    for _ in range(40):
        try: socket.create_connection(("127.0.0.1",9999),timeout=0.3).close(); break
        except OSError: time.sleep(0.2)
    args=["qemu-system-aarch64","-machine","virt","-cpu","max","-m","2G",
          "-display","none",
          "-device","virtio-gpu-device","-device","virtio-keyboard-device",
          "-netdev","user,id=net0","-device","virtio-net-device,netdev=net0",
          "-serial","mon:stdio","-kernel",str(KERNEL)]
    fp=open(LOG,"wb")
    c=pexpect.spawn(args[0],args[1:],timeout=90,logfile=fp,encoding=None)
    verdict="FAIL"
    try:
        c.expect(rb"\[bs\] flush done .+ entering input loop",timeout=60)
        time.sleep(0.3); c.sendline(b"batman")
        c.expect(PROMPT,timeout=30)
        c.sendline(b"cpol-flow-rate-selftest")
        c.expect([b"PASS",b"FAIL"], timeout=10)
        verdict=c.match.group(0).decode()
        try: c.expect(PROMPT, timeout=5)
        except pexpect.TIMEOUT: pass
        with open(LOG,"rb") as f:
            raw=f.read().decode("utf-8","replace")
        idx=raw.find("CPOL-FLOW-RATE SELF-TEST")
        if idx>=0:
            chunk=raw[idx:]
            end=chunk.find("sphragis >",40)
            print(chunk[:end if end>0 else 1200])
    except pexpect.TIMEOUT: print("[flowrate] TIMEOUT")
    finally:
        c.terminate(force=True); fp.close()
        daemon.terminate()
        try: daemon.wait(timeout=3)
        except subprocess.TimeoutExpired: daemon.kill()
    print(f"Log: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict=="PASS" else 1

if __name__=="__main__":
    sys.exit(main())
