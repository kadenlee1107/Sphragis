---
type: concept-note
topic: boot
---

# M4 boot path

> The single most-asked question about Sphragis is "wait, how does this even boot on Apple Silicon?" Asahi Linux's installer explicitly refuses M4 hardware as of 2026. Apple doesn't publish the boot interface. We boot anyway, via a chain that's been earned one fact at a time. This note is the one place the whole chain sits in order.

## The chain, in seven steps

1. **iBoot** runs (Apple's first-stage). Untouched. We don't fight it.
2. **macOS Recovery** is selected at boot via the standard boot-picker (hold the power button on the M4). One-fingerprint recovery, no fuses burned, no firmware re-flashed.
3. **`kmutil configure-boot`** was run once, in Recovery, with **Permissive Security** enabled. This installed the Asahi-style `m1n1` second-stage on the volume so a reboot picks it up.
4. **m1n1 stage 1** (vendored under [[external/m1n1]]) does the bring-up: parses ADT, pre-configures the AIC and PMGR, sets up the framebuffer, and exposes a USB proxy protocol for the host.
5. **m1n1 stage 2** is loaded over the USB proxy by the host running [[external/m1n1/proxyclient/tools/chainload.py]]. **The flag that matters here is `-S` / `--skip-secondary-cpus`** — without it, the M4 P-cluster SErrors on the RVBAR writes m1n1 normally does. The vendored copy of `chainload.py` has the flag pre-applied so we don't forget.
6. **Sphragis kernel image** (`target/aarch64-unknown-none/release/sphragis`) is uploaded by the same proxy and entered at its reset vector.
7. **`kernel_main_apple`** (in [[src/main.rs]]) is the first SPHRAGIS-authored function to execute on real M4 silicon. From there: cpu init, mmu setup, ADT walk, driver bring-up, SealFS mount, lock screen.

## What the host computer does

The proxy needs Linux. Windows doesn't enumerate m1n1's composite USB device without a vendor INF that Apple/Asahi don't publish; we tried, it's not worth fighting. macOS works (and is in fact the same machine that holds the M4 boot volume), but during a chainload session macOS is not running — the host must be a different machine. In practice, the persistent Ubuntu dev host pulls this duty.

Do not use `run_guest.py`, m1n1's other entrypoint. It initializes a hypervisor and writes `AMX_CONFIG_EL1`, which traps on M4 (no AMX; M4 uses SME). `chainload.py` is the supported path; `run_guest.py` is a footgun on this hardware.

## The first photo

Before the journal existed, the first thing that proved any of this worked was a photo of the M4 internal display showing `sphragis >` with the encrypted/offline/firewall status bar. That photo is kept at [[_generated/docs/photos/2026-04-17_first_m4_boot/INDEX.md]] (or in the repo at `docs/photos/2026-04-17_first_m4_boot/IMG_7118.jpg`). When power was lost mid-session, that photo was all that survived. It's the durable record that an M4 actually executed a kernel we wrote.

## Where the documented hex lives

Every register, every compatible string, every PMGR domain ID we depend on is transcribed into [[_generated/docs/M4_GROUND_TRUTH.md]]. When that document and Asahi documentation disagree, the document is authoritative — it was cross-checked against real hardware. Asahi's reference docs cover M1/M2 and were written before M4 shipped; they are useful as a starting point and dangerous as a final answer.

## Files in the chain (forward order)

- [[_generated/vendored/external_m1n1]] — the bootloader, vendored
- [[_generated/scripts/install_m1n1_on_m4.sh]] — Recovery-mode setup
- [[_generated/scripts/chainload.py]] — host-side proxy driver (note: the *real* one is `external/m1n1/proxyclient/tools/chainload.py`; scripts wrap it)
- [[_generated/src/main.rs]] — `kernel_main_apple`, the entry
- [[_generated/src/arch]] — aarch64 reset vector + early init
- [[_generated/src/drivers/apple]] — AIC, PMGR, ATC PHY, DCP, dockchannel UART, DWC3 XHCI

## What's still loose

- We have only chainloaded on **Mac16,1 / J604 / T8132 (Donan)**. Other M-series boards ship slightly different DCP versions; the boot transcript would tell us how much of the chain transfers.
- 11 of 12 PMGR domain IDs match Asahi's published table for the M4. One ATC variant doesn't. The boot log records which.
- No firmware re-flashing has been done and none is planned. The boot path is a *cooperative* one — Apple's iBoot agrees to hand off because Permissive Security says so. If Apple revokes that posture in a future macOS release, the chain breaks at step 3 and we re-evaluate.
