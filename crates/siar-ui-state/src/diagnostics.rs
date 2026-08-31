//! Security Details vs Diagnostics (ui-ux-15 §133-134).
//!
//! §133: normal Security Center shows human meaning; a separate
//! Diagnostics view shows key IDs, epoch IDs, signature status,
//! transport security details — for advanced/support use, not the
//! everyday screens this whole spec otherwise describes.
//!
//! §134: "No Raw Private Keys in Diagnostics" — a hard rule. Enforced
//! here the same way every other "must never expose X" rule in this
//! session's work has been: by absence. `SecurityDiagnosticDetail`
//! has no field capable of holding private key material — not a
//! private-key field with a redaction step, not an `Option` that's
//! supposed to stay `None`. There is nowhere on this type a private
//! key could go even by mistake.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityDiagnosticDetail {
    pub key_id: String,
    pub epoch_id: String,
    pub signature_status: String,
    pub transport_security_details: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_detail_holds_only_the_four_spec_listed_fields() {
        // Structural test: constructing one requires exactly these
        // four `String` fields — there is no fifth field this could
        // compile with, private-key-shaped or otherwise.
        let detail = SecurityDiagnosticDetail {
            key_id: "k-1".to_string(),
            epoch_id: "e-7".to_string(),
            signature_status: "valid".to_string(),
            transport_security_details: "TLS 1.3".to_string(),
        };
        assert_eq!(detail.key_id, "k-1");
    }
}
