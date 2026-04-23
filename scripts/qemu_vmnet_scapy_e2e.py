#!/usr/bin/env python3
"""Real-vmnet packet-pipeline E2E (no Docker, uses scapy on macOS host).

Docker Desktop on macOS runs containers inside a Linux VM, not on
the Mac's host network stack — so its macvlan driver can't attach
to a macOS-side bridge (the one QEMU's vmnet-host creates). The
previous `qemu_vmnet_docker_e2e.py` hits that limitation.

This script goes around it: we use the macOS host ITSELF as the
packet source. scapy crafts raw Ethernet frames and `sendp()` puts
them directly on the vmnet bridge, exactly as a container would.
Bat_OS's nic 1 receives them and the classifier runs end-to-end.

Flow:
  1. Ensure scapy is importable (pip-install if needed).
  2. Start batcaved.
  3. Launch QEMU with -netdev vmnet-host + slirp.
  4. Drive Bat_OS shell, set nat-bind + cpol rules.
  5. Find the new bridgeNN interface via ifconfig diff.
  6. scapy: send a stream of frames on that bridge:
       - allowed destination SYN           → allow + NAT
       - unallowed destination SYN         → drop_policy
       - burst toward the allowed dst      → drop_rate
       - TLS ClientHello w/ wrong SNI      → drop_sni
  7. Read nat-stats; verify counters moved in the right buckets.

Run:  sudo python3 scripts/qemu_vmnet_scapy_e2e.py
"""
import atexit, os, re, socket, subprocess, sys, time
from pathlib import Path
from datetime import datetime

if os.geteuid() != 0:
    print("ERROR: must run as sudo (vmnet + raw Ethernet send need it)")
    print("  sudo python3 scripts/qemu_vmnet_scapy_e2e.py")
    sys.exit(1)

REAL_USER = os.environ.get("SUDO_USER") or os.environ.get("USER") or "nobody"
ROOT = Path(__file__).resolve().parent.parent
KERNEL = ROOT / "target/aarch64-unknown-none/release/bat_os"
LOG_DIR = ROOT / "logs/vmnet-e2e"
LOG_DIR.mkdir(parents=True, exist_ok=True)
STAMP = datetime.now().strftime("%Y%m%d-%H%M%S")
QEMU_LOG = LOG_DIR / f"qemu-scapy-{STAMP}.log"
DAEMON_LOG = LOG_DIR / f"daemon-scapy-{STAMP}.log"

# ── Ensure scapy ───────────────────────────────────────────────────
try:
    from scapy.all import Ether, IP, TCP, sendp, conf  # noqa
except ImportError:
    print("[setup] scapy not installed; installing into the system python")
    r = subprocess.run([sys.executable, "-m", "pip", "install",
                        "--break-system-packages", "scapy"],
                       capture_output=True, text=True)
    if r.returncode != 0:
        print("ERROR: pip install scapy failed")
        print(r.stdout[-400:])
        print(r.stderr[-400:])
        sys.exit(1)
    from scapy.all import Ether, IP, TCP, sendp, conf  # noqa

# pexpect is used to drive the Bat_OS shell over QEMU's stdio.
try:
    import pexpect
except ImportError:
    print("ERROR: pexpect not installed; run `pip3 install pexpect`")
    sys.exit(1)

PROMPT = rb"bat_os\s*>\s*"
ANSI = re.compile(rb"\x1b\[[0-9;]*[A-Za-z]|\x1b\]\d+;[^\x07]*\x07")

CAVE_IP    = "192.168.77.10"
CAVE_NAME  = "vmnet-kali"
SUBNET     = "192.168.77.0/24"
GATEWAY    = "192.168.77.1"

def sudo_user(*cmd):
    return subprocess.run(["sudo", "-u", REAL_USER, *cmd],
                          capture_output=True, text=True)

def run_cmd(c, cmd, timeout=10):
    c.sendline(cmd.encode())
    c.expect(PROMPT, timeout=timeout)
    return ANSI.sub(b"", c.before or b"").decode("utf-8", "replace")

def parse_counter(stats, key):
    for line in stats.splitlines():
        if key in line:
            nums = [p for p in line.split() if p.isdigit()]
            if nums: return int(nums[-1])
    return 0

# ── Minimal TLS ClientHello builder (matches our nat parser) ──────

def tls_client_hello(sni_host):
    sni = sni_host.encode("ascii")
    list_len = 1 + 2 + len(sni)
    sn_ext = (bytes([(list_len>>8)&0xFF, list_len&0xFF])
              + b"\x00"
              + bytes([(len(sni)>>8)&0xFF, len(sni)&0xFF])
              + sni)
    ext = (b"\x00\x00"
           + bytes([(len(sn_ext)>>8)&0xFF, len(sn_ext)&0xFF])
           + sn_ext)
    ch = (b"\x03\x03" + b"\x00"*32 + b"\x00"
          + b"\x00\x02\xc0\x2c" + b"\x01\x00"
          + bytes([(len(ext)>>8)&0xFF, len(ext)&0xFF]) + ext)
    hs = (b"\x01"
          + bytes([(len(ch)>>16)&0xFF, (len(ch)>>8)&0xFF, len(ch)&0xFF])
          + ch)
    return (b"\x16\x03\x01"
            + bytes([(len(hs)>>8)&0xFF, len(hs)&0xFF])
            + hs)

# ── Lifecycle ──────────────────────────────────────────────────────

state = {"daemon": None, "qemu": None}
def cleanup():
    print("\n[e2e] cleanup")
    if state["qemu"] is not None:
        try: state["qemu"].terminate(force=True)
        except Exception: pass
    if state["daemon"] is not None and state["daemon"].poll() is None:
        state["daemon"].terminate()
        try: state["daemon"].wait(timeout=3)
        except subprocess.TimeoutExpired: state["daemon"].kill()
    print(f"[e2e] qemu log:   {QEMU_LOG}")
    print(f"[e2e] daemon log: {DAEMON_LOG}")
atexit.register(cleanup)

# ── Start batcaved ────────────────────────────────────────────────

print(f"[e2e] starting batcaved on :9999 (user {REAL_USER})")
daemon_fp = open(DAEMON_LOG, "wb")
state["daemon"] = subprocess.Popen(
    ["sudo", "-u", REAL_USER, "python3", str(ROOT / "scripts" / "batcaved.py")],
    stdout=daemon_fp, stderr=subprocess.STDOUT,
)
for _ in range(40):
    try:
        socket.create_connection(("127.0.0.1", 9999), timeout=0.3).close()
        break
    except OSError:
        time.sleep(0.2)
else:
    print("ERROR: batcaved didn't start")
    sys.exit(1)

# ── Capture bridge set BEFORE QEMU comes up ───────────────────────

pre_bridges = set()
try:
    r = subprocess.check_output(["ifconfig", "-l"], text=True)
    pre_bridges = {x for x in r.split() if x.startswith("bridge")}
except Exception: pass

# ── Launch QEMU with vmnet-host ───────────────────────────────────

qemu_args = [
    "qemu-system-aarch64",
    "-machine", "virt", "-cpu", "max", "-m", "2G",
    "-display", "none",
    "-device", "virtio-gpu-device",
    "-device", "virtio-keyboard-device",
    "-netdev", "user,id=hostnet",
    "-device", "virtio-net-device,netdev=hostnet",
    "-netdev",
    f"vmnet-host,id=cavenet,start-address={GATEWAY},end-address=192.168.77.254,subnet-mask=255.255.255.0",
    "-device", "virtio-net-device,netdev=cavenet",
    "-serial", "mon:stdio",
    "-kernel", str(KERNEL),
]
print(f"[e2e] launching QEMU with -netdev vmnet-host on {SUBNET}")
qemu_fp = open(QEMU_LOG, "wb")
state["qemu"] = pexpect.spawn(qemu_args[0], qemu_args[1:],
                               timeout=120, logfile=qemu_fp, encoding=None)
c = state["qemu"]
try:
    c.expect(rb"\[bs\] flush done .+ entering input loop", timeout=90)
except pexpect.TIMEOUT:
    print("ERROR: Bat_OS never reached auth prompt")
    sys.exit(1)
time.sleep(0.5)
c.sendline(b"batman")
try:
    c.expect(PROMPT, timeout=45)
except pexpect.TIMEOUT:
    print("ERROR: shell prompt never came up")
    sys.exit(1)
print("[e2e] Bat_OS shell ready")

# ── Find the new bridge interface ─────────────────────────────────

time.sleep(2)
post_bridges = set()
try:
    r = subprocess.check_output(["ifconfig", "-l"], text=True)
    post_bridges = {x for x in r.split() if x.startswith("bridge")}
except Exception: pass
new_bridges = sorted(post_bridges - pre_bridges)
bridge = new_bridges[0] if new_bridges else (sorted(post_bridges, reverse=True)[0] if post_bridges else "bridge100")
print(f"[e2e] vmnet bridge: {bridge}")
conf.iface = bridge

# ── Policy: pin example.com's IP; add a flood + SNI rule ──────────

example_ip = "93.184.216.34"
try:
    r = subprocess.check_output(["dig", "+short", "example.com"], text=True, timeout=5)
    first = [l for l in r.strip().splitlines() if re.match(r"^\d+\.\d+\.\d+\.\d+$", l)]
    if first: example_ip = first[0]
except Exception: pass
print(f"[e2e] example.com = {example_ip}")

run_cmd(c, "nat-reset")
run_cmd(c, f"nat-bind {CAVE_IP} {CAVE_NAME}")
# SNI-pinned allow rule for example.com.
run_cmd(c, f"cpol-add-sni {CAVE_NAME} {example_ip} 443 example.com")
# Rate limit so the flood test hits drop-rate not just drop-policy.
run_cmd(c, f"cpol-rate {CAVE_NAME} 5 10")

# ── Craft + send frames via scapy ─────────────────────────────────

# Fake cave MAC. Bat_OS's nic 1 accepts any frame (virtio-net driver
# doesn't filter by dst MAC), so broadcast is fine.
cave_mac  = "02:aa:00:00:00:10"
bcast_mac = "ff:ff:ff:ff:ff:ff"

def send_syn(dst_ip, dport, sport, payload=b""):
    """Craft SYN with optional payload, send on vmnet bridge."""
    flags = "S" if not payload else "PA"
    pkt = (Ether(src=cave_mac, dst=bcast_mac) /
           IP(src=CAVE_IP, dst=dst_ip) /
           TCP(sport=sport, dport=dport, flags=flags))
    if payload:
        pkt = pkt / payload
    sendp(pkt, iface=bridge, verbose=False)

print(f"[e2e] attack #1: SYN to {example_ip}:443 (allowed)")
send_syn(example_ip, 443, 51234)
time.sleep(0.3)

print(f"[e2e] attack #2: SYN to 203.0.113.66:4444 (C2 callback, should drop)")
for i in range(3):
    send_syn("203.0.113.66", 4444, 52000 + i)
time.sleep(0.3)

print(f"[e2e] attack #3: 40-SYN burst to allowed dst (shaper should cap)")
for i in range(40):
    send_syn(example_ip, 443, 53000 + i)
time.sleep(0.5)

print(f"[e2e] attack #4: TLS ClientHello SNI=attacker.com on allowed IP")
# First clear the rate so the ClientHello doesn't get dropped on the rate path.
run_cmd(c, "cpol-rate-clear vmnet-kali")
bad_ch = tls_client_hello("attacker.com")
for i in range(3):
    send_syn(example_ip, 443, 54000 + i, payload=bad_ch)
time.sleep(0.4)

# ── Verify counters ───────────────────────────────────────────────

stats = run_cmd(c, "nat-stats")
allow      = parse_counter(stats, "allow:")
drop_pol   = parse_counter(stats, "drop-policy")
drop_rate  = parse_counter(stats, "drop-rate")
drop_sni   = parse_counter(stats, "drop-sni")
drop_unk   = parse_counter(stats, "drop-unknown-src")

print()
print("── Bat_OS nat-stats after attacks ──")
print(f"   allow            = {allow}")
print(f"   drop-policy      = {drop_pol}")
print(f"   drop-rate        = {drop_rate}")
print(f"   drop-sni         = {drop_sni}")
print(f"   drop-unknown-src = {drop_unk}")

total_events = allow + drop_pol + drop_rate + drop_sni + drop_unk
print(f"   total classified = {total_events}")

ok = (drop_pol >= 3          # at least the three C2 callbacks
      and drop_rate >= 30    # burst attack bulk
      and drop_sni >= 1      # at least one SNI drop
      and allow >= 1)        # at least the first legit SYN

print()
if ok:
    print("═" * 62)
    print(" ✓ PASS — real vmnet + raw scapy frames flowed through")
    print("         Bat_OS's classifier. All four defense layers fired:")
    print(f"         policy={drop_pol}  rate={drop_rate}  sni={drop_sni}")
    print("═" * 62)
    sys.exit(0)
else:
    print("═" * 62)
    print(" ✗ FAIL — one or more counters didn't move as expected.")
    print("         Inspect QEMU log + batcaved log.")
    print("═" * 62)
    sys.exit(1)
