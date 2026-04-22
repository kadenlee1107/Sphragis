# AOP Bring-up Evidence Logs

Raw probe output from the 2026-04-21 → 2026-04-22 AOP/MTP keyboard
bring-up investigation. Each log corresponds to one boot cycle of the
M4 (J604 / T8132 "Donan") MacBook Pro via m1n1 chainload.

Referenced from:
- `docs/SESSION_JOURNAL.md` (2026-04-21 and 2026-04-22 entries)
- `docs/M4_GROUND_TRUTH.md` §3.6 (AOP ASCWrapV6 bring-up) and §3.7
- `scripts/hv/probe_aop_*.py` (the scripts that produced each log)

## Log naming

Pattern: `aop-<experiment>-<YYYYMMDD>-<HHMMSS>.log`

- `aop-pmgr-probe-*`       — v1: initial PMGR-reg hypothesis
- `aop-pmgr-v2-*`          — v2: dapf + time-series (discovered reg[3] is a timer)
- `aop-v3-doorbell-*`      — v3: DART.initialize() landmine (Mac reset)
- `aop-v4-nodart-*`        — v4: no DART + doorbell; FW takes FIQ but stalls
- `aop-v5-runonly-*`       — v5: RUN=1 alone; FW stays IDLE
- `aop-v6-inbox-no-doorbell-*` — v6: INBOX alone, no doorbell
- `aop-v7-3phase-*`        — v7: Ping / zero / SetIOPPower (handler dead for all)
- `aop-v8-smc-first-*`     — v8: SMC Hellos; AOP still silent
- `aop-v9-pac-patch-*`     — v9: attempted PAC-off patch; __TEXT SErrored
- `aop-v10-pure-*`         — v10: no stage; Mac crashed
- `aop-v11-write64-*`      — v11: 64-bit mailbox writes (no improvement)
- `aop-v12-longwait-*`     — v12: RUN without INBOX; Mac crashed
- `aop-v13-no-ob-reset-*`  — v13: skipped OB_CTRL reset
- `aop-v14-l4-fw-*`        — v14: _l4 firmware variant (wrong size, SError)
- `aop-v15-no-bootargs-*`  — v15: iBoot defaults; same stall
- `aop-v16-aopclient-*`    — v16: m1n1's own AOPClient.start() path; ASCTimeout
- `aop-mega-*`             — multi-IOP enumeration probe
- `aop-mega2-*`            — reg[4] + __DATA+0x498 dumps
- `aop-time-*`             — time-series FW-init progress (found page table)
- `aop-unstick-*`          — 7 unstick attempts (discovered +0x818 counter)
- `aop-818-*`              — +0x818 counter behavior study
- `aop-ping-*`             — Mgmt_Ping memory-diff probe

## Key raw evidence points

- **reg[0]+0x8 = 0x12345678** — debug magic cookie (aop-818 log)
- **__DATA+0x498 populates with 64 page-pointer entries** in <500ms
  (aop-time log)
- **+0x818 increments by 2 per INBOX write**, decrements on Ping
  (aop-unstick and aop-818 logs)
- **SMC Hellos via SMCClient.start() but AOP does not** — same code
  path, different firmware behavior (aop-v8 and aop-v16 logs)
