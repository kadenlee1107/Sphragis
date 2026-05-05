# Ladybird build environment for Bat_OS

A Linux ARM64 Docker container that builds [Ladybird](https://github.com/LadybirdBrowser/ladybird)'s
`WebContent` service + helpers for use as a Bat_OS guest binary.

Ladybird is a truly independent web browser — NOT a Chromium fork.
Its core libraries (LibWeb, LibJS, LibCrypto, LibTLS, LibIPC, …) come
from the SerenityOS project, and are built for non-Serenity systems
via the **Lagom** layer.

This port targets `aarch64-linux-gnu` against glibc, mirroring the
existing Chromium port's runtime environment. Bat_OS's cave loader
already handles dynamically-linked glibc binaries (see
`src/batcave/linux/loader.rs`), so the same in-cave runtime that
loads `content_shell` should load Ladybird's `WebContent` with the
same ELF + relocation pipeline.

## Why Ladybird vs Chromium

The Chromium port (`../chromium_port/`) booted, ran glibc init,
loaded V8 + Blink, and reached `FileURLLoader::Start` for
`file:///bin/hello.html`. It then walls on a
`pthread_cond_broadcast → CMP_REQUEUE → pthread_mutex_unlock`
chain where 10 specific high-VA mutex futexes never receive their
`FUTEX_WAKE` — a Mojo-IPC-adjacent threading interaction that
needs source-level reasoning to fix.

Ladybird sidesteps the whole class of issue:

| | Chromium | Ladybird |
|---|---|---|
| Binary size  | ~600 MB              | ~32 MB C++ + 2 MB Rust  |
| JS engine    | V8 (1 TB sandbox cage) | LibJS (no cage)        |
| IPC          | Mojo (Mach-style)    | LibIPC (Unix sockets)   |
| License      | BSD/Apache mix       | **BSD-2** (fork-friendly) |
| Init threads | 30+ cond-var cascade | Simpler, fewer threads  |
| Origin       | Built for prod OSes  | **Built alongside SerenityOS** |

The SerenityOS heritage is the killer feature: Ladybird's authors
already had to make their browser run on an OS they were writing
themselves. That's literally our exact situation.

## Strategy

1. **Lagom host build** — verify the Ladybird toolchain works at all
   on a Linux ARM64 host (Docker container).
2. **Cross-target tweak** — Lagom already produces `aarch64-linux-gnu`
   binaries when built on Linux/ARM64; nothing exotic.
3. **Bake into Bat_OS initrd** — `tools/bake_ladybird_initrd.sh`
   (TBD) produces a multi-file BATARCH archive the kernel's
   `load_archive_multi` can load alongside `WebContent`'s
   DT_NEEDED libs.
4. **Drive headlessly from the shell** — add a `ladybird` shell
   command that invokes the CLI in dump-DOM mode (Ladybird has
   `Tests/LibWeb/Ref` infrastructure that already does headless
   reference rendering — useful for our smoke).

## One-time setup (host machine — macOS Apple Silicon)

```sh
# 1. OrbStack (or Docker Desktop) installed and running
brew install --cask orbstack
open /Applications/OrbStack.app

# 2. Build the build container
cd ports/ladybird_port
docker build --platform linux/arm64 -t batos-ladybird-build:latest .
```

## Run the build

```sh
# Persistent volume for the source tree (~3 GB) and build output (~5 GB)
# — order of magnitude smaller than Chromium.
docker volume create batos-ladybird-src

docker run --rm -it \
    --platform linux/arm64 \
    -v batos-ladybird-src:/home/build/ladybird-src \
    -v "$(pwd)/build.sh:/home/build/build.sh:ro" \
    batos-ladybird-build:latest \
    /home/build/build.sh
```

The first run does:
1. `git clone https://github.com/LadybirdBrowser/ladybird.git` (~3 GB)
2. `Meta/ladybird.py rebuild` (cmake + ninja, ~30 min on M-series)
3. Copies the resulting binaries to `/out` (mounted to
   `ports/ladybird_port/out/` on the host)

## Output

After the build:

```
ports/ladybird_port/out/
├── WebContent          ← the renderer process (our content_shell equivalent)
├── RequestServer       ← network process
├── ImageDecoder        ← image decoding process
├── lib/                ← DT_NEEDED runtime deps (libstdc++, libssl, …)
└── share/              ← font/icon/cert resources
```

`tools/bake_ladybird_initrd.sh` (to be written) packs these into
a single `ladybird_initrd.bin` blob that the kernel embeds in its
image at link time, same flow as `tools/bake_chromium_initrd.sh`.

## Status

| Phase | Status |
|---|---|
| Branch + scaffolding | ✓ done |
| Build container | TODO |
| Lagom host build | TODO |
| `aarch64-linux-gnu` cross verify | TODO |
| Initrd bake script | TODO |
| Kernel `ladybird` shell command | TODO |
| First DOM dump | TODO |

## Pinned commit

For reproducibility, we'll pin to a known-good Ladybird commit once
we have a working build. Bump deliberately when upstream lands a
relevant change.

## Notes on the rendering surface

Ladybird's `WebContent` writes painted pixels into a shared-memory
region the UI process maps. For Bat_OS, we'll point that shared-memory
target at our existing `/batos/fb0` BGRA framebuffer node (see
`src/batcave/linux/vfs.rs`). That's the same approach we used for
Chromium's display bridge.
