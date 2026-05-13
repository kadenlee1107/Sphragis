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
pub const ISSUER_KEY_HASH: [u8; 32] = [14, 89, 103, 23, 128, 222, 147, 103, 52, 192, 169, 3, 88, 225, 33, 239, 110, 93, 113, 73, 4, 129, 46, 32, 89, 84, 133, 48, 55, 242, 78, 122];

pub const SERIAL_GOOD:    &[u8] = &[0x01];
pub const SERIAL_REVOKED: &[u8] = &[0x02];
