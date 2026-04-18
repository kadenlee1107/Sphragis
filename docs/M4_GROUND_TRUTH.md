# M4 (T8132 "Donan" / Mac16,1) — Bat_OS Ground Truth

**Purpose.** The single source of truth for everything we have reverse-
engineered about Apple M4 silicon while bringing up Bat_OS. Every fact
here was either observed on real hardware running Bat_OS via m1n1
chainload, or derived from m1n1's (thin) M4 scaffolding plus Asahi docs
plus our own ADT reconnaissance.

**Authority.** If something in our source code contradicts this file,
this file is correct. Update source to match, not the other way around.

**Provenance.** Primary data comes from a Bat_OS boot session captured
in `/tmp/batos_photos/IMG_7118.jpg` through `IMG_7231.jpg` (source: the
user's iPhone photos of the M4 screen running Bat_OS). Every register
address, every hex value, every compatible string below was seen on
real hardware. The originals are JPEGs; this is the greppable form.

Format note: addresses are given with underscore separators for
readability (e.g. `0x0000_0003_8070_0000`), which is how Rust literals
are written. When transcribed to C `#define` form, drop the underscores.

---

## 1. Machine identity

| Field | Value |
|---|---|
| Marketing name | Apple M4 base |
| Apple codename | "Donan" |
| Internal name | H16G |
| SoC chip ID | `0x8132` (T8132) |
| Mac model | Mac16,1 |
| Device-tree compatible | `"apple,j604"` |
| Total CPU cores | 10 (4 P + 6 E) |
| P-core MIDR part | `0x53` (Avalanche or its M4 successor) |
| E-core MIDR part | `0x52` (Blizzard or its M4 successor) |
| Boot CPU smp_id | `0x6` (a P-core) |
| MIDR observed | `0x611f0531` |
| RAM base | `0x0000_0100_0000_0000` |
| RAM top (observed) | `0x0000_0103_db6c_8000` (config-dependent, ≈15.86 GiB here) |
| Page size | 4 KiB |

## 2. Security-relevant M4 facts

- **No AMX**. Apple removed AMX (Apple Matrix eXtension) on M4 and uses
  SME (Scalable Matrix Extension) instead. Any code that writes
  `AMX_CONFIG_EL1` (e.g. m1n1's HV init in `hv/__init__.py` line ~1435)
  traps as a SYNC exception and kills the boot. We bypass by using
  `chainload.py` (no HV) rather than `run_guest.py` (has HV init).
- **P-cluster RVBAR SError.** m1n1's `chainload.py` tries to write
  RVBAR for every non-running CPU. On M4, writes to E-cluster CPU
  implementation registers at `0x210xx` succeed; writes to P-cluster
  at `0x211xx` SError, tearing down the serial link. We patch chainload
  with `--skip-secondary-cpus` / `-S` (applied in
  `external/m1n1/proxyclient/tools/chainload.py`). Single-core boot is
  fine for bring-up.
- **MCC and cpufreq are unknown versions.** m1n1 logs
  `MCC: Unsupported version:mcc,t8132` and
  `cpufreq: Chip 0x8132 is unsupported`. Non-fatal — m1n1 uses safe
  defaults. Long-running loads *have* caused spontaneous resets,
  which we suspect is the watchdog biting because cpufreq ramping
  isn't working. Avoid sustained full-tilt CPU until we port the
  tunables.

---

## 3. Memory map — **real, observed addresses**

All addresses below confirmed from ADT walks on live M4 hardware.

### 3.1 Core blocks

| Block | Base | Size | Notes |
|---|---|---|---|
| **PMGR (Power Manager)** | `0x0000_0003_8070_0000` | per-device | device regs at `pmgr_base + 0x00_04a8+` etc., see §6 |
| **Framebuffer** | `0x0000_0103_e005_0000` | 3024×1964×4B stride `0x2f40` | m1n1 leaves this active; 30bpp |
| **Dockchannel UART** | `0x0000_0003_8812_8000` | — | This is what m1n1 actually uses; NOT the classic PL011 at 0x9000000 (that's QEMU virt). Our Apple UART driver needs to target this. |

### 3.2 USB (DWC3 + XHCI + ATC PHY)

We found **three** USB-DRD (dual-role device) controllers on M4. They
skip index 2 — numbering is 0, 1, 3.

| ID | Controller base | DART base | ADT compatible |
|---|---|---|---|
| drd0 | `0x0000_0004_0228_0000` | `0x0000_0004_02f0_0000` | `usb-drd,t8132` |
| drd1 | `0x0000_0004_0a28_0000` | `0x0000_0004_0af0_0000` | `usb-drd,t8132` |
| drd3 | `0x0000_0004_1a28_0000` | `0x0000_0004_1af0_0000` | `usb-drd,t8132` |

Each controller has:
- `caplen=0x20`, `ver=0x0110`, `hcs1=0x0200047f` → max 127 slots, 2 ports
- Ports: `port-hs` (USB 2.0 high-speed) + `port-ss` (USB 3.x super-speed)
- `snps=0x33313130` in SNPSID register (`"3113"` ASCII — DWC3 IP rev)

ATC (Asynchronous Transmitter Controller) — USB-C Type-C mux + DP alt:

| Device | Base | Compatible |
|---|---|---|
| atc1-dpin0 | `0x0000_0000_495e_5000` | `atc-dpin,t8132` |
| atc1-dpin1 | `0x0000_0000_495e_5000` (same reg, pair) | `atc-dpin,t8132` |
| atc2-dpxbar | `0x0000_0000_41304_c800` | `atc-dpxbar,t602x` — note: **t602x, not t8132**. Apple shares this block between M4 base and M4 Pro/Max. |
| atc2-dphy0 | (no reg — logical node) | `atc-dphy,t8132` |
| atc3-dpxbar | `0x0000_0000_41b04_c800` | `atc-dpxbar,t602x` |

ATC PHY instances (the things carrying USB-C signalling):

| PHY | Base | Compatible |
|---|---|---|
| atc-phy0 | `0x0000_0004_02a3_0000` | `atc-phy,t8132` |
| atc-phy1 | `0x0000_0004_0aa3_0000` | `atc-phy,t8132` |
| atc-phy3 | `0x0000_0004_1aa5_0000` | `atc-phy,t8132` |

### 3.3 Other peripherals seen in ADT scan

| Device | Compatible | Notes |
|---|---|---|
| spi2 | `spi-1,spimc` | Hosts `mesa` biosensor child |
| spi4 | `spi-1,spimc` | Hosts `dp855 [parade,DP855]` display-bridge child |

### 3.4 What's still unknown (look up on next boot)

We do NOT yet have the real address of:
- AIC2 (current fallback in `soc.rs` is `0x2_8e10_0000` — almost certainly wrong, came from M1 docs)
- SEP
- ANS / DART-ANS
- DCP / DART-DISP0
- SMC / SIO / AOP

`discover_from_adt` will find these when we boot the next binary and
the ADT paths match. If they don't match, update the paths in
`src/drivers/apple/soc.rs::discover_from_adt`.

---

## 4. ATC PHY register window — atc-phy3 snapshot

Read live from `0x4_1aa5_0000` during Bat_OS boot. Identical values
observed on atc-phy0 and atc-phy1 — these are the factory-default
initial values.

### 4.1 reg[0] (4-dword window starting at base)

| Offset | Value | Name (inferred) |
|---|---|---|
| `+0x00` | `0x0000_0182` | USB2PHY_HOST/config — **this is what gets tuned** |
| `+0x04` | `0x0000_0000` | |
| `+0x08` | `0x01c0_0001` | |
| `+0x0c` | `0x0000_0000` | (unused) |
| `+0x10` | `0x0000_0000` | (unused) |
| `+0x14` | `0x0120_0120` | |
| `+0x18` | `0x0023_0113` | |
| `+0x1b` | `0x0616_fb10_0` | (truncated 32-bit actually — check alignment) |
| `+0x1c` | `0x000c_0813` | |

After applying `tunable_USB2PHY_HOST`:
- `+0x00` transitions `0x01c8_000f` → `0x01c8_700f` (this is from reg[0] on live boot; the above "pre" row shows it at `0x01c8_000f` once PMGR gates enabled it)
- mode bits `0x7003` at +0x00 = HOST mode

### 4.2 atc-phy3 ADT properties (complete)

Observed property list (size-in-bytes in brackets):

```
reg                       [64]    — MMIO windows: 0x1aa5_0000 + sub-regions
compatible                [14]    "atc-phy,t8132"
function-dock_parent       [8]    {phandle=0x1ab, fn="accP"}
AAPL,phandle              [4]    0x13c
instance                  [4]    0x3
port-number               [4]    0x3
clock-gates               [8]    0x80000130 0x80000100
device_type               [8]    "atc-phy"
port-type                 [8]    0x0000_7003  ← USB2 mode bits
tunable-ATCXRCV2AP       [500]   (elided from dump)
tunable-ATCXRCV2AP_LOCK_LOCK [56]  0x200012fc ...
tunable-ATCXRCV2AP_HR_DIS    [56]  0x200011f0 0x200014fc ...
tunable-ATCXRCV2AP_LIDA_ION  [56]  0x20000020 0x00000072d 0x2c 0x07 ...
tunable-ATC_FABRIC          [850]  (elided)
tunable-ATC_COMMON_CFG      [12]   0x20000c0 0x00 0x7c 0x30
tunable-USSCRY_REV          [12]   0x2000030 0x7003 0x00
tunable-USSCRY_MEXT         [12]   0x2000030 0x2020 0x00
tunable-USSCRY_USA          [12]   0x2000030 0x20000140 0x00
tunable-USSCRY_TOP          [24]   0x2000030 0x838 0x00 0x17
tunable-USPPLL_CORE         [72]   0x20000024 0x00 0x20000030 0x17 0x00010000 (+7 more)
tunable-USDPHX_FLL_TOP      [12]   0x20000030 0x100 0x00 (+16 more)
tunable-USDPHX_TOP          [12]   0x20000094 0x100 0x101 0x00 0x11
tunable-ACUPHY_FLL_TOP      [12]   0x20000030 0x00 0x00 0x11
tunable-ACUPHY_LANE_USSCX   [12]   0x20000000 0x0c00000 0x41000000 0x07
tunable-CID_SHM             [12]   0x20000084 0x001c0700 0x00000070 0x00
tunable-ACUPHY_USBON_LANE_CMD_USB_STUT [340]  0x20000049 0x000100c1 0x00 0x20000010 0x00ffffff
tunable-LN0_RX_TOP_USB_HPLF [340]  0x20022c0 0x20000000 0x33300003 0x20000030 0x00220033
```

**Action item for Phase 3.4b**: port these tunables into
`src/drivers/apple/dwc3.rs` / a new `atc_phy.rs`. m1n1 does not ship
them for t8132 — we derived them from ADT.

---

## 5. DWC3 + XHCI bring-up — observed sequence

Our Phase 3.4 DWC3 skeleton + Phase 3.1 DART bypass successfully got
XHCI to come up on all three controllers. Captured:

```
[usb] DWC3 bring-up: usb-control_nodes:
  drd0 @ 0x0000_0004_0228_0000 OK snps=0x33313130
      caplen=0x20 ver=0x0110 hcs1=0x0200047f ports=2
  drd1 @ 0x0000_0004_0a28_0000 OK snps=0x33313130  [same]
  drd3 @ 0x0000_0004_1a28_0000 OK snps=0x33313130  [same]

[usb] XHCI bring-up:
  drd0 ctl=0x02280000 dart=0x02f00000
      dwc3=OK dart=OK halt=OK reset=OK buf=OK start=OK portrst=OK
      USBSTS=0x0000_0018 PORTSC1=0x0020_02a0 slots=127 ports=2
  drd1 ctl=0x0a280000 dart=0x0af00000   [same shape]
  drd3 ctl=0x1a280000 dart=0x1af00000   [same shape]
```

Where `ctl` is the main DWC3 controller MMIO and `dart` is the DMA
translation DART in front of it. Both must be mapped before XHCI can
transfer anything.

TRB ring init captured:
```
TRB[0] cycle=1 type=34 (PSCE port=1) w0=0x01000000
TRB[1] cycle=1 type=34 (PSCE port=2) w0=0x02000000
```

where `type=34` = Port Status Change Event.

---

## 6. PMGR — clock-gate discovery and enable sequences

`pmgr_base = 0x0000_0003_8070_0000`.

Per-device gate registers sit at `pmgr_base + 0x004a0` onward, one u32
per device. Each register has layout:

```
[31:12]  reserved (usually 0x00000)
[11:4]   ACTUAL power-state (current)
[3:0]    TARGET power-state
```

Write to flip TARGET; poll until ACTUAL matches. `0x0f` = fully on.

### 6.1 Observed PMGR gate-enable sequence for ATC3 (USB3 prerequisite)

This is the exact dance Bat_OS needs to perform on real M4 before
drd3 / atc-phy3 are usable. No comparable code exists in any public
reference for this chip.

```
# Before:
ATC3_CIO      @ pmgr_base + 0x004a8 = 0x0000_0300  TARGET=0x00 ACTUAL=0x00
ATC3_CIO_PCIE @ pmgr_base + 0x004b0 = 0x0000_0300  TARGET=0x00 ACTUAL=0x00
ATC3_CIO_USB  @ pmgr_base + 0x004b8 = 0x0000_0300  TARGET=0x00 ACTUAL=0x00

# Write 0x0f to TARGET bits of ATC3_CIO (clears reserved, sets TARGET).
# Poll. A few µs later:
ATC3_CIO      @ pmgr_base + 0x004a8 = 0x0000_03ff  TARGET=0x0f ACTUAL=0x0f  OK
```

### 6.2 Observed clock-gate assignments (from ADT)

```
usb-drd3  clock-gates = 0x80000104 0x8000010d 0x80000176 0x80000179
atc-phy3  clock-gates = 0x80000130 0x80000100
dart-usb3 clock-gates = 0x8000012c 0x8000012c
acio3     clock-gates = 0x00000081 0x80000003 0x00000004 0x80000155
```

High bit `0x80000000` appears to be a "present/valid" flag; the low
bits are the PMGR device ID (matching the `id=` column below).

### 6.3 PMGR devices table (matched IDs so far)

From ADT walk: "matched 11 of 12 gate IDs; devices table has 392
entries." The 11 we resolved:

| id | id1 | id2 | psreg | psidx | name |
|---|---|---|---|---|---|
| `0x0081` | `0x00` | `0x0081` | 4 | `0x0014` | `ATC3_CIO` |
| `0x0083` | `0x00` | `0x0083` | 4 | `0x0016` | `ATC3_CIO_PCIE` |
| `0x0084` | `0x00` | `0x0084` | 4 | `0x0017` | `ATC3_CIO_USB` |
| `0x0155` | `0x00` | `0x0155` | 0 | `0x0000` | `CIO3_RECONFIG-V` |
| `0x0130` | `0x00` | `0x0130` | 0 | `0x0000` | `AUSB3_AONUSB-V` |
| `0x012c` | `0x00` | `0x012c` | 0 | `0x0000` | `ATC3_USB_DART` |
| `0x0104` | `0x00` | `0x0104` | 0 | `0x0000` | `ATC3_USB-V` |
| `0x0108` | `0x00` | `0x0108` | 0 | `0x0000` | `ATC3_COMMON-V` |
| `0x0176` | `0x16` | `0x0176` | 0 | `0x0000` | `ATC3_DCS_F3` |
| `0x0179` | `0x16` | `0x0179` | 0 | `0x0000` | `ATC3_DCS_F6` |
| `0x018d` | `0x00` | `0x018d` | 0 | `0x0000` | `USB-LLT-V` |

One more (device ID → name) is expected but didn't match on this
boot; likely an ATC0 or ATC1 variant. TBD next session.

### 6.4 Rule for gate enable sequencing

Observed: **gates must be enabled in the order clock-gates are listed
on the device node**. For drd3:

1. `0x80000104` — ATC3_USB-V
2. `0x8000010d` — USB-LLT-V (this one is actually `0x18d` per table; we need to cross-check whether the 0x10d listed earlier was a transcription error)
3. `0x80000176` — ATC3_DCS_F3
4. `0x80000179` — ATC3_DCS_F6

Also dart-usb3 must come up before drd3 (DART in front of the USB DART).

---

## 7. ATC PHY USB2PHY_HOST tunable apply — the exact register write

Captured live:

```
[atc-phy] applying tunable_USB2PHY_HOST to atc-phy3:
  base=0x0000_0004_1aa5_0000  +0x00
  before = 0x01c8_000f
  target = 0x01c8_700f
  after  = 0x01c8_700f   OK

USB2 mode bits 0x7003 → HOST
```

Interpreted:

- Reg at `base + 0x00` is a control/mode register.
- Bits `[15:12]` = USB2 mode selector.
  - `0x0` = off
  - `0x7` = HOST mode (drd3 acts as a USB host port)
- Setting these bits is required for the XHCI layer to see plugged-in devices.

---

## 8. Compatible-string inventory

Every `compatible =` value we have confirmed on M4. Ordered roughly
by subsystem. Use this list when choosing `discover_from_adt` path
strings.

| Subsystem | compatible |
|---|---|
| USB controller | `usb-drd,t8132` |
| USB DART (IOMMU) | (shares `apple,t8110-dart` family) |
| ATC display-bridge input | `atc-dpin,t8132` |
| ATC DP crossbar | `atc-dpxbar,t602x` (shared with M4 Pro/Max) |
| ATC DP PHY | `atc-dphy,t8132` |
| ATC PHY | `atc-phy,t8132` |
| SPI | `spi-1,spimc` |
| Biosensor | `biosensor,mesa` |
| Display bridge | `parade,DP855` |

---

## 9. What Bat_OS code is validated on real M4

From Bat_OS's own boot log during the captured session:

- ✅ `src/drivers/apple/boot_args.rs::parse` — accepts real m1n1 boot
  args (revision=3, version=2 observed)
- ✅ `src/drivers/apple/adt.rs` — walks the real M4 ADT end-to-end,
  traverses `/arm-io/*`, iterates children at scale
- ✅ `src/drivers/apple/dart.rs::set_bypass` — successfully enables
  bypass on dart-usb0/1/3 (XHCI DMA works through it)
- ✅ `src/drivers/apple/dwc3.rs` — initializes all three DWC3
  controllers, reads the right SNPSID, sees the right HCSPARAMS
- ✅ Our font + DCP splash code — renders a readable microkernel
  shell on the actual M4 display
- ✅ Boot-stub .text.apple_boot section fix — `_apple_start` runs at
  offset 0 under chainload, no Linux-header collision

What ran but we haven't confirmed in detail:
- AIC2 init (probably used wrong base address from our fallback)
- SPI keyboard init (didn't get to reading input before power loss)

What still needs to happen (Phase 3.4b onward):
- Port the ATC PHY tunables above into a Bat_OS `atc_phy.rs`
- Implement the PMGR gate-enable dance as the FIRST thing we do
  before touching drd3
- Switch UART driver to dockchannel at `0x3_8812_8000` (not the
  M1-era PL011 address)
- Verify AIC2 base from the real ADT next boot, not the fallback

---

## 10. Known gotchas going forward

1. **Don't use `run_guest.py`** on M4 — it inits HV which writes
   AMX_CONFIG_EL1 → SYNC trap → boot dies. Use `chainload.py`.
2. **Always use `-S --skip-secondary-cpus`** on chainload. P-cluster
   RVBAR writes SError on M4.
3. **Our vendored `external/m1n1/proxyclient/tools/chainload.py`** has
   the `-S` flag baked in. Use that copy, not upstream.
4. **Don't use `linux.py`** either — needs a J604 DTB that doesn't
   exist AND calls smp_start_secondaries (same P-cluster SError).
5. **File transfers happen on Mac, testing on Ubuntu.** Windows USB
   stack can't enumerate m1n1's composite device properly — vendor
   INF for VID_1209/PID_316D isn't distributed. Skip it entirely.
6. **Long-running CPU loads can reboot the Mac spontaneously.** Likely
   watchdog biting due to unported cpufreq. Don't run heavy workloads
   in bring-up.
7. **The real FB stride is `0x2f40`, not `width * 4`.** Our proof-of-
   life red fill paints a diagonal-stripe pattern because of this —
   visible = success, pretty isn't required here.

---

## 11. Where to update Bat_OS source to USE this

When the user is next at the keyboard and we're iterating:

| File | What to change |
|---|---|
| `src/drivers/apple/soc.rs` UART fallback | `UART0_BASE_FALLBACK = 0x0000_0003_8812_8000` (dockchannel, not the old `0x3_ad20_0000`) |
| `src/drivers/apple/soc.rs` PMGR base | Add `PMGR_BASE = 0x0000_0003_8070_0000` as a discoverable peripheral |
| `src/drivers/apple/soc.rs::discover_from_adt` | Add paths for atc-phy0/1/3, atc-dpxbar, each drd |
| NEW `src/drivers/apple/pmgr.rs` | Gate-enable primitives using §6 register layout |
| NEW `src/drivers/apple/atc_phy.rs` | USB2PHY_HOST tunable from §7 |
| `src/drivers/apple/dwc3.rs` | Call pmgr.enable(ATC3_*) + atc_phy.host_mode() BEFORE DWC3 bring-up |

---

**This file should grow every session.** Any new hex address, register
value, ADT path, or quirk we observe gets appended here BEFORE we fix
the code. The photos are the historical record; this file is the
working inventory.
