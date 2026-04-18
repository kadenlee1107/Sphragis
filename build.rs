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

    // Blink library available as standalone test binary.
    // Deep kernel integration will be done via shared memory IPC.
}
