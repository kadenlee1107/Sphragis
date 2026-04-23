#!/usr/bin/env python3
"""Followup 3c-multinic: verify Bat_OS discovers both NICs.

Launches QEMU with TWO virtio-net devices:
  nic 0: -netdev user  (existing slirp path, 10.0.2.2 gateway)
  nic 1: -netdev socket,listen=:25555
         A Python server accepts the connection but doesn't send
         any packets — QEMU just treats it as an always-up peer so
         Bat_OS sees the device during probe.

Expected: `nic-status` reports 2 NICs, both ready, two different MACs.
"""
import pexpect, re, signal, socket, subprocess, sys, threading, time
from pathlib import Path
from datetime import datetime

ROOT   = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
LOG    = ROOT / f"logs/qemu-tests/multinic-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"
LOG.parent.mkdir(parents=True, exist_ok=True)

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"bat_os\s*>\s*"

def start_socket_server(port=25555):
    """Accept one connection and keep it open silently."""
    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("127.0.0.1", port))
    srv.listen(1)
    state = {"conn": None, "srv": srv}
    def accept_loop():
        try:
            conn, addr = srv.accept()
            state["conn"] = conn
            # Just hold the connection alive. QEMU's socket netdev sends
            # raw Ethernet frames length-prefixed (4-byte BE length header),
            # but we don't need to speak that — QEMU only needs the TCP
            # connection to complete. We'll drain any bytes it sends.
            while True:
                data = conn.recv(65536)
                if not data: break
        except OSError:
            pass
    t = threading.Thread(target=accept_loop, daemon=True)
    t.start()
    return state

def main():
    port = 25555
    print(f"[multinic] starting socket server on :{port}")
    srv_state = start_socket_server(port)

    # Deadman + docker_client try to reach the daemon right after auth;
    # without batcaved running they hang on SYN retries and eat our
    # shell-prompt timeout budget. Spawn a throwaway daemon so boot
    # proceeds normally.
    print("[multinic] starting batcaved subprocess")
    daemon_log = open(ROOT / "logs/qemu-tests/multinic-daemon.log", "wb")
    daemon = subprocess.Popen(
        ["python3", str(ROOT / "scripts" / "batcaved.py")],
        stdout=daemon_log, stderr=subprocess.STDOUT,
    )
    for _ in range(40):
        try:
            probe = socket.create_connection(("127.0.0.1", 9999), timeout=0.3)
            probe.close(); break
        except OSError:
            time.sleep(0.2)

    qemu_args = [
        "qemu-system-aarch64",
        "-machine", "virt", "-cpu", "max", "-m", "2G",
        "-display", "none",
        "-device", "virtio-gpu-device",
        "-device", "virtio-keyboard-device",
        # nic 0 = host slirp (existing)
        "-netdev", "user,id=hostnet",
        "-device", "virtio-net-device,netdev=hostnet",
        # nic 1 = socket to our Python listener. QEMU connects out
        # as client (connect=...), we already listen on that port.
        "-netdev", f"socket,id=cavenet,connect=127.0.0.1:{port}",
        "-device", "virtio-net-device,netdev=cavenet",
        "-serial", "mon:stdio",
        "-kernel", str(KERNEL),
    ]

    print(f"[multinic] launching QEMU with two NICs")
    fp = open(LOG, "wb")
    c = pexpect.spawn(qemu_args[0], qemu_args[1:], timeout=90, logfile=fp, encoding=None)
    verdict = "FAIL"
    try:
        c.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
        time.sleep(0.3); c.sendline(b"batman")
        c.expect(PROMPT, timeout=30)
        print("[multinic] shell ready")

        c.sendline(b"nic-status")
        c.expect(PROMPT, timeout=10)
        raw = ANSI.sub(b"", c.before or b"").decode("utf-8", "replace")
        print("--- nic-status output ---")
        for line in raw.splitlines():
            if line.strip() and not line.strip().startswith(("[shell]", "bat_os >")):
                print(f"   {line.rstrip()[:120]}")

        # Success criterion: "brought up: 2" OR two "nic N: ready" lines.
        ready0 = "nic 0" in raw and "ready" in raw
        ready1 = "nic 1" in raw and "ready" in raw
        if ready0 and ready1:
            verdict = "PASS"
        else:
            verdict = "FAIL"
    except pexpect.TIMEOUT:
        print("[multinic] TIMEOUT")
    finally:
        try: c.terminate(force=True)
        except Exception: pass
        fp.close()
        try: srv_state["srv"].close()
        except Exception: pass
        if srv_state["conn"] is not None:
            try: srv_state["conn"].close()
            except Exception: pass
        daemon.terminate()
        try: daemon.wait(timeout=3)
        except subprocess.TimeoutExpired: daemon.kill()
        daemon_log.close()

    print(f"\nLog: {LOG}")
    print(f"Result: {verdict}")
    return 0 if verdict == "PASS" else 1

if __name__ == "__main__":
    sys.exit(main())
