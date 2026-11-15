use crate::model::{ConflictState, IntentDocument};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IntentValidationError {
    #[error("{field} must not be empty")]
    EmptyField { field: &'static str },
    #[error("{kind} contains duplicate identifier '{id}'")]
    DuplicateIdentifier { kind: &'static str, id: String },
    #[error("relationship references unknown target '{target_id}'")]
    UnknownRelationshipTarget { target_id: String },
    #[error("requirement '{requirement_id}' has no acceptance criteria")]
    MissingAcceptanceCriteria { requirement_id: String },
    #[error("conflict '{conflict_id}' must contain at least two positions")]
    InsufficientConflictPositions { conflict_id: String },
    #[error("resolved conflict '{conflict_id}' must include its resolution")]
    MissingConflictResolution { conflict_id: String },
    #[error("unresolved conflict '{conflict_id}' must not claim a resolution")]
    UnexpectedConflictResolution { conflict_id: String },
}

/// Validate structural integrity without inventing missing creator decisions.
pub fn validate_intent(
    intent: &IntentDocument,
) -> Result<(), Vec<IntentValidationError>> {
    let mut errors = Vec::new();

    check_nonempty(&mut errors, "schema_version", &intent.schema_version);
    check_nonempty(&mut errors, "intent_id", &intent.intent_id);
    check_nonempty(&mut errors, "source.source_id", &intent.source.source_id);
    check_nonempty(&mut errors, "source.media_type", &intent.source.media_type);
    check_nonempty(&mut errors, "source.content", &intent.source.content);
    check_nonempty(&mut errors, "objective", &intent.objective);

    if intent.revision == 0 {
        errors.push(IntentValidationError::EmptyField { field: "revision" });
    }
    if intent.targets.is_empty() {
        errors.push(IntentValidationError::EmptyField { field: "targets" });
    }

    let mut target_ids = HashSet::new();
    for target in &intent.targets {
        check_nonempty(&mut errors, "target.target_id", &target.target_id);
        check_nonempty(&mut errors, "target.kind", &target.kind);
        check_nonempty(&mut errors, "target.description", &target.description);
        if !target.target_id.trim().is_empty()
            && !target_ids.insert(target.target_id.as_str())
        {
            errors.push(IntentValidationError::DuplicateIdentifier {
                kind: "target",
                id: target.target_id.clone(),
            });
        }
        for claim in &target.properties {
            check_nonempty(&mut errors, "claim.name", &claim.name);
            check_nonempty(&mut errors, "claim.value", &claim.value);
            if let Some(provenance) = &claim.provenance {
                check_nonempty(&mut errors, "claim.provenance.source_id", &provenance.source_id);
            }
        }
    }

    for target in &intent.targets {
        for relationship in &target.relationships {
            check_nonempty(&mut errors, "relationship.predicate", &relationship.predicate);
            for id in [&relationship.subject_target_id, &relationship.object_target_id] {
                if !target_ids.contains(id.as_str()) {
                    errors.push(IntentValidationError::UnknownRelationshipTarget {
                        target_id: id.clone(),
                    });
                }
            }
        }
    }

    check_unique_ids(
        &mut errors,
        "requirement",
        intent.requirements.iter().map(|item| item.requirement_id.as_str()),
    );
    for requirement in &intent.requirements {
        check_nonempty(&mut errors, "requirement.requirement_id", &requirement.requirement_id);
        check_nonempty(&mut errors, "requirement.statement", &requirement.statement);
        if requirement.acceptance_criteria.is_empty()
            || requirement.acceptance_criteria.iter().any(|item| item.trim().is_empty())
        {
            errors.push(IntentValidationError::MissingAcceptanceCriteria {
                requirement_id: requirement.requirement_id.clone(),
            });
        }
        if let Some(provenance) = &requirement.provenance {
            check_nonempty(&mut errors, "requirement.provenance.source_id", &provenance.source_id);
        }
    }

    validate_statements(&mut errors, "constraint", &intent.constraints);
    validate_statements(&mut errors, "governing_law", &intent.governing_laws);
    validate_statements(&mut errors, "assumption", &intent.assumptions);
    validate_statements(&mut errors, "unknown", &intent.unknowns);

    check_unique_ids(
        &mut errors,
        "conflict",
        intent.conflicts.iter().map(|item| item.conflict_id.as_str()),
    );
    for conflict in &intent.conflicts {
        check_nonempty(&mut errors, "conflict.conflict_id", &conflict.conflict_id);
        check_nonempty(&mut errors, "conflict.subject", &conflict.subject);
        if conflict.positions.len() < 2
            || conflict.positions.iter().any(|item| item.trim().is_empty())
        {
            errors.push(IntentValidationError::InsufficientConflictPositions {
                conflict_id: conflict.conflict_id.clone(),
            });
        }
        match (conflict.state, conflict.resolution.as_deref()) {
            (ConflictState::Resolved, Some(value)) if !value.trim().is_empty() => {}
            (ConflictState::Resolved, _) => {
                errors.push(IntentValidationError::MissingConflictResolution {
                    conflict_id: conflict.conflict_id.clone(),
                });
            }
            (ConflictState::Unresolved, Some(value)) if !value.trim().is_empty() => {
                errors.push(IntentValidationError::UnexpectedConflictResolution {
                    conflict_id: conflict.conflict_id.clone(),
                });
            }
            (ConflictState::Unresolved, _) => {}
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn validate_statements(
    errors: &mut Vec<IntentValidationError>,
    kind: &'static str,
    statements: &[crate::model::Statement],
) {
    check_unique_ids(
        errors,
        kind,
        statements.iter().map(|item| item.statement_id.as_str()),
    );
    for item in statements {
        check_nonempty(errors, "statement.statement_id", &item.statement_id);
        check_nonempty(errors, "statement.statement", &item.statement);
        if let Some(provenance) = &item.provenance {
            check_nonempty(errors, "statement.provenance.source_id", &provenance.source_id);
        }
    }
}

fn check_nonempty(
    errors: &mut Vec<IntentValidationError>,
    field: &'static str,
    value: &str,
) {
    if value.trim().is_empty() {
        errors.push(IntentValidationError::EmptyField { field });
    }
}

fn check_unique_ids<'a>(
    errors: &mut Vec<IntentValidationError>,
    kind: &'static str,
    ids: impl Iterator<Item = &'a str>,
) {
    let mut seen = HashSet::new();
    for id in ids {
        if !id.trim().is_empty() && !seen.insert(id) {
            errors.push(IntentValidationError::DuplicateIdentifier {
                kind,
                id: id.to_owned(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Claim, ClaimStatus, Conflict, IntentDocument, Relationship, Requirement, SourceRecord,
        Statement, TargetSpec,
    };
    use std::collections::BTreeMap;

    fn sample() -> IntentDocument {
        IntentDocument {
            schema_version: "1.0".into(),
            intent_id: "INT-001".into(),
            revision: 1,
            source: SourceRecord {
                source_id: "SRC-001".into(),
                media_type: "text/plain".into(),
                content: "Create a self-contained artifact.".into(),
            },
            objective: "Create a self-contained artifact.".into(),
            targets: vec![TargetSpec {
                target_id: "artifact".into(),
                kind: "artifact".into(),
                description: "A generic constructed object.".into(),
                properties: vec![Claim {
                    name: "finish".into(),
                    value: "matte".into(),
                    status: ClaimStatus::Declared,
                    provenance: None,
                }],
                relationships: vec![],
            }],
            requirements: vec![Requirement {
                requirement_id: "REQ-001".into(),
                statement: "The artifact must be inspectable.".into(),
                acceptance_criteria: vec!["An inspection representation exists.".into()],
                provenance: None,
            }],
            constraints: vec![],
            governing_laws: vec![],
            assumptions: vec![],
            unknowns: vec![Statement {
                statement_id: "UNK-001".into(),
                statement: "Internal composition is unspecified.".into(),
                provenance: None,
            }],
            conflicts: vec![],
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn accepts_valid_domain_neutral_intent() {
        assert!(validate_intent(&sample()).is_ok());
    }

    #[test]
    fn rejects_missing_creator_source() {
        let mut intent = sample();
        intent.source.content.clear();
        assert!(validate_intent(&intent).is_err());
    }

    #[test]
    fn rejects_zero_revision() {
        let mut intent = sample();
        intent.revision = 0;
        assert!(validate_intent(&intent).is_err());
    }

    #[test]
    fn rejects_duplicate_target_ids() {
        let mut intent = sample();
        intent.targets.push(intent.targets[0].clone());
        assert!(validate_intent(&intent).is_err());
    }

    #[test]
    fn rejects_relationship_to_unknown_target() {
        let mut intent = sample();
        intent.targets[0].relationships.push(Relationship {
            subject_target_id: "artifact".into(),
            predicate: "contains".into(),
            object_target_id: "missing".into(),
        });
        assert!(validate_intent(&intent).is_err());
    }

    #[test]
    fn rejects_requirement_without_acceptance_criteria() {
        let mut intent = sample();
        intent.requirements[0].acceptance_criteria.clear();
        assert!(validate_intent(&intent).is_err());
    }

    #[test]
    fn rejects_conflict_with_only_one_position() {
        let mut intent = sample();
        intent.conflicts.push(Conflict {
            conflict_id: "CON-001".into(),
            subject: "Scale".into(),
            positions: vec!["Large".into()],
            state: ConflictState::Unresolved,
            resolution: None,
        });
        assert!(validate_intent(&intent).is_err());
    }

    #[test]
    fn preserves_unknown_as_a_valid_state() {
        assert_eq!(sample().unknowns.len(), 1);
        assert!(validate_intent(&sample()).is_ok());
    }

    #[test]
    fn permits_multiple_extensible_target_kinds() {
        let mut intent = sample();
        let mut second = intent.targets[0].clone();
        second.target_id = "environment".into();
        second.kind = "user-defined-environment".into();
        intent.targets.push(second);
        assert!(validate_intent(&intent).is_ok());
    }
}
