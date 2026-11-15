//! Domain-independent capture, validation, and identity for creator intent.
//!
//! This crate preserves the creator's source and distinguishes declared,
//! derived, and assumed claims. It does not infer intent or decide whether a
//! requested construction has been achieved.

mod digest;
mod model;
mod validation;

pub use digest::{DigestError, IntentDigest};
pub use model::{
    Claim, ClaimStatus, Conflict, ConflictState, IntentDocument, Provenance, Relationship,
    Requirement, SourceRecord, Statement, TargetSpec,
};
pub use validation::{IntentValidationError, validate_intent};

impl IntentDocument {
    /// Validate this revision before it is passed to planning or execution.
    pub fn validate(&self) -> Result<(), Vec<IntentValidationError>> {
        validate_intent(self)
    }

    /// Compute a stable, domain-separated digest of this exact serialized revision.
    ///
    /// Collection order is significant and preserved. Metadata keys are ordered
    /// by BTreeMap, so equivalent metadata maps serialize consistently.
    pub fn digest(&self) -> Result<IntentDigest, DigestError> {
        digest::digest_intent(self)
    }
}
