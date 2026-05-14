//! Tab completion for the Sphragis shell.
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
    "audit-chain",
    "audit-flush",
    "caves",
    "caves-fw-allow",
    "caves-fw-deny",
    "caves-fw-list",
    "batfs-quota-selftest",
    "blk-selftest",
    "blk-status",
    "block-on-selftest",
    "c++",
    "caps",
    "cat",
    "cave-policy-selftest",
    "cave-private-selftest",
    "cave-quota",
    "cave-seal-selftest",
    "cave-syscall-allow",
    "cave-syscall-clear",
    "cave-syscall-deny",
    "cave-syscall-list",
    "cave-syscall-selftest",
    "cave-usage",
    "clear",
    "clip",
    "cls",
    "comms",
    "conntrack-selftest",
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
    "date",
    "delete",
    "dmesg",
    "dns",
    "edit",
    "fds",
    "fetch",
    "files",
    "firewall",
    "fw",
    "gcm-selftest",
    "hash",
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
    "mount-ns",
    "mount-ns-selftest",
    "nat-beacon-reset",
    "nat-beacon-selftest",
    "nat-beacons",
    "nat-bind",
    "nat-bindings",
    "nat-forward",
    "nat-frag-selftest",
    "nat-gc-force",
    "nat-gc-selftest",
    "nat-pump",
    "nat-reply",
    "nat-reset",
    "nat-rewrite-selftest",
    "nat-selftest",
    "nat-stats",
    "nat-sync",
    "nat-table",
    "net",
    "nic-status",
    "ocsp-selftest",
    "origin",
    "origin-allow",
    "origin-mode",
    "otp-consume",
    "otp-dump",
    "otp-stats",
    "panic",
    "ping",
    "pipe-selftest",
    "pkg",
    "posix",
    "pq-comms-selftest",
    "pq-selftest",
    "pq-sig-selftest",
    "pq-tls-selftest",
    "procs",
    "ps",
    "quota-selftest",
    "read",
    "redirect-selftest",
    "release-pubkey",
    "release-verify",
    "resolve",
    "rm",
    "scheduler-selftest",
    "screen",
    "secstatus",
    "secure-ipc-selftest",
    "secure-ipc-wire-selftest",
    "shm-selftest",
    "status",
    "sys-caves-selftest",
    "sys-wg-ipc-selftest",
    "sys-wg-selftest",
    "task",
    "tcp-list",
    "tcp-listen",
    "tcp-selftest",
    "threads",
    "time-selftest",
    "time-sync-https",
    "tz",
    "uname",
    "unix-sock-selftest",
    "uptime",
    "verify",
    "wg-dispatch-selftest",
    "wg-endpoint-selftest",
    "wg-initiator-e2e-selftest",
    "wg-initiator-selftest",
    "wg-replay-selftest",
    "wg-selftest",
    "wg-test-outbound",
    "wg-wire-selftest",
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

// ─── Argument completion ──────────────────────────────────────────────
//
// Past the first space the user is typing arguments, not a command
// name. Different commands take different argument types: file names
// (`read`, `cat`, `rm`, `verify`, `edit`, `write`), cave names
// (`cpol-show`, `cpol-add-sni`, `cpol-clear`, `cave-syscall-*`,
// `caves-fw-*`), test-binary names (`run`), or nothing for v1.
//
// Candidates are runtime-enumerated (the file table or cave
// registry), so this type owns its candidate bytes rather than
// borrowing `&'static str` like `Completion` does.

/// Per-name buffer width for the argument-candidate list. Sized for
/// the longest realistic argument:
/// - batfs filenames cap at `batfs::FILE_NAME_LEN` = 64 bytes
/// - cave names cap at 32 bytes by `caves::cave::MAX_NAME_LEN`
const MAX_ARG_NAME: usize = 64;

/// What kind of argument the next token of `cmd` expects. Drives
/// which enumerator the shell should pull candidates from.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArgKind {
    /// Command takes a file name from BatFS as its next argument.
    File,
    /// Command takes a cave name (registered via `cave_policy`) as
    /// its next argument.
    Cave,
    /// Command takes a test-binary name from the small hardcoded
    /// set the cave loader includes via `include_bytes!`.
    Binary,
    /// Argument is one of a fixed list of literal keywords — a
    /// subcommand (`pkg <install|list|...>`), a small enum
    /// (`kbd-trace <on|off>`), or a common-path suggestion list
    /// (`tz <-8|-5|+0|...>`).
    Literal(&'static [&'static str]),
    /// No completable argument (or out of v1 scope).
    None,
}

// ── Subcommand keyword tables ────────────────────────────────────────
//
// One table per multi-arm command. These drive both the subcommand
// completion at `arg_index == 0` and the dispatch into
// `arg_kind_for` for the next argument (`arg_index >= 1`).

/// `pkg <install|list|remove|stage>`
const SUB_PKG: &[&str] = &["install", "list", "remove", "stage"];

/// `comms <connect|send|identify|pin|my-id>`
const SUB_COMMS: &[&str] = &["connect", "identify", "my-id", "pin", "send"];

/// `mount-ns <ls|write|read|rm>` (cave-scoped BatFS view).
const SUB_MOUNT_NS: &[&str] = &["ls", "read", "rm", "write"];

/// `clip <set|yank-back|push|pull|clear|show>`
const SUB_CLIP: &[&str] = &["clear", "pull", "push", "set", "show", "yank-back"];

/// `audit` (only `all` is a literal — numeric N is free-form).
const SUB_AUDIT: &[&str] = &["all"];

/// `cookies <clear>` (bare `cookies` dumps the jar).
const SUB_COOKIES: &[&str] = &["clear"];

/// `kbd-trace <on|off>`
const SUB_KBD_TRACE: &[&str] = &["off", "on"];

/// `screen <scale>` — typical zoom levels for the FB dump.
const SUB_SCREEN: &[&str] = &["1", "2", "4", "8", "16"];

/// `tz <offset>` — common UTC offsets with sign.
const SUB_TZ: &[&str] = &[
    "-10", "-8", "-7", "-6", "-5", "-4", "-3",
    "+0", "+1", "+2", "+3", "+4", "+5:30", "+8", "+9", "+10",
];

/// `hash <algo> <file>` — first arg is the algorithm.
const SUB_HASH_ALGO: &[&str] = &[
    "blake3", "sha256", "sha384", "sha3-256", "sha3-384", "sha3-512",
];

/// `caves <enter|stop|...>` — keep alphabetised for easy diff.
const SUB_BATCAVE: &[&str] = &[
    "bb",       "busybox",       "create",        "destroy",
    "display",  "docker-create", "docker-destroy", "docker-list",
    "docker-ping", "docker-run", "enter",         "grant",
    "gui",      "install",       "ipc",           "kits",
    "list",     "mkdir",         "pipe",          "revoke",
    "run",      "seal",          "stop",          "test",
    "uname",
];

/// Look up the argument kind for the next token of `parts[0]`.
/// `parts` is the line's tokens parsed left-to-right, where
/// `parts[0]` is the command and `parts[1..arg_index]` are the
/// already-typed prior arguments. `arg_index` is the index of the
/// token currently being completed.
///
/// Returning `ArgKind::Literal(slice)` for a subcommand position
/// gives `complete_argument` a static candidate list to filter.
pub fn arg_kind_for_parts(parts: &[&str], arg_index: usize) -> ArgKind {
    let cmd = parts.first().copied().unwrap_or("");
    let sub = parts.get(1).copied().unwrap_or("");

    // ── First-arg subcommand keyword tables ──
    if arg_index == 0 {
        match cmd {
            "pkg"        => return ArgKind::Literal(SUB_PKG),
            "comms"      => return ArgKind::Literal(SUB_COMMS),
            "mount-ns"   => return ArgKind::Literal(SUB_MOUNT_NS),
            "clip"       => return ArgKind::Literal(SUB_CLIP),
            "audit"      => return ArgKind::Literal(SUB_AUDIT),
            "cookies"    => return ArgKind::Literal(SUB_COOKIES),
            "kbd-trace"  => return ArgKind::Literal(SUB_KBD_TRACE),
            "screen"     => return ArgKind::Literal(SUB_SCREEN),
            "tz"         => return ArgKind::Literal(SUB_TZ),
            "hash"       => return ArgKind::Literal(SUB_HASH_ALGO),
            "caves"    => return ArgKind::Literal(SUB_BATCAVE),
            _ => {}
        }
    }

    match (cmd, sub, arg_index) {
        // ── File-taking first arguments ──
        ("read"|"cat"|"rm"|"delete"|"verify"|"edit"|"write", _, 0) => ArgKind::File,

        // ── Cave-taking first arguments ──
        ("cpol-show"|"cpol-clear"|"cpol-sync"|"cpol-rate-show"
            |"cpol-rate-clear"|"cpol-daemon-show", _, 0) => ArgKind::Cave,
        ("cpol-add"|"cpol-add-sni"|"cpol-check"
            |"cpol-rate"|"cpol-byte-rate"|"cpol-flow-rate", _, 0) => ArgKind::Cave,
        ("cave-syscall-deny"|"cave-syscall-allow"
            |"cave-syscall-list"|"cave-syscall-clear", _, 0) => ArgKind::Cave,
        ("caves-fw-allow"|"caves-fw-deny", _, 0) => ArgKind::Cave,

        // ── Binary-taking first arguments ──
        ("run", _, 0) => ArgKind::Binary,

        // ── Multi-arg dispatch: (cmd, subcommand) -> next arg's kind ──
        // `pkg install <bundle.bpkg>` and `pkg remove <bundle>` —
        // both are BatFS files. `pkg stage <name> <ip:port>` is a
        // fresh filename, no completion. `pkg list` takes nothing.
        ("pkg", "install"|"remove", 1) => ArgKind::File,

        // `mount-ns <read|rm|write> <file>` — all three target a
        // file in the active cave's mount namespace.
        ("mount-ns", "read"|"rm"|"write", 1) => ArgKind::File,

        // `caves <enter|stop|destroy|grant|revoke|seal|gui
        // |display|uname|test|run|kits|install|pipe|ipc|bb
        // |busybox|mkdir> <cave>` — second token is a cave name.
        ("caves",
            "enter"|"stop"|"destroy"|"grant"|"revoke"|"seal"
            |"gui"|"display"|"uname"|"test"|"run"|"kits"
            |"install"|"pipe"|"ipc"|"bb"|"busybox"|"mkdir"
            |"docker-destroy"|"docker-run"|"docker-ping", 1) => ArgKind::Cave,

        // `hash <algo> <file>` — algo is arg 0 (Literal above);
        // arg 1 is the file in BatFS.
        ("hash", _, 1) => ArgKind::File,

        _ => ArgKind::None,
    }
}

/// Backward-compat alias for callers that only know the
/// `(cmd, arg_index)` shape. Routes through `arg_kind_for_parts`
/// with `parts = [cmd]`, so subcommand-aware dispatch only kicks in
/// at `arg_index == 0`. New callers should pass the full parts
/// slice via `arg_kind_for_parts`.
#[allow(dead_code)]
pub fn arg_kind_for(cmd: &str, arg_index: usize) -> ArgKind {
    arg_kind_for_parts(&[cmd], arg_index)
}

/// Argument completion result. Same shape as `Completion` but the
/// candidate list is owned (the byte buffer lives in this struct)
/// rather than `&'static`, so it can hold runtime-enumerated names.
#[derive(Clone, Copy)]
pub struct ArgCompletion {
    pub match_count: u8,
    pub extension: [u8; MAX_EXTENSION_LEN],
    pub extension_len: u8,
    /// Up to MAX_CANDIDATES names. Each row is a flat byte buffer.
    pub names: [[u8; MAX_ARG_NAME]; MAX_CANDIDATES],
    pub name_lens: [u8; MAX_CANDIDATES],
    pub names_len: u8,
}

impl ArgCompletion {
    pub const fn empty() -> Self {
        Self {
            match_count: 0,
            extension: [0; MAX_EXTENSION_LEN],
            extension_len: 0,
            names: [[0; MAX_ARG_NAME]; MAX_CANDIDATES],
            name_lens: [0; MAX_CANDIDATES],
            names_len: 0,
        }
    }

    pub fn extension_bytes(&self) -> &[u8] {
        &self.extension[..self.extension_len as usize]
    }

    /// Borrow a candidate's bytes by index.
    pub fn name_at(&self, i: usize) -> &[u8] {
        &self.names[i][..self.name_lens[i] as usize]
    }

    fn try_push(&mut self, name: &[u8]) {
        if (self.names_len as usize) >= MAX_CANDIDATES { return; }
        let n = name.len().min(MAX_ARG_NAME);
        let row = self.names_len as usize;
        self.names[row][..n].copy_from_slice(&name[..n]);
        self.name_lens[row] = n as u8;
        self.names_len += 1;
    }
}

/// Run argument completion. Calls the right enumerator based on
/// `kind`, filters by the `current` prefix, and computes the
/// extension (unique completion or longest common prefix).
pub fn complete_argument(kind: ArgKind, current: &str) -> ArgCompletion {
    let mut out = ArgCompletion::empty();
    let prefix = current.as_bytes();
    let mut count: u32 = 0;
    let prefix_for_filter = prefix; // Captured for closures.
    let mut consider = |name: &[u8]| {
        if name.starts_with(prefix_for_filter) {
            count = count.saturating_add(1);
            out.try_push(name);
        }
    };

    match kind {
        ArgKind::None => return out,
        ArgKind::File => {
            // gap-audit 032: tab completion scopes to the active
            // cave's mount namespace — completing `cat <TAB>` from
            // inside a cave only suggests that cave's files.
            crate::fs::batfs::ns_list(|name, _size, _enc| consider(name.as_bytes()));
        }
        ArgKind::Cave => {
            crate::caves::cave::list(|cv| consider(cv.name_str().as_bytes()));
        }
        ArgKind::Binary => {
            // Hardcoded set the loader includes — keep in sync with
            // ui::shell::execute()'s `cmd_run_elf` arms.
            for name in &[b"hello".as_slice(), b"hello_libc", b"hello_threads",
                          b"posix", b"cxx"] {
                consider(name);
            }
        }
        ArgKind::Literal(words) => {
            // Subcommand keywords / small enum literals / common-path
            // suggestions. The candidate set is `'static` so we read
            // straight from the slice without an extra enumerator.
            for w in words.iter() {
                consider(w.as_bytes());
            }
        }
    }
    out.match_count = count.min(255) as u8;

    if count == 0 {
        return out;
    }
    if count == 1 {
        // Unique — extension is name[prefix.len()..].
        let row = out.name_lens[0] as usize;
        let name_bytes = &out.names[0][..row];
        let tail = &name_bytes[prefix.len()..];
        let take = tail.len().min(MAX_EXTENSION_LEN);
        out.extension[..take].copy_from_slice(&tail[..take]);
        out.extension_len = take as u8;
        return out;
    }

    // Common prefix across all visible candidates.
    let n = out.names_len as usize;
    let prefix_len = prefix.len();
    let mut common = 0usize;
    'outer: loop {
        if common >= MAX_EXTENSION_LEN { break; }
        let pos = prefix_len + common;
        let first_len = out.name_lens[0] as usize;
        if pos >= first_len { break; }
        let want = out.names[0][pos];
        for i in 1..n {
            let cl = out.name_lens[i] as usize;
            if pos >= cl || out.names[i][pos] != want {
                break 'outer;
            }
        }
        out.extension[common] = want;
        common += 1;
    }
    out.extension_len = common as u8;
    out
}

/// Parse the current input buffer and figure out:
///   * whether the cursor is in the first token (command name) or
///     past a space (argument)
///   * if past a space, which command we're in and which argument
///     index this is
///
/// Returns `None` for "still inside the command word" — caller falls
/// through to `complete_command()`.
#[allow(dead_code)]
pub fn split_for_completion(line: &str) -> Option<(&str, usize, &str)> {
    let bytes = line.as_bytes();
    let first_space = bytes.iter().position(|&b| b == b' ')?;
    let cmd = &line[..first_space];
    // Walk over space-separated args; index = how many full args
    // already typed (i.e. the index of the current trailing token).
    let rest = &line[first_space + 1..];
    let mut idx = 0usize;
    let mut last_token_start = 0usize;
    let rest_bytes = rest.as_bytes();
    for i in 0..rest_bytes.len() {
        if rest_bytes[i] == b' ' {
            // A space terminates the previous token. The character
            // at i+1 is the start of the next token (if any).
            idx += 1;
            last_token_start = i + 1;
        }
    }
    let current = &rest[last_token_start..];
    Some((cmd, idx, current))
}

/// Maximum prior-arg tokens we surface to the dispatcher. Larger
/// grammars (`caves docker-create <name> <image> <caps-csv>`) cap
/// at this width; beyond it the new arg falls through to `None`.
pub const MAX_PRIOR_PARTS: usize = 8;

/// Result of `split_for_completion_parts`. Carries a small array of
/// fully-typed tokens (parts[0] = command, parts[1..arg_index] =
/// prior args), the current partial token at `parts[arg_index]`,
/// and the count of populated slots.
pub struct SplitParts<'a> {
    pub parts: [&'a str; MAX_PRIOR_PARTS],
    pub parts_len: usize,
    pub arg_index: usize,
    pub current: &'a str,
}

/// Extended companion to `split_for_completion` that also exposes
/// the prior arguments. Required by `arg_kind_for_parts` to make
/// subcommand-aware dispatch decisions (e.g. `pkg install <TAB>`
/// has to know about `install` to pick `ArgKind::File`).
///
/// Returns `None` for "still inside the command word" — caller
/// falls through to `complete_command()`.
pub fn split_for_completion_parts(line: &str) -> Option<SplitParts<'_>> {
    let first_space = line.bytes().position(|b| b == b' ')?;
    let cmd = &line[..first_space];
    let rest = &line[first_space + 1..];

    let mut info = SplitParts {
        parts: [""; MAX_PRIOR_PARTS],
        parts_len: 1,
        arg_index: 0,
        current: "",
    };
    info.parts[0] = cmd;

    // Walk `rest`, splitting on each space. `idx` counts spaces
    // seen so the trailing token (parts[arg_index]) is the one
    // the user is currently typing.
    let mut tok_start = 0usize;
    let mut idx = 0usize;
    let rest_bytes = rest.as_bytes();
    for i in 0..rest_bytes.len() {
        if rest_bytes[i] == b' ' {
            let tok = &rest[tok_start..i];
            // Record into parts[1 + idx] if there's room.
            if 1 + idx < MAX_PRIOR_PARTS {
                info.parts[1 + idx] = tok;
                info.parts_len = (1 + idx + 1).max(info.parts_len);
            }
            idx += 1;
            tok_start = i + 1;
        }
    }
    let current = &rest[tok_start..];
    info.current = current;
    info.arg_index = idx;
    // The trailing-token slot (where the user is typing now) also
    // belongs in parts so callers can include it in dispatch if
    // they need to. arg_index is the index into parts AFTER
    // accounting for the leading command at parts[0].
    if 1 + idx < MAX_PRIOR_PARTS {
        info.parts[1 + idx] = current;
        info.parts_len = (1 + idx + 1).max(info.parts_len);
    }
    Some(info)
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
