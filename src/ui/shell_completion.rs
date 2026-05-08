//! Tab completion for the Bat_OS shell.
//!
//! Both `main::serial_shell` and `ui::shell::run` read keystrokes one
//! byte at a time. When the user presses Tab (`0x09`) we want to
//! complete the partial token they've typed against the set of known
//! command names. This module owns:
//!
//! * `COMMAND_NAMES` — the canonical sorted list of every command the
//!   shell's `execute()` dispatcher accepts. Aliases (`ls` / `files`,
//!   `cat` / `read`, etc.) are listed separately so completion
//!   surfaces every name the user might type.
//!
//! * `complete_command()` — given a prefix, returns a `Completion`
//!   describing what the input loop should do (extend the buffer to
//!   a unique match, list candidates and extend to the common
//!   prefix on multiple, or no-op on zero).
//!
//! `Completion` is fixed-size + `Copy` — no heap, no allocation. The
//! caller copies bytes out of the returned struct.

#![allow(clippy::too_many_arguments)]

/// Every shell command name. Keep alphabetically sorted; if you add
/// a new arm to `ui::shell::execute()`, add the literal here too.
/// Order is not load-bearing for correctness (we scan linearly), but
/// keeping it sorted makes it trivial to grep + diff.
pub static COMMAND_NAMES: &[&str] = &[
    "audit",
    "audit-flush",
    "batcave",
    "batcave-fw-allow",
    "batcave-fw-deny",
    "batcave-fw-list",
    "blk-selftest",
    "blk-status",
    "c++",
    "cat",
    "cave-policy-selftest",
    "cave-seal-selftest",
    "cave-syscall-allow",
    "cave-syscall-clear",
    "cave-syscall-deny",
    "cave-syscall-list",
    "cave-syscall-selftest",
    "clear",
    "cls",
    "cookies",
    "cpol-add",
    "cpol-add-sni",
    "cpol-byte-rate",
    "cpol-check",
    "cpol-clear",
    "cpol-daemon-list",
    "cpol-daemon-show",
    "cpol-flow-rate",
    "cpol-flow-rate-selftest",
    "cpol-list",
    "cpol-rate",
    "cpol-rate-clear",
    "cpol-rate-list",
    "cpol-rate-selftest",
    "cpol-rate-show",
    "cpol-show",
    "cpol-sni-selftest",
    "cpol-sync",
    "cxx",
    "delete",
    "dns",
    "edit",
    "fetch",
    "files",
    "firewall",
    "fw",
    "gcm-selftest",
    "hello",
    "help",
    "ifconfig",
    "ipc-selftest",
    "kbd",
    "kbd-stats",
    "kbd-trace",
    "ls",
    "mem",
    "memory",
    "nat-beacon-reset",
    "nat-beacon-selftest",
    "nat-beacons",
    "nat-gc-force",
    "nat-gc-selftest",
    "nat-rewrite-selftest",
    "nat-selftest",
    "net",
    "nic-status",
    "origin",
    "origin-allow",
    "origin-mode",
    "otp-consume",
    "otp-dump",
    "otp-stats",
    "panic",
    "ping",
    "posix",
    "pq-interop",
    "pq-selftest",
    "pq-sig-selftest",
    "pq-tls-selftest",
    "read",
    "resolve",
    "rm",
    "scheduler-selftest",
    "screen",
    "secure-ipc-selftest",
    "secure-ipc-wire-selftest",
    "status",
    "tcp-list",
    "tcp-listen",
    "tcp-selftest",
    "threads",
    "uname",
    "uptime",
    "verify",
    "whoami",
    "write",
    "x509-selftest",
];

/// Maximum bytes we'll ever extend the input buffer by in a single
/// completion. Sized to fit the longest command name (currently
/// `cpol-flow-rate-selftest` at 23 bytes) plus headroom.
pub const MAX_EXTENSION_LEN: usize = 32;

/// Maximum number of candidates we surface on multi-match. Beyond
/// this we just truncate the list (the user can refine the prefix).
pub const MAX_CANDIDATES: usize = 32;

/// Result of looking up `prefix` against the command table.
///
/// Layout chosen for `Copy` + no_std: caller reads `match_count`
/// first and switches on it. Bytes in `extension[..extension_len]`
/// are valid ASCII (the command names are ASCII-only).
#[derive(Clone, Copy)]
pub struct Completion {
    /// 0 = no match, 1 = unique, otherwise the number of matches
    /// (clamped to `MAX_CANDIDATES` for the visible list).
    pub match_count: u8,
    /// Bytes to append to the cmd buffer. For `match_count == 1`,
    /// this is the rest of the unique command. For `match_count > 1`,
    /// this is the longest common prefix beyond the input.
    pub extension: [u8; MAX_EXTENSION_LEN],
    pub extension_len: u8,
    /// On `match_count > 1`, the candidate list (truncated at
    /// `MAX_CANDIDATES`). Each entry is `'static` since they all
    /// point into `COMMAND_NAMES`.
    pub candidates: [&'static str; MAX_CANDIDATES],
    pub candidates_len: u8,
}

impl Completion {
    pub const fn empty() -> Self {
        Self {
            match_count: 0,
            extension: [0; MAX_EXTENSION_LEN],
            extension_len: 0,
            candidates: [""; MAX_CANDIDATES],
            candidates_len: 0,
        }
    }

    pub fn extension_bytes(&self) -> &[u8] {
        &self.extension[..self.extension_len as usize]
    }

    pub fn candidate_slice(&self) -> &[&'static str] {
        &self.candidates[..self.candidates_len as usize]
    }
}

/// Find completions for a command-name prefix.
///
/// `prefix` is the bytes the user has typed so far in the FIRST token
/// of the line (autofill of arguments lands in a follow-up). Empty
/// prefix lists everything.
///
/// Returns a `Completion` describing the action the input loop
/// should take.
pub fn complete_command(prefix: &str) -> Completion {
    let mut out = Completion::empty();
    let mut count: u32 = 0;

    // First pass: count + collect matches into `candidates`.
    for &name in COMMAND_NAMES {
        if !name.starts_with(prefix) {
            continue;
        }
        if (out.candidates_len as usize) < MAX_CANDIDATES {
            out.candidates[out.candidates_len as usize] = name;
            out.candidates_len += 1;
        }
        count = count.saturating_add(1);
    }

    out.match_count = count.min(255) as u8;

    if count == 0 {
        return out;
    }

    if count == 1 {
        // Unique match — extension is `name[prefix.len()..]`.
        let only = out.candidates[0];
        let tail = &only.as_bytes()[prefix.len()..];
        let take = tail.len().min(MAX_EXTENSION_LEN);
        out.extension[..take].copy_from_slice(&tail[..take]);
        out.extension_len = take as u8;
        return out;
    }

    // Multiple matches — extension is the longest common prefix
    // beyond the input. Walk one byte at a time across the candidates
    // until they disagree. Snapshot length + first-candidate bytes
    // into locals so the loop's writes to `out.extension` don't
    // borrow-conflict with reads of `out.candidates`.
    let n = out.candidates_len as usize;
    let prefix_len = prefix.len();
    let mut common = 0usize;
    'outer: loop {
        if common >= MAX_EXTENSION_LEN { break; }
        let pos = prefix_len + common;
        let first = out.candidates[0].as_bytes();
        if pos >= first.len() { break; }
        let want = first[pos];
        for i in 1..n {
            let cb = out.candidates[i].as_bytes();
            if pos >= cb.len() || cb[pos] != want {
                break 'outer;
            }
        }
        out.extension[common] = want;
        common += 1;
    }
    out.extension_len = common as u8;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_prefix_lists_everything() {
        let r = complete_command("");
        assert!(r.match_count as usize >= COMMAND_NAMES.len().min(MAX_CANDIDATES));
    }

    #[test]
    fn unique_prefix_completes_to_full_name() {
        // `pq-interop` is the only command starting with `pq-i`.
        let r = complete_command("pq-i");
        assert_eq!(r.match_count, 1);
        assert_eq!(r.extension_bytes(), b"nterop");
    }

    #[test]
    fn ambiguous_prefix_extends_to_common() {
        // All `cpol-rate*` share the prefix `cpol-rate` — typing `cpol-r`
        // should extend us to `cpol-rate` (the common prefix).
        let r = complete_command("cpol-r");
        assert!(r.match_count > 1);
        assert_eq!(r.extension_bytes(), b"ate");
    }

    #[test]
    fn no_match_returns_zero() {
        let r = complete_command("zz-nothing");
        assert_eq!(r.match_count, 0);
        assert_eq!(r.extension_len, 0);
    }

    #[test]
    fn command_table_is_sorted() {
        for w in COMMAND_NAMES.windows(2) {
            assert!(w[0] < w[1], "out of order: {:?} >= {:?}", w[0], w[1]);
        }
    }
}
