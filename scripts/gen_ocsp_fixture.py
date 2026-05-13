#!/usr/bin/env python3
"""Generate an OCSP response DER fixture for `ocsp-selftest`.

Builds a self-signed test CA, issues one cert under it, then crafts
an `OcspResponse` (RFC 6960 §4.2.1) containing two `SingleResponse`
entries:

  * serial 0x01 → Good
  * serial 0x02 → Revoked (revocationTime = now)

The fixture is consumed in-kernel by `src/net/ocsp.rs` via
`ingest_basic_response(der)`. We also emit a tiny header file with
the issuer key hash (SHA-256 of the responder's subjectPublicKey
BIT STRING contents) so the selftest knows what key to query.

Output:
  src/net/ocsp_fixtures/test_response.der
  src/net/ocsp_fixtures/test_response.rs
"""
from __future__ import annotations

from datetime import datetime, timedelta, timezone
import hashlib
import pathlib
import textwrap

from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.x509 import oid as x509_oid
from cryptography.x509 import ocsp


def now_utc() -> datetime:
    return datetime.now(tz=timezone.utc).replace(microsecond=0)


def make_self_signed_ca() -> tuple[x509.Certificate, rsa.RSAPrivateKey]:
    key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    name = x509.Name([x509.NameAttribute(x509_oid.NameOID.COMMON_NAME, "Bat_OS OCSP Test CA")])
    cert = (
        x509.CertificateBuilder()
        .subject_name(name)
        .issuer_name(name)
        .public_key(key.public_key())
        .serial_number(0xCAFE)
        .not_valid_before(now_utc() - timedelta(days=1))
        .not_valid_after(now_utc() + timedelta(days=365 * 10))
        .add_extension(x509.BasicConstraints(ca=True, path_length=None), critical=True)
        .sign(private_key=key, algorithm=hashes.SHA256())
    )
    return cert, key


def make_subject_cert(serial: int, ca_cert: x509.Certificate, ca_key: rsa.RSAPrivateKey) -> x509.Certificate:
    key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    name = x509.Name([x509.NameAttribute(x509_oid.NameOID.COMMON_NAME, f"Bat_OS OCSP Test Cert #{serial}")])
    return (
        x509.CertificateBuilder()
        .subject_name(name)
        .issuer_name(ca_cert.subject)
        .public_key(key.public_key())
        .serial_number(serial)
        .not_valid_before(now_utc() - timedelta(days=1))
        .not_valid_after(now_utc() + timedelta(days=30))
        .sign(private_key=ca_key, algorithm=hashes.SHA256())
    )


def main() -> None:
    out_dir = pathlib.Path(__file__).resolve().parent.parent / "src/net/ocsp_fixtures"
    out_dir.mkdir(parents=True, exist_ok=True)

    ca_cert, ca_key = make_self_signed_ca()
    cert_good = make_subject_cert(0x01, ca_cert, ca_key)
    cert_revoked = make_subject_cert(0x02, ca_cert, ca_key)

    # The cryptography library only emits one SingleResponse per
    # OCSPResponse, so we generate two separate fixtures — one for
    # the Good cert, one for the Revoked cert. The in-kernel
    # selftest ingests both and queries each.
    def build(builder_status, target_cert, revocation_time, revocation_reason):
        b = ocsp.OCSPResponseBuilder().add_response(
            cert=target_cert,
            issuer=ca_cert,
            algorithm=hashes.SHA256(),
            cert_status=builder_status,
            this_update=now_utc(),
            next_update=now_utc() + timedelta(days=7),
            revocation_time=revocation_time,
            revocation_reason=revocation_reason,
        ).responder_id(ocsp.OCSPResponderEncoding.HASH, ca_cert)
        return b.sign(ca_key, hashes.SHA256()).public_bytes(serialization.Encoding.DER)

    der_good = build(ocsp.OCSPCertStatus.GOOD, cert_good, None, None)
    der_revoked = build(ocsp.OCSPCertStatus.REVOKED, cert_revoked,
                        now_utc(), x509.ReasonFlags.key_compromise)

    good_path = out_dir / "test_good.der"
    revoked_path = out_dir / "test_revoked.der"
    good_path.write_bytes(der_good)
    revoked_path.write_bytes(der_revoked)
    print(f"wrote {good_path} ({len(der_good)} bytes)")
    print(f"wrote {revoked_path} ({len(der_revoked)} bytes)")

    # The issuer_key_hash the OCSP response carries is SHA-256 of
    # the responder CA's subjectPublicKey BIT STRING CONTENTS (not
    # the wrapping SPKI struct) — RFC 6960 §4.1.1. Both responses
    # share the same issuer key hash since they're under the same CA.
    parsed = ocsp.load_der_ocsp_response(der_good)
    ikh = parsed.issuer_key_hash
    assert len(ikh) == 32, f"expected 32-byte SHA-256 issuer key hash, got {len(ikh)}"

    rs_path = out_dir / "test_response.rs"
    rs_body = textwrap.dedent(f"""\
        // GENERATED — do not edit. Regenerate via scripts/gen_ocsp_fixture.py.
        //
        // OCSP test responses covering two serials under one issuer:
        //   * serial 0x01 -> Good        (test_good.der)
        //   * serial 0x02 -> Revoked     (test_revoked.der, reason: keyCompromise)
        //
        // Used by `ocsp-selftest` to exercise the DER ingest path of
        // `crate::net::ocsp::ingest_basic_response`.

        pub const TEST_OCSP_GOOD_DER:    &[u8] = include_bytes!("test_good.der");
        pub const TEST_OCSP_REVOKED_DER: &[u8] = include_bytes!("test_revoked.der");

        /// SHA-256 of the responder CA's subjectPublicKey BIT STRING
        /// contents — the same value the OCSP response carries as
        /// `issuerKeyHash`. Selftest queries the cache with this.
        pub const ISSUER_KEY_HASH: [u8; 32] = {list(ikh)!r};

        pub const SERIAL_GOOD:    &[u8] = &[0x01];
        pub const SERIAL_REVOKED: &[u8] = &[0x02];
    """)
    rs_path.write_text(rs_body)
    print(f"wrote {rs_path}")


if __name__ == "__main__":
    main()
