# Sphragis Quickstart

Fast path to running the things this repo can do today.

> Prereqs: macOS (M-series), `qemu-system-aarch64` (`brew install qemu`),
> nightly Rust, `pexpect` (`pip3 install pexpect`).

## Build the kernel

```bash
make build
```

Equivalent to:

```
SPHRAGIS_ALLOW_UNSIGNED_INITRD=1 SPHRAGIS_PASSPHRASE=batman SPHRAGIS_KEEP_GOING=1 \
    cargo build --release --features gicv3
```

## Bake the Chromium archive

Drops `content_shell` + 12 shared libraries into a BATCHROM-framed
initrd at `target/aarch64-unknown-none/release/chromium_initrd.bin`.
Adds any `*.html` next to `content_shell` so the shell can `render`
them.

```bash
make initrd
```

## Render an HTML page to a PNG

```bash
make render                                    # default: hello.html
make render URL=file:///bin/showcase.html      # the showcase page
```

Output: `logs/qemu-tests/render-<timestamp>.png` — open with `open …`.
The runner boots Sphragis under QEMU + HVF, sends `render <url>` to the
shell, captures the base64-encoded BGRA dump on the serial port, and
writes a PNG.

Pipeline (kernel side):

```
HTML bytes (initrd archive)
  → browser::html::parser
  → browser::layout::build
  → browser::paint::paint    (TrueType glyphs, backgrounds, borders…)
  → 800×600 BGRA framebuffer in BSS
  → base64 over UART
  → host: scripts/render_to_png.py decodes to PNG
```

## Dump the DOM tree (text)

Faster than `render`; skips layout + paint.

```bash
make dom                                  # default: hello.html
make dom URL=file:///bin/showcase.html
```

## Run the Chromium pipeline smoke

Boots Sphragis, runs `chromium --dump-dom file:///bin/hello.html`,
captures every `[SKIP …]` event with `SPHRAGIS_KEEP_GOING=1` so you can
see exactly where Chromium is currently failing.

```bash
make smoke
```

The smoke is a stress test — Chromium gets through libc init, V8
startup, the worker thread pool, Skia, HarfBuzz text layout, and
FileURLLoader::Start before stalling in Mojo IPC. See
`docs/SESSION_JOURNAL.md` for the latest blocker.

## Where things live

```
src/                      kernel + browser + drivers
├── browser/              native HTML parser, DOM, CSS, layout, paint
├── caves/linux/        Linux compat layer (syscalls, futex, epoll, …)
├── kernel/               mm, scheduler, arch, sync
├── drivers/              uart, virtio (gpu/net/blk/keyboard), apple/*
└── ui/                   shell, console, font, truetype, apps/

scripts/                  host-side runners
├── render_to_png.py      `make render`
├── dump_dom.py           `make dom`
├── qemu_chromium_pipeline_smoke.py   `make smoke`
├── batcaved.py           control-channel daemon (auto-spawned)
└── lib/qemu_boot.py      shared boot harness used by the above

tools/                    build/bake scripts
├── bake_chromium_archive.sh   `make initrd`
└── …

ports/chromium_port/      content_shell + libs + the .html files we render
docs/                     SESSION_JOURNAL.md, M4_GROUND_TRUTH.md, …
logs/qemu-tests/          all generated logs + PNGs
```

## Common environment variables

| Var                              | Default | Purpose                                                   |
| -------------------------------- | ------- | --------------------------------------------------------- |
| `SPHRAGIS_ALLOW_UNSIGNED_INITRD`   | `1`     | Skip initrd signature check during dev                    |
| `SPHRAGIS_PASSPHRASE`              | `batman`| Build-time passphrase for BatFS / auth                    |
| `SPHRAGIS_KEEP_GOING`              | `1`     | Skip-and-log on cave-fatal events instead of teardown     |
| `URL` (Make var)                 | `file:///bin/hello.html` | Target for `make render` / `make dom` |

Override on the command line:

```bash
make build SPHRAGIS_KEEP_GOING=                 # production-style build
make render URL=file:///bin/showcase.html
```

## Where the renders go

Every run drops two artifacts:

- `logs/qemu-tests/<command>-<timestamp>.log` — full serial transcript
- `logs/qemu-tests/<command>-<timestamp>.png` — the rendered framebuffer
  (only for `make render`)

To clean these up:

```bash
rm -rf logs/qemu-tests/render-* logs/qemu-tests/dumpdom-*
```

(They're already gitignored.)
