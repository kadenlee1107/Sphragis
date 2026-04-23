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
  CREATE <name> <image> <caps-csv>        # response: OK <id>  /  ERR <reason>
  RUN <name> <cmdline…>                   # streams stdout/stderr; ends with EOF <rc>
  LIST                                    # lines of <name>\t<image>\t<status>, then EOF
  DESTROY <name>                          # OK / ERR
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
NETWORK_NAME = None     # created on first CREATE, torn down on last DESTROY
HEARTBEAT    = {"last": time.time(), "deadman_s": 0}  # deadman_s=0 → disabled

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

def ensure_network():
    global NETWORK_NAME
    with STATE_LOCK:
        if NETWORK_NAME:
            return NETWORK_NAME
        name = f"batnet-{uuid.uuid4().hex[:8]}"
        r = docker("network", "create", "--driver", "bridge", name)
        if r.returncode != 0:
            raise RuntimeError(f"network create failed: {r.stderr.strip()}")
        NETWORK_NAME = name
        log(f"network created: {name}")
        return name

def teardown_network():
    global NETWORK_NAME
    with STATE_LOCK:
        if NETWORK_NAME:
            docker("network", "rm", NETWORK_NAME, check=False)
            log(f"network removed: {NETWORK_NAME}")
            NETWORK_NAME = None

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

def cmd_create(name: str, image: str, caps_csv: str) -> tuple[bool, str]:
    """Create a BatCave container. `caps_csv` is a comma-separated list of
    Linux capabilities to add (e.g. "NET_RAW,NET_ADMIN")."""
    if not name.replace("-", "").replace("_", "").isalnum():
        return False, "invalid name (alnum + - + _)"
    if not image:
        return False, "image required"
    cname = CAVE_PREFIX + name

    # Check for collision
    r = docker("ps", "-a", "--format", "{{.Names}}", "--filter", f"name=^{cname}$")
    if r.returncode == 0 and cname in r.stdout.splitlines():
        return False, f"cave '{name}' already exists"

    network = ensure_network()

    caps = [c.strip() for c in caps_csv.split(",") if c.strip()]
    cap_args = [f for c in caps for f in ("--cap-add", c)]

    # Build the run command. We run `sleep infinity` as the entrypoint so
    # the container stays alive; tools are invoked via `docker exec`.
    cmd = [
        "run", "-d", "--rm",
        "--name", cname,
        "--network", network,
        "--dns", "1.1.1.1", "--dns", "8.8.8.8",
        *cap_args,
        "--entrypoint", "sleep",
        image, "infinity",
    ]
    r = docker(*cmd)
    if r.returncode != 0:
        return False, r.stderr.strip() or "docker run failed"

    container_id = r.stdout.strip()[:12]
    log(f"CREATE {name} image={image} caps={caps_csv or '(default)'} → {container_id}")
    return True, container_id

def cmd_run_stream(name: str, argv: list[str], writeln, write_raw):
    """Exec `argv` inside cave `name`, stream stdout/stderr to `writeln`."""
    cname = CAVE_PREFIX + name
    # Check existence
    r = docker("inspect", "--format", "{{.State.Running}}", cname)
    if r.returncode != 0:
        writeln(f"ERR cave '{name}' does not exist")
        return 127
    if r.stdout.strip() != "true":
        writeln(f"ERR cave '{name}' is not running")
        return 127

    # Stream output live via Popen
    cmd = ["docker", "exec", cname, *argv]
    log(f"RUN {name} :: {' '.join(shlex.quote(x) for x in argv)}")
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                            text=True, bufsize=1)
    assert proc.stdout is not None
    for line in proc.stdout:
        write_raw(line)
    proc.wait()
    return proc.returncode

def cmd_destroy(name: str) -> tuple[bool, str]:
    cname = CAVE_PREFIX + name
    r = docker("rm", "-f", cname)
    if r.returncode != 0:
        return False, r.stderr.strip() or "rm failed"
    log(f"DESTROY {name}")
    # If no caves remain, tear down the shared network so restart-after-wipe
    # is clean.
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
                        self._send("ERR usage: CREATE <name> <image> [caps-csv]")
                        continue
                    name, image = parts[0], parts[1]
                    caps = parts[2] if len(parts) > 2 else ""
                    ok, msg = cmd_create(name, image, caps)
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
