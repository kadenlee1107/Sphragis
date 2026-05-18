#!/usr/bin/env python3
"""Generate X.509 test chain fixtures for the chain-validator selftest.

Produces a fixed set of ECDSA P-256 DER fixtures under
`src/net/x509_fixtures/` that exercise the 6 TDD scenarios mandated
by the 2026-05-17 push plan §3 (Eng-1):

  1. valid 3-level chain (leaf -> intermediate -> root)
  2. signature mismatch (leaf signed by wrong key vs intermediate)
  3. expired intermediate (notAfter in the past)
  4. unknown root (root not in trust store)
  5. BasicConstraints violation (leaf with CA:TRUE)
  6. revocation stub (any valid chain; verifier defaults to no rev)

The fixtures are deterministic only up to the random ECDSA keys; we
regenerate when the validator's accept criteria change. Each .der is
a single DER-encoded certificate ready for `Certificate::from_der`.

Output:
  src/net/x509_fixtures/valid_root.der
  src/net/x509_fixtures/valid_intermediate.der
  src/net/x509_fixtures/valid_leaf.der
  src/net/x509_fixtures/badsig_root.der
  src/net/x509_fixtures/badsig_intermediate.der
  src/net/x509_fixtures/badsig_leaf.der
  src/net/x509_fixtures/expired_root.der
  src/net/x509_fixtures/expired_intermediate.der
  src/net/x509_fixtures/expired_leaf.der
  src/net/x509_fixtures/unknown_root.der
  src/net/x509_fixtures/unknown_intermediate.der
  src/net/x509_fixtures/unknown_leaf.der
  src/net/x509_fixtures/bcleaf_root.der
  src/net/x509_fixtures/bcleaf_intermediate.der
  src/net/x509_fixtures/bcleaf_leaf.der
  src/net/x509_fixtures/test_chains.rs

Each leaf carries a SubjectAltName dNSName of `selftest.sphragis.test`
so the hostname check in `verify_chain` passes for valid chains and
the test harness can use a single hostname constant.
"""
from __future__ import annotations

from datetime import datetime, timedelta, timezone
import pathlib
import textwrap

from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.x509 import oid as x509_oid


SELFTEST_HOST = "selftest.sphragis.test"


def now_utc() -> datetime:
    return datetime.now(tz=timezone.utc).replace(microsecond=0)


def gen_key() -> ec.EllipticCurvePrivateKey:
    return ec.generate_private_key(ec.SECP256R1())


def name(cn: str) -> x509.Name:
    return x509.Name(
        [x509.NameAttribute(x509_oid.NameOID.COMMON_NAME, cn)],
    )


def build_self_signed_root(
    cn: str,
    *,
    not_before: datetime,
    not_after: datetime,
) -> tuple[x509.Certificate, ec.EllipticCurvePrivateKey]:
    key = gen_key()
    builder = (
        x509.CertificateBuilder()
        .subject_name(name(cn))
        .issuer_name(name(cn))
        .public_key(key.public_key())
        .serial_number(0x01)
        .not_valid_before(not_before)
        .not_valid_after(not_after)
        .add_extension(
            x509.BasicConstraints(ca=True, path_length=None),
            critical=True,
        )
        .add_extension(
            x509.KeyUsage(
                digital_signature=False,
                content_commitment=False,
                key_encipherment=False,
                data_encipherment=False,
                key_agreement=False,
                key_cert_sign=True,
                crl_sign=True,
                encipher_only=False,
                decipher_only=False,
            ),
            critical=True,
        )
    )
    cert = builder.sign(private_key=key, algorithm=hashes.SHA256())
    return cert, key


def build_intermediate(
    cn: str,
    issuer_cert: x509.Certificate,
    issuer_key: ec.EllipticCurvePrivateKey,
    *,
    not_before: datetime,
    not_after: datetime,
    serial: int = 0x02,
) -> tuple[x509.Certificate, ec.EllipticCurvePrivateKey]:
    key = gen_key()
    builder = (
        x509.CertificateBuilder()
        .subject_name(name(cn))
        .issuer_name(issuer_cert.subject)
        .public_key(key.public_key())
        .serial_number(serial)
        .not_valid_before(not_before)
        .not_valid_after(not_after)
        .add_extension(
            x509.BasicConstraints(ca=True, path_length=0),
            critical=True,
        )
        .add_extension(
            x509.KeyUsage(
                digital_signature=False,
                content_commitment=False,
                key_encipherment=False,
                data_encipherment=False,
                key_agreement=False,
                key_cert_sign=True,
                crl_sign=True,
                encipher_only=False,
                decipher_only=False,
            ),
            critical=True,
        )
    )
    cert = builder.sign(private_key=issuer_key, algorithm=hashes.SHA256())
    return cert, key


def build_leaf(
    cn: str,
    issuer_cert: x509.Certificate,
    issuer_key: ec.EllipticCurvePrivateKey,
    *,
    not_before: datetime,
    not_after: datetime,
    leaf_is_ca: bool = False,
    serial: int = 0x03,
    san: str = SELFTEST_HOST,
) -> tuple[x509.Certificate, ec.EllipticCurvePrivateKey]:
    key = gen_key()
    builder = (
        x509.CertificateBuilder()
        .subject_name(name(cn))
        .issuer_name(issuer_cert.subject)
        .public_key(key.public_key())
        .serial_number(serial)
        .not_valid_before(not_before)
        .not_valid_after(not_after)
        .add_extension(
            # Leaf MUST be ca=False; the "BC violation" fixture flips
            # this to True to exercise BasicConstraintsViolation.
            x509.BasicConstraints(ca=leaf_is_ca, path_length=None),
            critical=True,
        )
        .add_extension(
            x509.SubjectAlternativeName([x509.DNSName(san)]),
            critical=False,
        )
        .add_extension(
            x509.ExtendedKeyUsage([x509_oid.ExtendedKeyUsageOID.SERVER_AUTH]),
            critical=False,
        )
    )
    cert = builder.sign(private_key=issuer_key, algorithm=hashes.SHA256())
    return cert, key


def der(cert: x509.Certificate) -> bytes:
    return cert.public_bytes(serialization.Encoding.DER)


def main() -> None:
    out_dir = pathlib.Path(__file__).resolve().parent.parent / "src/net/x509_fixtures"
    out_dir.mkdir(parents=True, exist_ok=True)

    today = now_utc()
    one_year_ago = today - timedelta(days=365)
    five_years_ahead = today + timedelta(days=365 * 5)
    # "Expired" anchor: pick a date safely in the past relative to any
    # plausible `SPHRAGIS_BUILD_UNIX` value the kernel might see. The
    # build.rs floor is 2026-01-01, and a stale build can pin the
    # kernel's clock arbitrarily far behind wall-clock — so we use a
    # date that pre-dates the floor (2025-01-01). The validity-period
    # check fails Expired iff `now_unix > notAfter`, and the floor
    # guarantees `now_unix >= 1_735_689_600 (2025-01-01 UTC)`, so this
    # cert is reliably "expired" without depending on a fresh build.
    far_past = datetime(2024, 1, 1, tzinfo=timezone.utc)
    far_past_minus_year = datetime(2023, 1, 1, tzinfo=timezone.utc)

    # ============================================================
    # Fixture 1 — VALID 3-level chain (leaf <- int <- root).
    # ============================================================
    v_root, v_root_key = build_self_signed_root(
        "Sphragis x509-selftest Valid Root",
        not_before=one_year_ago,
        not_after=five_years_ahead,
    )
    v_int, v_int_key = build_intermediate(
        "Sphragis x509-selftest Valid Intermediate",
        v_root, v_root_key,
        not_before=one_year_ago,
        not_after=five_years_ahead,
    )
    v_leaf, _ = build_leaf(
        "Sphragis x509-selftest Valid Leaf",
        v_int, v_int_key,
        not_before=one_year_ago,
        not_after=five_years_ahead,
    )

    (out_dir / "valid_root.der").write_bytes(der(v_root))
    (out_dir / "valid_intermediate.der").write_bytes(der(v_int))
    (out_dir / "valid_leaf.der").write_bytes(der(v_leaf))

    # ============================================================
    # Fixture 2 — SIG MISMATCH. We build a normal valid root + int,
    # then construct a leaf cert whose `signature_algorithm` and
    # `signature` are produced by a STRANGER key (not the
    # intermediate's). The leaf's `issuer` field still names the
    # intermediate, so `verify_chain` walks to the intermediate and
    # tries to verify the leaf's signature with the intermediate's
    # pubkey — which fails because the signature was made with a
    # different key.
    # ============================================================
    bs_root, bs_root_key = build_self_signed_root(
        "Sphragis x509-selftest BadSig Root",
        not_before=one_year_ago,
        not_after=five_years_ahead,
    )
    bs_int, bs_int_key = build_intermediate(
        "Sphragis x509-selftest BadSig Intermediate",
        bs_root, bs_root_key,
        not_before=one_year_ago,
        not_after=five_years_ahead,
    )
    # Stranger key (NOT the intermediate's). We sign the leaf with
    # this key but use the intermediate's *name* as the issuer so the
    # chain is structurally valid up to the signature check.
    stranger_key = gen_key()
    bs_leaf = (
        x509.CertificateBuilder()
        .subject_name(name("Sphragis x509-selftest BadSig Leaf"))
        .issuer_name(bs_int.subject)
        .public_key(gen_key().public_key())
        .serial_number(0x03)
        .not_valid_before(one_year_ago)
        .not_valid_after(five_years_ahead)
        .add_extension(
            x509.BasicConstraints(ca=False, path_length=None),
            critical=True,
        )
        .add_extension(
            x509.SubjectAlternativeName([x509.DNSName(SELFTEST_HOST)]),
            critical=False,
        )
        .add_extension(
            x509.ExtendedKeyUsage([x509_oid.ExtendedKeyUsageOID.SERVER_AUTH]),
            critical=False,
        )
        .sign(private_key=stranger_key, algorithm=hashes.SHA256())
    )

    (out_dir / "badsig_root.der").write_bytes(der(bs_root))
    (out_dir / "badsig_intermediate.der").write_bytes(der(bs_int))
    (out_dir / "badsig_leaf.der").write_bytes(der(bs_leaf))

    # ============================================================
    # Fixture 3 — EXPIRED INTERMEDIATE. The intermediate's notAfter
    # is set to 2024-01-01 — comfortably before the build.rs floor
    # of 2026-01-01, so the kernel's `now_unix()` is guaranteed to
    # be greater than this cert's notAfter regardless of how stale
    # the binary is. The chain walk should flag the intermediate as
    # Expired. (See `validity` comment in `verify_chain` for the
    # rationale on constant-cost abort.)
    # ============================================================
    e_root, e_root_key = build_self_signed_root(
        "Sphragis x509-selftest Expired Root",
        not_before=far_past_minus_year,
        not_after=five_years_ahead,
    )
    e_int, e_int_key = build_intermediate(
        "Sphragis x509-selftest Expired Intermediate",
        e_root, e_root_key,
        not_before=far_past_minus_year,
        # Expired comfortably before any plausible build-time clock.
        not_after=far_past,
    )
    # Leaf with validity inside the intermediate's window so the
    # cryptography library accepts the chain construction. The
    # validator should NEVER reach the leaf's date check — it will
    # short-circuit at the intermediate's Expired flag.
    e_leaf, _ = build_leaf(
        "Sphragis x509-selftest Expired-IntChain Leaf",
        e_int, e_int_key,
        not_before=far_past_minus_year,
        not_after=far_past - timedelta(seconds=1),
    )

    (out_dir / "expired_root.der").write_bytes(der(e_root))
    (out_dir / "expired_intermediate.der").write_bytes(der(e_int))
    (out_dir / "expired_leaf.der").write_bytes(der(e_leaf))

    # ============================================================
    # Fixture 4 — UNKNOWN ROOT. Build a fully valid chain whose root
    # is NOT in the trust store. The test harness will NOT add this
    # root to its test-anchor slice, so `verify_chain` returns
    # UntrustedRoot.
    # ============================================================
    u_root, u_root_key = build_self_signed_root(
        "Sphragis x509-selftest Unknown Root",
        not_before=one_year_ago,
        not_after=five_years_ahead,
    )
    u_int, u_int_key = build_intermediate(
        "Sphragis x509-selftest Unknown Intermediate",
        u_root, u_root_key,
        not_before=one_year_ago,
        not_after=five_years_ahead,
    )
    u_leaf, _ = build_leaf(
        "Sphragis x509-selftest Unknown Leaf",
        u_int, u_int_key,
        not_before=one_year_ago,
        not_after=five_years_ahead,
    )

    (out_dir / "unknown_root.der").write_bytes(der(u_root))
    (out_dir / "unknown_intermediate.der").write_bytes(der(u_int))
    (out_dir / "unknown_leaf.der").write_bytes(der(u_leaf))

    # ============================================================
    # Fixture 5 — BASIC CONSTRAINTS VIOLATION. Build a chain whose
    # leaf has `cA: TRUE`. `verify_chain` must reject the chain at
    # the BasicConstraints check.
    # ============================================================
    bc_root, bc_root_key = build_self_signed_root(
        "Sphragis x509-selftest BC-leaf-CA Root",
        not_before=one_year_ago,
        not_after=five_years_ahead,
    )
    bc_int, bc_int_key = build_intermediate(
        "Sphragis x509-selftest BC-leaf-CA Intermediate",
        bc_root, bc_root_key,
        not_before=one_year_ago,
        not_after=five_years_ahead,
    )
    bc_leaf, _ = build_leaf(
        "Sphragis x509-selftest BC-leaf-CA Leaf",
        bc_int, bc_int_key,
        not_before=one_year_ago,
        not_after=five_years_ahead,
        leaf_is_ca=True,  # ← the violation
    )

    (out_dir / "bcleaf_root.der").write_bytes(der(bc_root))
    (out_dir / "bcleaf_intermediate.der").write_bytes(der(bc_int))
    (out_dir / "bcleaf_leaf.der").write_bytes(der(bc_leaf))

    # ============================================================
    # Emit the Rust module that re-exports the fixtures.
    # ============================================================
    rust = textwrap.dedent(
        """\
        // GENERATED — do not edit. Regenerate via scripts/gen_x509_test_chains.py.
        //
        // X.509 chain-validator test fixtures backing the 6 scenarios from
        // the 2026-05-17 multi-team-push plan §3 (Eng-1).
        //
        // All certs are ECDSA P-256 with SHA-256 signatures. The leaf SAN
        // is `selftest.sphragis.test` so the hostname check passes for the
        // valid chain. Each fixture group is independent: a different
        // synthetic root anchors each chain, so swapping which root is in
        // the test-trust-store flips a chain between Untrusted and Ok.

        // Scenario 1 — valid 3-level chain.
        pub const VALID_ROOT_DER:         &[u8] = include_bytes!("valid_root.der");
        pub const VALID_INTERMEDIATE_DER: &[u8] = include_bytes!("valid_intermediate.der");
        pub const VALID_LEAF_DER:         &[u8] = include_bytes!("valid_leaf.der");

        // Scenario 2 — signature mismatch (leaf signed by a stranger).
        pub const BADSIG_ROOT_DER:         &[u8] = include_bytes!("badsig_root.der");
        pub const BADSIG_INTERMEDIATE_DER: &[u8] = include_bytes!("badsig_intermediate.der");
        pub const BADSIG_LEAF_DER:         &[u8] = include_bytes!("badsig_leaf.der");

        // Scenario 3 — expired intermediate.
        pub const EXPIRED_ROOT_DER:         &[u8] = include_bytes!("expired_root.der");
        pub const EXPIRED_INTERMEDIATE_DER: &[u8] = include_bytes!("expired_intermediate.der");
        pub const EXPIRED_LEAF_DER:         &[u8] = include_bytes!("expired_leaf.der");

        // Scenario 4 — unknown root (root NOT in test trust store).
        pub const UNKNOWN_ROOT_DER:         &[u8] = include_bytes!("unknown_root.der");
        pub const UNKNOWN_INTERMEDIATE_DER: &[u8] = include_bytes!("unknown_intermediate.der");
        pub const UNKNOWN_LEAF_DER:         &[u8] = include_bytes!("unknown_leaf.der");

        // Scenario 5 — BasicConstraints violation (leaf with CA:TRUE).
        pub const BCLEAF_ROOT_DER:         &[u8] = include_bytes!("bcleaf_root.der");
        pub const BCLEAF_INTERMEDIATE_DER: &[u8] = include_bytes!("bcleaf_intermediate.der");
        pub const BCLEAF_LEAF_DER:         &[u8] = include_bytes!("bcleaf_leaf.der");

        // Hostname every leaf SAN covers.
        pub const SELFTEST_HOSTNAME: &[u8] = b"selftest.sphragis.test";
        """
    )
    (out_dir / "test_chains.rs").write_text(rust)
    print(f"wrote 16 files to {out_dir}")


if __name__ == "__main__":
    main()
