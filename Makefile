# Sphragis — top-level entry points.
#
# Run `make` (or `make help`) for the menu.

# ─── Build env ──────────────────────────────────────────────────────────
SPHRAGIS_ALLOW_UNSIGNED_INITRD ?= 1
SPHRAGIS_PASSPHRASE            ?= batman
SPHRAGIS_KEEP_GOING            ?= 1

CARGO_FEATURES ?= gicv3
CARGO_FLAGS    ?= --release --features $(CARGO_FEATURES)

ENV = \
    SPHRAGIS_ALLOW_UNSIGNED_INITRD=$(SPHRAGIS_ALLOW_UNSIGNED_INITRD) \
    SPHRAGIS_PASSPHRASE=$(SPHRAGIS_PASSPHRASE) \
    SPHRAGIS_KEEP_GOING=$(SPHRAGIS_KEEP_GOING)

# ─── Paths ──────────────────────────────────────────────────────────────
ROOT     := $(shell pwd)
TARGET   := target/aarch64-unknown-none/release
KERNEL   := $(TARGET)/sphragis

# ─── Phony targets ──────────────────────────────────────────────────────
.PHONY: help build clean watch info

# Default: print the menu.
help:
	@printf "Sphragis — common commands\n\n"
	@printf "  make build       — cargo build the kernel (release, gicv3)\n"
	@printf "  make clean       — cargo clean\n"
	@printf "  make watch       — cargo watch (rebuild on src changes)\n"
	@printf "  make info        — show paths + env that the targets use\n\n"
	@printf "Examples:\n"
	@printf "  make build SPHRAGIS_KEEP_GOING=  (production-ish build)\n"

# ─── Build ──────────────────────────────────────────────────────────────
build: $(KERNEL)

$(KERNEL): $(shell find src -name '*.rs' 2>/dev/null) Cargo.toml linker.ld
	@$(ENV) cargo build $(CARGO_FLAGS)

# ─── Misc ───────────────────────────────────────────────────────────────
clean:
	cargo clean

watch:
	@cargo watch -x 'build $(CARGO_FLAGS)'

info:
	@echo "ROOT     : $(ROOT)"
	@echo "KERNEL   : $(KERNEL)"
	@echo "ENV      : $(ENV)"
