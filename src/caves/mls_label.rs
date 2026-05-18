//! MLS (multi-level security) label primitive — gov-grade §3.2 type wrapper.
//!
//! Pairs the existing `cave::Sensitivity` (Bell-LaPadula confidentiality
//! lattice) and `cave::Integrity` (Biba integrity lattice) into a single
//! `MlsLabel`, and exposes the *dominance* relation those lattices
//! together induce.
//!
//! Vocabulary:
//!   * **A dominates B** (`A >= B`) iff `A.sensitivity >= B.sensitivity`
//!     AND `A.integrity >= B.integrity`. The Bell-LaPadula "no read up"
//!     and Biba "no write up" rules then both reduce to a single
//!     dominance check on the (subject, object) pair, with the direction
//!     decided by the operation.
//!
//! The lattices are kept independent so a cave can be Secret/Untrusted
//! (cleared for secrets but its output isn't authoritative) or
//! Unclassified/HighIntegrity (no secret access, but emits artefacts
//! the kernel signs).
//!
//! The existing `cave::can_flow` / `cave::can_flow_integrity` keep
//! working — this module adds the *typed* `LabelViolation` shape the
//! §3 charter calls for so cross-cave call sites can pattern-match on
//! `Err(LabelViolation::ReadUp)` / `Err(LabelViolation::WriteUp)`
//! directly.
//!
//! No on-disk format. The label that gets persisted is the
//! `Cave.sensitivity` + `Cave.integrity` u8 pair already covered by
//! `persist.rs`; this module is a typed view over those bytes.

#![allow(dead_code)]

use crate::caves::cave::{self, Integrity, MlsOp, Sensitivity, MAX_CAVES};

/// Reason an IPC flow was rejected by the MLS lattice. The §3 charter
/// names the two scenarios `ReadUp` (Bell-LaPadula simple-security)
/// and `WriteUp` (Biba *-integrity). We also surface the two
/// orthogonal violations for completeness even though the charter
/// only requires the named pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelViolation {
    /// Bell-LaPadula simple-security: subject's sensitivity is BELOW
    /// the object's. "No read up." (`subject.sensitivity < object.sensitivity`)
    ReadUp,
    /// Bell-LaPadula *-property: subject's sensitivity is ABOVE the
    /// object's. "No write down." (`subject.sensitivity > object.sensitivity`)
    WriteDown,
    /// Biba *-integrity: subject's integrity is BELOW the object's.
    /// "No write up." (`subject.integrity < object.integrity`)
    WriteUp,
    /// Biba simple-integrity: subject's integrity is ABOVE the
    /// object's. "No read down." (`subject.integrity > object.integrity`)
    ReadDown,
}

impl LabelViolation {
    pub fn as_str(self) -> &'static str {
        match self {
            LabelViolation::ReadUp    => "blp: read-up",
            LabelViolation::WriteDown => "blp: write-down",
            LabelViolation::WriteUp   => "biba: write-up",
            LabelViolation::ReadDown  => "biba: read-down",
        }
    }
}

/// Typed MLS label. The two axes are kept independent — see module
/// docs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MlsLabel {
    pub sensitivity: Sensitivity,
    pub integrity:   Integrity,
}

impl MlsLabel {
    pub const fn new(sensitivity: Sensitivity, integrity: Integrity) -> Self {
        Self { sensitivity, integrity }
    }

    /// Look up the label currently attached to `cave_id`. Out-of-range
    /// ids yield the bottom of both lattices, matching the underlying
    /// `cave::sensitivity_of` / `cave::integrity_of` defaults.
    pub fn of_cave(cave_id: u16) -> Self {
        Self {
            sensitivity: cave::sensitivity_of(cave_id),
            integrity:   cave::integrity_of(cave_id),
        }
    }

    /// `self` dominates `other` iff both axes are at least as high.
    /// Reflexive: every label dominates itself. Antisymmetric:
    /// dominance is a partial order, not a total one — Secret/Untrusted
    /// and Unclassified/HighIntegrity are incomparable.
    pub fn dominates(&self, other: &Self) -> bool {
        self.sensitivity >= other.sensitivity
            && self.integrity   >= other.integrity
    }

    /// Strict dominance: dominates AND not equal. Useful for tests
    /// that want to assert a step UP the lattice.
    pub fn strictly_dominates(&self, other: &Self) -> bool {
        self.dominates(other) && self != other
    }
}

/// Type-of-flow tag for `check_flow`. Distinguishes a *read* (subject
/// pulls data from object) from a *write* (subject pushes data to
/// object). The rules differ by axis:
///
///   * Confidentiality (Bell-LaPadula):
///       - Read:  subject.sensitivity >= object.sensitivity (no read up)
///       - Write: subject.sensitivity <= object.sensitivity (no write down)
///   * Integrity (Biba):
///       - Read:  subject.integrity   <= object.integrity   (no read down)
///       - Write: subject.integrity   >= object.integrity   (no write up)
///
/// Both axes are checked. Sensitivity is evaluated FIRST so a
/// confidentiality violation surfaces as `ReadUp` / `WriteDown`
/// rather than masking the integrity failure (matches the §3
/// charter, which keys its test names off BLP for read paths).
pub fn check_flow(
    subject: &MlsLabel,
    object:  &MlsLabel,
    op:      MlsOp,
) -> Result<(), LabelViolation> {
    match op {
        MlsOp::Read => {
            if subject.sensitivity < object.sensitivity {
                return Err(LabelViolation::ReadUp);
            }
            if subject.integrity > object.integrity {
                return Err(LabelViolation::ReadDown);
            }
        }
        MlsOp::Write => {
            if subject.sensitivity > object.sensitivity {
                return Err(LabelViolation::WriteDown);
            }
            if subject.integrity < object.integrity {
                return Err(LabelViolation::WriteUp);
            }
        }
    }
    Ok(())
}

/// Convenience: typed view of the §3 cross-cave IPC check. A call
/// from `subject_cave` to `object_cave` carrying data of `op` flavour
/// is permitted iff the MLS lattices allow it both ways.
///
/// Tests / call sites that want the dominance check without going
/// through the full `mls_ipc::call_with_token` path call this
/// directly.
pub fn check_cross_cave(
    subject_cave: u16,
    object_cave:  u16,
    op:           MlsOp,
) -> Result<(), LabelViolation> {
    if (subject_cave as usize) >= MAX_CAVES || (object_cave as usize) >= MAX_CAVES {
        // Out-of-range cave: bottom-lattice defaults apply, but rather
        // than silently producing an "Ok" by accident we report the
        // most conservative violation. Callers in the IPC path already
        // pre-check ids and return `MlsIpcError::BadId`; this branch
        // is belt-and-suspenders for direct callers.
        return Err(LabelViolation::ReadUp);
    }
    check_flow(&MlsLabel::of_cave(subject_cave), &MlsLabel::of_cave(object_cave), op)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lbl(s: Sensitivity, i: Integrity) -> MlsLabel { MlsLabel::new(s, i) }

    // ── Scenario 1: a label always dominates itself. ──
    #[test]
    fn test_label_dominance_self() {
        let cases = [
            lbl(Sensitivity::Unclassified, Integrity::Untrusted),
            lbl(Sensitivity::Confidential, Integrity::Sandboxed),
            lbl(Sensitivity::Secret,       Integrity::SystemTrusted),
            lbl(Sensitivity::TopSecret,    Integrity::HighIntegrity),
            // Cross-axis: BLP=Secret, Biba=Untrusted (cleared for secrets,
            // not trusted to write them).
            lbl(Sensitivity::Secret,       Integrity::Untrusted),
            // Mirror: BLP=Unclassified, Biba=HighIntegrity.
            lbl(Sensitivity::Unclassified, Integrity::HighIntegrity),
        ];
        for c in &cases {
            assert!(c.dominates(c), "label {:?} fails reflexive dominance", c);
            // Reflexive but NOT strict — every label is equal to
            // itself, so strict dominance must reject.
            assert!(!c.strictly_dominates(c),
                "label {:?} reports strict self-dominance", c);
        }
    }

    // ── Scenario 2: strict ascending chain through the BLP axis,
    //    with integrity pinned at each step's natural counterpart. ──
    #[test]
    fn test_label_dominance_strict() {
        // Bell-LaPadula axis: U < C < S < TS. Integrity walks in
        // parallel here so each pair is comparable (matches §3
        // "with compartments where appropriate" — we don't yet model
        // compartments; the lattice on the two-axis pair is enough
        // for the strict chain).
        let chain = [
            lbl(Sensitivity::Unclassified, Integrity::Untrusted),
            lbl(Sensitivity::Confidential, Integrity::Sandboxed),
            lbl(Sensitivity::Secret,       Integrity::SystemTrusted),
            lbl(Sensitivity::TopSecret,    Integrity::HighIntegrity),
        ];
        for hi in 0..chain.len() {
            for lo in 0..hi {
                assert!(chain[hi].dominates(&chain[lo]),
                    "{:?} should dominate {:?}", chain[hi], chain[lo]);
                assert!(chain[hi].strictly_dominates(&chain[lo]),
                    "{:?} should strictly-dominate {:?}", chain[hi], chain[lo]);
                assert!(!chain[lo].dominates(&chain[hi]),
                    "{:?} must NOT dominate {:?}", chain[lo], chain[hi]);
            }
        }
        // Incomparability: Secret/Untrusted vs Unclassified/HighIntegrity
        // — neither dominates the other.
        let a = lbl(Sensitivity::Secret,       Integrity::Untrusted);
        let b = lbl(Sensitivity::Unclassified, Integrity::HighIntegrity);
        assert!(!a.dominates(&b), "{:?} should NOT dominate {:?}", a, b);
        assert!(!b.dominates(&a), "{:?} should NOT dominate {:?}", b, a);
    }

    // ── Scenario 3: Bell-LaPadula "no read up" — a CONFIDENTIAL
    //    subject reading from a SECRET object fails with ReadUp. ──
    #[test]
    fn test_bell_lapadula_read_up_denied() {
        // The §3 charter phrases this in terms of "cave at CONFIDENTIAL
        // tries to read from SECRET cave". We model it at the label
        // level here (and at the cross-cave level in mls_ipc tests).
        let subject = lbl(Sensitivity::Confidential, Integrity::Sandboxed);
        let object  = lbl(Sensitivity::Secret,       Integrity::Sandboxed);
        match check_flow(&subject, &object, MlsOp::Read) {
            Err(LabelViolation::ReadUp) => {}
            other => panic!("expected ReadUp, got {:?}", other),
        }
        // Same-level read is OK on both axes.
        assert!(check_flow(&object, &object, MlsOp::Read).is_ok());
    }

    // ── Scenario 4: Biba "no write up" — a CONFIDENTIAL subject
    //    writing to an UNCLASSIFIED object. Biba protects INTEGRITY,
    //    so the rule is opposite of BLP-write's direction. The §3
    //    charter names this scenario WriteUp; the integrity axis
    //    drives the verdict (subject integrity below object integrity).
    #[test]
    fn test_biba_write_up_denied() {
        // Subject and object share sensitivity (no BLP issue); the
        // integrity gradient drives the failure. Use a stronger
        // gradient than the names suggest because Biba operates on
        // the integrity axis, not BLP. The scenario's "Confidential
        // -> Unclassified" framing is about the BLP direction; the
        // Biba violation surfaces because integrity goes UP.
        let subject = lbl(Sensitivity::Confidential, Integrity::Untrusted);
        let object  = lbl(Sensitivity::Unclassified, Integrity::SystemTrusted);
        match check_flow(&subject, &object, MlsOp::Write) {
            Err(LabelViolation::WriteUp) => {}
            other => panic!("expected WriteUp, got {:?}", other),
        }
        // If we then permit subject's integrity to match the object's,
        // the BLP write-down kicks in instead (Confidential -> Unclassified
        // is a confidentiality violation on the write path).
        let bumped = lbl(Sensitivity::Confidential, Integrity::SystemTrusted);
        match check_flow(&bumped, &object, MlsOp::Write) {
            Err(LabelViolation::WriteDown) => {}
            other => panic!("expected WriteDown, got {:?}", other),
        }
    }
}
