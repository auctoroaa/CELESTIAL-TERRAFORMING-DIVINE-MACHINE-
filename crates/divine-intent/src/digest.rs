use crate::model::IntentDocument;
use thiserror::Error;

const DIGEST_DOMAIN: &[u8] = b"celestial-terraforming-divine-intent:v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentDigest(pub String);

#[derive(Debug, Error)]
pub enum DigestError {
    #[error("intent serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub(crate) fn digest_intent(intent: &IntentDocument) -> Result<IntentDigest, DigestError> {
    let bytes = serde_json::to_vec(intent)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(DIGEST_DOMAIN);
    hasher.update(&bytes);
    Ok(IntentDigest(hasher.finalize().to_hex().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{IntentDocument, SourceRecord, TargetSpec};
    use std::collections::BTreeMap;

    fn intent() -> IntentDocument {
        IntentDocument {
            schema_version: "1.0".into(),
            intent_id: "INT-1".into(),
            revision: 1,
            source: SourceRecord {
                source_id: "SRC-1".into(),
                media_type: "text/plain".into(),
                content: "Make an artifact".into(),
            },
            objective: "Make an artifact".into(),
            targets: vec![TargetSpec {
                target_id: "T-1".into(),
                kind: "artifact".into(),
                description: "An artifact".into(),
                properties: vec![],
                relationships: vec![],
            }],
            requirements: vec![],
            constraints: vec![],
            governing_laws: vec![],
            assumptions: vec![],
            unknowns: vec![],
            conflicts: vec![],
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn digest_is_stable_for_the_same_revision() {
        assert_eq!(digest_intent(&intent()).unwrap(), digest_intent(&intent()).unwrap());
    }

    #[test]
    fn digest_changes_when_intent_changes() {
        let original = intent();
        let mut revised = original.clone();
        revised.revision = 2;
        assert_ne!(digest_intent(&original).unwrap(), digest_intent(&revised).unwrap());
    }

    #[test]
    fn digest_has_64_hex_characters() {
        assert_eq!(digest_intent(&intent()).unwrap().0.len(), 64);
    }
}
