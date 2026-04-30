"""Shared Bat_OS QEMU/HVF boot harness.

All the per-command runners (render, dump-dom, smoke) want the same
six setup steps before they can talk to the bat_os shell:

  1. Make sure `target/aarch64-unknown-none/release/bat_os.bin`
     (the flat Image) is fresh relative to the linked ELF.
  2. Spawn `scripts/batcaved.py` so the kernel can answer the
     control-channel handshake.
  3. Wait for the daemon's TCP port (127.0.0.1:9999) to listen.
  4. Spawn QEMU with the standard HVF + GICv3 args.
  5. Wait for the `bat_os > ` prompt.
  6. Tear it all down on exit.

This module wraps that. Callers do:

    from scripts.lib.qemu_boot import boot
    with boot() as session:
        session.run(b"dump-dom file:///bin/hello.html")
        session.expect_prompt(timeout=30)

`session.log` is the path of the captured serial log.
"""
from __future__ import annotations

import os
import shutil
import socket
import subprocess
import sys
import time
from contextlib import contextmanager
from datetime import datetime
from pathlib import Path

import pexpect


ROOT   = Path(__file__).resolve().parents[2]
TARGET = ROOT / "target/aarch64-unknown-none/release"
PROMPT = rb"bat_os\s*>\s*"


def _find_objcopy() -> Path:
    """Locate rust-objcopy/llvm-objcopy. Order matters: rustup nightly's
    rust-objcopy is the canonical choice; fall back to whatever's on PATH
    or in Homebrew's llvm cellar."""
    candidates = [
        Path.home()
        / ".rustup/toolchains/nightly-aarch64-apple-darwin/"
        / "lib/rustlib/aarch64-apple-darwin/bin/rust-objcopy",
        Path("/opt/homebrew/Cellar/llvm/22.1.3/bin/llvm-objcopy"),
    ]
    for c in candidates:
        if c.exists():
            return c
    on_path = shutil.which("llvm-objcopy") or shutil.which("objcopy")
    if on_path:
        return Path(on_path)
    raise FileNotFoundError("no rust-objcopy/llvm-objcopy in $PATH")


def _refresh_bat_os_bin(elf: Path, bin_path: Path) -> None:
    """Re-objcopy the flat Image if older than the linked ELF."""
    if bin_path.exists() and bin_path.stat().st_mtime >= elf.stat().st_mtime:
        return
    objcopy = _find_objcopy()
    print(f"[boot] {objcopy.name} -O binary {elf.name} {bin_path.name}")
    subprocess.run([str(objcopy), "-O", "binary", str(elf), str(bin_path)],
                   check=True)


def _wait_for_daemon(port: int = 9999, attempts: int = 40) -> None:
    for _ in range(attempts):
        try:
            socket.create_connection(("127.0.0.1", port), timeout=0.3).close()
            return
        except OSError:
            time.sleep(0.2)
    print(f"[boot] WARNING: daemon never listened on :{port}", file=sys.stderr)


class Session:
    """Live `pexpect.spawn` of QEMU + a path to the captured serial log."""

    def __init__(self, child: "pexpect.spawn", log: Path):
        self.child = child
        self.log   = log

    def run(self, line: bytes) -> None:
        self.child.sendline(line)

    def expect_prompt(self, timeout: int = 30) -> None:
        self.child.expect(PROMPT, timeout=timeout)

    def expect(self, pattern, timeout: int = 30):
        return self.child.expect(pattern, timeout=timeout)


@contextmanager
def boot(*, log_prefix: str = "session", timeout: int = 120,
         initrd: Path | None = None):
    """Bring up Bat_OS under QEMU/HVF and yield a `Session`.

    `log_prefix` controls the on-disk log filename:
      logs/qemu-tests/<log_prefix>-<timestamp>.log

    `initrd` defaults to `target/.../chromium_initrd.bin`. Pass a
    different path to test alternate archive bakes.
    """
    log_dir = ROOT / "logs/qemu-tests"
    log_dir.mkdir(parents=True, exist_ok=True)
    ts = datetime.now().strftime("%Y%m%d-%H%M%S")
    log_path = log_dir / f"{log_prefix}-{ts}.log"

    elf      = TARGET / "bat_os"
    kernel   = TARGET / "bat_os.bin"
    initrd   = initrd or (TARGET / "chromium_initrd.bin")
    if not elf.exists():
        raise FileNotFoundError(
            f"no kernel ELF at {elf}.\n"
            "  build with: BAT_OS_ALLOW_UNSIGNED_INITRD=1 \\\n"
            "              BAT_OS_PASSPHRASE=batman \\\n"
            "              BAT_OS_KEEP_GOING=1 \\\n"
            "              cargo build --release --features gicv3"
        )
    if not initrd.exists():
        raise FileNotFoundError(
            f"no initrd at {initrd}.\n"
            f"  bake with: tools/bake_chromium_archive.sh "
            f"ports/chromium_port/out/content_shell "
            f"ports/chromium_port/out/lib_runtime"
        )

    _refresh_bat_os_bin(elf, kernel)

    daemon = subprocess.Popen(
        ["python3", str(ROOT / "scripts" / "batcaved.py")],
        stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT,
    )
    _wait_for_daemon()

    args = [
        "qemu-system-aarch64",
        "-accel", "hvf",
        "-machine", "virt,gic-version=3",
        "-cpu", "host",
        "-m", "4G",
        "-display", "none",
        "-serial", "mon:stdio",
        "-kernel", str(kernel),
        "-initrd", str(initrd),
    ]

    fp = open(log_path, "wb")
    child = pexpect.spawn(args[0], args[1:], timeout=timeout,
                          logfile=fp, encoding=None)
    session = Session(child, log_path)
    try:
        session.expect_prompt(timeout=60)
        yield session
    finally:
        try:
            child.terminate(force=True)
        except Exception:
            pass
        fp.close()
        daemon.terminate()
        try:
            daemon.wait(timeout=3)
        except subprocess.TimeoutExpired:
            daemon.kill()
