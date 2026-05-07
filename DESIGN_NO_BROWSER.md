# DESIGN: Bat_OS Has No Browser

**Status:** Active strategy as of 2026-05-07.
**Supersedes:** `DESIGN_BROWSER.md`, `DESIGN_CHROMIUM.md`.

## The decision

Bat_OS is **a secure workstation, not a secure laptop**. Web browsing
happens on the host. Bat_OS never ships a browser.

The 30K+ lines of browser code currently in tree (native engine,
Ladybird port, stream-client) are deleted.

## Why

### Security-first eats every other principle

The project's mandate is "every byte auditable" for government and
private-client deployments. A browser violates that mandate at every
sane scope:

- **Chromium-style** drags 30M lines of unaudited C++ into the trust
  boundary. Every Chrome CVE is your CVE. Antithetical to the mission.
- **Ladybird-style** is smaller (~32 MB C++ + 2 MB Rust) but still
  introduces a large, third-party C++ codebase that you do not own
  the security posture of. Even sandboxed in a cave, the cave is your
  attack surface, not theirs.
- **Stream-client-style** (Mac-side Chromium displaying pixels in
  Bat_OS) makes you structurally dependent on a Mac you cannot audit.
  Whatever Chrome's CVE list is for the month, that's your CVE list,
  except the bug runs on hardware outside your control.
- **Native pure-Rust engine** is the only browser that fits the
  security mandate. But "build a real web engine" is a 1–2 year
  effort to reach modern parity, and *modern parity is the wrong
  goal anyway* — see next point.

### "Browser" is a wrong success metric for this OS

The implicit target — "Chrome but secure" — is wrong. A government /
private-client user on a security-first workstation does not need to
render TikTok. The web they actually need (banking portals,
internal dashboards, mail clients, document viewers) can be opened
on their Mac. Bat_OS's purpose is the things the Mac *can't* be
trusted with.

### Scope discipline > feature creep

Three browser paths exist because no single one was sufficient and
each addressed a different perceived gap. That's a symptom of an
unclear product identity, not three good options. Removing the
browser collapses the ambiguity and lets the OS focus on what it
*does* uniquely well.

## What Bat_OS is instead

Bat_OS is a **secure workstation / personal HSM**:

- **Cryptographic operations** — keys never leave the OS. Argon2id,
  Ed25519, P-384, RSA-PSS, ML-KEM-768, ML-DSA. Already in tree.
- **Secure file management** — BatFS encrypted filesystem; persistent
  caves; auditable VFS. Already in tree.
- **Secure document handling** — view, sign, encrypt, decrypt local
  files. No web fetching.
- **Secure communications** — TLS 1.3 + hybrid PQ, audited stack.
  Used for explicit machine-to-machine work, not browsing.
- **Vetted workload execution** — caves run specific Linux ELFs that
  the user explicitly trusts. Not arbitrary code from the internet.

Web browsing is **explicitly delegated to the host**. The user has a
Mac (or Linux box, or whatever); they browse there. Bat_OS occupies
the slot of "the trusted device next to the laptop," not "the laptop."

## What gets removed

Concretely:

- `src/browser/` (~16K LOC) — native HTML/CSS/JS engine, paint pipeline,
  media decoders for web content, font rasterizer used by browser path.
- `src/ui/apps/browser.rs` (~2.4K LOC including the in-flight remote-mode
  changes) — Browser app from the WM.
- `ports/ladybird_port/` (346 MB build artifacts + scaffolding,
  Dockerfile, build.sh, baked initrd).
- `ports/chromium_port/` (306 MB build artifacts).
- `ports/chromium/` (1.3 GB source snapshot — already gitignored,
  delete the local working copy).
- NetSurf reference tree (`ports/netsurf/`, `ports/libcss/`,
  `ports/libdom/`, `ports/libhubbub/`, `ports/libnsfb/`,
  `ports/libparserutils/`, `ports/libwapcaplet/`, `ports/libnsutils/`)
  — never integrated, no longer reference for anything.
- `scripts/browser_proxy.py` — Mac-side stream-client proxy.
- `scripts/qemu_ladybird_*.py`, `scripts/qemu_chromium_*.py` — Ladybird
  and Chromium QEMU drivers.
- `tools/bake_ladybird_initrd.sh`, `tools/bake_chromium_*.sh`.
- Shell commands: `web`, `webwin`, `ladybird`, `ladybird-js`,
  `ladybird-dump`, `chromium`, `chromium-version`, `render`,
  `dump-dom` — gone from `src/ui/shell.rs`.
- `docs/LADYBIRD_AUTOPILOT.md` — autopilot is no longer driving
  Ladybird iters. (Autopilot infrastructure can stay; it'll target
  other STUMPs.)
- The current 1,613-LOC uncommitted stream-client iter 1 work — **not
  committed**. Discarded as part of this decision.
- `/batos/fb0` ChromiumFb VFS node + `chromium_blit` kthread in
  `src/drivers/virtio/gpu.rs` — the path was specifically for browser
  rendering. WM still uses virtio-gpu directly without the blit-on-
  write path.

## What stays

- The TLS 1.3 + hybrid-PQ stack in `src/net/tls.rs` etc. **Used for
  non-browser TLS** (machine-to-machine, secure file transfer).
- The HTTPS fetch helpers in `src/net/fetch.rs` etc. Same justification.
- The TrueType font rasterizer in `src/ui/{font,truetype}.rs`. Used by
  the WM/shell, not just the browser.
- All crypto, BatFS, caves, syscall layer. Untouched.
- The autopilot infrastructure in `scripts/autopilot.sh` —
  reusable for other tracks.

## What we're explicitly NOT doing

- **No "local HTML viewer."** Hard delete means hard delete. If you
  need to read an HTML doc, open it on the Mac. Adding a local HTML
  viewer reintroduces a parser-based attack surface for web content,
  which is the thing we just decided to remove.
- **No PDF reader (in this iteration).** PDF is its own beast and a
  separate decision; not bundled into this one. If a clear use case
  arises later, it's a separate spec.
- **No markdown-renders-like-a-browser creep.** Markdown displayed
  in the editor or shell is fine; markdown rendered to a styled
  page (with images, links, etc.) starts pulling in the web layout
  surface area we just removed.
- **No "Ladybird in a totally locked-down cave for special users."**
  We considered and rejected the escape-valve framing. If it's in the
  tree it's in the trust boundary. Out means out.

## Reversibility

This decision is documented and the deletion happens via git, so the
work is not lost: `port/ladybird` HEAD as of `2d434b32` preserves the
full Ladybird port + native engine + stream-client. Anyone can revive
any of the three by checking out a tag at that commit. Tag
`pre-no-browser-2026-05-07` will be applied immediately before the
deletion commits land.

If a future use case makes a strong case for re-introducing local
HTML rendering, this doc gets superseded by a new spec that explains
what changed. Until then: no browser.

## Implementation plan

A separate plan doc handles the actual deletion (which files, what
order, what tests verify nothing else broke, how to handle the
shell-command removal cleanly). This design doc is the *why*; the
plan is the *how*.

🦇
