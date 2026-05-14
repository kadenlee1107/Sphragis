//! CALIPSO — RFC 5570 Common Architecture Label IPv6 Security Option.
//!
//! IPv6 equivalent of CIPSO (which we emit on IPv4 in `ip.rs`).
//! CALIPSO rides inside an IPv6 Hop-by-Hop Options header
//! (next-header = 0). This module is the pure-function half:
//! encode an option block given a DOI + sensitivity level, and
//! parse one out of a candidate options buffer. The Sphragis IP
//! stack today is v4-only — wiring CALIPSO into a `send6` /
//! `handle6` mirror of `ip.rs` waits on v6 landing in tree.
//!
//! RFC 5570 §5 wire format (after the 2-byte option header):
//!
//!     +0  Domain of Interpretation (4 bytes, big-endian)
//!     +4  Sensitivity level         (1 byte)
//!     +5  Compartment-length        (1 byte, in 4-octet units)
//!     +6  Checksum                  (2 bytes — ones-complement
//!                                    of the option's bytes, with
//!                                    the checksum field treated as
//!                                    zero during compute)
//!     +8  Compartment bitmap        (0..N×4 bytes, optional)
//!
//! The outer option header is the standard 2-byte (option_type,
//! option_length) IPv6 TLV — option_type 0x07 for CALIPSO, with
//! the top-3-bit "action" set to "skip if unknown" (option_type
//! 0x07 = 0b00000111 already encodes that).
//!
//! No compartments today; a single sensitivity byte is the
//! minimum CALIPSO needs to carry. Compartment-bitmap support
//! plugs into the same encoder when we have a richer policy.

#![allow(dead_code)]

/// IPv6 Hop-by-Hop option type for CALIPSO. RFC 5570 §5.1.
pub const CALIPSO_OPT_TYPE: u8 = 0x07;

/// Sphragis DOI. We pick a private value (same byte string as our
/// CIPSO DOI so a single trusted-Sphragis-network policy applies to
/// both v4 and v6 traffic in a future mixed deployment).
pub const CALIPSO_DOI_SPHRAGIS: u32 = 0x42_42_4F_53;

/// Smallest CALIPSO encoding (no compartments): 2 (TLV) + 8
/// (DOI/level/cmpt-len/checksum) = 10 bytes.
pub const MIN_CALIPSO_LEN: usize = 10;

/// Encode a CALIPSO option with the given sensitivity byte and
/// no compartments. Writes `MIN_CALIPSO_LEN` bytes starting at
/// `out[0]` and returns the count, or 0 if `out` is too small.
pub fn encode(level: u8, out: &mut [u8]) -> usize {
    if out.len() < MIN_CALIPSO_LEN { return 0; }
    out[0] = CALIPSO_OPT_TYPE;
    out[1] = (MIN_CALIPSO_LEN - 2) as u8; // option data length = 8
    out[2..6].copy_from_slice(&CALIPSO_DOI_SPHRAGIS.to_be_bytes());
    out[6] = level;
    out[7] = 0;          // compartment-length = 0 octets
    out[8] = 0; out[9] = 0; // checksum placeholder
    let cksum = checksum(&out[..MIN_CALIPSO_LEN]);
    out[8..10].copy_from_slice(&cksum.to_be_bytes());
    MIN_CALIPSO_LEN
}

/// Parse a CALIPSO option from `data` (which must begin with the
/// 2-byte TLV header — caller's responsibility to find it inside
/// the Hop-by-Hop options header). Returns the sensitivity byte
/// if the DOI matches Sphragis's and the checksum verifies;
/// otherwise `None`. Defensive about lengths so a crafted option
/// can't overrun.
pub fn parse(data: &[u8]) -> Option<u8> {
    if data.len() < MIN_CALIPSO_LEN { return None; }
    if data[0] != CALIPSO_OPT_TYPE { return None; }
    let opt_data_len = data[1] as usize;
    let total = 2 + opt_data_len;
    if total > data.len() || total < MIN_CALIPSO_LEN { return None; }
    let doi = u32::from_be_bytes([data[2], data[3], data[4], data[5]]);
    if doi != CALIPSO_DOI_SPHRAGIS { return None; }
    let level    = data[6];
    let cmpt_len = data[7] as usize * 4;
    // total includes the 2-byte TLV header. Option data is
    // 8 fixed bytes (DOI+level+cmptlen+chk) + cmpt_len bytes of
    // bitmap, so expected total = 2 + 8 + cmpt_len = 10 + cmpt_len.
    if total != MIN_CALIPSO_LEN + cmpt_len { return None; }
    if checksum(&data[..total]) != 0 { return None; }
    Some(level)
}

/// One's-complement sum across 16-bit words, used both to compute
/// the checksum at encode time and verify it at parse time.
/// Treats the 2-byte checksum field as part of the data — when
/// the field is zero, the result is the checksum to insert; when
/// the field already holds the inserted value, the verifier sees
/// `sum == 0xFFFF` and `(!sum as u16) == 0`.
fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += u16::from_be_bytes([data[i], data[i + 1]]) as u32;
        i += 2;
    }
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}
