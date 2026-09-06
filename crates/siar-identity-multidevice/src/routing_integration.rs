//! §172 "Routing Integration", §173 "DTN Integration".

use siar_domain::{AccountId, DeviceId};

use crate::directory::{DeviceDirectory, DeviceEndpoint, DeviceStatus};

/// §172's own three-step resolution chain
/// (`AccountId → active devices → transport endpoints`), as one
/// function rather than three call sites a router would otherwise
/// have to assemble itself — and, critically, filtered to
/// [`DeviceStatus::Active`] only. A router calling
/// `directory.devices` directly could accidentally include a revoked
/// device's stale endpoints; this function is the one place that
/// filter is guaranteed to happen.
pub fn resolve_account_endpoints(
    directory: &DeviceDirectory,
) -> Vec<(DeviceId, Vec<DeviceEndpoint>)> {
    directory
        .devices
        .iter()
        .filter(|entry| entry.status == DeviceStatus::Active)
        .map(|entry| (entry.device_id, entry.transport_endpoints.clone()))
        .collect()
}

/// §173: "opaque identifiers derived from AccountId/DeviceId/routing
/// capability... relay peers do not need full account metadata." A
/// one-way BLAKE3 hash, matching
/// [`crate::discovery_privacy::RotatingDiscoveryToken`]'s own
/// derivation style for the identical concern one layer up (short-
/// range discovery there, DTN destinations here) — a relay holding
/// only this value learns nothing about which account or device it
/// actually names.
pub fn dtn_opaque_identifier(
    account_id: AccountId,
    device_id: DeviceId,
    routing_capability: &[u8],
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"siar-identity-multidevice/dtn-opaque-identifier/v1");
    hasher.update(account_id.to_string().as_bytes());
    hasher.update(device_id.to_string().as_bytes());
    hasher.update(routing_capability);
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::DeviceCapabilitySet;
    use crate::certificate::DeviceCertificate;
    use crate::directory::DeviceDirectoryEntry;
    use crate::root_key::RootIdentityKey;

    fn entry(
        root: &RootIdentityKey,
        account: AccountId,
        device_id: DeviceId,
        status: DeviceStatus,
        endpoints: Vec<DeviceEndpoint>,
    ) -> DeviceDirectoryEntry {
        let certificate = DeviceCertificate::issue(
            root,
            account,
            device_id,
            [0u8; 32],
            0,
            None,
            DeviceCapabilitySet(0),
            1,
        );
        DeviceDirectoryEntry {
            device_id,
            certificate,
            status,
            transport_endpoints: endpoints,
        }
    }

    #[test]
    fn resolution_excludes_revoked_devices_endpoints() {
        let root = RootIdentityKey::generate();
        let account = AccountId::new();
        let active_device = DeviceId::new();
        let revoked_device = DeviceId::new();
        let directory = DeviceDirectory::sign(
            &root,
            account,
            1,
            vec![
                entry(
                    &root,
                    account,
                    active_device,
                    DeviceStatus::Active,
                    vec![DeviceEndpoint(vec![1])],
                ),
                entry(
                    &root,
                    account,
                    revoked_device,
                    DeviceStatus::Revoked,
                    vec![DeviceEndpoint(vec![2])],
                ),
            ],
        );

        let resolved = resolve_account_endpoints(&directory);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].0, active_device);
    }

    #[test]
    fn dtn_identifier_reveals_nothing_derivable_back_to_the_inputs() {
        let account = AccountId::new();
        let device = DeviceId::new();
        let id_a = dtn_opaque_identifier(account, device, b"lan");
        let id_b = dtn_opaque_identifier(account, device, b"ble");
        assert_ne!(
            id_a, id_b,
            "different routing capabilities must not collide onto the same opaque identifier"
        );
    }

    #[test]
    fn dtn_identifier_is_deterministic_for_the_same_inputs() {
        let account = AccountId::new();
        let device = DeviceId::new();
        assert_eq!(
            dtn_opaque_identifier(account, device, b"lan"),
            dtn_opaque_identifier(account, device, b"lan")
        );
    }
}
