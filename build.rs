// V8-ROOT-5: tell cargo to re-run the build whenever the operator-supplied
// passphrase / duress envs change. Without these, `cargo build` caches an
// old passphrase across env changes and the operator thinks they've
// rotated credentials when they haven't.
//
// Consumed by src/main.rs via option_env!("BAT_OS_PASSPHRASE") /
// option_env!("BAT_OS_DURESS"). Leave unset in production — main.rs falls
// through to the interactive UART prompt + kernel-image-hash derivation
// for duress (see DEV_FALLBACK_LABEL / DURESS_LABEL).
fn main() {
    println!("cargo:rerun-if-env-changed=BAT_OS_PASSPHRASE");
    println!("cargo:rerun-if-env-changed=BAT_OS_DURESS");

    // Dev-only opt-in for loading an unsigned initrd (Chromium content_shell
    // stand-in while the real signing pipeline isn't wired up). Consumed by
    // src/batcave/linux/runner.rs via option_env!. Without this rerun hint,
    // cargo would cache the previous `ALLOW_UNSIGNED_INITRD` boolean across
    // env flips and the operator would see a stale FATAL refusal.
    println!("cargo:rerun-if-env-changed=BAT_OS_ALLOW_UNSIGNED_INITRD");
    println!("cargo:rerun-if-env-changed=BAT_OS_DISABLE_INIT_TRAMPOLINE");

    // Gap-audit item 034: build-time release-engineer Ed25519 pubkey
    // (64 hex chars). Used by the `release-verify` shell command to
    // check signed kernel images / packages. Generated via
    // `scripts/release_sign.py keygen`. Absent in dev builds — the
    // verifier refuses to run without it.
    println!("cargo:rerun-if-env-changed=BAT_OS_RELEASE_PUBKEY");

    // STUMP #87: cargo doesn't natively re-link when linker.ld changes
    // because the script is consumed by rustc via -Tlinker.ld in
    // .cargo/config.toml — Cargo treats it as opaque. Hint here so a
    // stack-size or section-layout change actually lands in the binary
    // instead of being silently cached. Symptom that bit us: bumped
    // kernel stack 512KB → 8MB to chase a JS compile_script hang, the
    // build was suspiciously fast (0.13s), and the hang persisted
    // because the new stack never made it into bat_os.bin.
    println!("cargo:rerun-if-changed=linker.ld");

    // Blink library available as standalone test binary.
    // Deep kernel integration will be done via shared memory IPC.

    // x509-hardening-a: bake the build host's current Unix epoch
    // seconds into the binary so validity-period checks have a
    // monotonic floor without a wall-clock RTC. Bat_OS is bare-metal
    // and has no NTP/RTC; this is the lower bound the verifier uses
    // to reject certs whose notBefore is in the future. Operators
    // can override at runtime once a verified time source lands
    // (out of scope for this PR).
    let build_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        // 2026-01-01 floor — a clock-skewed build host must not
        // accidentally produce a binary that accepts pre-2026 certs
        // as "from the future".
        .unwrap_or(1_735_689_600);
    println!("cargo:rustc-env=BAT_OS_BUILD_UNIX={build_unix}");
    // The env is implicit-input to the compile, so cargo doesn't know
    // to rerun on time changes. Force a rerun whenever build.rs itself
    // changes — that's the closest cheap signal.
    println!("cargo:rerun-if-changed=build.rs");
}
