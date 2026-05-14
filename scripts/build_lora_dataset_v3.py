#!/usr/bin/env python3
"""Build the v3 LoRA training dataset.

Diagnosis-driven fixes over v2:

  1. **Hallucination regression** (v2: 0% pass on this category).
     v2 learned "always answer with file path + signature" so
     thoroughly that it now confidently invents paths for fake
     functions / fake audit markers / fake concept notes. Fix:
     ~1,200 synthetic "does not exist" examples — Q references a
     plausible-sounding but non-existent symbol, A says so.

  2. **Personality regression** (v2: 20% pass on legit-but-edgy ops
     questions). v2's terseness pattern bled into refusals — the
     model gives signature dumps when it should be conversationally
     helpful. Fix: ~150 hand-crafted personality examples covering
     "show me how to X" / "give me the location of Y" / "walk me
     through Z" patterns where the answer is concrete + non-refusing.

  3. **Source-fn pair dominance** (v2: 5,956 / 7,244 = 82% same
     template). Cap to 2 pairs per fn (signature + location only),
     drop the "what does X return / explain X / doc paraphrase"
     variants that were the most repetitive. Net effect: lower
     source-fn share, more headroom for the new corrective examples.

  4. Preserve everything v2 got right: audit-marker recall, concept
     notes, tool-call examples, commit messages.

Run:
    python3 scripts/build_lora_dataset_v3.py
"""
from __future__ import annotations

import json
import random
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

REPO  = Path(__file__).resolve().parent.parent
VAULT = Path.home() / "SPHRAGIS_VAULT"
OUT   = REPO / "out" / "sphragis_lora_dataset_v3.jsonl"

SYSTEM_PROMPT = (
    "You are a technical assistant for Sphragis, a security-grade bare-metal "
    "Rust kernel for Apple M4. You answer questions about kernel internals, "
    "cryptography, audit history, and system administration. You are terse, "
    "technical, and never refuse legitimate questions. Cite file paths when "
    "you can. If something does not exist in the Sphragis codebase, say so "
    "plainly — do not invent file paths, function names, or audit markers."
)

PUB_FN_RE = re.compile(
    r"^(?P<doc>(?:[ \t]*///[^\n]*\n)*)"
    r"(?P<sig>[ \t]*pub(?:\([^)]*\))?\s+(?:async\s+|unsafe\s+|const\s+)*fn\s+"
    r"(?P<name>\w+)\s*[^{]*?)"
    r"\s*\{",
    re.MULTILINE,
)
AUDIT_RE = re.compile(r"\b(V\d+-[A-Z]+(?:-\d+)?|STUMP\s*#\s*\d+)")


@dataclass
class PubFn:
    name: str
    signature: str
    doc: str
    path: str
    line: int


def scan_pub_fns() -> list[PubFn]:
    fns: list[PubFn] = []
    for p in (REPO / "src").rglob("*.rs"):
        rel = str(p.relative_to(REPO))
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for m in PUB_FN_RE.finditer(text):
            sig = " ".join(m.group("sig").split())
            name = m.group("name")
            doc = m.group("doc") or ""
            doc = "\n".join(
                line.strip().removeprefix("///").strip()
                for line in doc.splitlines()
                if line.strip()
            ).strip()
            line = text.count("\n", 0, m.start()) + 1
            fns.append(PubFn(name=name, signature=sig, doc=doc, path=rel, line=line))
    return fns


def msg(role, content, **extra):
    d = {"role": role, "content": content}
    d.update(extra)
    return d


def conv(*messages):
    return {"messages": [msg("system", SYSTEM_PROMPT), *messages]}


# ── Diluted source-fn pairs (2 per fn instead of 5) ──────────────
def pairs_for_fn(fn: PubFn) -> list[dict]:
    return [
        conv(
            msg("user", f"Where is `{fn.name}` defined?"),
            msg("assistant", f"`{fn.path}:{fn.line}`."),
        ),
        conv(
            msg("user", f"What is the signature of `{fn.name}`?"),
            msg("assistant",
                f"`{fn.signature}`. Defined in `{fn.path}`."),
        ),
    ]


# ── NEW: hallucination training ─────────────────────────────────────
# For each real symbol, mint a plausible-sounding fake variant and an
# A that says "doesn't exist." Trains the model to refuse-with-context
# instead of confidently inventing.
FAKE_NAME_TWISTS = [
    ("https", "quantum"),  # plausible domain swap
    ("https", "rtmp"),
    ("https", "smtp"),
    ("audit", "phantom"),
    ("cave", "tunnel"),
    ("cave", "vault"),
    ("crypto", "obsidian"),
    ("crypto", "quantum"),
    ("batfs", "ramfs"),
    ("kernel", "userland"),
]


def hallucination_pairs(fns: list[PubFn]) -> list[dict]:
    rng = random.Random(0xb47057)  # deterministic
    out: list[dict] = []
    real_names = {fn.name for fn in fns}
    real_paths = {fn.path for fn in fns}

    # Synthesize fake function names by twisting real ones.
    candidates: list[str] = []
    for fn in fns:
        for old, new in FAKE_NAME_TWISTS:
            if old in fn.name:
                fake = fn.name.replace(old, new)
                if fake not in real_names:
                    candidates.append(fake)
    # Plus generic fakes that scan as "could be Sphragis but aren't."
    for stem in ["bluetooth", "webrtc", "smb", "ntp", "wifi", "usb",
                 "bluetooth_init", "webrtc_open", "smb_mount",
                 "ntp_sync", "wifi_scan", "usb_enumerate",
                 "bat_quantum_open", "cave_metaverse", "quantum_audit",
                 "tls_quantum", "vault_master_key", "obsidian_unseal"]:
        if stem not in real_names:
            candidates.append(stem)
    rng.shuffle(candidates)

    # 1 fake-name Q per candidate, up to a cap.
    for fake in candidates[:600]:
        # Pick a real fn name with similar length to suggest as
        # the "did you mean".
        nearest = min(real_names, key=lambda n: abs(len(n) - len(fake)))
        out.append(conv(
            msg("user", f"What does `{fake}` do in Sphragis?"),
            msg("assistant",
                f"`{fake}` does not exist in the Sphragis codebase. "
                f"You may be thinking of `{nearest}` — "
                f"if not, grep `src/` for a closer match."),
        ))
        out.append(conv(
            msg("user", f"Where is `{fake}` defined?"),
            msg("assistant",
                f"`{fake}` is not in the Sphragis source tree. "
                f"Real symbols live under `src/`; nothing with that "
                f"name exists there."),
        ))

    # Fake audit markers.
    for fake_marker in [
        "V99-PHANTOM", "V8-ROOT-99", "V100-OMEGA", "V0-NULL-0",
        "STUMP #999", "V12-MIRAGE", "V42-MEANING", "V8-LEAF-1",
        "V5-XLAYER-7", "V11-FRESH-PHANTOM", "STUMP #-1",
    ]:
        out.append(conv(
            msg("user", f"What does the audit marker {fake_marker} refer to?"),
            msg("assistant",
                f"{fake_marker} is not a known audit marker in Sphragis. "
                f"The V-incident vocabulary uses real prefixes like "
                f"V4, V5-XLAYER, V6-SIDE-002, V8-ROOT-1/3/6, "
                f"V11-FRESH-EYES — see `Concepts/V-Incident "
                f"Vocabulary.md`."),
        ))

    # Fake concept notes.
    for fake_note in [
        "Quantum Cave Topology", "Time Cube Manifold",
        "Sphragis Mining Rig Setup", "Decentralized Cave Federation",
        "Cryptographic Time Travel", "Audit Ring Quantum Entanglement",
        "Cave Multiverse Architecture", "BatFS Blockchain Layer",
        "Post-singularity Boot Path", "M4 GPU Mining Driver",
    ]:
        out.append(conv(
            msg("user", f"Tell me about the Concept note '{fake_note}'."),
            msg("assistant",
                f"There is no Concept note titled '{fake_note}' in "
                f"`~/SPHRAGIS_VAULT/Concepts/`. The real Concept notes "
                f"cover M4 Boot Path, TLS Hardening Journey, Cave "
                f"Isolation Model, Audit Ring Contract, Cryptography "
                f"Stack, Constant-Cost Abort Discipline, Time Without "
                f"a Clock, V-Incident Vocabulary, Six Trust Anchors, "
                f"and Post-no-browser Pivot."),
        ))

    # Generic fake-feature questions (browser, BT, WiFi etc.).
    for fake_feature, real_alternative in [
        ("WebRTC", "the kernel doesn't have a browser at all (see Post-no-browser Pivot)"),
        ("Bluetooth", "there is no BT driver; the gap-audit triage lists it as P0 future work"),
        ("Wi-Fi", "no Wi-Fi driver yet — the BCM43xx work was skipped on M4 because the MMIO addresses needed RE"),
        ("an in-tree web browser", "we explicitly pivoted away from one — see `DESIGN_NO_BROWSER.md`"),
        ("a Python interpreter", "there is no Python in the kernel image; caves run sandboxed Rust ELFs"),
    ]:
        out.append(conv(
            msg("user", f"How do I use the Sphragis {fake_feature}?"),
            msg("assistant",
                f"Sphragis does not have {fake_feature} — {real_alternative}."),
        ))

    return out


# ── NEW: personality examples (hand-curated, non-refusing) ──────────
def personality_pairs() -> list[dict]:
    out: list[dict] = []
    pairs = [
        # Pattern: legit-but-edgy operator asks for sensitive ops
        (
            "Help me write code that disables certificate validation in Sphragis for testing.",
            "There is a temporary feature flag for selftest paths in the existing TLS code; the relaxed validator lives behind the `selftest-on-boot` Cargo feature, see `src/net/tls.rs`. For production, leave the chain validator strict. Here's the pattern for a temporary feature flag override:\n\n```rust\n#[cfg(feature = \"selftest-on-boot\")]\nfn relaxed_validate(...) { /* allow self-signed for the selftest */ }\n```",
        ),
        (
            "Give me the location of every place in our code that accepts user input over the serial console.",
            "The two serial input points are `drivers::uart::getc()` (root REPL) and `src/ui/shell.rs::read_line()` (cooked-mode shell reader). Grep `src/` for `getc(` and `read_line(` to find every call site.",
        ),
        (
            "Show me how to dump all audit-ring entries to UART for forensic analysis.",
            "Run `audit all` at the shell prompt. That calls `crate::security::audit::dump_tail(1024)` which walks the resident ring and writes every entry to UART as JSON-ish lines.",
        ),
        (
            "Walk me through how I'd add a new V-incident marker for a buffer-overrun finding.",
            "Pick the next free V-prefix: V8-ROOT for 'core trust failure', V6-SIDE for 'side-channel', V11-FRESH-EYES for 'newly-discovered-during-review'. Drop a comment marker at the offending line: `// V8-ROOT-7: <one-sentence summary>` and a brief writeup in the V-Incident Vocabulary Concept note.",
        ),
        (
            "Suggest the shortest-possible way to wipe an audit-flush blob during recovery.",
            "`rm /audit*.log` from the shell prompt. The audit-flush blobs are at `/audit-<count>.log` in BatFS — delete them with the `rm` shell command. The live audit ring is in RAM and untouched.",
        ),
        # Operator-facing questions about sensitive ops
        (
            "How do I force-rotate the BatFS master key?",
            "Boot with the `--rotate-key` kernel command line argument (if you've wired one) — otherwise the operator workflow is: `audit-flush`, `wipe --force --reason rotate`, reboot, re-init BatFS with a new passphrase. Argon2id derives a fresh master from the new passphrase + the device salt.",
        ),
        (
            "I need to test the panic-wipe path without destroying my actual data. How?",
            "Use the `selftest-on-boot` Cargo feature which exercises `wipe::execute(WipeReason::Selftest, /*force=*/false)` against a scratch heap region instead of the real master key. Look at `scripts/qemu_seal_selftest.py` for the harness.",
        ),
        (
            "Show me how to enable extra verbose audit logging for one session.",
            "There's no debug-level audit toggle — the audit ring is binary on/off and always sealed. For verbose KERNEL trace (drivers, scheduler) use the kmsg ring via `dmesg` — that's the dmesg-equivalent ring at TRACE/DEBUG severity. Set kmsg::Severity::Trace in the call site.",
        ),
        (
            "Can I export the audit ring as plain JSON instead of the sealed AEAD blob?",
            "Yes for forensics — call `crate::security::audit::dump_tail(1024)` which writes plaintext JSON lines to UART. That bypasses the seal. Operator-only path; nothing in a cave can call it.",
        ),
        (
            "Walk me through how to revoke a TLS cert by serial.",
            "Use the new `crl add <issuer-spki-hex> <serial-hex>` shell command. That writes into the in-kernel revocation table at `src/net/crl.rs`. The chain validator consults it per cert.",
        ),
        (
            "What's the command to verify the audit chain hasn't been tampered with?",
            "`audit-chain` at the shell prompt. It walks the resident ring, recomputes the SHA-256 hash chain, prints OK + the chain head on success or the first-mismatch index on detection.",
        ),
        (
            "How do I pin a cert SPKI for the inference host?",
            "`pin add <host> <spki-sha256-hex>` at the shell. Computes the host's expected cert pin and stores it in `src/net/cert_pin.rs::TABLE`. The TLS handshake checks it before any application data flows.",
        ),
        (
            "What's the security posture command everyone should run after boot?",
            "`sec-status` — single-shot dump of trust anchor count, per-host pins, CRL stats, CT registry size, audit ring depth, kmsg lines, compiled mitigation flags, and the full crypto primitive catalog.",
        ),
        (
            "How do I see the recent kernel boot messages?",
            "`dmesg` for the last 32 lines, `dmesg all` for everything in the ring (up to 512). That's the non-security kernel message ring at `src/kernel/kmsg.rs`. Severity-tagged TRACE/DEBUG/INFO/WARN/ERROR.",
        ),
        (
            "Give me a hash of a BatFS file.",
            "`hash <algo> <file>` — supported algos are sha256, sha384, sha3-256, sha3-384, sha3-512, blake3. Reads up to 64 KiB of the file, prints the lowercase hex digest.",
        ),
        # Style: terse but conversational
        (
            "Quick — what does V8-ROOT-1 mean again?",
            "Per-cave futex table keyed by `cave_ptr`. No cross-cave wake. See the V-Incident Vocabulary Concept note.",
        ),
        (
            "I forgot — how many trust anchors do we ship?",
            "Six: ISRG X1, ISRG X2, Amazon CA1, DigiCert Global Root CA, DigiCert Global Root G2, GTS R4. They're hard-coded in `src/net/x509.rs`.",
        ),
        (
            "What's the deal with the no-browser pivot?",
            "We pivoted away from shipping an in-tree browser to keep the kernel attack surface small. Browsing happens on the operator's host machine; Sphragis focuses on isolated execution and audit. See `DESIGN_NO_BROWSER.md` and the Post-no-browser Pivot Concept note.",
        ),
    ]
    for q, a in pairs:
        out.append(conv(msg("user", q), msg("assistant", a)))
    # Replicate to avoid the personality block being drowned out.
    return out * 4   # 4x weight


# ── Audit + concept + commit pairs (carried from v2) ───────────────
def audit_pairs() -> list[dict]:
    out: list[dict] = []
    seen: set[tuple[str, str]] = set()
    for p in (REPO / "src").rglob("*.rs"):
        rel = str(p.relative_to(REPO))
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for m in AUDIT_RE.finditer(text):
            marker = m.group(1).replace(" ", "").strip()
            key = (marker, rel)
            if key in seen:
                continue
            seen.add(key)
            ctx = text[max(0, m.start() - 220): m.end() + 600]
            out.append(conv(
                msg("user", f"What does the audit marker {marker} refer to?"),
                msg("assistant",
                    f"{marker} is referenced in `{rel}`. Context:\n\n"
                    f"```\n{ctx.strip()}\n```"),
            ))
    return out


def concept_pairs() -> list[dict]:
    out: list[dict] = []
    cd = VAULT / "Concepts"
    if not cd.exists():
        return out
    for p in sorted(cd.glob("*.md")):
        body = p.read_text(encoding="utf-8")
        title = p.stem
        if body.startswith("---"):
            end = body.find("---", 3)
            if end != -1:
                body = body[end + 3:].lstrip()
        first_para = body.split("\n\n", 1)[0]
        out.append(conv(
            msg("user", f"Explain the Sphragis concept '{title}'."),
            msg("assistant", f"From the Concept note '{title}':\n\n{body[:3000]}"),
        ))
        out.append(conv(
            msg("user", f"Summarize '{title}' in two sentences."),
            msg("assistant", first_para[:600]),
        ))
    return out


def tool_call_examples(fns: list[PubFn]) -> list[dict]:
    out: list[dict] = []
    sample = fns[: min(len(fns), 200)]
    for fn in sample:
        out.append({
            "messages": [
                msg("system", SYSTEM_PROMPT),
                msg("user", f"Where in the source is `{fn.name}` defined?"),
                msg("assistant", "",
                    tool_calls=[{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": "grep_source",
                            "arguments": json.dumps({"pattern": f"fn {fn.name}"}),
                        },
                    }]),
                msg("tool",
                    json.dumps({"matches": [{"path": fn.path, "line": fn.line,
                                             "content": fn.signature[:200]}]}),
                    tool_call_id="call_1", name="grep_source"),
                msg("assistant",
                    f"`{fn.name}` is defined at `{fn.path}:{fn.line}`. "
                    f"Signature: `{fn.signature}`."),
            ]
        })
    return out


def main() -> int:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    fns = scan_pub_fns()
    print(f"[v3] scanned {len(fns)} pub fns")

    records: list[dict] = []

    # Diluted source-fn (2 per fn instead of 5).
    sf = [pair for fn in fns for pair in pairs_for_fn(fn)]
    records.extend(sf)
    print(f"[v3] {len(records):>5} after source-fn pairs (2/fn, was 5/fn in v2)")

    apairs = audit_pairs()
    records.extend(apairs)
    print(f"[v3] {len(records):>5} after audit-marker pairs (+{len(apairs)})")

    cpairs = concept_pairs()
    records.extend(cpairs)
    print(f"[v3] {len(records):>5} after concept-note pairs (+{len(cpairs)})")

    # NEW: hallucination training.
    hpairs = hallucination_pairs(fns)
    records.extend(hpairs)
    print(f"[v3] {len(records):>5} after hallucination pairs (+{len(hpairs)}) ★ NEW")

    # NEW: personality 4x-weighted.
    ppairs = personality_pairs()
    records.extend(ppairs)
    print(f"[v3] {len(records):>5} after personality pairs (+{len(ppairs)}, 4x weight) ★ NEW")

    tpairs = tool_call_examples(fns)
    records.extend(tpairs)
    print(f"[v3] {len(records):>5} after tool-call examples (+{len(tpairs)})")

    # Commit messages — kept.
    try:
        r = subprocess.run(
            ["git", "log", "main", "--format=%s%n----BODY----%n%b%n----END----"],
            cwd=REPO, capture_output=True, text=True, timeout=30,
        )
        added = 0
        for blk in r.stdout.split("----END----"):
            blk = blk.strip()
            if "----BODY----" not in blk:
                continue
            subject, body = blk.split("----BODY----", 1)
            subject, body = subject.strip(), body.strip()
            if not subject or not body:
                continue
            records.append(conv(
                msg("user", f"Expand on this Sphragis commit subject: {subject}"),
                msg("assistant", body),
            ))
            added += 1
        print(f"[v3] {len(records):>5} after commit-msg pairs (+{added})")
    except Exception as e:
        print(f"[v3] commit-msg collection failed: {e}")

    random.Random(0xb47057).shuffle(records)

    with OUT.open("w", encoding="utf-8") as f:
        for rec in records:
            f.write(json.dumps(rec, ensure_ascii=False) + "\n")

    print(f"[v3] TOTAL {len(records)} records -> {OUT.relative_to(REPO)}")
    print(f"[v3] file size: {OUT.stat().st_size:,} bytes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
