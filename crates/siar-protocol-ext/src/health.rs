//! spec §93 "Error Code Strategy", §94 "Extension Health", §95
//! "Recovery Semantics".

use crate::identifier::ProtocolId;
use crate::lifecycle::ExtensionLifecycle;

/// spec §93's own worked example ranges, verbatim — core gets
/// 0x0000–0x00FF, each extension after it a further 0x0100 block.
/// "Prefer extension-scoped stable codes... or local error spaces
/// scoped by negotiated extension ID" — this is the fixed, documented
/// global-range half of that choice; a "local error space scoped by
/// negotiated [`crate::descriptor::SessionLocalExtensionId`]" is the
/// other, and this type doesn't force a choice between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ErrorCodeRange {
    pub start: u16,
    pub end_inclusive: u16,
}

pub const CORE_ERROR_CODE_RANGE: ErrorCodeRange = ErrorCodeRange {
    start: 0x0000,
    end_inclusive: 0x00FF,
};
pub const MESSAGING_ERROR_CODE_RANGE: ErrorCodeRange = ErrorCodeRange {
    start: 0x0100,
    end_inclusive: 0x01FF,
};
pub const FILES_ERROR_CODE_RANGE: ErrorCodeRange = ErrorCodeRange {
    start: 0x0200,
    end_inclusive: 0x02FF,
};

/// spec §93: "Human-readable text is diagnostic only." Structural, not
/// just a convention: [`ErrorCode`] always carries a stable numeric
/// `code`, and `diagnostic_text` is a plain, separate `String` field —
/// nothing here lets a caller match on the text instead of the code,
/// which is the actual failure mode this rule guards against (a
/// diagnostic message wording change silently breaking error-handling
/// logic elsewhere).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ErrorCode {
    pub code: u16,
    pub diagnostic_text: String,
}

impl ErrorCodeRange {
    pub fn contains(&self, code: u16) -> bool {
        code >= self.start && code <= self.end_inclusive
    }
}

/// spec §94, verbatim struct — `state` uses this crate's real
/// [`ExtensionLifecycle`] (§23) rather than an unspecified
/// `ExtensionState`, since spec §94 never defines that type separately
/// and this crate already has the real one.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtensionHealth {
    pub extension: ProtocolId,
    pub state: ExtensionLifecycle,
    pub queue_depth: usize,
    pub active_operations: usize,
    pub last_error: Option<ExtensionErrorSummary>,
}

/// spec §94's `Option<ExtensionErrorSummary>` field, given a real
/// shape: an [`ErrorCode`] plus when it happened — "diagnostics,
/// automated recovery, support bundles, UI health indicators" (§94's
/// own four listed uses) all need a timestamp to be useful, not just
/// the error itself.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExtensionErrorSummary {
    pub error: ErrorCode,
    pub occurred_at_millis: u64,
}

/// spec §95, verbatim four-class taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OperationRecoveryClass {
    Retryable,
    Resumable,
    Discardable,
    Expired,
}

/// spec §95's own five worked examples, verbatim.
pub fn spec_95_example_classification(operation_name: &str) -> Option<OperationRecoveryClass> {
    match operation_name {
        "text message" => Some(OperationRecoveryClass::Retryable),
        "file transfer" => Some(OperationRecoveryClass::Resumable),
        "typing" | "video frame" => Some(OperationRecoveryClass::Discardable),
        "expired SOS" => Some(OperationRecoveryClass::Expired),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifier::{NamespaceId, ProtocolMajor, ProtocolName};

    #[test]
    fn spec_93_ranges_do_not_overlap_at_their_boundaries() {
        let boundary = CORE_ERROR_CODE_RANGE.end_inclusive;
        assert!(!MESSAGING_ERROR_CODE_RANGE.contains(boundary));
        let boundary = MESSAGING_ERROR_CODE_RANGE.end_inclusive;
        assert!(!FILES_ERROR_CODE_RANGE.contains(boundary));
    }

    #[test]
    fn spec_93_a_messaging_code_is_recognized_by_its_range_not_its_text() {
        let error = ErrorCode {
            code: 0x0142,
            diagnostic_text: "delivery receipt timeout".to_string(),
        };
        assert!(MESSAGING_ERROR_CODE_RANGE.contains(error.code));
        assert!(!FILES_ERROR_CODE_RANGE.contains(error.code));
    }

    #[test]
    fn spec_94_health_carries_a_real_lifecycle_state() {
        let health = ExtensionHealth {
            extension: ProtocolId::new(
                NamespaceId::new("org.example").unwrap(),
                ProtocolName::new("messaging").unwrap(),
                ProtocolMajor(1),
            ),
            state: ExtensionLifecycle::Active,
            queue_depth: 3,
            active_operations: 1,
            last_error: None,
        };
        assert_eq!(health.state, ExtensionLifecycle::Active);
    }

    #[test]
    fn spec_95_five_worked_examples_classify_as_given() {
        assert_eq!(
            spec_95_example_classification("text message"),
            Some(OperationRecoveryClass::Retryable)
        );
        assert_eq!(
            spec_95_example_classification("file transfer"),
            Some(OperationRecoveryClass::Resumable)
        );
        assert_eq!(
            spec_95_example_classification("typing"),
            Some(OperationRecoveryClass::Discardable)
        );
        assert_eq!(
            spec_95_example_classification("video frame"),
            Some(OperationRecoveryClass::Discardable)
        );
        assert_eq!(
            spec_95_example_classification("expired SOS"),
            Some(OperationRecoveryClass::Expired)
        );
    }
}
