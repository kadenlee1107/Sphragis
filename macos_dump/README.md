# macOS research dumps

Apple-signed binaries and reverse-engineering reference data extracted
from live macOS on the M4 MacBook Pro (Mac16,1 / J604 / T8132 "Donan").
Used by the AOP/MTP bring-up investigation documented in
`docs/SESSION_JOURNAL.md` and `docs/M4_GROUND_TRUTH.md`.

## Layout

- `kernelcache.mac16j.im4p`  — im4p-wrapped kernelcache (32 MB)
- `kernelcache.mac16j.bin`   — unwrapped, split into 90 MB chunks:
    - `kernelcache.mac16j.bin.part00` (90 MB)
    - `kernelcache.mac16j.bin.part01` (26 MB)
    - Reassemble: `cat kernelcache.mac16j.bin.part?? > kernelcache.mac16j.bin`
    - SHA-256: (computed at split time) — verify with `sha256sum`
- `SystemKernelExtensions.kc` — system kext cache, split (345 MB):
    - `SystemKernelExtensions.kc.part00..part03`
    - Reassemble: `cat SystemKernelExtensions.kc.part?? > SystemKernelExtensions.kc`
- `BootKernelExtensions.kc`  — boot kexts (67 MB, under 100 MB limit)
- `kexts/`                   — extracted kext mach-o + syms files
- `batos_dump/`              — iBoot-visible ADT + device tree dump
- `batos_dump.tar.gz`        — archive of the above
- `dtrace_traces/`           — live-macOS mailbox traces (2026-04-22)

## Reassembly script

```sh
cd macos_dump/
cat kernelcache.mac16j.bin.part?? > kernelcache.mac16j.bin
cat SystemKernelExtensions.kc.part?? > SystemKernelExtensions.kc
```

## Provenance

Extracted from macOS 26.3 / iBoot-13822.81.10 running on Mac16,1.

Tracked in-repo because the repo is private and these files are the
only reference for several findings in the session journal. If the
repo ever goes public, these must be removed first (Apple IP).

## Verified SHA-256 of reassembled files

After running the reassembly commands above, verify with:

```
80ebbb73d8d644e9a199668bc0aac1565add6f1fe56bbfece330628d8eec887f  kernelcache.mac16j.bin
336c38eb5ecd91f00064da502660b4847893dd746c344af1c143675a11fd356c  SystemKernelExtensions.kc
```
