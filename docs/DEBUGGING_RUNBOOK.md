# Bat_OS debugging runbook

Every failure mode we've hit, with the exact recovery step. Check here
before guessing — most of these are already solved.

## Table of contents

- [1. m1n1 won't reach "Running proxy..."](#1-m1n1-wont-reach-running-proxy)
- [2. `chainload.py` can't find the serial port](#2-chainloadpy-cant-find-the-serial-port)
- [3. Mac spontaneously reboots mid-chainload](#3-mac-spontaneously-reboots-mid-chainload)
- [4. Bat_OS resets silently on entry](#4-bat_os-resets-silently-on-entry)
- [5. `run_guest.py` crashes with AMX SYNC exception](#5-run_guestpy-crashes-with-amx-sync-exception)
- [6. Exception loop stuck on SYNC from EL2h](#6-exception-loop-stuck-on-sync-from-el2h)
- [7. USB CDC device not enumerating on Windows](#7-usb-cdc-device-not-enumerating-on-windows)
- [8. `cargo build` errors about rust-src / build-std](#8-cargo-build-errors-about-rust-src--build-std)
- [9. Chromium Docker container killed (exit 137)](#9-chromium-docker-container-killed-exit-137)
- [10. Docker Chromium build restarted from [1/N] after resume](#10-docker-chromium-build-restarted-from-1n-after-resume)
- [11. GitHub push rejected due to >100 MB file](#11-github-push-rejected-due-to-100-mb-file)
- [12. Mac stuck on m1n1, can't get back to macOS](#12-mac-stuck-on-m1n1-cant-get-back-to-macos)

---

## 1. m1n1 won't reach "Running proxy..."

**Symptom:** boot the Mac, see the Apple boot-picker briefly, then
black screen or stuck on m1n1 bat logo, no USB serial device appears.

**Causes to check in order:**

1. **Mac booted macOS instead of m1n1.** During boot, do you see the
   macOS progress bar or the bat logo? If macOS — reboot, hold power
   button at start, select the Asahi/m1n1 entry from the boot picker.

2. **m1n1 panicked during hardware init.** M4 has unsupported MCC/
   cpufreq versions; usually non-fatal, but sustained CPU load
   sometimes triggers a watchdog. Power-cycle and try again.

3. **Permissive Security got reverted.** If macOS pushed an update,
   the security policy may revert to Full. Boot into Recovery,
   Utilities → Startup Security Utility → re-enable Permissive +
   user-managed kexts. Then `kmutil configure-boot` the m1n1.bin
   again (must be done from Recovery on Apple Silicon).

## 2. `chainload.py` can't find the serial port

**Symptom:**
```
could not open port /dev/m1n1: [Errno 2] No such file or directory
```

**Fix:** `M1N1DEVICE` env var isn't set. Use:
```bash
sudo M1N1DEVICE=/dev/ttyACM0 python3 proxyclient/tools/chainload.py ...
```

Replace `ttyACM0` if the device enumerated somewhere else. Check with:
```bash
ls /dev/ttyACM* /dev/ttyUSB* 2>/dev/null
dmesg | grep -i 'cdc\|ttyACM' | tail -5
```

## 3. Mac spontaneously reboots mid-chainload

**Symptom:** serial link drops suddenly, Mac goes through full iBoot
sequence again, m1n1 re-loads from scratch.

**Cause:** Almost always the **P-cluster RVBAR SError** on M4. m1n1's
`chainload.py` tries to write RVBAR for every non-running CPU. E-cluster
writes at `0x210xx` succeed; P-cluster writes at `0x211xx` SError.

**Fix:** Use the `-S` / `--skip-secondary-cpus` flag:
```bash
sudo M1N1DEVICE=/dev/ttyACM0 python3 proxyclient/tools/chainload.py \
    --raw --entry-point 0 -S \
    bat_os_apple.bin
```

Our vendored `external/m1n1/proxyclient/tools/chainload.py` has this
flag pre-added.

## 4. Bat_OS resets silently on entry

**Symptom:** chainload completes ("Jumping to entry point"), serial
goes silent, iBoot sequence restarts from scratch.

**Cause #1: Linux-image-header collision at offset 0.**
Previously our build script produced a raw binary where the Linux
kernel Image header (from `linux_header.s`) was at offset 0. When
chainload's `--entry-point 0` jumped there, it ran the Linux boot
stub which interprets x0 as an FDT — but m1n1 puts Apple boot_args
there, so first deref faulted.

**Fix (already applied):** `apple/boot.s` is in `.text.apple_boot`
section which `linker_apple.ld` places BEFORE `.text.boot`. Verify:
```bash
xxd target/bat_os_apple.bin | head -2
# first bytes should be `f4 03 00 aa` (mov x20, x0) — start of _apple_start
# NOT `... 41 52 4d 64 ...` ("ARM\x64" magic)

file target/bat_os_apple.bin
# should say 'data', NOT 'Linux kernel ARM64 boot executable Image'
```

**Cause #2: bad load address or wrong entry offset.** Confirm:
- `chainload.py --raw --entry-point 0` (0, not 2048)
- The binary's first instruction is `mov x20, x0` (= 0xaa0003f4 LE)

## 5. `run_guest.py` crashes with AMX SYNC exception

**Symptom:**
```
Exception: SYNC, from EL2h, ESR: 0x2000000 (unknown), PC: 0x10005fafa78
```
in `hv/__init__.py` around line 1435 at `self.u.msr(AMX_CONFIG_EL1, ...)`.

**Cause:** M4 doesn't have AMX. Apple switched to SME. Writing
`AMX_CONFIG_EL1` on M4 traps.

**Fix:** Don't use `run_guest.py`. Use `chainload.py` — it doesn't init
the HV and therefore never touches AMX_CONFIG_EL1. We want Bat_OS
to run as the actual OS, not inside a hypervisor, so this is the right
choice anyway.

## 6. Exception loop stuck on SYNC from EL2h

**Symptom:** After a previous run_guest.py or a panicked chainload,
serial keeps spitting `Exception: SYNC, from EL2h...` repeatedly,
USB serial dead.

**Fix:** Power-cycle the Mac (hold power 5+ seconds to force
shutdown). Boot again — m1n1 comes up fresh, proxy becomes usable
again. No data loss, no recovery needed.

## 7. USB CDC device not enumerating on Windows

**Symptom:** m1n1 is running, USB-C cable connected to Windows PC,
but Device Manager shows "Unknown USB Device" with code 43, or nothing
appears at all, or only a partial device enumerates.

**Cause:** m1n1 advertises as a composite USB device (VID 1209 /
PID 316D). Windows needs a vendor INF file for one of the interfaces
(the non-CDC one). That INF doesn't exist in m1n1's repo and isn't
distributed anywhere.

**Fix:** **Don't use Windows as the proxy host.** Ubuntu (live USB or
persistent) handles this device out-of-the-box because Linux CDC-ACM
doesn't care about the non-compliant interface.

## 8. `cargo build` errors about rust-src / build-std

**Symptom (building m1n1 Rust lib):**
```
error: "/.../stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/Cargo.lock"
does not exist, unable to build with the standard library
```

Or: `can't find crate for 'alloc'`.

**Cause:** m1n1's Rust portion needs `-Z build-std=alloc,core` which
requires nightly Rust + rust-src component.

**Fix:**
```bash
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
# Make sure rust-toolchain.toml at m1n1 repo ROOT (not just rust/)
# pins channel = "nightly"
make RELEASE=1 BUILDSTD=1 -j4
```

The `BUILDSTD=1` flag is critical — without it m1n1's Makefile resets
CARGO_FLAGS and drops the build-std flag.

## 9. Chromium Docker container killed (exit 137)

**Symptom:** `docker ps -a` shows the batos-chromium-build container
as `Exited (137)`.

**Cause:** Memory pressure. Usually triggered when we run other
heavy jobs (QEMU HVF, multi-agent Claude work) on the same Mac.

**Fix:** Just restart it. ninja is incremental; it resumes from the
last completed object file.
```bash
docker start batos-chromium-build
docker logs --tail 3 batos-chromium-build
```

If many resumes happen, snapshot the container state periodically:
```bash
docker commit batos-chromium-build batos-chromium-build:snap-$(date +%Y%m%d-%H%M)
```

## 10. Docker Chromium build restarted from [1/N] after resume

**Symptom:** Resumed a stopped container but ninja shows `[1/15843]`
again instead of resuming where it left off.

**Cause:** Either the build-dir volume was recreated (e.g. you ran
`docker run` again with different flags instead of `docker start`),
or gn's config changed forcing a regen.

**Fix:**
- Always use `docker start batos-chromium-build` (not `docker run`).
- Roll back to a snapshot that had progress:
  ```bash
  docker stop batos-chromium-build
  docker rm batos-chromium-build
  docker run -d --name batos-chromium-build \
    --memory=14g --memory-swap=20g --cpus=8 \
    batos-chromium-build:<last-good-snapshot-tag> \
    bash -c "cd /home/build/chromium/src && exec ninja -C out/BatOs -j2 content_shell"
  ```

## 11. GitHub push rejected due to >100 MB file

**Symptom:** `remote: error: File ... is X MB; this exceeds GitHub's
file size limit of 100.00 MB`.

**Causes we've hit:**
- `ports/chromium/.git/objects/pack/pack-*.pack` — the internal git
  metadata of the vendored Chromium clone. Excluded in `.gitignore`
  (we ignore `/ports/chromium` entirely).

**Fix:** Add the offending path to `.gitignore`, remove from index
with `git rm --cached <path>`, commit, try push again. If you
genuinely need the file in the repo, set up Git LFS.

## 12. Mac stuck on m1n1, can't get back to macOS

**Symptom:** Mac boots directly into m1n1 every time, no boot picker
offered, need to get back to macOS for Mac Claude work.

**Fix:** Hold the power button at startup for ~8 seconds — NOT a tap,
a hold. This shows "Loading startup options..." and then the boot
picker. Click Options → Continue → pick "Macintosh HD" (or whatever
your macOS volume is named) from the list. Back to macOS, no changes
to m1n1 install.

If power-button-hold isn't producing the picker, your m1n1 install
might have broken the macOS boot entry. Recovery: boot into Recovery
via the same long-hold trick, then in Terminal:
```bash
sudo bless --mount /Volumes/Macintosh\ HD --setBoot
```
Reboot — back to macOS as default boot target.
