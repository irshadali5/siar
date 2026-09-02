//! spec §84 "Typed Extension Configuration", §85 "Configuration
//! Validation", §86 "Extension Dependencies".

use crate::descriptor::ExtensionLimits;

/// spec §84, verbatim struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MessagingConfig {
    pub max_message_size: usize,
    pub receipts: bool,
    pub editing: bool,
}

/// spec §84, verbatim struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FileConfig {
    pub chunk_size: usize,
    pub max_parallel_chunks: usize,
    pub max_file_size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    /// spec §85's own worked example, verbatim: "chunk size > extension
    /// maximum frame size."
    #[error("chunk_size {chunk_size} exceeds this extension's max_frame_size {max_frame_size}")]
    ChunkSizeExceedsMaxFrameSize {
        chunk_size: usize,
        max_frame_size: usize,
    },
}

/// spec §85: "Validate before opening network listeners... Fail fast
/// during runtime construction." A plain function callers are expected
/// to run at construction time, not something this crate can enforce
/// they actually call — the "before opening network listeners" part is
/// a caller-ordering discipline this type can't structurally guarantee
/// any more than any validation function can.
pub fn validate_file_config(config: &FileConfig, limits: &ExtensionLimits) -> Result<(), ConfigError> {
    if config.chunk_size > limits.max_frame_size {
        return Err(ConfigError::ChunkSizeExceedsMaxFrameSize {
            chunk_size: config.chunk_size,
            max_frame_size: limits.max_frame_size,
        });
    }
    Ok(())
}

/// spec §86's own two-item list, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExtensionDependencyKind {
    RequiredDependency,
    OptionalIntegration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{CapabilityId, CapabilitySet};
    use crate::descriptor::{ExtensionDescriptor, ExtensionRequirement, ExtensionStability, ExtensionVersion};
    use crate::identifier::{NamespaceId, ProtocolId, ProtocolMajor, ProtocolMinor, ProtocolName};
    use crate::negotiation::{negotiate, RemoteAdvertisement};
    use crate::security::SecurityRequirements;

    #[test]
    fn spec_85_oversized_chunk_is_rejected() {
        let config = FileConfig {
            chunk_size: 100_000,
            max_parallel_chunks: 4,
            max_file_size: 1_000_000_000,
        };
        let limits = ExtensionLimits {
            max_frame_size: 65536,
            max_in_flight_frames: 32,
            max_concurrent_streams: 4,
            max_buffered_bytes: 1 << 20,
        };
        let err = validate_file_config(&config, &limits).unwrap_err();
        assert_eq!(
            err,
            ConfigError::ChunkSizeExceedsMaxFrameSize {
                chunk_size: 100_000,
                max_frame_size: 65536,
            }
        );
    }

    #[test]
    fn spec_85_chunk_within_frame_size_validates() {
        let config = FileConfig {
            chunk_size: 4096,
            max_parallel_chunks: 4,
            max_file_size: 1_000_000_000,
        };
        let limits = ExtensionLimits {
            max_frame_size: 65536,
            max_in_flight_frames: 32,
            max_concurrent_streams: 4,
            max_buffered_bytes: 1 << 20,
        };
        assert!(validate_file_config(&config, &limits).is_ok());
    }

    #[test]
    fn spec_86_messaging_negotiates_fine_with_no_files_extension_advertised_at_all() {
        // spec §86's own example: "messaging: optional integration ->
        // files. Messaging itself must continue working without
        // files." Proven directly: negotiate messaging alone, with
        // `remote` never mentioning files at all (not even as an
        // unsupported entry) — messaging must still succeed.
        let messaging_id = ProtocolId::new(
            NamespaceId::new("org.example").unwrap(),
            ProtocolName::new("messaging").unwrap(),
            ProtocolMajor(1),
        );
        let local = vec![ExtensionDescriptor {
            id: messaging_id.clone(),
            version: ExtensionVersion {
                major: ProtocolMajor(1),
                minor: ProtocolMinor(0),
            },
            capabilities: CapabilitySet::new([CapabilityId(1)]),
            required_capabilities: CapabilitySet::default(),
            requirement: ExtensionRequirement::Required,
            limits: ExtensionLimits {
                max_frame_size: 65536,
                max_in_flight_frames: 32,
                max_concurrent_streams: 4,
                max_buffered_bytes: 1 << 20,
            },
            security: SecurityRequirements::messaging_default(),
            stability: ExtensionStability::Stable,
        }];
        let remote = vec![RemoteAdvertisement {
            id: messaging_id,
            capabilities: CapabilitySet::new([CapabilityId(1)]),
        }];

        let result = negotiate(&local, &remote).unwrap();
        assert_eq!(result.len(), 1);
    }
}
