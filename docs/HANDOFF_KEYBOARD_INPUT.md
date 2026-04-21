# Keyboard input handoff — next session start here

Last session (2026-04-20 21:00–22:15) landed two wins:
  1. M4 ~118 s watchdog permanently disabled (commit 72c606f4).
     HV sessions now run as long as you want.
  2. Tab-to-X shutdown UI shipped in the code (commit 877502e4).
     Tab cycles SH→…→BC→X; Enter on X triggers `halt_bat_os`.

The X-button code is correct by inspection but wasn't
demonstrable end-to-end, because the host couldn't reliably
inject keystrokes into the running HV. This doc describes the
tooling added this session to make keystroke injection work
next time, and the exact steps to validate.

## What changed for input plumbing

### 1. `scripts/hv/batos_hv_interactive.py` — stdin_forwarder always on

Previously gated `if not stims and sys.stdin.isatty()`. Now just
`if sys.stdin.isatty()`. So when you run the interactive with a
stimulus like `BATOS_HV_STIMULUS=batman`, the stim still fires at
startup AND your terminal's keyboard stays connected to the guest
vuart for the whole session.

That means: after `[security] AUTH PASSED` shows up, you can type
Tab/Enter/whatever directly into the terminal running
`batos_hv_interactive.py` and it reaches Bat_OS.

Gotchas:
  - Ctrl+] detaches cleanly (keeps Mac alive).
  - Tab is 0x09 — the byte your terminal sends when you press Tab
    (assuming xterm-style line discipline). If it sends `\t\t` or
    gets eaten by the shell, use `inject_keys.py` instead.

### 2. `scripts/hv/inject_keys.py` — non-interactive keystroke sender

For when the HV session is running in the background and you want to
inject keystrokes from a separate shell without killing it. Opens
`/dev/ttyACM2` in raw mode (non-exclusive — coexists with the
interactive script's pyserial open), writes each byte with a small
delay, exits.

```bash
# Type "batman\r" to log in:
sg dialout -c "/usr/bin/python3 scripts/hv/inject_keys.py \"batman\" enter"

# Tab 9 times + Enter = shutdown via X button:
sg dialout -c "/usr/bin/python3 scripts/hv/inject_keys.py \
    tab tab tab tab tab tab tab tab tab enter"

# Any single byte by hex:
sg dialout -c "/usr/bin/python3 scripts/hv/inject_keys.py 0x11"  # Ctrl+Q

# Help:
sg dialout -c "/usr/bin/python3 scripts/hv/inject_keys.py --help"
```

Token grammar:
  `tab | enter | esc | bs | space` — named keys
  `ctrl+X` (lowercase) — control code for letter X
  `0xNN` — raw hex byte
  `"quoted string"` — literal bytes one by one
  any single char — its ASCII byte

Delay between bytes is `INJECT_DELAY` (default 0.08 s) — tunable
via env if you need faster/slower.

### 3. Why `printf > /dev/ttyACM2` doesn't work

USB CDC-ACM via kernel's default ldisc ends up in cooked/canonical
mode. Line-buffering holds the byte until a newline. Tab never
flushes. That's why the direct-redirect approach failed last
session. The inject_keys.py script sets raw termios on its fresh fd.

## Validation checklist for next session

1. Power-cycle Mac, wait for stock m1n1.
2. Chainload patched m1n1:
   ```
   sudo -n M1N1DEVICE=/dev/ttyACM1 M1N1WAIT=1 \
     /usr/bin/python3 external/m1n1/proxyclient/tools/chainload.py \
     -S external/m1n1/build/m1n1.macho
   ```
   Watch for `[hv_init] M15 AP-WDT regs: … r2 0000007c->ffffffff …`
   confirming the watchdog disable lands.
3. Start interactive in a terminal where Kaden can type:
   ```
   BATOS_KEEP_FB=1 BATOS_HV_STIMULUS=batman BATOS_HV_TIMEOUT=3600 \
     sg dialout -c "/usr/bin/python3 scripts/hv/batos_hv_interactive.py"
   ```
4. Wait for `[security] AUTH PASSED — launching shell` in the serial
   log — means batman auth went through and Bat_OS is at the desktop.
5. Either (a) type Tab x9, Enter — watch M4 screen for the
   "BAT_OS HALTED" banner, OR (b) from a separate shell:
   ```
   sg dialout -c "/usr/bin/python3 scripts/hv/inject_keys.py \
     tab tab tab tab tab tab tab tab tab enter"
   ```
6. Grep the serial log for `[BATOS] halt requested via UI close
   button` — that's the marker `halt_bat_os` writes when it triggers.
7. Watch the framebuffer — should show the shutdown banner.
8. Mac stays alive (watchdog is off); power-cycle when you're done.

## Where the code lives

  - `src/ui/wm.rs` — `CLOSE_FOCUSED` state, X-rendering branch.
  - `src/ui/desktop.rs` — Tab handler cycles apps then onto X;
    Enter on X → `halt_bat_os()`.
  - `src/ui/desktop.rs` bottom — `halt_bat_os()` banner + wfe loop.
  - `external/m1n1/src/hv.c` end of `hv_init` — the 4 MMIO writes
    that disable the AP watchdog. Don't remove these — they're
    what let the Mac stay alive past 118 s.
  - `scripts/hv/inject_keys.py` — the new helper.
  - `scripts/hv/batos_hv_interactive.py` — stdin always forwarded.

## Known leftovers

  - `src/security/boot_screen.rs` has `[bs] …` uart trace prints
    left in from last session's debugging. Harmless but noisy; can
    delete if the UI tests show the login renders reliably.
  - The tab cycle wraps back to app 0 when Tab is pressed ON the X.
    If you want Shift+Tab to go backwards, that's a future nicety.
  - `halt_bat_os` only stops the guest CPU. Host-side proxy keeps
    running — the HV session is "parked". To actually return to
    stock m1n1, power-cycle the Mac.
