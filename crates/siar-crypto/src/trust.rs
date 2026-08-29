//! Peer trust states and trust sources (Part 28 §40, §41).
//!
//! Both types are close to literal transcriptions of the spec's own
//! sketches — there's little to design here beyond making sure nothing
//! about the Rust representation quietly adds behavior the spec didn't
//! ask for. §41's one substantive rule — "trust is not automatically
//! transitive" — isn't something a type system can enforce on its own
//! (nothing here stops a caller from writing code that treats it as
//! transitive); it's a rule for whatever caller consumes
//! `PeerTrustState`/`TrustSource`, so it's documented, not encoded as
//! a method this module doesn't have enough context to implement
//! correctly (a real transitivity check needs a trust graph and a
//! policy for how far to walk it, which belongs with whatever owns
//! that graph, not with these two enums).

use serde::{Deserialize, Serialize};

/// §40, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerTrustState {
    Unknown,
    Seen,
    Verified,
    Trusted,
    Revoked,
    Blocked,
}

/// §41's five listed sources, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustSource {
    QrSasVerification,
    NfcBootstrap,
    OrganizationCertificate,
    ExistingVerifiedDevice,
    ManualFingerprintComparison,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_states_round_trip_through_postcard() {
        for state in [
            PeerTrustState::Unknown,
            PeerTrustState::Seen,
            PeerTrustState::Verified,
            PeerTrustState::Trusted,
            PeerTrustState::Revoked,
            PeerTrustState::Blocked,
        ] {
            let bytes = postcard::to_allocvec(&state).unwrap();
            let decoded: PeerTrustState = postcard::from_bytes(&bytes).unwrap();
            assert_eq!(decoded, state);
        }
    }

    #[test]
    fn trust_sources_round_trip_through_postcard() {
        for source in [
            TrustSource::QrSasVerification,
            TrustSource::NfcBootstrap,
            TrustSource::OrganizationCertificate,
            TrustSource::ExistingVerifiedDevice,
            TrustSource::ManualFingerprintComparison,
        ] {
            let bytes = postcard::to_allocvec(&source).unwrap();
            let decoded: TrustSource = postcard::from_bytes(&bytes).unwrap();
            assert_eq!(decoded, source);
        }
    }
}
