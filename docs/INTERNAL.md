# Internal documentation

Some of this project's documentation lives in a **separate private
repository** rather than in this one. That's deliberate: the
material in question is Tier 3 under the project's
[disclosure-posture rules](https://github.com/kadenlee1107/sphragis-internal/blob/main/docs/DISCLOSURE_POSTURE.md)
— trade-secret content whose value depends on not being publicly
accessible.

## Where the internal docs live

[`kadenlee1107/sphragis-internal`](https://github.com/kadenlee1107/sphragis-internal)
(private — request access by emailing the project contact in the
[`README`](../README.md)).

| Document | What it is |
|---|---|
| `docs/M4_GROUND_TRUTH.md` | Verified Apple M4 hardware reverse-engineering — addresses, PMGR sequences, ATC PHY tunables, etc. |
| `docs/SESSION_JOURNAL.md` | Chronological development log across all sessions. |
| `docs/DISCLOSURE_POSTURE.md` | The Tier 1 / 2 / 3 classification rules + per-mechanism categorisation. |
| `docs/LICENSING.md` | Internal strategy doc on the AGPL-3.0 + commercial dual-license posture. |

## Why this split exists

Sphragis is open-source (AGPL-3.0-or-later) and benefits from
public visibility, third-party citation, and grant eligibility.
But several specific artifacts — primarily the M4 hardware
reverse-engineering — represent trade secrets whose competitive
value evaporates the moment they're public. Splitting them into
a private companion repo preserves both properties:
**the OS itself is open; the hard-won reverse-engineering work
that made it possible isn't volunteered for free.**
