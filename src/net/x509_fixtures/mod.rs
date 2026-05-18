//! Test fixtures for the X.509 chain validator — DER certs generated
//! by `scripts/gen_x509_test_chains.py`. Backs the 6 TDD scenarios
//! from the 2026-05-17 push plan §3 (Eng-1). Only the selftest path
//! uses these; never link from production code.

#![allow(dead_code)]

include!("test_chains.rs");
