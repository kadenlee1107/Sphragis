#!/usr/bin/env python3
"""Keep Bat_OS running on M4 under m1n1 HV **across resets**.

The SoC (we think Apple's PMU or an AOP watchdog) resets the Mac
roughly every 60-96 s of HV idle. This is a known, unfixed, probably-
external-to-us problem. Until we build a proper AOP RTKit driver,
this supervisor hides the reset behind automatic recovery:

  1. Wait for stock m1n1 to enumerate its USB CDC composite
     (/dev/ttyACM1 + /dev/ttyACM2 become live).
  2. Chainload external/m1n1/build/m1n1.macho with the -S flag.
  3. Run batos_hv_interactive.py with the user's chosen stimulus,
     under a hard timeout.
  4. When the session dies (USB drops / timeout hit), note the
     duration, update running stats, loop back to step 1.

From the user's perspective: start the supervisor, walk away, come
back whenever — Bat_OS has been booting, running, and rebooting
forever. Each cycle prints one line of metrics, so you can watch
convergence in real time.

Env vars (all optional, pass-through to the interactive session):
  BATOS_KEEP_FB        "1"   — hold the FB live under HV
  BATOS_HV_STIMULUS    str   — canned ;; -separated shell input to
                               replay on every cycle. Default: the
                               passphrase then a slow uptime poll.
  BATOS_HV_TIMEOUT     int   — per-cycle hard timeout in seconds
                               (default 360).
  BATOS_HV_MAX_CYCLES  int   — stop after N cycles (default: run
                               forever).
  BATOS_HV_LOG_DIR     str   — where to tee per-cycle logs
                               (default /tmp/batos_hv_supervisor).
  BATOS_PASSPHRASE     str   — rebuild-time passphrase info only,
                               not used by the supervisor itself.

Usage:
  sg dialout -c "python3 scripts/hv/batos_hv_supervisor.py"

Ctrl+C between cycles to stop; during a cycle the running HV
session gets the signal and terminates cleanly.
"""
import os
import pathlib
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
M1N1_MACHO = ROOT / "external/m1n1/build/m1n1.macho"
CHAINLOAD = ROOT / "external/m1n1/proxyclient/tools/chainload.py"
INTERACTIVE = ROOT / "scripts/hv/batos_hv_interactive.py"

ACM_PRIMARY = "/dev/ttyACM1"
ACM_VUART = "/dev/ttyACM2"

DEFAULT_STIMULUS = ";;".join([
    "batman",
    *["uptime"] * 40,
])
DEFAULT_TIMEOUT = 360


def _log(msg: str):
    ts = time.strftime("%H:%M:%S")
    print(f"[supervisor {ts}] {msg}", flush=True)


def _wait_for_stock_m1n1(deadline_seconds: float = 420.0) -> bool:
    """Poll for both /dev/ttyACM1 and /dev/ttyACM2 to exist + be
    readable by our group. Timeout → return False.

    420 s budget: a clean Mac reset → iBoot → stock m1n1 cycle is
    typically 60-120 s, but if the Mac decided to jump back into
    macOS or hit a nasty iBoot branch, we can wait minutes. 420 s
    is long enough that a genuinely dead Mac is unambiguous.

    Special-case (loud message after 150 s): if /dev/ttyACM0 is
    the only ACM visible and has been that way for >150 s, the
    Mac has almost certainly booted into macOS rather than
    chainloading stock m1n1. The supervisor can't fix this from
    Ubuntu — a human needs to poke the boot picker.
    """
    start = time.time()
    last_progress_msg = 0.0
    notified_macos = False
    while time.time() - start < deadline_seconds:
        if os.path.exists(ACM_PRIMARY) and os.path.exists(ACM_VUART):
            # ttyACM1 / ACM2 appeared together → stock m1n1 is up.
            elapsed = time.time() - start
            _log(f"Mac back at m1n1 after {elapsed:.0f}s")
            return True
        now = time.time()
        elapsed = now - start
        if now - last_progress_msg > 30:
            _log(f"waiting for Mac to reboot into m1n1 "
                 f"({ACM_PRIMARY}, {ACM_VUART}) — "
                 f"{int(elapsed)}s elapsed")
            last_progress_msg = now
        if elapsed > 150 and not notified_macos:
            only_acm0 = (
                os.path.exists("/dev/ttyACM0")
                and not os.path.exists(ACM_PRIMARY)
            )
            if only_acm0:
                _log("────────────────────────────────────────────")
                _log("Mac seems to have booted into macOS instead")
                _log("of m1n1. Hold the power button, select your")
                _log("m1n1 volume in the boot picker, then let it")
                _log("chainload — we'll pick up automatically.")
                _log("────────────────────────────────────────────")
                notified_macos = True
        time.sleep(2.0)
    return False


def _chainload() -> bool:
    """Run chainload.py with the -S flag. Returns True on success."""
    _log(f"chainloading {M1N1_MACHO.name}")
    env = dict(os.environ)
    env["M1N1DEVICE"] = ACM_PRIMARY
    env["M1N1WAIT"] = "1"
    cmd = [
        "sudo", "-n",
        "--preserve-env=M1N1DEVICE,M1N1WAIT",
        "/usr/bin/python3",
        str(CHAINLOAD),
        "-S",
        str(M1N1_MACHO),
    ]
    try:
        # chainload.py runs ~20-40 s; 180 s budget is generous.
        r = subprocess.run(cmd, env=env, stdout=subprocess.PIPE,
                           stderr=subprocess.STDOUT, timeout=180,
                           text=True, errors="replace")
    except subprocess.TimeoutExpired:
        _log("chainload: TIMEOUT (>180 s) — probably Mac wedged")
        return False
    ok = "Proxy is alive again" in r.stdout
    if not ok:
        tail = r.stdout.splitlines()[-6:]
        _log("chainload: FAILED, tail=")
        for ln in tail:
            _log(f"  {ln}")
    return ok


def _run_session(cycle: int, log_dir: pathlib.Path) -> tuple[int, int]:
    """Run one interactive-HV session. Returns (last_heartbeat_secs,
    total_wallclock_secs). last_heartbeat_secs = -1 if HV never
    started (chainload→run race lost).
    """
    stim = os.environ.get("BATOS_HV_STIMULUS", DEFAULT_STIMULUS)
    keep_fb = os.environ.get("BATOS_KEEP_FB", "1")
    timeout = int(os.environ.get("BATOS_HV_TIMEOUT", DEFAULT_TIMEOUT))

    log_path = log_dir / f"cycle_{cycle:04d}.log"
    _log(f"cycle {cycle}: starting HV session → {log_path.name}")

    env = dict(os.environ)
    env["BATOS_KEEP_FB"] = keep_fb
    env["BATOS_HV_STIMULUS"] = stim

    cmd = ["sg", "dialout", "-c",
           f"timeout {timeout} /usr/bin/python3 {INTERACTIVE}"]

    t0 = time.time()
    with open(log_path, "w") as lf:
        # We can't easily Ctrl+C the `sg dialout` child once started,
        # but its `timeout N` bound guarantees termination. Parent
        # Ctrl+C will propagate.
        try:
            subprocess.run(cmd, env=env, stdout=lf,
                           stderr=subprocess.STDOUT,
                           timeout=timeout + 15)
        except subprocess.TimeoutExpired:
            _log(f"cycle {cycle}: outer timeout fired (>{timeout + 15}s)")
    elapsed = int(time.time() - t0)

    # Read last heartbeat from the log.
    last_hb = -1
    try:
        for line in log_path.read_text(errors="replace").splitlines():
            if "HV alive t=" in line:
                # e.g. "TTY> HV alive t=83s traps=26072798"
                t = line.split("HV alive t=", 1)[1].split("s", 1)[0]
                try:
                    last_hb = int(t)
                except ValueError:
                    pass
    except FileNotFoundError:
        pass
    return last_hb, elapsed


def _fmt_stats(durations):
    if not durations:
        return "no data yet"
    arr = sorted(durations)
    n = len(arr)
    avg = sum(arr) / n
    return (f"n={n} min={arr[0]}s max={arr[-1]}s "
            f"p50={arr[n // 2]}s avg={avg:.0f}s")


def main():
    log_dir = pathlib.Path(os.environ.get(
        "BATOS_HV_LOG_DIR", "/tmp/batos_hv_supervisor"))
    log_dir.mkdir(parents=True, exist_ok=True)

    max_cycles = int(os.environ.get("BATOS_HV_MAX_CYCLES", "0"))

    if not M1N1_MACHO.exists():
        print(f"[supervisor] ERROR: {M1N1_MACHO} not found. Run "
              f"`make -C external/m1n1` first.", file=sys.stderr)
        return 1

    _log(f"supervisor starting. Logs → {log_dir}")
    _log(f"m1n1.macho mtime={time.ctime(M1N1_MACHO.stat().st_mtime)}")
    _log(f"max_cycles={'∞' if max_cycles == 0 else max_cycles}")

    heartbeats = []
    cycle = 0
    session_start = time.time()

    try:
        while max_cycles == 0 or cycle < max_cycles:
            cycle += 1
            _log(f"─── cycle {cycle} ───")

            if not _wait_for_stock_m1n1():
                _log(f"cycle {cycle}: Mac never came back "
                     f"after the 420s budget. Sleeping 10 s and "
                     f"trying again — user may need to hold power.")
                time.sleep(10)
                continue

            if not _chainload():
                _log(f"cycle {cycle}: chainload failed, loop.")
                # Give Mac a chance to reset back to stock.
                time.sleep(5)
                continue

            last_hb, wall = _run_session(cycle, log_dir)

            if last_hb >= 0:
                heartbeats.append(last_hb)
            _log(f"cycle {cycle}: last_hb={last_hb}s wall={wall}s "
                 f"| stats over {len(heartbeats)} session(s): "
                 f"{_fmt_stats(heartbeats)}")
            _log(f"total supervisor uptime: {int(time.time() - session_start)}s")
    except KeyboardInterrupt:
        _log("Ctrl+C — exiting.")
    finally:
        _log(f"shutdown. cycles={cycle} stats={_fmt_stats(heartbeats)}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
