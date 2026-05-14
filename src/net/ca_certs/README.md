# Sphragis embedded CA roots

Curated DER-encoded root certs that Sphragis's `x509::TRUST_STORE`
ships with as of STUMP #139. Each file is the raw DER bytes of a
self-signed root CA certificate fetched from the CA's official
publication endpoint.

## Roots in this directory

| File | Subject | Algorithm | Source |
|---|---|---|---|
| `isrg_root_x1.der` | ISRG Root X1 (Let's Encrypt) | RSA 4096 | https://letsencrypt.org/certs/isrgrootx1.der |
| `isrg_root_x2.der` | ISRG Root X2 (Let's Encrypt ECDSA) | ECDSA P-384 | https://letsencrypt.org/certs/isrg-root-x2.der |
| `amazon_root_ca1.der` | Amazon Root CA 1 | RSA 2048 | https://www.amazontrust.com/repository/AmazonRootCA1.cer |
| `digicert_global_root_ca.der` | DigiCert Global Root CA | RSA 2048 | https://cacerts.digicert.com/DigiCertGlobalRootCA.crt |
| `digicert_global_root_g2.der` | DigiCert Global Root G2 | RSA 2048 | https://cacerts.digicert.com/DigiCertGlobalRootG2.crt |
| `gts_root_r4.der` | GTS Root R4 (Google Trust Services) | ECDSA P-384 | https://i.pki.goog/r4.crt (SHA-256 34:9D:FA:40:58:C5:E2:63:12:3B:39:8A:E7:95:57:3C:4E:13:13:C8:3F:E6:8F:93:55:6C:D5:E8:03:1B:3C:7D) |

This is a deliberately small starter set, not a full Mozilla CA
bundle. Adding the other ~150 roots is a follow-up STUMP. The six
above cover Let's Encrypt + Amazon + DigiCert + Google Trust
Services chains, which is most of the public web in practice.

## Refreshing

Roots rarely rotate, but when they do (or if a CA gets distrusted)
re-fetch from the URLs above and rebuild. The `tools/fetch_ca_bundle.sh`
helper does this in one go.

## RSA caveat (until STUMP #140 lands)

`crypto/sig.rs::verify_pubkey` only validates ECDSA P-256 / P-384
leaf signatures as of STUMP #139. Three of the roots above are RSA,
which means today they only anchor chains where the leaf is ECDSA
(common — Let's Encrypt issues both). Pure-RSA leaf chains fall
through with `UnsupportedSigAlg` and the pin-based path is the only
defence. STUMP #140 adds RSA verify and unlocks the rest of these
roots.
