# Bat_OS — Chromium build environment
# Linux ARM64, Debian Bookworm, with depot_tools + Chromium build deps.
# Used to produce a static-PIE content_shell binary for Bat_OS.

FROM --platform=linux/arm64 debian:bookworm

ENV DEBIAN_FRONTEND=noninteractive
ENV LANG=C.UTF-8
ENV LC_ALL=C.UTF-8

# Core build deps (depot_tools + Chromium prerequisites for ARM64 Linux)
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        wget \
        git \
        python3 \
        python3-pip \
        python3-distutils-extra \
        lsb-release \
        sudo \
        file \
        less \
        vim \
        xz-utils \
        unzip \
        zip \
        bzip2 \
        gzip \
        tar \
        rsync \
        pkg-config \
        gnupg \
        gperf \
        bison \
        flex \
        ninja-build \
        cmake \
        make \
        gcc \
        g++ \
        binutils \
        libc6-dev \
        libstdc++-12-dev \
        ccache \
    && rm -rf /var/lib/apt/lists/*

# Chromium-specific build prereqs (we install only what depot_tools'
# install-build-deps.sh needs for headless ARM64 — no X11/Wayland).
RUN apt-get update && apt-get install -y --no-install-recommends \
        libnss3-dev \
        libgbm-dev \
        libdrm-dev \
        libxkbcommon-dev \
        libpango1.0-dev \
        libcairo2-dev \
        libcups2-dev \
        libxss-dev \
        libxtst-dev \
        libasound2-dev \
        libatk1.0-dev \
        libatk-bridge2.0-dev \
        libatspi2.0-dev \
        libgtk-3-dev \
        libnspr4-dev \
        libxcomposite-dev \
        libxdamage-dev \
        libxrandr-dev \
        libxfixes-dev \
        libxinerama-dev \
        libxcursor-dev \
        libxi-dev \
        libxext-dev \
        libxrender-dev \
        libxcb1-dev \
        libx11-xcb-dev \
        libxcb-glx0-dev \
        libgl1-mesa-dev \
        libgles2-mesa-dev \
        libegl1-mesa-dev \
        libglu1-mesa-dev \
        libssl-dev \
        libdbus-1-dev \
        libudev-dev \
        libjpeg62-turbo-dev \
        libpng-dev \
        libwebp-dev \
        libavcodec-dev \
        libavformat-dev \
        libavutil-dev \
        libswscale-dev \
        libsqlite3-dev \
        libxml2-dev \
        libxslt1-dev \
        libcurl4-openssl-dev \
        libfreetype6-dev \
        libfontconfig1-dev \
        libharfbuzz-dev \
        libicu-dev \
        qemu-user-static \
        qemu-user \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js v24.12.0 EXACTLY — Chromium M132 has a strict version check
# (NodeSource's setup_24.x ships latest 24.x which won't match). Download from
# the official Node.js distribution mirror.
RUN cd /tmp \
    && curl -fsSLO https://nodejs.org/dist/v24.12.0/node-v24.12.0-linux-arm64.tar.xz \
    && tar -xJf node-v24.12.0-linux-arm64.tar.xz -C /opt \
    && ln -sf /opt/node-v24.12.0-linux-arm64/bin/node /usr/local/bin/node \
    && ln -sf /opt/node-v24.12.0-linux-arm64/bin/npm  /usr/local/bin/npm \
    && ln -sf /opt/node-v24.12.0-linux-arm64/bin/npx  /usr/local/bin/npx \
    && rm -f /tmp/node-v24.12.0-linux-arm64.tar.xz \
    && node --version

# Enable x86_64 multi-arch so Chromium's bundled x86_64 prebuilts (clang,
# tsc, etc.) can run under emulation. Required for ARM64 host builds.
RUN dpkg --add-architecture amd64 \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        libc6:amd64 \
        libstdc++6:amd64 \
        zlib1g:amd64 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root build user (matches host uid for volume sanity)
ARG USER_UID=1000
ARG USER_GID=1000
RUN groupadd -g $USER_GID build \
    && useradd -m -u $USER_UID -g $USER_GID -s /bin/bash build \
    && echo "build ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

USER build
WORKDIR /home/build

# Install Rust via rustup (ARM64 native; Chromium's bundled Rust is x86_64-only)
ENV RUSTUP_HOME=/home/build/.rustup \
    CARGO_HOME=/home/build/.cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
        sh -s -- -y --default-toolchain nightly --profile minimal \
    && /home/build/.cargo/bin/rustc --version

# Fetch depot_tools — Google's Chromium build/sync wrapper
RUN git clone --depth 1 https://chromium.googlesource.com/chromium/tools/depot_tools.git \
        /home/build/depot_tools

ENV PATH="/home/build/.cargo/bin:/home/build/depot_tools:${PATH}"
ENV DEPOT_TOOLS_UPDATE=1
ENV GCLIENT_PY3=1

# Source tree mountpoint (provided as a Docker volume from outside)
VOLUME ["/home/build/chromium"]
WORKDIR /home/build/chromium

CMD ["/bin/bash"]
