# First Bat_OS boot on M4 — 2026-04-17

Phone photos of the M4 display captured during the session where Bat_OS
first booted end-to-end on real M4 hardware via m1n1 chainload. Power
went out before the Ubuntu session could save anything; these photos
are the durable record of what worked.

All hex addresses, register layouts, and compatible strings visible in
these photos are transcribed into `../../M4_GROUND_TRUTH.md`. When the
text conflicts with what you see here, the photo is authoritative.

## Photo index

| File | Contents |
|---|---|
| `IMG_7118.jpg` | Bat_OS microkernel shell rendered on M4 display. `bat_os >` prompt, keybindings, "ENCRYPTED \| OFFLINE \| FW:DENY_ALL" status bar. **Proof of full boot.** |
| `IMG_7141.jpg` | `[usb] DWC3 bring-up: usb-control_nodes:` — three DWC3 controllers (drd0/1/3) with SNPSID, HCSPARAMS. |
| `IMG_7150.jpg` | XHCI bring-up output — `dwc3=OK dart=OK halt=OK reset=OK buf=OK start=OK portrst=OK`, USBSTS/PORTSC values. |
| `IMG_7157.jpg` | `[adt-scan] input devices:` — spi2/mesa biosensor, spi4/dp855 parade bridge, all three usb-drd controllers. |
| `IMG_7164.jpg` | DWC3 bring-up with control-node dump — caplen/ver/hcs1 per controller, and atc1/2/3 dpin/dpxbar/dphy families (t8132 + t602x variants). |
| `IMG_7177.jpg` | `[atc-phy] register snapshot (read-only)` — atc-phy0/1/3 reg window values at +0x00..+0x1c. |
| `IMG_7179.jpg` | `[adt] USB-related property dumps` — full ps-regs / function-pmp_control tables (100s of lines of hex). |
| `IMG_7184.jpg` | `[adt] PMGR clock-gate discovery:` — gate IDs per device + matched pmgr.devices table (11/12 gates resolved). |
| `IMG_7187.jpg` | Same as 7184 + early `[pmgr] ATC3 gate enable (drd3 pre-req)` start. |
| `IMG_7195.jpg` | `[adt] /arm-io/atc-phy3 reconnaissance (full prop list)` — every property with hex-dumped value (reg, compatible, tunables). |
| `IMG_7199.jpg` | Same as 7184 (duplicate capture). |
| `IMG_7208.jpg` | PMGR ATC3 gate-enable register dance + start of ATC-PHY tunable replay. |
| `IMG_7213.jpg` | atc-phy3 property list continuation — ACUPHY/USPPLL/USDPHX tunables. |
| `IMG_7226.jpg` | PMGR gate-enable complete + USB2PHY_HOST tunable applied. |
| `IMG_7230.jpg` | Clean close-up of PMGR ATC3 gate-enable and USB2PHY tunable apply. |
| `IMG_7231.jpg` | atc-phy3 all-reg-window scan with USB2PHY_HOST before/after values. |

## Why this matters

Nobody else in the open-source world has booted a non-Apple OS on M4.
Apple doesn't document these registers, Asahi hasn't released M4
support, m1n1 has ~6 lines of M4 awareness. The hex addresses and
register sequences captured here are new research derived from a
live boot.
