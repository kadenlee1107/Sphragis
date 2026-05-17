# DESIGN: Sphragis Productization UX Architecture

**Document version:** 1.0 (covers SP-UX-001 through SP-UX-010, 2026-05-16)
**Status:** Design lock for the full UX productization track. Each `SP-UX-N` from the master plan implements one chapter of this document.
**Audience:** UX implementers landing future sub-projects + gov buyers evaluating the "real OS" feel claim.
**Companion docs:** `DESIGN_CAVE_ISOLATION.md` (caves are the units a window manager exposes), `docs/OPERATOR_RUNBOOK.md` (operator-facing how-to that this UX makes accessible), `ANTI_FEATURES.md` (no AI-in-kernel, etc.).

## Why this document exists

Track D of the master plan is the entire productization UX layer:

| REQ | Sub-project | Status today |
|---|---|---|
| UX-001 | Multi-app concurrent UI + window manager | MISSING |
| UX-002 | Installer / boot ISO | MISSING |
| UX-003 | Settings / system-management app | MISSING |
| UX-004 | User accounts beyond lock-screen passphrase | MISSING |
| UX-005 | Package manager | MISSING |
| UX-006 | Analyst POSIX toolbox | MISSING |
| UX-007 | External display / multi-monitor | PARTIAL (DCP driver exists; no WM-side integration) |
| UX-008 | Bluetooth / WiFi userspace | PARTIAL (bcm_wifi driver exists; no UX) |
| UX-009 | Audit-review console | PARTIAL (security app exists; needs filter UI) |
| UX-010 | Cave-management console extensions | PARTIAL (caves_mgr exists; needs attestation/policy/quota UI) |

Today: 7 in-OS apps switched one-at-a-time via numeric keys. Lock-screen + single passphrase. No installer; install requires a Sphragis dev to build + chainload via m1n1. No package manager; no POSIX toolbox; no settings app.

For a gov buyer to take the demo seriously, this needs to feel like a "real OS." For an AO to approve deployment, the multi-user + audit-review + cave-management surfaces need first-class UI. Track D delivers both.

This document is the architectural plan. Each chapter is a future SP-UX-N that lands a piece.

---

## 1. UX-001 — Window Manager + Multi-app Concurrent UI

### Goal

Replace the current one-app-at-a-time numeric-key cycle with concurrent multi-app rendering. Each app runs in its own cave (already true). Each cave can render into its own window. Operator switches focus instead of switching app-state.

### Design choice: tiling

**Picked:** tiling window manager (Sway/i3/dwm model).

**Rejected:** floating (macOS/Windows-style overlapping windows).

**Why:** Tiling matches the cave model — caves are deterministic isolation containers, not free-form workspaces. Tiling makes the "which cave is which" question visually obvious. Floating windows would require z-order + occlusion management that the cave-isolation discipline doesn't want at the kernel level. Operator efficiency on keyboard-driven workflows is also higher with tiling.

### Architecture

```
┌─────────────────────────────────────────────┐
│ Compositor (src/ui/compositor.rs — NEW)     │
│  - damage tracking + region invalidation    │
│  - per-window framebuffer mappings          │
│  - hardware-accelerated via AGX on M4       │
│    where available; SW-rendered fallback    │
└─────────────────────────────────────────────┘
                    ↑
        per-window paint() callbacks
                    ↓
┌─────────────────────────────────────────────┐
│ Window manager (src/ui/wm.rs — NEW)         │
│  - tiling tree (binary partition)            │
│  - per-window cave_id binding                │
│  - input routing to focused window           │
│  - global keybinding handler                 │
└─────────────────────────────────────────────┘
                    ↑
                input
                    ↓
┌─────────────────────────────────────────────┐
│ Existing apps (src/ui/apps/*.rs)             │
│  - Refactor: each app's paint() takes a      │
│    WindowRect (already does — keep) +        │
│    operates within that rect, not full       │
│    screen                                    │
│  - Each app's per-cave reset_for_cave_switch │
│    hook stays — fires on cave switch         │
└─────────────────────────────────────────────┘
```

### Keybindings (configurable; defaults match Sway)

| Binding | Action |
|---|---|
| Mod+Enter | Spawn a new app in a new window (prompts for which cave) |
| Mod+Tab | Cycle focus through windows |
| Mod+H/J/K/L | Move focus left/down/up/right |
| Mod+Shift+H/J/K/L | Move window left/down/up/right |
| Mod+R | Toggle tiling/stacking layout for this group |
| Mod+Shift+Q | Close focused window |
| Mod+1..9 | Jump to workspace 1..9 (multi-monitor: per-monitor workspaces) |
| Ctrl+L | Lock (existing) |
| Ctrl+W | Emergency wipe (existing) |

Mod key defaults to Super; operator-configurable.

### Status bar

Per-monitor status bar at the top (or operator-configurable position). Components:

- Cave indicator: which cave owns the currently-focused window (cave_id + name)
- Attestation status: "rooted ✓" / "in-memory ⚠️"
- Audit event count: rolling count since last `audit-flush`
- Network status: per-cave or per-window
- Clock + uptime

### Implementation phasing

**SP-UX-001.IMPL.A** (minimal viable): one workspace, no multi-monitor, single tiling layout, default keybindings hard-coded. Validates the compositor + WM architecture.

**SP-UX-001.IMPL.B**: multi-monitor + per-monitor workspaces (uses M4 DCP for the multi-monitor side; SP-UX-007 lives here).

**SP-UX-001.IMPL.C**: configurable keybindings via Settings (depends on SP-UX-003).

**SP-UX-001.IMPL.D**: status-bar customization.

---

## 2. UX-002 — Installer / Boot ISO

### Goal

A first-time operator can install Sphragis on a target device without needing a Sphragis-dev at the keyboard.

### Architecture

Two delivery formats:

| Target | Format | Tool |
|---|---|---|
| x86_64 (future, SP-HW-002) | UEFI-bootable ISO | `genisoimage` or `xorriso` produces hybrid ISO (boots from USB + DVD) |
| M4 (today's primary) | m1n1 chainload bundle | `.dmg` for macOS install-helper + raw `.img` for advanced operators |

### First-boot flow

```
Powered-on device with Sphragis installer media
        │
        │ 1. Bootloader loads installer kernel
        │
        │ 2. Installer presents:
        │    - Language selection
        │    - Hardware probe (lists detected RAM, storage, network adapters)
        │    - Operator-CA pubkey selection (paste / scan QR / generate)
        │    - Unlock passphrase setup (Argon2id-protected)
        │    - Initial cave creation (defaults: "primary" cave with default policy)
        │    - Audit-trail seal: first chain head emitted to UART for operator to capture
        │
        │ 3. Installer writes:
        │    - Sphragis kernel + initial cave config to BatFS
        │    - Operator-CA pubkey at /attest/operator-ca.pub
        │    - First-time-init marker (consumed by next-boot kernel to skip setup)
        │
        │ 4. Reboot into installed Sphragis
```

### Implementation phasing

**SP-UX-002.IMPL.A** (M4 path first): m1n1-chainload bundle + macOS install-helper `.dmg`. Operator uses macOS Recovery + custom-build m1n1 to install. Validates the installer flow without needing x86_64 hardware.

**SP-UX-002.IMPL.B** (x86_64): depends on SP-HW-002 x86_64 port landing first. Then add UEFI ISO build.

**SP-UX-002.IMPL.C**: TPM-sealed / SEP-sealed installer state (so a stolen installer media can't be replayed against another device).

---

## 3. UX-003 — Settings / System Management App

### Goal

Unified UI for the operator-facing configuration surfaces.

### Sections

| Section | Reads from / writes to |
|---|---|
| Networking | `src/net/firewall.rs`, `src/net/cave_policy.rs`, `src/net/cave_shaper.rs` |
| Audit log review | `audit::recent`, `audit::recent_for_cave`, audit-flush, audit-seal |
| Cave management | `src/caves/cave.rs` (create/destroy/enter); links to UX-010 console |
| Attestation status | `src/security/attest.rs::current_cave_identity`, kernel measurement, endorsement validity |
| Updates | (UX-005 dependency); shows update-status, package-list, last-applied |
| User accounts | (UX-004 dependency); shows registered users + TPI officers |
| Crypto status | `cmd_sec_status` + on-demand `lms-kat` / `attest-smoke` triggers |
| Time & locale | NTP-style time sync (or operator-pinned offset); keyboard layout; date format |

### Implementation phasing

**SP-UX-003.IMPL.A**: read-only first cut — Settings displays current state of all sections. No edits.

**SP-UX-003.IMPL.B**: read-write for non-privileged settings (keyboard layout, audit-display preferences).

**SP-UX-003.IMPL.C**: read-write for privileged settings (network rules, cave policies) — gated by TPI quorum.

---

## 4. UX-004 — Multi-user Accounts

### Goal

Replace the single-passphrase model with per-user identity backed by operator-CA attestation.

### Identity model

Each user has:
- A unique `user_id` (UUID)
- A human-readable username
- A passphrase (Argon2id-protected)
- An optional hardware-key second factor (FIDO2-style)
- A set of capabilities (which caves they can enter; which TPI roles they hold)
- A creation audit trail (who created, when)

User-account state stored in BatFS at `/users/<user_id>/` — per-user-encrypted under a key derived from `(operator-CA-attested-master, user_id, user_passphrase)`. Master-CA-attested means the user roster is itself attestable; an attacker can't add ghost users.

### Privilege escalation

Replaces today's coarse "single operator" model:

| Role | Capabilities |
|---|---|
| **Operator** (today's default) | Same as current — full system access |
| **CryptoOfficer** | TPI role for key-rotation + master-key ops |
| **AuditOfficer** | TPI role for audit-wipe + audit-seal |
| **CaveAdmin** | Create / destroy caves; manage cave policy |
| **User** | Use existing caves they're authorized to enter; cannot create new caves |

A user holds one or more roles. Two-person-integrity ops still require quorum from distinct users in the required-role pair.

### Implementation phasing

**SP-UX-004.IMPL.A**: data-model + storage (multi-user `/users/` directory + per-user BatFS encryption).

**SP-UX-004.IMPL.B**: lock-screen username + passphrase UI (replaces today's single-passphrase prompt).

**SP-UX-004.IMPL.C**: hardware-key second-factor (FIDO2 / WebAuthn).

**SP-UX-004.IMPL.D**: capability-set + role mgmt UI (Settings section in SP-UX-003).

---

## 5. UX-005 — Package Manager

### Goal

In-OS install / update / remove of additional Sphragis-blessed packages. The vehicle for shipping POSIX toolbox (SP-UX-006), updates (UpdateApply audit category), and operator-extensions.

### Protocol

**TUF** (The Update Framework) over HTTPS with CNSA-2.0 cipher suites only (gov-strict). Why TUF:

- Standard for software-package supply chain integrity
- Multi-key threshold for repository signing (operator can't forge updates with one stolen key)
- Snapshot + targets + timestamp roles separate concerns
- Compatible with sigstore-signed targets (per `DESIGN_SIGSTORE_REKOR.md`)

### Package format

- Tarball with manifest (name, version, dependencies, capabilities-required, LMS signature)
- Installed into a per-package read-only mount inside a target cave (chosen by operator at install time)

### Repository

Sphragis-blessed repository hosted at `packages.sphragis.org` (operator-installable mirror infrastructure). Initial package set is the analyst POSIX toolbox from UX-006.

### CLI + UI

- CLI inside any cave: `sphragis-pkg install <name>` / `update` / `remove`
- UI in Settings (SP-UX-003) section "Updates"

### Implementation phasing

**SP-UX-005.IMPL.A**: read-only TUF client + offline-bundle install (operator places packages in `/packages/` manually).

**SP-UX-005.IMPL.B**: HTTPS-fetched packages (gated on SP-B1.6.1.tls — TLS to a remote peer with gov-strict).

**SP-UX-005.IMPL.C**: dependency-resolution + version-pinning.

---

## 6. UX-006 — Analyst POSIX Toolbox

### Goal

Ship the daily-driver POSIX tools a security analyst expects.

### Minimum viable toolset

| Tool | Purpose |
|---|---|
| vim / nano | Text editing |
| git | Version control |
| python3 (CPython) | Scripting + analysis |
| ssh / scp (OpenSSH) | Remote shell |
| tmux | Terminal multiplexing |
| curl + jq | HTTP + JSON inspection |
| coreutils equivalents (grep / sed / awk / find) | BusyBox or full GNU coreutils |
| make | Build tooling |
| bash (or dash) | Shell |

### Distribution

Via SP-UX-005 package manager. Each tool packaged separately so operator picks what they need. Sphragis-gov ships the toolbox default-installed; community is opt-in.

### Linux ABI shim

Each tool runs inside a cave under the Linux ABI shim (`src/caves/linux/`). The shim is already partial; SP-UX-006 extends to whatever syscalls these specific tools require. **Bound expansion**: do NOT extend the shim to "run everything." Each new syscall added is a deliberate decision documented in the shim's accepted-surface table.

### Implementation phasing

**SP-UX-006.IMPL.A**: cross-compile + package vim + tmux + curl + jq + busybox coreutils.

**SP-UX-006.IMPL.B**: python3 + git (heavier dependencies).

**SP-UX-006.IMPL.C**: ssh (network-side hardening + cave-policy validation).

---

## 7. UX-007 — Multi-monitor

### Goal

Use external displays connected via M4 HDMI/DisplayPort.

### Architecture

The M4 `src/drivers/apple/dcp.rs` driver already supports the display coprocessor surface. SP-UX-007 wires the window manager (SP-UX-001) to:
- Detect each connected monitor
- Treat each monitor as an independent workspace set
- Drag-between-monitors gesture (Mod+Shift+arrow)

### Implementation phasing

Lands as SP-UX-001.IMPL.B (per §1).

---

## 8. UX-008 — Bluetooth / WiFi Userspace

### Goal

Settings-app UI for the existing `src/drivers/apple/bcm_wifi.rs` driver (and a future BT driver).

### Why BT is P2 (not P0)

Gov SCIF deployments typically disable Bluetooth. BT support is a low-priority polish item.

### WiFi UX scope

- Scan + present available networks
- Connect (operator selects + provides passphrase)
- Configure per-cave WiFi-access policy (which caves can use WiFi at all)

### Implementation phasing

**SP-UX-008.IMPL.A**: WiFi scan + connect UI in Settings.

**SP-UX-008.IMPL.B**: per-cave WiFi-access policy enforcement at the firewall layer.

**SP-UX-008.IMPL.C** (P2): Bluetooth pairing + per-cave BT-access policy.

---

## 9. UX-009 — Audit-review Console

### Goal

A dedicated app surfacing the audit ring with filtering + chain-integrity verification.

### Features

- Live tail of recent events (paginated)
- Filter by category, severity, time range, cave_id
- Per-cave view (uses `audit::recent_for_cave` from SP-ISO-009)
- Inline chain-verify (`audit::verify_chain`)
- Trigger `audit-flush` from the UI (TPI-gated)
- Trigger `audit-seal` from the UI (TPI-gated)
- Export to operator's SIEM (HTTPS POST per cave-policy)

### Implementation phasing

**SP-UX-009.IMPL.A**: refactor existing `src/ui/apps/security.rs` to be the audit-review console. Add filter UI.

**SP-UX-009.IMPL.B**: SIEM export integration.

---

## 10. UX-010 — Cave-management Console Extensions

### Goal

Extend the existing `src/ui/apps/caves_mgr.rs` to surface the post-2026-05-16 cave primitives.

### Features added beyond current caves_mgr

- Per-cave attestation status (registered via `attest::register_cave_identity`?)
- Per-cave information-flow policy editor
- Per-cave resource quotas display + edit (memory, CPU time, network bandwidth)
- "Freeze cave" for forensic capture (snapshot of cave state without destroying)
- Per-cave audit-subset view (links to UX-009 console pre-filtered)

### Implementation phasing

**SP-UX-010.IMPL.A**: attestation-status + policy editor.

**SP-UX-010.IMPL.B**: resource quotas.

**SP-UX-010.IMPL.C**: freeze-cave forensic snapshot (requires careful zeroize discipline + memory-map serialization).

---

## Cross-cutting: accessibility + internationalization

Not first-class in this design today. Documented as REQ-UX-NEW-001 to add in a future spec revision.

## Cross-cutting: gov-strict UI restrictions

Under `gov-strict`, the package manager (UX-005) only fetches from operator-signed mirrors; the WiFi UI (UX-008) defaults to disabled; the Settings app (UX-003) shows the gov-strict mode prominently in the status bar.

## REQ traceability

This document is the design lock for REQ-UX-001 through REQ-UX-010. Each REQ remains MISSING (or PARTIAL where indicated) until its corresponding SP-UX-N.IMPL.* lands.

## References

- Sway window manager: https://swaywm.org/ — closest model for the cave-bound tiling pattern
- TUF (The Update Framework): https://theupdateframework.io/
- BusyBox: https://www.busybox.net/
- FIDO2 / WebAuthn: https://www.w3.org/TR/webauthn-3/
- DESIGN_CAVE_ISOLATION.md — the underlying cave primitive each window-bound app uses
- DESIGN_HSM_OPERATOR_CA.md — the attestation chain UX-004 user identities tie into
- DESIGN_LMS_KERNEL_SIGNING.md — the signing chain UX-005 package signatures use
