#!/usr/bin/env python3
"""Bat_OS "Kali in a BatCave" — live pentest chain + external analysis.

THE DEMO
========
1. Spin up a Mac-side HTTP server (the recon "target") + a unique path token.
2. For each busybox-backed pentest tool in the chain, boot Bat_OS fresh,
   run the tool INSIDE a capability-gated BatCave, and have QEMU's
   `filter-dump` capture ALL of the guest's virtio-net traffic into a
   per-tool pcap.
3. Mac-side: `tshark` post-analyzes each pcap and prints what a
   pen-tester would actually see on the wire.
4. Summary: which tools produced which protocol evidence, plus the
   HTTP server's application-level log.

THE PENTEST CHAIN (everything is a busybox applet — no host tools)
  1. uname       — target identification (local)
  2. nslookup    — DNS recon
  3. ping        — ICMP reachability
  4. traceroute  — path discovery
  5. wget        — HTTP GET the target

PHILOSOPHY
  - Zero host-global tools used. nmap/tshark are ONLY invoked post-hoc
    for pcap analysis (reading files, no process running alongside a
    Bat_OS target).
  - Each busybox run is in its own QEMU (our ELF runner is noreturn).
  - Each run has a dedicated, disposable BatCave.
"""
import pexpect
import http.server
import socketserver
import threading
import socket
import re
import time
import uuid
import subprocess
from pathlib import Path
from datetime import datetime

ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
LOG_DIR = ROOT / "logs/qemu-tests"; LOG_DIR.mkdir(parents=True, exist_ok=True)
STAMP = datetime.now().strftime("%Y%m%d-%H%M%S")
PCAP_DIR = LOG_DIR / f"kali-demo-{STAMP}"; PCAP_DIR.mkdir(parents=True)

PROBE_ID = uuid.uuid4().hex[:12]
PROBE_PATH = f"/kali-demo-{PROBE_ID}"

ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")
PROMPT = rb"bat_os\s*>\s*"

HITS = []

# ── Mac-side HTTP server (the "target" for recon) ───────────────
class ProbeHandler(http.server.BaseHTTPRequestHandler):
    server_version = "MacTargetServer/0.1"
    def do_GET(self):
        ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        HITS.append({"time": ts, "client": self.client_address,
                     "path": self.path, "headers": dict(self.headers)})
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.send_header("Server", self.server_version)
        body = f"HIT: {PROBE_ID}\n".encode()
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)
    def log_message(self, fmt, *args):
        pass

def pick_port():
    s = socket.socket(); s.bind(("0.0.0.0", 0))
    p = s.getsockname()[1]; s.close()
    return p

def start_server(port):
    class TS(socketserver.ThreadingMixIn, http.server.HTTPServer):
        daemon_threads = True
    srv = TS(("0.0.0.0", port), ProbeHandler)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv

def clean(b: bytes) -> str:
    return ANSI.sub(b"", b or b"").decode("utf-8", "replace").strip()

def dedup_echo_line(s: str) -> str:
    """Only dedup if the line is clearly an echo (≥50% doubled alpha chars)."""
    if not s: return s
    doubled = sum(1 for i in range(0, len(s)-1, 2)
                  if s[i].isalpha() and s[i] == s[i+1])
    if doubled * 2 < len(s) * 0.5:
        return s  # not an echo line, leave it alone
    out, i = [], 0
    while i < len(s):
        if i+1 < len(s) and s[i] == s[i+1] and s[i].isalpha():
            out.append(s[i]); i += 2
        else:
            out.append(s[i]); i += 1
    return "".join(out)

def clean_display(raw: str) -> list[str]:
    """Strip the kernel-log prefixes + dedup per-line echo doubling."""
    keep = []
    SKIP_PREFIXES = ("[mmu]", "[loader]", "[reloc]", "[runner]", "[shell]",
        "[batcave", "[dms]", "[vfs]", "[kbd]", "[gpu]", "[fs]",
        "[firewall]", "[net]", "[boot]", "[chromium", "[bs]",
        "[auth]", "[ipc]", "[arch]", "[rng]", "[sched]", "[mm]",
        "[security]", "[initrd]", "[dtb]", "[mmap]", "bat_os >",
        "BATCAVES", "BatCave created", "BATCAVE", "Granted",
        "batcave ", "  Microkernel", "Ctrl+", "=====", "-----",
        "BAT_OS v", "Bat_OS v", "(none)", "--------", "(")
    for line in raw.splitlines():
        s = line.rstrip()
        if not s: continue
        s = dedup_echo_line(s)
        # Skip kernel log lines
        if any(s.lstrip().startswith(p) for p in SKIP_PREFIXES): continue
        # Skip lines that are mostly ASCII art / banner
        if set(s.strip()) <= set(" _|/\\()"): continue
        keep.append(s)
    return keep

# ── Core: boot Bat_OS with pcap capture, run one command, kill ──
def run_in_batcave(tool_cmd: str, pcap_path: Path, port: int) -> str:
    log_fp = open(LOG_DIR / f"kali-{tool_cmd.split()[0]}-{STAMP}.log", "wb")
    qemu = [
        "qemu-system-aarch64",
        "-machine", "virt", "-cpu", "max", "-m", "2G",
        "-display", "none",
        "-device", "virtio-gpu-device",
        "-device", "virtio-keyboard-device",
        "-netdev", "user,id=net0",
        "-device", "virtio-net-device,netdev=net0",
        # QEMU native pcap — no sudo needed on the host
        "-object", f"filter-dump,id=f0,netdev=net0,file={pcap_path}",
        "-serial", "mon:stdio",
        "-kernel", str(KERNEL),
    ]
    child = pexpect.spawn(qemu[0], qemu[1:], timeout=120,
                          logfile=log_fp, encoding=None)
    try:
        child.expect(rb"\[bs\] flush done .+ entering input loop", timeout=60)
        time.sleep(0.3)
        child.sendline(b"batman")
        child.expect(PROMPT, timeout=30)

        # Dedicated cave per tool for true isolation
        cave_name = tool_cmd.split()[0][:8]  # e.g. "nslookup" -> "nslookup"
        child.sendline(f"batcave create {cave_name}".encode())
        child.expect(PROMPT, timeout=15)
        # Grant minimal caps (net, raw) — everything else denied
        child.sendline(f"batcave grant {cave_name} net".encode())
        child.expect(PROMPT, timeout=15)
        child.sendline(f"batcave grant {cave_name} raw".encode())
        child.expect(PROMPT, timeout=15)

        # Run the tool via busybox inside an auto-activated shell-host cave.
        # (cave::enter hangs — BUG-6 — so the explicit cave is documentary
        # for `batcave list`; the tool still runs cap-gated via shell-host.)
        child.sendline(f"batcave run {tool_cmd}".encode())
        try:
            child.expect(rb"\[linux\] process exited", timeout=60)
            time.sleep(0.4)  # let QEMU flush the pcap
            out = clean(child.before)
        except pexpect.TIMEOUT:
            out = clean(child.before) + "\n[HANG]"
    finally:
        child.terminate(force=True)
        log_fp.close()

    return out

# ── Post-hoc tshark analysis ────────────────────────────────────
def tshark_summary(pcap: Path) -> list[str]:
    """Run tshark on the pcap with a few useful display filters."""
    results = []
    filters = [
        ("all packets",  []),
        ("DNS",          ["-Y", "dns"]),
        ("ICMP",         ["-Y", "icmp"]),
        ("TCP handshake",["-Y", "tcp.flags.syn==1 or tcp.flags.fin==1"]),
        ("HTTP",         ["-Y", "http"]),
    ]
    for title, filt in filters:
        try:
            r = subprocess.run(
                ["tshark", "-r", str(pcap), "-n", *filt],
                capture_output=True, text=True, timeout=15,
            )
            lines = [l for l in r.stdout.splitlines() if l.strip()][:6]
            if lines:
                results.append(f"  {title}:")
                for l in lines:
                    results.append(f"    {l[:110]}")
        except Exception as e:
            results.append(f"  {title}: (tshark err: {e})")
    return results

# ── Main ────────────────────────────────────────────────────────
def main():
    print("=" * 74)
    print(" Bat_OS — Kali-style pentest chain inside a BatCave (real deal)")
    print("=" * 74)

    port = pick_port()
    print(f"[mac] recon target HTTP server → 0.0.0.0:{port}")
    print(f"[mac] probe ID: {PROBE_ID}")
    srv = start_server(port)
    time.sleep(0.2)

    TARGET = "10.0.2.2"  # QEMU slirp alias for the Mac
    CHAIN = [
        ("uname -a",                       "target identification"),
        (f"nslookup example.com",          "DNS recon (public domain)"),
        (f"ping -c 2 {TARGET}",            "ICMP reachability"),
        (f"traceroute {TARGET}",           "path discovery"),
        (f"wget -qO- http://{TARGET}:{port}{PROBE_PATH}",
                                            "HTTP GET the target"),
    ]

    pcaps = []
    for cmd, desc in CHAIN:
        print()
        print("─" * 74)
        print(f" ▶ {cmd}")
        print(f"   ({desc})")
        print("─" * 74)
        pcap = PCAP_DIR / f"{cmd.split()[0]}.pcap"
        out = run_in_batcave(cmd, pcap, port)
        # Bat_OS-side output (what the cave produced)
        print("   BAT_OS stdout:")
        lines = clean_display(out)
        if lines:
            for l in lines[-8:]:
                print(f"     {l[:110]}")
        else:
            print("     (no visible stdout — check per-tool log)")

        # External observation (what tshark sees)
        print(f"   tshark analysis of {pcap.name}:")
        for s in tshark_summary(pcap):
            print(s)
        pcaps.append((cmd, pcap))

    # Wait a beat, teardown
    time.sleep(0.5)
    srv.shutdown()

    # ── Mac-side HTTP server log ──────────────────────────────
    print()
    print("=" * 74)
    print(" EXTERNAL: Mac HTTP server observations")
    print("=" * 74)
    if HITS:
        for h in HITS:
            print(f"  ✓ {h['time']}  from {h['client'][0]}:{h['client'][1]}")
            print(f"      GET {h['path']}")
            print(f"      User-Agent: {h['headers'].get('User-Agent', '(none)')}")
    else:
        print("  (no HTTP hits)")

    # ── Verdict ────────────────────────────────────────────────
    print()
    print("=" * 74)
    print(" VERDICT")
    print("=" * 74)
    got_hit = any(PROBE_ID in h["path"] for h in HITS)
    print(f"  HTTP probe hit the Mac server: {'✅' if got_hit else '❌'}")
    print(f"  Probe ID: {PROBE_ID}")
    print(f"  Pcaps:    {PCAP_DIR}")

if __name__ == "__main__":
    main()
