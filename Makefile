# Bat_OS — top-level entry points.
#
# Run `make` (or `make help`) for the menu. The defaults below match the
# environment variables the build expects when running the Chromium port
# under HVF on Apple M4: signed-initrd off, dev passphrase, KEEP_GOING
# enabled. Override on the command line if you want different behavior:
#
#     make build BAT_OS_KEEP_GOING=
#     make render URL=file:///bin/showcase.html
#

# ─── Build env ──────────────────────────────────────────────────────────
BAT_OS_ALLOW_UNSIGNED_INITRD ?= 1
BAT_OS_PASSPHRASE            ?= batman
BAT_OS_KEEP_GOING            ?= 1

CARGO_FEATURES ?= gicv3
CARGO_FLAGS    ?= --release --features $(CARGO_FEATURES)

ENV = \
    BAT_OS_ALLOW_UNSIGNED_INITRD=$(BAT_OS_ALLOW_UNSIGNED_INITRD) \
    BAT_OS_PASSPHRASE=$(BAT_OS_PASSPHRASE) \
    BAT_OS_KEEP_GOING=$(BAT_OS_KEEP_GOING)

# ─── Paths ──────────────────────────────────────────────────────────────
ROOT     := $(shell pwd)
TARGET   := target/aarch64-unknown-none/release
KERNEL   := $(TARGET)/bat_os
INITRD   := $(TARGET)/chromium_initrd.bin

CONTENT_SHELL := ports/chromium_port/out/content_shell
LIB_RUNTIME   := ports/chromium_port/out/lib_runtime

# ─── Phony targets ──────────────────────────────────────────────────────
.PHONY: help build initrd render render-live dom smoke clean watch info

# Default: print the menu.
help:
	@printf "Bat_OS — common commands\n\n"
	@printf "  make build       — cargo build the kernel (release, gicv3)\n"
	@printf "  make initrd      — bake content_shell + libs into chromium_initrd.bin\n"
	@printf "  make render      — render an HTML page to a PNG (URL=file:///bin/hello.html)\n"
	@printf "  make dom         — dump the DOM tree to stdout (URL=...)\n"
	@printf "  make smoke       — run the chromium pipeline smoke under HVF\n"
	@printf "  make clean       — cargo clean\n"
	@printf "  make info        — show paths + env that the targets use\n\n"
	@printf "Examples:\n"
	@printf "  make render URL=file:///bin/showcase.html\n"
	@printf "  make smoke   (keep going & log all skip events)\n"
	@printf "  make build BAT_OS_KEEP_GOING=  (production-ish build)\n"

# ─── Build ──────────────────────────────────────────────────────────────
build: $(KERNEL)

$(KERNEL): $(shell find src -name '*.rs' 2>/dev/null) Cargo.toml linker.ld
	@$(ENV) cargo build $(CARGO_FLAGS)

initrd: $(INITRD)

$(INITRD): $(KERNEL) $(CONTENT_SHELL) $(wildcard $(LIB_RUNTIME)/*)
	@if [ -f "$(CONTENT_SHELL)" ] && [ -d "$(LIB_RUNTIME)" ]; then \
	    tools/bake_chromium_archive.sh $(CONTENT_SHELL) $(LIB_RUNTIME); \
	else \
	    echo "[initrd] $(CONTENT_SHELL) or $(LIB_RUNTIME) missing — falling back to tests/hello"; \
	    tools/bake_chromium_initrd.sh tests/hello; \
	fi

# ─── Run targets ────────────────────────────────────────────────────────
URL ?= file:///bin/hello.html

render: build initrd
	@python3 scripts/render_to_png.py "$(URL)"

# Sprint 1.4: live render path. Boots QEMU with virtio-gpu attached and
# the host's native display so the rendered page appears in a window
# instead of only as a base64 PNG dump. Pass `URL=...` the same way as
# `make render`. Rendering is single-page (first 1900 px); scrolling
# is the next milestone. Closes the window with Cmd-Q (Cocoa) /
# Ctrl-Alt-Q (GTK) — kernel keeps running but you've gone back to
# headless.
render-live: build initrd
	@python3 scripts/render_live.py "$(URL)"

dom: build initrd
	@python3 scripts/dump_dom.py "$(URL)"

smoke: build initrd
	@python3 scripts/qemu_chromium_pipeline_smoke.py

# ─── Misc ───────────────────────────────────────────────────────────────
clean:
	cargo clean

watch:
	@cargo watch -x 'build $(CARGO_FLAGS)'

info:
	@echo "ROOT     : $(ROOT)"
	@echo "KERNEL   : $(KERNEL)"
	@echo "INITRD   : $(INITRD)"
	@echo "URL      : $(URL)"
	@echo "ENV      : $(ENV)"
