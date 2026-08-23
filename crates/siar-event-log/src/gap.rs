//! §26 "Gap Detection".

/// §26's own worked example: "stream expects version 44 but receives
/// 46 → mark the stream incomplete and request reconciliation." A pure
/// function, not a method on any stateful type — this crate's actual
/// stores don't call it internally (an in-memory/SQL append naturally
/// can't produce a gap in its own local stream), it's meant for the
/// *remote ingestion* path (§23), where a caller applying incoming
/// events one at a time checks each arrival against what it expected
/// next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamGap {
    pub expected_version: u64,
    pub received_version: u64,
}

/// Returns `Some` if `received_version` skips ahead of
/// `expected_next_version` (a real gap — one or more versions are
/// missing), `None` if it's exactly the expected next version or, per
/// §24, a duplicate/stale version that idempotency already handles
/// elsewhere (not this function's concern — see its own doc comment on
/// scope).
pub fn detect_gap(expected_next_version: u64, received_version: u64) -> Option<StreamGap> {
    if received_version > expected_next_version {
        Some(StreamGap { expected_version: expected_next_version, received_version })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spec_own_worked_example_is_detected_as_a_gap() {
        assert_eq!(detect_gap(44, 46), Some(StreamGap { expected_version: 44, received_version: 46 }));
    }

    #[test]
    fn the_exact_expected_version_is_not_a_gap() {
        assert_eq!(detect_gap(44, 44), None);
    }

    #[test]
    fn a_version_at_or_before_expected_is_not_a_gap() {
        assert_eq!(detect_gap(44, 43), None); // stale/duplicate — §24's concern, not this function's
    }
}
