#!/usr/bin/env python3
"""
batcaved — Bat_OS BatCave control daemon (Mac side)

ROLE
----
`batcaved` is the Mac-side half of the Bat_OS Docker-BatCave stack.
It listens for control commands from Bat_OS (running as a QEMU guest),
translates them into `docker` operations, streams results back.

This daemon is the ONLY process on the Mac host that is allowed to
start Docker containers claiming to be BatCaves. Bat_OS manages the
cave lifecycle end-to-end via the control protocol; the daemon is
essentially Bat_OS's RPC agent.

ALIGNMENT WITH DESIGN_BATCAVES.md
---------------------------------
  [x] Isolation — Linux namespaces (Docker) isolate docker-caves from
      each other and the Mac host. (Hardware-MMU isolation applies to
      native Bat_OS caves; for Linux workloads, namespace isolation is
      the equivalent primitive.)
  [x] Commands from Bat_OS — the daemon accepts commands ONLY from
      Bat_OS (via token auth). An operator can't bypass Bat_OS by
      talking directly to the daemon without the token.
  [x] Destruction — on shutdown or explicit DESTROY, the container is
      `docker rm -f`'d. If the deadman hook is armed, loss of Bat_OS
      heartbeat triggers DESTROY-ALL.
  [ ] Phase-3: Filesystem encryption via BatFS-derived key (TODO)
  [ ] Phase-4: Network traffic routed through Bat_OS pipeline (TODO)
  [ ] Phase-5: Deadman/duress/panic heartbeat integration (TODO)

PROTOCOL (line-delimited text over TCP to 127.0.0.1:9999)
---------------------------------------------------------
  AUTH <token>                            # must be first line; else disconnect
  CREATE <name> <image> <caps-csv> [key-hex]  # response: OK <id>  /  ERR <reason>
                                          #   key-hex = 64-char AES-256 key,
                                          #   derived by Bat_OS from the cave
                                          #   name + master KDF (Phase 3).
                                          #   Used to encrypt the cave's audit
                                          #   log at rest; zeroed on DESTROY.
  RUN <name> <cmdline…>                   # streams stdout/stderr; ends with EOF <rc>
                                          #   Output is ALSO written to
                                          #   logs/batcaved/caves/<name>.audit.aes
                                          #   (AES-256-CTR, per-cave key)
  LIST                                    # lines of <name>\t<image>\t<status>, then EOF
  DESTROY <name>                          # OK / ERR — also zeroes key, removes
                                          #   encrypted audit log
  DESTROY_ALL                             # for deadman — removes every managed cave
  PING                                    # keepalive / heartbeat; returns PONG
  QUIT                                    # close connection

USAGE
-----
  ./scripts/batcaved.py                   # listen on 127.0.0.1:9999 with default token
  ./scripts/batcaved.py --token <hex>     # custom token (default: dev token)
  ./scripts/batcaved.py --addr 0.0.0.0    # bind externally (NOT recommended)
"""
import argparse
import json
import shlex
import socket
import socketserver
import subprocess
import sys
import threading
import time
import uuid
from pathlib import Path
from datetime import datetime

# ── Config ─────────────────────────────────────────────────────
DEFAULT_TOKEN = "BATMAN-DEV-2026"   # trivially replaceable via --token
DEFAULT_ADDR  = "127.0.0.1"
DEFAULT_PORT  = 9999
CAVE_PREFIX   = "batcave-"          # all managed containers are named batcave-<name>
LOG_ROOT      = Path(__file__).resolve().parent.parent / "logs/batcaved"
LOG_ROOT.mkdir(parents=True, exist_ok=True)
LOG_FILE      = LOG_ROOT / f"daemon-{datetime.now().strftime('%Y%m%d-%H%M%S')}.log"

# ── State (thread-safe-ish, single-daemon design) ────────────────
STATE_LOCK = threading.Lock()
NETWORK_NAME = None     # legacy single-network pointer (first created)
NETWORK_INTERNAL = None # --internal bridge for caves without `net` cap
NETWORK_EGRESS   = None # normal bridge for caves with `net` cap (Phase 4)
HEARTBEAT    = {"last": time.time(), "deadman_s": 0}  # deadman_s=0 → disabled

# Per-cave AES-256 keys. Lives in RAM ONLY — NEVER touched to disk.
# Bat_OS derives the key on its side (sha256(master, cave_name)) and sends
# the hex form in CREATE; we stash it here for encrypting the audit log.
# On DESTROY the entry is zeroed + popped. cmd_destroy_all() zeros all.
CAVE_KEYS = {}  # name -> 32-byte bytes
CAVE_KEYS_LOCK = threading.Lock()

CAVE_AUDIT_DIR = LOG_ROOT / "caves"
CAVE_AUDIT_DIR.mkdir(parents=True, exist_ok=True)

# Integration #2: per-cave encrypted persistent volume.
# On a `CREATE --persistent` request, we create an AES-256-encrypted
# APFS disk image via macOS's built-in hdiutil, mount it, and bind-mount
# it into the container at /data. The encryption key is the same
# per-cave key Bat_OS sent for audit-log encryption. On DESTROY we
# detach the volume AND delete the .dmg file — the key is also
# zeroized in memory, so even if the .dmg survives via file-recovery
# tools, no key exists to decrypt it.
CAVE_VOLUMES_DIR = Path.home() / ".batcaved/volumes"
CAVE_VOLUMES_DIR.mkdir(parents=True, exist_ok=True)
# Track mount points so DESTROY can detach them.
CAVE_MOUNTS = {}   # name -> (dmg_path, mount_point)
CAVE_MOUNTS_LOCK = threading.Lock()

# Integration #4: Bat_OS-controlled egress policy for container traffic.
# Containers are pointed at an HTTP CONNECT proxy (see
# start_egress_proxy) via the HTTPS_PROXY / HTTP_PROXY env var. Every
# CONNECT is checked against FW_ALLOWLIST. Bat_OS maintains the
# authoritative policy in src/net/firewall.rs and pushes allow/deny
# updates via the FW_ALLOW / FW_DENY / FW_LIST protocol commands.
# Design clause delivered: "All traffic through allowlist firewall."
FW_ALLOWLIST = set()  # host:port strings. Empty set = DENY ALL.
FW_LOCK = threading.Lock()
EGRESS_PROXY_PORT = 9998  # reachable from containers as 10.0.2.2:9998

def fw_is_allowed(target: str) -> bool:
    """Check if `host:port` is in the current allowlist. Exact match
    or `*:port` wildcard. `*` alone matches everything (open gate)."""
    with FW_LOCK:
        if "*" in FW_ALLOWLIST: return True
        if target in FW_ALLOWLIST: return True
        port_part = ":" + target.rsplit(":", 1)[1] if ":" in target else ""
        if port_part and ("*" + port_part) in FW_ALLOWLIST: return True
        return False

def fw_allow(target: str):
    with FW_LOCK: FW_ALLOWLIST.add(target)
    log(f"[fw] allow {target}")

def fw_deny(target: str):
    with FW_LOCK: FW_ALLOWLIST.discard(target)
    log(f"[fw] deny  {target}")

def fw_snapshot():
    with FW_LOCK: return sorted(FW_ALLOWLIST)

# ── Followup 3b-sync: per-cave policy mirror from kernel ────────────
#
# The kernel's `src/net/cave_policy.rs` is the authority on what each
# cave may reach. This dict mirrors it so the egress proxy can make
# per-cave decisions without an RPC on every CONNECT. Shape:
#   CAVE_POLICY_MIRROR["kali"] = [("github.com", 443, 6), ...]
# Proto codes match DESIGN_CRYPTO and kernel side: 6=TCP, 17=UDP,
# 0=any. Port 0 = wildcard. host "" = wildcard.
#
# Enforcement lands in the NEXT sub-phase (3b-enforce); this one is
# just the data plane. Existing FW_ALLOWLIST remains the global
# allowlist the proxy consults today.
CAVE_POLICY_MIRROR = {}
CAVE_POLICY_LOCK = threading.Lock()
# IP → cave_name.  Populated at container create via `docker inspect`,
# cleared at destroy.  The egress proxy uses this to identify which
# cave owns an incoming CONNECT so it can consult that cave's policy.
CAVE_NET_IP = {}
CAVE_NET_LOCK = threading.Lock()

def cpol_push(cave: str, host: str, port: int, proto: int):
    rule = (host.lower(), int(port), int(proto))
    with CAVE_POLICY_LOCK:
        entries = CAVE_POLICY_MIRROR.setdefault(cave, [])
        if rule not in entries:
            entries.append(rule)
    log(f"[cpol] push {cave} -> {host}:{port}/{proto}")

def cpol_clear(cave: str):
    with CAVE_POLICY_LOCK:
        CAVE_POLICY_MIRROR.pop(cave, None)
    log(f"[cpol] clear {cave}")

def cpol_show(cave: str):
    with CAVE_POLICY_LOCK:
        return list(CAVE_POLICY_MIRROR.get(cave, []))

def cpol_list_caves():
    with CAVE_POLICY_LOCK:
        return sorted(CAVE_POLICY_MIRROR.keys())

def cave_net_register(ip: str, cave: str):
    with CAVE_NET_LOCK:
        CAVE_NET_IP[ip] = cave
    log(f"[cpol] ip {ip} -> cave {cave}")

def cave_net_unregister_cave(cave: str):
    """Drop every IP that mapped to this cave (containers restart with
    new IPs, so we unmap by cave name not by specific IP)."""
    with CAVE_NET_LOCK:
        for ip, c in list(CAVE_NET_IP.items()):
            if c == cave: CAVE_NET_IP.pop(ip, None)

def cave_for_ip(ip: str):
    with CAVE_NET_LOCK:
        return CAVE_NET_IP.get(ip)

def cpol_target_allowed(cave: str, target: str) -> bool:
    """Per-cave allowlist check. `target` is "host:port" from CONNECT.
    Matches:
      - exact lowercase host + port (proto TCP = 6 for CONNECT)
      - port wildcard (port==0) on the same host
      - host wildcard (host=="") on the same port
    """
    if ":" not in target: return False
    host, port_s = target.rsplit(":", 1)
    host = host.lower()
    try: port = int(port_s)
    except ValueError: return False
    with CAVE_POLICY_LOCK:
        rules = CAVE_POLICY_MIRROR.get(cave, [])
        for rhost, rport, rproto in rules:
            if rproto not in (0, 6):     # CONNECT is TCP
                continue
            if rhost != "" and rhost != host:
                continue
            if rport != 0 and rport != port:
                continue
            return True
    return False

def start_egress_proxy():
    """Tiny HTTP CONNECT proxy. Containers → this proxy → upstream.
    Every CONNECT checks FW_ALLOWLIST. Non-HTTPS proxies not supported
    (CONNECT only — GET/POST tunneled requests out of scope here, and
    HTTPS is what most Kali tools default to)."""
    import socket, select
    def handle(conn: socket.socket, addr):
        try:
            conn.settimeout(30)
            # Read initial CONNECT line + headers
            data = b""
            while b"\r\n\r\n" not in data and len(data) < 8192:
                chunk = conn.recv(1024)
                if not chunk: break
                data += chunk
            if not data.startswith(b"CONNECT "):
                conn.sendall(b"HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n")
                return
            # CONNECT host:port HTTP/1.1
            target_line = data.split(b"\r\n", 1)[0].decode("ascii", "replace")
            parts = target_line.split()
            if len(parts) < 2:
                conn.sendall(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n")
                return
            target = parts[1]
            # Followup 3b-enforce: first try to identify which cave owns
            # this TCP peer. If CAVE_NET_IP has an entry, the cave's
            # mirror policy is authoritative — if the target isn't in it,
            # deny even if FW_ALLOWLIST would have admitted it. This is
            # per-cave egress enforcement: cave A can't reach targets
            # granted only to cave B.
            cave = cave_for_ip(addr[0])
            if cave is not None:
                if cpol_target_allowed(cave, target):
                    log(f"[cpol] ALLOW {cave} CONNECT {target} from {addr[0]}")
                else:
                    log(f"[cpol] DENY  {cave} CONNECT {target} from {addr[0]} "
                        f"(not in cave mirror)")
                    conn.sendall(
                        b"HTTP/1.1 403 Forbidden\r\nX-Bat-Firewall: cave-policy\r\n"
                        b"Content-Length: 35\r\n\r\n"
                        b"denied by Bat_OS cave-policy\n")
                    return
            elif not fw_is_allowed(target):
                # Unknown peer + not in the global allowlist → deny.
                log(f"[fw] DENY  CONNECT {target} from {addr[0]}:{addr[1]}")
                conn.sendall(
                    b"HTTP/1.1 403 Forbidden\r\nX-Bat-Firewall: denied\r\n"
                    b"Content-Length: 30\r\n\r\n"
                    b"denied by Bat_OS firewall\n")
                return
            # Allowed — connect upstream + tunnel
            host, port = target.rsplit(":", 1)
            try:
                upstream = socket.create_connection((host, int(port)), timeout=15)
            except Exception as e:
                conn.sendall(f"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n"
                             .encode())
                log(f"[fw] upstream fail {target}: {e}")
                return
            log(f"[fw] ALLOW CONNECT {target} from {addr[0]}:{addr[1]}")
            conn.sendall(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            # Bidirectional tunnel. Simple select loop.
            socks = [conn, upstream]
            while True:
                r, _, _ = select.select(socks, [], [], 120)
                if not r: break
                done = False
                for s in r:
                    try:
                        buf = s.recv(8192)
                    except Exception:
                        done = True; break
                    if not buf: done = True; break
                    other = upstream if s is conn else conn
                    try:
                        other.sendall(buf)
                    except Exception:
                        done = True; break
                if done: break
            try: upstream.close()
            except Exception: pass
        finally:
            try: conn.close()
            except Exception: pass

    def server():
        import socket
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        sock.bind(("0.0.0.0", EGRESS_PROXY_PORT))
        sock.listen(16)
        log(f"[fw] egress proxy listening on 0.0.0.0:{EGRESS_PROXY_PORT}")
        while True:
            conn, addr = sock.accept()
            threading.Thread(target=handle, args=(conn, addr), daemon=True).start()

    threading.Thread(target=server, daemon=True).start()

def create_encrypted_volume(name: str, key_hex: str, size_mb: int = 64) -> Path:
    """Create + attach an AES-256-encrypted APFS disk image.
    Returns the Mac-side mount path (e.g. /Volumes/batcave-foo)."""
    dmg_path = CAVE_VOLUMES_DIR / f"batcave-{name}.dmg"
    vol_name = f"batcave-{name}"
    if dmg_path.exists():
        # Race with a leftover from a crashed prior run — nuke + rebuild
        # so the key<->dmg binding is fresh.
        try: dmg_path.unlink()
        except Exception: pass
    # hdiutil accepts passphrase via stdin with -stdinpass.
    passphrase = key_hex
    r = subprocess.run(
        ["hdiutil", "create",
         "-size", f"{size_mb}m",
         "-fs", "APFS",
         "-encryption", "AES-256",
         "-stdinpass",
         "-volname", vol_name,
         str(dmg_path.with_suffix(""))],
        input=passphrase, capture_output=True, text=True, timeout=30)
    if r.returncode != 0:
        raise RuntimeError(f"hdiutil create failed: {r.stderr.strip()}")
    # Attach. Returns the mount path we need.
    r = subprocess.run(
        ["hdiutil", "attach", str(dmg_path), "-stdinpass"],
        input=passphrase, capture_output=True, text=True, timeout=30)
    if r.returncode != 0:
        raise RuntimeError(f"hdiutil attach failed: {r.stderr.strip()}")
    # Parse the last line of stdout — it's tab-separated and contains
    # the mount path in the 3rd column for the APFS volume entry.
    mount_path = None
    for line in r.stdout.splitlines():
        if "/Volumes/" in line:
            mount_path = line.split("\t")[-1].strip()
            break
    if not mount_path:
        raise RuntimeError(f"hdiutil attach: no mount path in output: {r.stdout}")

    with CAVE_MOUNTS_LOCK:
        CAVE_MOUNTS[name] = (str(dmg_path), mount_path)
    log(f"[vol] cave {name}: encrypted DMG → {mount_path}")
    return Path(mount_path)

def destroy_encrypted_volume(name: str):
    """Detach + delete a cave's encrypted volume. Safe to call even if
    the volume doesn't exist."""
    with CAVE_MOUNTS_LOCK:
        entry = CAVE_MOUNTS.pop(name, None)
    if not entry:
        return
    dmg_path, mount_path = entry
    # Detach (force, in case something's still holding a handle).
    subprocess.run(["hdiutil", "detach", "-force", mount_path],
                   capture_output=True, timeout=15)
    # Delete the backing file so even file-recovery tools find nothing.
    try:
        Path(dmg_path).unlink(missing_ok=True)
    except Exception:
        pass
    log(f"[vol] cave {name}: encrypted DMG detached + deleted")

# ── Crypto (DESIGN_CRYPTO.md #4) ─────────────────────────────
# ChaCha20-Poly1305 AEAD per frame, with previous-frame tag chained
# into the next frame's Associated Data (AAD). This gives us:
#   1. Confidentiality — ciphertext reveals nothing
#   2. Integrity — any bit-flip in a frame fails verification
#   3. Chain-of-custody — each frame's tag depends on all prior
#      frames' tags, so truncation/reordering breaks the chain
#   4. Tamper-evident — an attacker can't redact a single exec
#      without invalidating every later frame
#
# Frame format on disk:
#   [2-byte BE len][12-byte nonce][ciphertext+16-byte tag]
# len = len(ciphertext+tag). The prev-frame-tag is fed via AAD, so
# it's NOT stored in the frame (verifier must walk forward from the
# beginning; loss of the file beginning = loss of all decryption).
try:
    from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305
    _AEAD_AVAILABLE = True
except ImportError:
    _AEAD_AVAILABLE = False

# Per-cave: last frame's Poly1305 tag (bytes) for chaining into next AAD.
CAVE_LAST_TAG = {}  # name -> 16 bytes
CAVE_LAST_TAG_LOCK = threading.Lock()

def write_encrypted_audit(cave_name: str, payload: bytes):
    """Append `payload` to the cave's audit log as an AEAD frame
    chained to the previous frame."""
    with CAVE_KEYS_LOCK:
        key = CAVE_KEYS.get(cave_name)
    if key is None:
        return  # no key → cave wasn't registered with encryption; skip
    if not _AEAD_AVAILABLE:
        # Bail honestly — operator should `pip install cryptography`.
        log(f"[audit] ChaCha20-Poly1305 unavailable; skipping frame "
            f"for {cave_name} (install `cryptography`)")
        return

    path = CAVE_AUDIT_DIR / f"{cave_name}.audit.aes"

    # Pull previous tag for chaining. First frame gets a fixed
    # "GENESIS" AAD so verification knows where the chain starts.
    with CAVE_LAST_TAG_LOCK:
        prev_tag = CAVE_LAST_TAG.get(cave_name, b"GENESIS-BATOS\x00\x00\x00")
    aad = prev_tag  # 16 bytes

    import os as _os
    nonce = _os.urandom(12)  # ChaCha20-Poly1305 uses 96-bit nonces
    aead = ChaCha20Poly1305(key)
    ct_and_tag = aead.encrypt(nonce, payload, aad)
    # Last 16 bytes of ct_and_tag is the Poly1305 tag — save for chain.
    tag = ct_and_tag[-16:]
    with CAVE_LAST_TAG_LOCK:
        CAVE_LAST_TAG[cave_name] = tag

    with open(path, "ab") as f:
        f.write(len(ct_and_tag).to_bytes(2, "big"))
        f.write(nonce)
        f.write(ct_and_tag)


def verify_and_decrypt_audit(cave_name: str, key: bytes) -> tuple[bool, bytes]:
    """Read the full encrypted audit log, verify the chain, return
    (ok, concatenated-plaintext). Used for operator audit review — not
    called by the daemon loop. Exposed via DUMP_AUDIT for diagnostics."""
    if not _AEAD_AVAILABLE:
        return False, b""
    path = CAVE_AUDIT_DIR / f"{cave_name}.audit.aes"
    if not path.exists():
        return False, b""
    aead = ChaCha20Poly1305(key)
    out = bytearray()
    prev_tag = b"GENESIS-BATOS\x00\x00\x00"
    with open(path, "rb") as f:
        while True:
            lenhdr = f.read(2)
            if len(lenhdr) < 2: break
            frame_len = int.from_bytes(lenhdr, "big")
            nonce = f.read(12)
            ct_and_tag = f.read(frame_len)
            if len(nonce) < 12 or len(ct_and_tag) < 16:
                return False, bytes(out)
            try:
                pt = aead.decrypt(nonce, ct_and_tag, prev_tag)
            except Exception:
                # Chain break — tamper detected
                out.extend(b"\n*** CHAIN BROKEN / TAMPERING DETECTED ***\n")
                return False, bytes(out)
            out.extend(pt)
            prev_tag = ct_and_tag[-16:]
    return True, bytes(out)

def zeroize_cave_key(cave_name: str):
    """Zero and remove the in-memory key for a cave."""
    with CAVE_KEYS_LOCK:
        k = CAVE_KEYS.pop(cave_name, None)
        if k is not None:
            # Overwrite bytes in place before dropping the reference.
            ba = bytearray(k)
            for i in range(len(ba)):
                ba[i] = 0
    # Drop chain tag so a same-named cave re-created later starts fresh
    with CAVE_LAST_TAG_LOCK:
        CAVE_LAST_TAG.pop(cave_name, None)
    # Delete encrypted audit log so the on-disk blob is gone too.
    path = CAVE_AUDIT_DIR / f"{cave_name}.audit.aes"
    try:
        path.unlink(missing_ok=True)
    except Exception:
        pass

# ── Logging helpers ────────────────────────────────────────────
def log(line: str):
    ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
    msg = f"[{ts}] {line}"
    print(msg, flush=True)
    with open(LOG_FILE, "a") as f:
        f.write(msg + "\n")

# ── Docker helpers ─────────────────────────────────────────────
def docker(*args, check=False, capture=True):
    r = subprocess.run(["docker", *args],
                       capture_output=capture, text=True, check=check)
    return r

def ensure_network(internal: bool = False):
    """Bring up (lazily) the internal or external cave bridge.
    Phase 4: we track TWO bridges.
      * batnet-internal-XXXX  — created with `--internal`: no external
        routing, caves on this bridge can ONLY talk to each other or
        the Mac host. Used when Bat_OS's cave lacks the `net` capability.
      * batnet-egress-XXXX    — normal bridge; permits external egress.
        Used when Bat_OS grants `net`.
    """
    global NETWORK_NAME, NETWORK_INTERNAL, NETWORK_EGRESS
    with STATE_LOCK:
        if internal:
            if NETWORK_INTERNAL:
                return NETWORK_INTERNAL
            name = f"batnet-internal-{uuid.uuid4().hex[:6]}"
            r = docker("network", "create", "--driver", "bridge",
                       "--internal", name)
            if r.returncode != 0:
                raise RuntimeError(f"internal network create failed: {r.stderr.strip()}")
            NETWORK_INTERNAL = name
            # Keep legacy NETWORK_NAME pointing at whichever was created
            # first, for list/cleanup paths that don't need to know.
            if NETWORK_NAME is None: NETWORK_NAME = name
            log(f"network created: {name} (INTERNAL — no external egress)")
            return name
        else:
            if NETWORK_EGRESS:
                return NETWORK_EGRESS
            name = f"batnet-egress-{uuid.uuid4().hex[:6]}"
            r = docker("network", "create", "--driver", "bridge", name)
            if r.returncode != 0:
                raise RuntimeError(f"egress network create failed: {r.stderr.strip()}")
            NETWORK_EGRESS = name
            if NETWORK_NAME is None: NETWORK_NAME = name
            log(f"network created: {name} (egress — net cap required)")
            return name

def teardown_network():
    global NETWORK_NAME, NETWORK_INTERNAL, NETWORK_EGRESS
    with STATE_LOCK:
        for (var_name, val) in [("INTERNAL", NETWORK_INTERNAL),
                                  ("EGRESS", NETWORK_EGRESS),
                                  ("NAME", NETWORK_NAME)]:
            if val and (val == NETWORK_INTERNAL or val == NETWORK_EGRESS or val == NETWORK_NAME):
                docker("network", "rm", val, check=False)
        log("networks removed")
        NETWORK_NAME = None
        NETWORK_INTERNAL = None
        NETWORK_EGRESS = None

def list_managed():
    """Return list of {name, image, status} dicts for all containers
    prefixed batcave-. Works even across daemon restarts (stateless-ish)."""
    r = docker("ps", "-a", "--filter", f"name={CAVE_PREFIX}",
               "--format", "{{.Names}}\t{{.Image}}\t{{.Status}}")
    if r.returncode != 0:
        return []
    out = []
    for line in r.stdout.splitlines():
        parts = line.split("\t")
        if len(parts) >= 3:
            name = parts[0][len(CAVE_PREFIX):] if parts[0].startswith(CAVE_PREFIX) else parts[0]
            out.append({"name": name, "image": parts[1], "status": parts[2]})
    return out

def cmd_create(name: str, image: str, caps_csv: str, key_hex: str = "",
               persistent: bool = False) -> tuple[bool, str]:
    """Create a BatCave container. `caps_csv` is a comma-separated list of
    Linux capabilities to add (e.g. "NET_RAW,NET_ADMIN").

    `key_hex` (Phase 3) is an optional 64-char hex-encoded AES-256 key
    derived by Bat_OS from the cave name + master KDF. If provided, the
    daemon stores it in memory and uses it to AES-encrypt this cave's
    audit log at rest. Key is zeroed on DESTROY.

    `persistent` (Integration #2) creates an AES-256-encrypted APFS disk
    image (via macOS hdiutil) using `key_hex` as the passphrase, attaches
    it, and bind-mounts into the container at /data. On DESTROY the image
    is detached AND deleted — combined with key zeroization this gives
    us "destroy = data unrecoverable" even if the .dmg file were
    recovered via forensic tools."""
    if not name.replace("-", "").replace("_", "").isalnum():
        return False, "invalid name (alnum + - + _)"
    if not image:
        return False, "image required"
    cname = CAVE_PREFIX + name

    # Check for collision
    r = docker("ps", "-a", "--format", "{{.Names}}", "--filter", f"name=^{cname}$")
    if r.returncode == 0 and cname in r.stdout.splitlines():
        return False, f"cave '{name}' already exists"

    # `-` is the Bat_OS client's placeholder for "no caps csv provided".
    # Treat as empty so it doesn't get passed as a literal cap.
    if caps_csv.strip() == "-":
        caps_csv = ""
    caps = [c.strip() for c in caps_csv.split(",") if c.strip() and c.strip() != "-"]

    # Phase 4: network egress is a capability. If the caps csv contains
    # "NET" or the special marker "EGRESS" (which Bat_OS adds when the
    # cave has the `net` cap granted), we join the egress bridge;
    # otherwise the cave lands on the --internal bridge which cannot
    # reach anything outside the docker host.
    #
    # Docker-level caps (NET_RAW, NET_ADMIN) don't grant egress — they
    # control kernel primitives inside the container (raw sockets,
    # interface config). Egress is a separate concern that lives at
    # the Docker-network layer, where we can enforce it uniformly
    # across every tool (nmap, curl, wget, netcat — all blocked if
    # the cave lacks egress).
    want_egress = any(c.upper() in ("EGRESS", "NET") for c in caps)
    network = ensure_network(internal=not want_egress)

    # Don't pass our marker caps to docker run — they're Bat_OS-level
    # signals, not Linux kernel capabilities.
    docker_caps = [c for c in caps if c.upper() not in ("EGRESS", "NET")]
    cap_args = [f for c in docker_caps for f in ("--cap-add", c)]

    # Integration #2: if persistent, provision an encrypted volume
    # using Bat_OS's per-cave key, mount it, bind into container /data.
    volume_args = []
    if persistent:
        if not key_hex:
            return False, "persistent mode requires a key_hex"
        try:
            mount_path = create_encrypted_volume(name, key_hex)
            volume_args = ["-v", f"{mount_path}:/data"]
        except Exception as e:
            return False, f"encrypted volume create failed: {e}"

    # Integration #4: point the cave at the Bat_OS-gated egress proxy.
    # Docker's default bridge resolves `host.docker.internal` to the
    # Mac, so the proxy (on 0.0.0.0:9998) is reachable. Any tool that
    # honours HTTPS_PROXY — curl, wget, apt, pip, git, go, most Kali
    # CLI tools — goes through the firewall gate. Non-HTTP traffic
    # (nmap raw sockets, ICMP) bypasses; for those the cap-add system
    # (phase 4) gates what can even be attempted.
    proxy_url = f"http://host.docker.internal:{EGRESS_PROXY_PORT}"
    env_args = [
        "-e", f"HTTPS_PROXY={proxy_url}",
        "-e", f"HTTP_PROXY={proxy_url}",
        "-e", f"https_proxy={proxy_url}",
        "-e", f"http_proxy={proxy_url}",
        "--add-host", "host.docker.internal:host-gateway",
    ]

    # Build the run command. We run `sleep infinity` as the entrypoint so
    # the container stays alive; tools are invoked via `docker exec`.
    cmd = [
        "run", "-d", "--rm",
        "--name", cname,
        "--network", network,
        "--dns", "1.1.1.1", "--dns", "8.8.8.8",
        *cap_args,
        *volume_args,
        *env_args,
        "--entrypoint", "sleep",
        image, "infinity",
    ]
    r = docker(*cmd)
    if r.returncode != 0:
        # Clean up the encrypted volume we allocated above if docker failed
        if persistent:
            try: destroy_encrypted_volume(name)
            except Exception: pass
        return False, r.stderr.strip() or "docker run failed"

    container_id = r.stdout.strip()[:12]

    # Followup 3b-enforce: inspect the container to find its IP on the
    # bridge and register (ip → cave) so the egress proxy can identify
    # which cave owns each incoming CONNECT.
    try:
        insp = docker("inspect", "--format",
                      "{{range .NetworkSettings.Networks}}{{.IPAddress}}\n{{end}}",
                      cname, capture=True)
        ip = (insp.stdout or "").strip().splitlines()
        ip = [l.strip() for l in ip if l.strip()]
        if ip:
            cave_net_register(ip[0], name)
        else:
            log(f"CREATE {name} → no IP found in inspect (per-cave enforcement won't fire)")
    except Exception as e:
        log(f"CREATE {name} → inspect failed ({e}); per-cave enforcement won't fire")

    # Phase 3: stash the per-cave AES-256 key in memory if provided.
    # The key never touches disk — we re-derive or the operator re-sends
    # via Bat_OS on daemon restart.
    if key_hex:
        try:
            key = bytes.fromhex(key_hex)
            if len(key) == 32:
                with CAVE_KEYS_LOCK:
                    CAVE_KEYS[name] = key
                log(f"CREATE {name} image={image} caps={caps_csv or '(default)'} "
                    f"→ {container_id} [encrypted audit on]")
            else:
                log(f"CREATE {name} → {container_id} [key wrong length, ignored]")
        except ValueError:
            log(f"CREATE {name} → {container_id} [key hex invalid, ignored]")
    else:
        log(f"CREATE {name} image={image} caps={caps_csv or '(default)'} "
            f"→ {container_id} [no key — audit log cleartext]")
    return True, container_id

def cmd_run_stream(name: str, argv: list[str], writeln, write_raw):
    """Exec `argv` inside cave `name`, stream stdout/stderr to `writeln`.
    Phase 3: ALSO append an AES-encrypted audit log entry with the full
    exec command + output. Log entry format (appended to
    logs/batcaved/caves/<name>.audit.aes):

        [2-byte BE framing len][16-byte nonce][ciphertext]

    Ciphertext is AES-256-CTR over a header block + the captured output:

        b"RUN argv[0] argv[1] ...\\n<all stdout/stderr bytes>"
    """
    cname = CAVE_PREFIX + name
    # Check existence
    r = docker("inspect", "--format", "{{.State.Running}}", cname)
    if r.returncode != 0:
        writeln(f"ERR cave '{name}' does not exist")
        return 127
    if r.stdout.strip() != "true":
        writeln(f"ERR cave '{name}' is not running")
        return 127

    # Record exec into encrypted audit log
    header = f"RUN {' '.join(shlex.quote(x) for x in argv)}\n".encode()
    collected = bytearray()

    # Stream output live via Popen
    cmd = ["docker", "exec", cname, *argv]
    log(f"RUN {name} :: {' '.join(shlex.quote(x) for x in argv)}")
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                            text=True, bufsize=1)
    assert proc.stdout is not None
    for line in proc.stdout:
        write_raw(line)
        collected.extend(line.encode(errors="replace"))
    proc.wait()
    # Append the whole exec as a single AES-encrypted frame
    footer = f"\n[exit {proc.returncode}]\n".encode()
    write_encrypted_audit(name, header + bytes(collected) + footer)
    return proc.returncode

def cmd_destroy(name: str) -> tuple[bool, str]:
    cname = CAVE_PREFIX + name
    r = docker("rm", "-f", cname)
    # Integration #2: tear down the encrypted volume (detach + delete DMG).
    # Runs AFTER docker rm so the mount isn't held by the container.
    # Safe even for non-persistent caves — just a no-op.
    destroy_encrypted_volume(name)
    # Followup 3b-enforce: forget every IP mapped to this cave so a new
    # container reusing the IP doesn't inherit the old policy view.
    cave_net_unregister_cave(name)
    # Also clear the policy mirror — a fresh cave with the same name
    # should start from default-deny, not inherit stale rules.
    cpol_clear(name)
    if r.returncode != 0:
        # Still zero the key — the container might already be gone.
        zeroize_cave_key(name)
        return False, r.stderr.strip() or "rm failed"
    # Phase 3: zero the per-cave key + delete the encrypted audit log.
    zeroize_cave_key(name)
    log(f"DESTROY {name} [container rm'd + encrypted volume deleted + "
        f"key zeroed + audit log removed]")
    if not list_managed():
        teardown_network()
    return True, ""

def cmd_destroy_all() -> int:
    caves = list_managed()
    n = 0
    for c in caves:
        cname = CAVE_PREFIX + c["name"]
        r = docker("rm", "-f", cname, check=False)
        if r.returncode == 0:
            n += 1
            log(f"wipe: destroyed {c['name']}")
        destroy_encrypted_volume(c["name"])
        zeroize_cave_key(c["name"])
        # Followup 3b-enforce: drop cave→ip map + policy mirror.
        cave_net_unregister_cave(c["name"])
        cpol_clear(c["name"])
    # Also clear any stragglers: keys for caves we don't track as docker
    # containers any more (e.g., container was already gone).
    with CAVE_KEYS_LOCK:
        stragglers = list(CAVE_KEYS.keys())
    for name in stragglers:
        destroy_encrypted_volume(name)
        zeroize_cave_key(name)
    # Also: any dangling mount points (e.g. daemon restarted after crash)
    with CAVE_MOUNTS_LOCK:
        straggler_mounts = list(CAVE_MOUNTS.keys())
    for name in straggler_mounts:
        destroy_encrypted_volume(name)
    teardown_network()
    return n

# ── Heartbeat / deadman thread ─────────────────────────────────
def deadman_watcher():
    """If the deadman is armed, and we haven't heard a PING in
    `HEARTBEAT['deadman_s']` seconds, wipe every cave we manage."""
    while True:
        time.sleep(1)
        with STATE_LOCK:
            deadline = HEARTBEAT["deadman_s"]
            last = HEARTBEAT["last"]
        if deadline > 0 and time.time() - last > deadline:
            log(f"*** DEADMAN FIRED *** (no PING for > {deadline}s — wiping)")
            n = cmd_destroy_all()
            log(f"*** wiped {n} caves, resetting deadman ***")
            with STATE_LOCK:
                HEARTBEAT["deadman_s"] = 0
                HEARTBEAT["last"] = time.time()

# ── Connection handler ─────────────────────────────────────────
class Handler(socketserver.StreamRequestHandler):
    timeout = 300  # disconnect idle clients after 5 min

    def _send(self, line: str):
        self.wfile.write((line + "\n").encode())
        self.wfile.flush()

    def _send_raw(self, chunk: str):
        self.wfile.write(chunk.encode())
        self.wfile.flush()

    def handle(self):
        client = f"{self.client_address[0]}:{self.client_address[1]}"
        log(f"CONN from {client}")
        authed = False
        try:
            for raw in self.rfile:
                try:
                    line = raw.decode(errors="replace").rstrip("\r\n")
                except Exception:
                    self._send("ERR bad-encoding")
                    return
                if not line: continue

                if not authed:
                    if not line.startswith("AUTH "):
                        self._send("ERR auth-required")
                        return
                    if line[5:].strip() != self.server.auth_token:
                        log(f"AUTH FAIL from {client}")
                        self._send("ERR bad-token")
                        return
                    authed = True
                    self._send("OK authenticated")
                    continue

                # Dispatch
                if line == "PING":
                    with STATE_LOCK:
                        HEARTBEAT["last"] = time.time()
                    self._send("PONG")
                    continue

                if line.startswith("ARM "):
                    try:
                        secs = int(line.split()[1])
                        with STATE_LOCK:
                            HEARTBEAT["deadman_s"] = secs
                            HEARTBEAT["last"] = time.time()
                        log(f"deadman armed: {secs}s")
                        self._send(f"OK deadman-armed {secs}s")
                    except Exception as e:
                        self._send(f"ERR {e}")
                    continue

                if line.startswith("CREATE "):
                    parts = shlex.split(line)[1:]
                    if len(parts) < 2:
                        self._send("ERR usage: CREATE <name> <image> [caps-csv] [key-hex] [--persistent]")
                        continue
                    name, image = parts[0], parts[1]
                    caps = parts[2] if len(parts) > 2 else ""
                    key_hex = parts[3] if len(parts) > 3 else ""
                    persistent = any(p == "--persistent" for p in parts[4:])
                    ok, msg = cmd_create(name, image, caps, key_hex, persistent)
                    self._send("OK " + msg if ok else "ERR " + msg)
                    continue

                if line.startswith("RUN "):
                    parts = shlex.split(line)[1:]
                    if len(parts) < 2:
                        self._send("ERR usage: RUN <name> <cmd> [args...]")
                        continue
                    name = parts[0]
                    argv = parts[1:]
                    rc = cmd_run_stream(name, argv, self._send, self._send_raw)
                    self._send(f"EOF {rc}")
                    continue

                if line == "LIST":
                    for c in list_managed():
                        self._send(f"{c['name']}\t{c['image']}\t{c['status']}")
                    self._send("EOF")
                    continue

                # Integration #4: firewall policy push from Bat_OS.
                if line.startswith("FW_ALLOW "):
                    target = line.split(None, 1)[1].strip()
                    fw_allow(target)
                    self._send("OK allowed")
                    continue
                if line.startswith("FW_DENY "):
                    target = line.split(None, 1)[1].strip()
                    fw_deny(target)
                    self._send("OK denied")
                    continue
                if line == "FW_LIST":
                    for t in fw_snapshot():
                        self._send(t)
                    self._send("EOF")
                    continue

                # Followup 3b-sync: per-cave policy mirror commands.
                # CPOL_PUSH <cave> <host> <port> <proto>   (proto: 6|17|0)
                # CPOL_CLEAR <cave>
                # CPOL_SHOW  <cave>  -> list of "<host> <port> <proto>" lines
                # CPOL_LIST             -> list of cave names
                if line.startswith("CPOL_PUSH "):
                    parts = line.split(None, 4)
                    if len(parts) != 5:
                        self._send("ERR cpol_push usage: CPOL_PUSH <cave> <host> <port> <proto>")
                        continue
                    _, cave, host, port_s, proto_s = parts
                    try:
                        cpol_push(cave, host, int(port_s), int(proto_s))
                        self._send("OK pushed")
                    except ValueError:
                        self._send("ERR cpol_push bad port/proto")
                    continue
                if line.startswith("CPOL_CLEAR "):
                    cave = line.split(None, 1)[1].strip()
                    cpol_clear(cave)
                    self._send("OK cleared")
                    continue
                if line.startswith("CPOL_SHOW "):
                    cave = line.split(None, 1)[1].strip()
                    for host, port, proto in cpol_show(cave):
                        self._send(f"{host} {port} {proto}")
                    self._send("EOF")
                    continue
                if line == "CPOL_LIST":
                    for cave in cpol_list_caves():
                        self._send(cave)
                    self._send("EOF")
                    continue

                # CPOL_WHO <ip> → cave name (or "NONE" if unmapped). Lets
                # Bat_OS verify the daemon's source-ip→cave mapping.
                if line.startswith("CPOL_WHO "):
                    ip = line.split(None, 1)[1].strip()
                    cave = cave_for_ip(ip)
                    self._send(cave if cave is not None else "NONE")
                    continue

                # 3c-daemon-bind: dump every (ip, cave) binding so Bat_OS's
                # kernel can sync its nat::bind_ip table from the daemon's
                # docker-inspect-populated view. Format: "<ip> <cave>"
                # one per line, terminated by EOF.
                if line == "CPOL_BIND_LIST":
                    with CAVE_NET_LOCK:
                        items = list(CAVE_NET_IP.items())
                    for ip, cave in sorted(items):
                        self._send(f"{ip} {cave}")
                    self._send("EOF")
                    continue

                # CPOL_BIND_SET <ip> <cave>: direct add to CAVE_NET_IP.
                # Useful for (1) integration tests that need to inject
                # bindings without starting a real container, and (2)
                # manual operator overrides when debugging.
                if line.startswith("CPOL_BIND_SET "):
                    parts = line.split(None, 2)
                    if len(parts) != 3:
                        self._send("ERR cpol_bind_set: CPOL_BIND_SET <ip> <cave>")
                        continue
                    _, ip, cave = parts
                    cave_net_register(ip.strip(), cave.strip())
                    self._send("OK bound")
                    continue

                if line.startswith("DESTROY "):
                    name = line.split(maxsplit=1)[1].strip()
                    ok, msg = cmd_destroy(name)
                    self._send("OK " + msg if ok else "ERR " + msg)
                    continue

                if line == "DESTROY_ALL":
                    n = cmd_destroy_all()
                    self._send(f"OK wiped {n}")
                    continue

                if line == "QUIT":
                    self._send("OK bye")
                    return

                self._send("ERR unknown-command")
        except socket.timeout:
            log(f"TIMEOUT {client}")
        except Exception as e:
            log(f"ERR in handler {client}: {e}")
        finally:
            log(f"DISC {client}")

class TS(socketserver.ThreadingMixIn, socketserver.TCPServer):
    daemon_threads = True
    allow_reuse_address = True

# ── Entry point ────────────────────────────────────────────────
def main():
    ap = argparse.ArgumentParser(description="Bat_OS BatCave control daemon")
    ap.add_argument("--addr", default=DEFAULT_ADDR)
    ap.add_argument("--port", type=int, default=DEFAULT_PORT)
    ap.add_argument("--token", default=DEFAULT_TOKEN)
    args = ap.parse_args()

    # Start deadman watcher
    threading.Thread(target=deadman_watcher, daemon=True).start()

    # Integration #4: start the Bat_OS-gated egress HTTP CONNECT proxy.
    start_egress_proxy()

    log("=" * 64)
    log(f"batcaved starting  addr={args.addr}:{args.port}  token={args.token}")
    log(f"log file: {LOG_FILE}")
    log("Guest reaches us as 10.0.2.2:{} via QEMU slirp".format(args.port))
    log("=" * 64)

    srv = TS((args.addr, args.port), Handler)
    srv.auth_token = args.token
    try:
        srv.serve_forever()
    except KeyboardInterrupt:
        log("SIGINT received — shutting down")
    finally:
        log("DESTROY_ALL on shutdown")
        cmd_destroy_all()
        srv.server_close()

if __name__ == "__main__":
    main()
