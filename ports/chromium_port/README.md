# Chromium build environment for Bat_OS

A Linux ARM64 Docker container that builds Chromium's `content_shell`
for use as a Bat_OS guest binary. See `../../CHROMIUM_PORT_PLAN.md`
for the full port plan.

## One-time setup (host machine — macOS Apple Silicon)

```sh
# 1. OrbStack (or Docker Desktop) installed
brew install --cask orbstack
open /Applications/OrbStack.app   # let it finish first-time setup

# 2. Build the build container
cd ports/chromium_port
docker build --platform linux/arm64 -t batos-chromium-build:latest .
```

## Run the build

```sh
# Persistent volume for the source tree (~30 GB) and build output (~30 GB)
docker volume create batos-chromium-src

# Drop into a shell in the container
docker run --rm -it \
    --platform linux/arm64 \
    -v batos-chromium-src:/home/build/chromium \
    -v "$(pwd)/.gn-args:/home/build/.gn-args:ro" \
    -v "$(pwd)/build.sh:/home/build/build.sh:ro" \
    batos-chromium-build:latest \
    /home/build/build.sh
```

The first run does:
1. `fetch chromium` — downloads source (~30 GB, 30-60 min)
2. `gclient sync` — pulls third-party deps (~20 min)
3. `gn gen` — configures the build
4. `autoninja content_shell` — compiles (4-8 hours on M-series)

Subsequent runs reuse the volume so only changed files rebuild.

## Output

The built binary lives at:
```
/home/build/chromium/src/out/BatOs/content_shell
```

Inside the container, copy it out:
```sh
docker run --rm -v batos-chromium-src:/src batos-chromium-build:latest \
    cat /src/src/out/BatOs/content_shell > content_shell
```

That's the file we'll embed into Bat_OS in Phase 7.

## Files

| File         | What                                               |
|--------------|----------------------------------------------------|
| `Dockerfile` | Linux ARM64 build environment                      |
| `.gn-args`   | Pinned GN arguments for the content_shell build    |
| `build.sh`   | Drives gclient sync + ninja inside the container   |
| `patches/`   | Chromium source patches we apply (Ozone, etc.)     |

## Disk usage warning

This build needs ~60 GB of free disk on the host. Check with `df -h`
before starting. The `batos-chromium-src` Docker volume is where the
source + build output lives.

## Resetting

```sh
docker volume rm batos-chromium-src   # nuke source + build, start over
docker rmi batos-chromium-build       # rebuild the image too
```
