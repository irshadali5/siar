# 02 — Multi-Device Identity & Trust

> **Corresponding Specifications:** [`sys-arch/02-multi-device-identity-architecture.md`](../sys-arch/02-multi-device-identity-architecture.md), [`sys-arch/15-qr-nfc-bootstrap-pairing-architecture.md`](../sys-arch/15-qr-nfc-bootstrap-pairing-architecture.md), [`sys-arch/ui-ux-15-security-center-devices-keys-recovery-architecture.md`](../sys-arch/ui-ux-15-security-center-devices-keys-recovery-architecture.md)  
> **Key Crates:** [`crates/siar-identity-multidevice`](../crates/siar-identity-multidevice), [`crates/siar-crypto`](../crates/siar-crypto), [`crates/siar-domain`](../crates/siar-domain)

---

## 1. Cryptographic Identity Model

In SIAR, an identity is not bound to a phone number, email, or username registered on a centralized server. Instead, every user account is anchored in a sovereign **Root Identity Key** (`RootKey`):

```
                       +-------------------------------+
                       |    Account Root Keypair       |
                       |    Ed25519 (Private / Pub)    |
                       | (Stored in HSM / StrongBox)   |
                       +---------------+---------------+
                                       | Signs
          +----------------------------+----------------------------+
          |                                                         |
+---------v---------+                                     +---------v---------+
| Primary Phone     |                                     | Laptop Desktop    |
| Device Key & Cert |                                     | Device Key & Cert |
| (Gen 1, Active)   |                                     | (Gen 2, Active)   |
+-------------------+                                     +-------------------+
```

### Identity Principles
1. **Offline Key Generation**: Keys are generated locally using OS-level secure hardware entropy (`getrandom` / Android `KeyStore` / Secure Enclave).
2. **Delegated Device Certificates**: The root key signs individual device keys via cryptographic certificates (`DeviceCert`). The root private key can remain offline or locked in hardware storage.
3. **Monotonic Device Generations**: Every device certificate carries a strictly increasing generation counter (`u64`) to prevent rollback and replay attacks.

---

## 2. Multi-Device Trust Store & Certificate Chains

Each node maintains a verified `DeviceTrustStore` that indexes valid peer certificates and revoked tombstones:

```rust
pub struct DeviceCert {
    pub account_id: AccountId,
    pub device_id: DeviceId,
    pub device_public_key: PublicKey,
    pub generation: u64,
    pub capabilities: CapabilityBitmask,
    pub issued_at: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub signature: Signature, // Signed by Account RootKey
}
```

### Certificate Validation Rules
1. The `signature` over the certificate payload must verify against the account's known `RootPublicKey`.
2. The `generation` must be greater than or equal to the last recorded generation for that `DeviceId`.
3. If a `RevocationTombstone` exists for the given `DeviceId` with an equal or higher generation, the certificate is rejected immediately.

---

## 3. Device Pairing & SAS Out-of-Band Verification

When a user links a secondary device (e.g., adding a desktop app to an existing phone account), they execute a zero-knowledge, out-of-band mutual authentication protocol:

```mermaid
sequenceDiagram
    autonumber
    actor Alice as Primary Device (Phone)
    actor Bob as New Device (Desktop)

    Bob->>Bob: Generate Ephemeral Keypair (e_bob)
    Bob->>Bob: Display Dynamic Pairing QR Code
    Alice->>Alice: Scan QR Code (Captures e_bob + DeviceId)
    Alice->>Bob: Direct BLE / LAN Link Proposal (Signed by RootKey)
    Alice->>Bob: Key Exchange Handshake (Diffie-Hellman)
    Alice->>Alice: Compute SAS Code = HKDF(shared_secret, 6-digits)
    Bob->>Bob: Compute SAS Code = HKDF(shared_secret, 6-digits)
    Note over Alice,Bob: Both display: "492 108" + 3 Emoji Words
    Alice->>Alice: User confirms SAS match
    Bob->>Bob: User confirms SAS match
    Alice->>Bob: Issue Signed DeviceCert (Gen N+1)
    Bob->>Bob: Persist DeviceCert & Root Anchor
```

---

## 4. Revocation & Tombstone Propagation

When a device is lost, stolen, or decommissioned, the account root key issues a `RevocationTombstone`:

```rust
pub struct RevocationTombstone {
    pub account_id: AccountId,
    pub device_id: DeviceId,
    pub revoked_generation: u64,
    pub reason: RevocationReason,
    pub timestamp: Timestamp,
    pub signature: Signature, // Signed by RootKey
}
```

### Tombstone Gossip Protocol
1. Revocation tombstones are marked with `Priority::High` and gossiped aggressively across all active mesh links and DTN bundles.
2. Any node receiving a tombstone updates its local `DeviceTrustStore` and immediately drops all existing sessions, TLS/Iroh tunnels, and MLS group key material associated with the revoked `DeviceId`.
3. Once revoked, a device cannot rejoin groups without a fresh provisioning flow and newly signed certificate.
