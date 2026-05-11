---
type: concept-note
topic: project · history
---

# Post-no-browser pivot

> May 2026. The project deliberately stopped trying to ship a web browser. This note explains what was deleted, why, and what's still in the tree as archaeology.

## The pre-pivot state

For most of the project's life, Bat_OS was going to ship its own web browser. The thinking: a security workstation that can't browse is half a workstation. The plan was to vendor a small browser engine, sandbox it inside a cave, and let the cave's default-deny policy + kernel-mediated HTTPS do the heavy lifting on the network side.

To that end, the repo accumulated:

- A `ports/` tree containing NetSurf, libcss, libdom, libhubbub, libnsfb, libnsutils, libparserutils, libwapcaplet — vendored snapshots of a small browser stack. Deleted at the pivot; if you ever need to reconstitute, the upstream releases are still on the original projects' sites.
- An attempt at vendoring Chromium content_shell (parked at >1 GB; never tracked in git).
- [[_generated/DESIGN_PACKET_PIPELINE.md]] — design doc for the network pipeline that would feed a browser.
- A `port/ladybird` branch that experimented with porting the Ladybird browser to bare metal.

The default branch was named `feat/js-engine-browser-posix` — the literal git branch implied a JS engine on a POSIX-ish surface for a browser to run on.

## What broke the plan

Three forces collected over a few weeks:

1. **The attack surface was the point.** A workstation that ships a browser is a workstation whose biggest attack surface is "every web page the operator visits." That's the opposite of what a security-graded workstation is for.
2. **The engineering surface was enormous.** Even a "small" browser is hundreds of thousands of lines we didn't write. Auditing them to the same standard as the kernel was infeasible.
3. **A simpler model existed.** *Browse on a different device.* The workstation does the work that needs strong isolation (drafting, review, signed comms, encrypted storage). Browsing is allowed to live on a separate, untrusted device.

The pivot was: **delete the browser, double down on the kernel.**

## What got deleted

- The browser app inside the kernel was removed from the default image.
- The Ladybird port branch was abandoned.
- The Chromium-content_shell experiment was deleted (it never made it into git).
- The default-branch name was changed from `feat/js-engine-browser-posix` to `main` (this happened on 2026-05-08; see [[_generated/CLAUDE.md]]).

## What stayed

- The `ports/` tree was **deleted** at the pivot — keeping a few hundred MB of dormant browser-engine source in the repo just to "maybe come back to it" failed the cost/benefit test. If you ever need any of those libraries, they're each one upstream-release away.
- Network pipeline design docs are still relevant — Hush, Mesh, Sonar, the kernel-mediated HTTPS syscall all came out of work that was originally browser-driven and survived the pivot intact. See [[Concepts/TLS Hardening Journey]].
- The cave isolation model became *more* important after the pivot, not less. Without a browser, every cave's policy is the entire surface; getting that right is the project. See [[Concepts/Cave Isolation Model]].

## What replaces "browser" in the marketing

> Bat_OS deliberately omits a web browser. Browsing belongs on a separate device. What remains is the work that needs strong isolation: drafting, review, encrypted storage, signed communications.

That sentence is on every version of the marketing site since the pivot. The website also lives outside this repo (at `~/Downloads/index.html`) and is the place where the no-browser stance is most visible.

## What this changed about the roadmap

After the pivot merged (PR #1 ish — the early no-browser commits), the roadmap collapsed into three tracks in priority order:

1. **TLS X.509 chain validation hardening** — see [[Concepts/TLS Hardening Journey]] for status. PR #21 is in.
2. **Scheduler `block_on()`** — see [[_generated/DESIGN_SCHEDULER_BLOCK_ON.md]]. Required so the kernel-mediated HTTPS syscall can be properly synchronous.
3. **Captures cleanup** — `captures/` has 7800+ PNGs from the browser-era visual debugging work. These are now noise; they need a separate decision (delete vs. archive).

## Files for archaeology

- [[_generated/DESIGN_NO_BROWSER.md]] — the design doc that landed the pivot
- [[_generated/vendored/ports]] — the vendored browser-era libraries, dormant
- [[_generated/CLAUDE.md]] — references the pivot in its onboarding for future Claude sessions
