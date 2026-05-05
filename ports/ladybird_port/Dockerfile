# Bat_OS — Ladybird build environment
# Linux ARM64, Debian Bookworm, with Ladybird's build prereqs.
# Produces a dynamically-linked WebContent binary (+ helpers + DT_NEEDED
# libs) for use as a Bat_OS guest binary, mirroring how the chromium_port
# produces content_shell.
#
# Ladybird's prereqs per Documentation/BuildInstructionsLadybird.md:
#   * C++23 compiler — clang-21 or gcc-14
#   * CMake 3.30+
#   * Rust toolchain
#   * Qt6 (UI; we'll skip the UI build, only need Services)
#   * nasm, ninja, autotools
#   * libgl, libdrm (vendored vcpkg-style for the WebContent process)

FROM --platform=linux/arm64 ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV LANG=C.UTF-8
ENV LC_ALL=C.UTF-8

# Bookworm ships gcc-12 by default. Ladybird needs gcc-14 OR clang-21+.
# Pull from Kitware (cmake) and LLVM apt repos per Ladybird's install
# guide. We use clang-21 + ld.lld to keep the toolchain consistent.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates curl wget git gnupg lsb-release sudo file \
        python3 python3-pip python3-venv \
        # vcpkg / autotools / nasm — Ladybird's bootstrap build
        autoconf autoconf-archive automake build-essential ccache \
        curl git libtool nasm ninja-build pkg-config tar unzip zip \
        # Runtime deps the WebContent service links to
        libdrm-dev libgl1-mesa-dev libpulse-dev \
        # Qt6 — Ladybird's CMakeLists.txt always tries to find_package(Qt6)
        # for its UI/Qt subdir, even when we only build js / Services.
        # Installing the dev package satisfies cmake configure; the
        # binaries we actually use (js, WebContent) don't link to Qt.
        qt6-base-dev qt6-tools-dev qt6-tools-dev-tools \
        # Fonts (Ladybird requires liberation fonts at runtime)
        fonts-liberation2 \
    && rm -rf /var/lib/apt/lists/*

# Kitware cmake 3.30+
RUN wget -qO- https://apt.kitware.com/keys/kitware-archive-latest.asc \
      | gpg --dearmor - > /usr/share/keyrings/kitware-archive-keyring.gpg \
 && echo "deb [signed-by=/usr/share/keyrings/kitware-archive-keyring.gpg] \
          https://apt.kitware.com/ubuntu/ $(lsb_release -cs) main" \
      > /etc/apt/sources.list.d/kitware.list \
 && apt-get update \
 && apt-get install -y --no-install-recommends cmake \
 && rm -rf /var/lib/apt/lists/*

# LLVM 21 (clang + lld + clang-tidy)
RUN wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key \
      > /usr/share/keyrings/llvm-snapshot.gpg.key \
 && echo "deb [signed-by=/usr/share/keyrings/llvm-snapshot.gpg.key] \
          https://apt.llvm.org/$(lsb_release -cs)/ \
          llvm-toolchain-$(lsb_release -cs)-21 main" \
      > /etc/apt/sources.list.d/llvm.list \
 && apt-get update \
 && apt-get install -y --no-install-recommends \
        clang-21 lld-21 clang-tools-21 \
 && rm -rf /var/lib/apt/lists/* \
 && ln -sf /usr/bin/clang-21    /usr/bin/clang \
 && ln -sf /usr/bin/clang++-21  /usr/bin/clang++ \
 && ln -sf /usr/bin/ld.lld-21   /usr/bin/ld.lld

# Rust toolchain. Ubuntu 24.04 ships rustc 1.75 + cargo 1.75 which is
# new enough for Ladybird's small Rust corner. Avoid rustup-as-root —
# its $HOME/.cargo isn't readable by the build user, and Ladybird's
# CMake `find_program(RUST_CARGO cargo)` then fails. apt installs
# system-wide into /usr/bin which is in PATH for any user.
RUN apt-get update && apt-get install -y --no-install-recommends \
        rustc cargo \
    && rm -rf /var/lib/apt/lists/*

# Google's `gn` build tool. Required by the vcpkg `skia` port —
# Skia uses GN to generate ninja files, and vcpkg-gn (the cmake
# wrapper) silently passes `None` for the gn binary path if it
# can't find one. On Ubuntu the package is `generate-ninja`; the
# binary is still named `gn`.
RUN apt-get update && apt-get install -y --no-install-recommends generate-ninja \
    && rm -rf /var/lib/apt/lists/*

# Build user (matches chromium_port's `build` user). Ubuntu 24.04 ships
# with a default `ubuntu` user at uid 1000; delete it first so our
# `build` user can claim that uid.
RUN userdel -r ubuntu 2>/dev/null || true && \
    useradd -m -u 1000 -s /bin/bash build && \
    echo "build ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/build
USER build
WORKDIR /home/build

# Persistent ccache to speed up rebuilds across container invocations
ENV CCACHE_DIR=/home/build/.ccache
RUN mkdir -p ${CCACHE_DIR}

# Default entrypoint just drops to a shell; the actual build is driven
# by build.sh which the host mounts via -v.
CMD ["/bin/bash"]
