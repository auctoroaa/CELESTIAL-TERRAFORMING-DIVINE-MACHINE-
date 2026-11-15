use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Versioned, source-preserving representation of one creator intent revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentDocument {
    pub schema_version: String,
    pub intent_id: String,
    pub revision: u64,
    pub source: SourceRecord,
    pub objective: String,
    pub targets: Vec<TargetSpec>,
    pub requirements: Vec<Requirement>,
    pub constraints: Vec<Statement>,
    pub governing_laws: Vec<Statement>,
    pub assumptions: Vec<Statement>,
    pub unknowns: Vec<Statement>,
    pub conflicts: Vec<Conflict>,
    pub metadata: BTreeMap<String, String>,
}

/// Original creator input. The content is retained verbatim by this model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRecord {
    pub source_id: String,
    pub media_type: String,
    pub content: String,
}

/// A domain-neutral construction target. Kind is an extensible domain type,
/// not a closed enum, so new construction domains do not require core changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetSpec {
    pub target_id: String,
    pub kind: String,
    pub description: String,
    pub properties: Vec<Claim>,
    pub relationships: Vec<Relationship>,
}

/// A property claim with an explicit epistemic origin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    pub name: String,
    pub value: String,
    pub status: ClaimStatus,
    pub provenance: Option<Provenance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStatus {
    Declared,
    Derived,
    Assumed,
}

/// A typed relationship between two targets in the same intent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    pub subject_target_id: String,
    pub predicate: String,
    pub object_target_id: String,
}

/// A requirement that can later be linked to verification evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    pub requirement_id: String,
    pub statement: String,
    pub acceptance_criteria: Vec<String>,
    pub provenance: Option<Provenance>,
}

/// A source reference for a claim or requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub source_id: String,
    pub locator: Option<String>,
}

/// A recorded constraint, law, assumption, or unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Statement {
    pub statement_id: String,
    pub statement: String,
    pub provenance: Option<Provenance>,
}

/// An explicit incompatibility. It remains visible until resolved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
    pub conflict_id: String,
    pub subject: String,
    pub positions: Vec<String>,
    pub state: ConflictState,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictState {
    Unresolved,
    Resolved,
}
