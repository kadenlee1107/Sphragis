# Bat_OS Licensing Posture

**Status as of 2026-05-13.** Internal strategy doc. Lives in the
private repo. NOT a public LICENSE file.

## Today's state

- **Repo visibility:** PRIVATE on GitHub (`kadenlee1107/Bat_OS`).
- **No `LICENSE` file. No `license` field in `Cargo.toml`.**
- **Legal effect:** default copyright applies — *all rights reserved*
  to the copyright holder (Kaden Lee). Nobody may use, copy, modify,
  or distribute any part of this code without explicit written
  permission. This is the *most protective* possible default. The
  absence of a license is not an oversight; it's a deliberate hold
  on the licensing decision until we're ready to make it once.

## Plan when going public

When we open the repo (Phase A complete → Phase B rollout, weeks
from now), Bat_OS will be released under a **dual license**:

1. **AGPL-3.0** for open-source / research / non-commercial use.
2. **Commercial license** sold separately by Kaden Lee (the copyright
   holder) for any party that wants to use Bat_OS in a closed-source
   product or that finds AGPL's network-copyleft clause incompatible
   with their distribution model.

### Why this combination

| Concern | Why AGPL + commercial works |
|---|---|
| **Grant eligibility.** NLnet / Sovereign Tech Fund / Mozilla MIEL / GitHub Accelerator. | AGPL is OSI-approved open source; every relevant grant program accepts it. |
| **Defense / enterprise customers.** Don't want to open-source their integration; will pay for a commercial license. | The commercial license tier exists for exactly them. MongoDB, Sentry, GitLab, Mattermost all run this playbook. |
| **Discourage freeloading.** A contractor cannot quietly fork Bat_OS into a closed product without either open-sourcing their fork OR paying us. | AGPL's "network-use is distribution" clause closes the SaaS loophole that GPL leaves open. |
| **Future flexibility.** Can we still grant non-AGPL terms to a specific partner if it makes business sense? | Yes — as the sole copyright holder we retain the right to license our own code under any terms we choose. AGPL only binds *other people's* use, never our own. |

### Why NOT the alternatives

- **Apache-2.0 / MIT (permissive).** Anyone can fork Bat_OS into a
  closed commercial product, sell it, and owe us nothing. This
  maximizes adoption and forecloses our revenue option in one move.
- **BUSL / PolyForm Noncommercial (source-available).** Not
  OSI-approved → disqualifies us from NLnet, Sovereign Tech Fund,
  most other grants. The optics of "source visible" without the
  reality of "open source" buys us little and costs us grant money.
- **GPL-3.0 (no network clause).** Closes the binary-distribution
  loophole but leaves the SaaS loophole open — any cloud provider
  can host a Bat_OS-derived service without ever re-distributing
  the source. AGPL-3.0 covers both.

## Dependency policy

**Bat_OS source code may not absorb code under copyleft licenses
(GPL, AGPL, LGPL, MPL, EUPL, etc.).** Linking against a copyleft
crate transitively forces Bat_OS to inherit that license, which
would foreclose our commercial license tier — once a single AGPL
contribution lands, we can no longer offer commercial-license
terms to a partner without their separate AGPL compliance.

Permissive licenses are fine: MIT, Apache-2.0, BSD (2/3-clause),
ISC, Unlicense, CC0, MIT-0, zlib.

**Verified clean as of 2026-05-13.** A `cargo tree --no-default-features
--features gicv3 -e normal --format "{p} | {l}"` audit lists every
transitive dependency under permissive terms only. No GPL / AGPL /
LGPL / MPL anywhere in the tree.

A re-audit should run before every public release and after any
non-trivial `Cargo.toml` change. One-liner:

```sh
cargo tree --target aarch64-unknown-none --no-default-features \
  --features gicv3 -e normal --prefix none --format '{p} | {l}' \
  | sort -u | grep -iE 'gpl|agpl|copyleft|mpl|eupl'
```

Empty output = clean. Any matches require explicit review before
merge.

## When the time comes

1. Add `LICENSE` file containing the full AGPL-3.0-or-later text.
2. Add `LICENSE-COMMERCIAL.md` describing the commercial-license
   process (single point of contact, indicative pricing tiers,
   indemnification language).
3. Add `license = "AGPL-3.0-or-later"` to `Cargo.toml`.
4. Add SPDX headers (`// SPDX-License-Identifier: AGPL-3.0-or-later`)
   to each source file. This can be automated with `addlicense`
   or a one-off Python script.
5. Update `README.md` with the licensing summary and contact
   address for commercial inquiries.
6. (Optional, recommended) Sign a `cla-assistant`-style CLA
   workflow into the repo so any future contributors assign
   copyright OR grant an unlimited license to the project. Without
   this, contributor commits add joint-copyright complications that
   make future re-licensing harder.

## Trademark posture (separate from copyright)

The name "Bat_OS" and any project logo should be claimed as a
common-law trademark once first used publicly (Phase B). Formal
USPTO registration (~$250-350 application fee for a single
class + a few hundred more for the lawyer or self-filing) is
optional but cheap insurance against squatters and lookalike
products. Domain reservation (bat-os.dev / batos.dev / etc.) is
roughly $12-15/yr and worth doing the moment we can spare $15.
