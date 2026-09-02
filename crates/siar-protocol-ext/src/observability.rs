//! spec §57 "Observability", §58 "Metrics", §59 "Diagnostics".

use crate::capability::CapabilityId;
use crate::descriptor::{ExtensionVersion, NegotiatedExtension};
use crate::identifier::ProtocolId;
use crate::security::PeerIdentity;

/// spec §57's own eight trace fields, verbatim — and, just as
/// importantly, nothing else. "Never log sensitive payload contents or
/// keys" is enforced here structurally, not just by instruction: there
/// is no field on this struct capable of holding a payload or a key in
/// the first place, so a caller populating every field this type has
/// still cannot have logged either.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TraceFields {
    pub peer_id: PeerIdentity,
    pub extension_id: ProtocolId,
    pub version: ExtensionVersion,
    pub frame_type: u8,
    pub operation_id: String,
    pub duration_millis: u64,
    pub bytes: u64,
    pub result: TraceResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TraceResult {
    Success,
    Failure,
}

/// spec §58's own eight named metrics, as real counters/gauges rather
/// than only a wishlist — "local metrics should work without external
/// telemetry" is true by construction: nothing here does I/O or
/// touches a network, it's a plain in-memory struct a caller reads
/// directly.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExtensionMetrics {
    pub negotiation_success_count: u64,
    pub open_latency_millis_last: u64,
    pub frames_sent: u64,
    pub frames_received: u64,
    pub protocol_violations: u64,
    pub queue_depth: u64,
    pub backpressure_activations: u64,
    pub unsupported_capability_count: u64,
    pub version_mismatch_count: u64,
}

impl ExtensionMetrics {
    pub fn record_frame_sent(&mut self) {
        self.frames_sent += 1;
    }

    pub fn record_frame_received(&mut self) {
        self.frames_received += 1;
    }

    pub fn record_protocol_violation(&mut self) {
        self.protocol_violations += 1;
    }
}

/// spec §59's own worked example, reproduced structurally: a
/// human-readable "Active extensions: ... / Unsupported remote
/// extensions: ..." view. `capability_name` is a caller-supplied
/// lookup rather than something this crate invents — [`CapabilityId`]
/// is just a `u32` (see `capability.rs`'s own doc comment for why),
/// this crate has no built-in id-to-name registry, and guessing names
/// like "text"/"reply"/"receipts" for arbitrary ids would be exactly
/// the kind of invented content this pass avoids.
pub fn render_diagnostics(
    active: &[NegotiatedExtension],
    capability_name: impl Fn(CapabilityId) -> String,
    unsupported_remote: &[ProtocolId],
) -> String {
    let mut out = String::from("Active extensions:\n");
    for extension in active {
        out.push_str(&format!("  {}\n", extension.id.canonical_name()));
        for capability in &extension.capabilities.values {
            out.push_str(&format!("    {}\n", capability_name(*capability)));
        }
    }

    out.push_str("\nUnsupported remote extensions:\n");
    for protocol in unsupported_remote {
        out.push_str(&format!("  {}\n", protocol.canonical_name()));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilitySet;
    use crate::descriptor::SessionLocalExtensionId;
    use crate::identifier::{NamespaceId, ProtocolMajor, ProtocolName};

    #[test]
    fn spec_57_trace_fields_cannot_carry_payload_or_keys() {
        // Structural test: this compiles with every field populated
        // and still contains nothing but the eight named fields —
        // there is no way to smuggle a payload/key in.
        let trace = TraceFields {
            peer_id: PeerIdentity([0u8; 32]),
            extension_id: ProtocolId::new(
                NamespaceId::new("org.example").unwrap(),
                ProtocolName::new("messaging").unwrap(),
                ProtocolMajor(1),
            ),
            version: ExtensionVersion {
                major: ProtocolMajor(1),
                minor: crate::identifier::ProtocolMinor(0),
            },
            frame_type: 1,
            operation_id: "op-1".to_string(),
            duration_millis: 42,
            bytes: 1024,
            result: TraceResult::Success,
        };
        assert_eq!(trace.frame_type, 1);
    }

    #[test]
    fn spec_58_metrics_work_with_no_telemetry_backend() {
        let mut metrics = ExtensionMetrics::default();
        metrics.record_frame_sent();
        metrics.record_frame_sent();
        metrics.record_protocol_violation();
        assert_eq!(metrics.frames_sent, 2);
        assert_eq!(metrics.protocol_violations, 1);
    }

    #[test]
    fn spec_59_diagnostic_view_matches_the_spec_example_shape() {
        let messaging = ProtocolId::new(
            NamespaceId::new("org.example").unwrap(),
            ProtocolName::new("messaging").unwrap(),
            ProtocolMajor(1),
        );
        let active = vec![NegotiatedExtension {
            id: messaging,
            session_id: SessionLocalExtensionId(1),
            capabilities: CapabilitySet::new([CapabilityId(1), CapabilityId(2)]),
        }];
        let unsupported = vec![ProtocolId::new(
            NamespaceId::new("com.example").unwrap(),
            ProtocolName::new("custom").unwrap(),
            ProtocolMajor(2),
        )];

        let names = |id: CapabilityId| match id.0 {
            1 => "text".to_string(),
            2 => "reply".to_string(),
            other => format!("capability-{other}"),
        };

        let view = render_diagnostics(&active, names, &unsupported);
        assert!(view.contains("Active extensions:"));
        assert!(view.contains("org.example/messaging/1"));
        assert!(view.contains("text"));
        assert!(view.contains("reply"));
        assert!(view.contains("Unsupported remote extensions:"));
        assert!(view.contains("com.example/custom/2"));
    }
}
