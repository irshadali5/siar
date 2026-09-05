//! §101 "Emergency Identity", §102 "Authority Identity", §103
//! "Identity Claims", §104 "Claim Type".
//!
//! §101's own rule shapes this whole module: "the identity layer
//! should support different principal types without embedding
//! emergency business rules." [`PrincipalType`] is therefore a plain
//! label with no behavior attached to any variant — nothing here
//! decides, say, that an `Authority` principal is automatically
//! trusted or that a `Responder` gets elevated capabilities. Whatever
//! an application does with a principal type is the application's
//! business rule, not this crate's.

use serde::{Deserialize, Serialize};

use crate::error::IdentityError;
use crate::root_key::RootPublicKey;
use siar_domain::AccountId;
use siar_event_log::ids::Timestamp;

/// §101's own four named principal types, verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrincipalType {
    Person,
    Authority,
    Responder,
    AnonymousLimited,
}

/// §103: "keep optional claims separate from core identity... do not
/// hard-code them into `AccountId`." Four named examples plus an
/// escape hatch for anything an application needs that these don't
/// name — §103 explicitly calls its list "examples," not an exhaustive
/// set, and a closed enum would force every future claim kind through
/// a change to this crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimType {
    Organization,
    Role,
    Department,
    /// §102's three examples ("Police Device" / "Hospital Device" /
    /// "Rescue Coordinator") are values of this claim type, not
    /// separate claim types of their own — the distinction §102 draws
    /// is in the (issuer, value) pair, not in a dedicated enum
    /// variant per authority kind.
    EmergencyAuthority,
    Other(String),
}

/// Deliberately just a `String` — §103/§104 never specify a claim
/// value's shape beyond "organization" / "role" / "Police Device"
/// style free text, and inventing structure here that spec doesn't ask
/// for would only make `ClaimValue` harder for an application to use
/// for a claim kind this crate didn't anticipate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimValue(pub String);

/// §104: "applications decide which issuers they trust" — so this is
/// just `AccountId` under a name that reads correctly at claim
/// call-sites, not a new identity concept. Any account able to hold a
/// [`crate::root_key::RootIdentityKey`] can issue a claim; whether a
/// given application chooses to trust a particular issuer for a
/// particular [`ClaimType`] is entirely that application's policy,
/// checked via [`IdentityClaim::is_valid`] with a caller-supplied
/// issuer key and trust decision — never by this type.
pub type IssuerId = AccountId;

/// §104's struct, verbatim field-for-field, with `Signature`
/// materialized as `Vec<u8>` — same reasoning as every other signed
/// type in this crate (see [`crate::directory::DeviceDirectory`]'s own
/// doc comment): a fixed `[u8; 64]` doesn't round-trip through serde
/// derives without extra machinery, and the stored/verified form only
/// ever needs to be bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityClaim {
    pub subject: AccountId,
    pub claim_type: ClaimType,
    pub value: ClaimValue,
    pub issuer: IssuerId,
    pub expires_at: Option<Timestamp>,
    pub signature: Vec<u8>,
}

impl IdentityClaim {
    fn signing_payload(
        subject: AccountId,
        claim_type: &ClaimType,
        value: &ClaimValue,
        issuer: IssuerId,
        expires_at: Option<Timestamp>,
    ) -> Vec<u8> {
        #[derive(Serialize)]
        struct Payload<'a> {
            subject: AccountId,
            claim_type: &'a ClaimType,
            value: &'a ClaimValue,
            issuer: IssuerId,
            expires_at: Option<Timestamp>,
        }
        postcard::to_allocvec(&Payload {
            subject,
            claim_type,
            value,
            issuer,
            expires_at,
        })
        .expect("postcard encoding of a fixed-shape struct cannot fail")
    }

    /// An issuer mints a claim about `subject`. Signed with whatever
    /// root key the issuer account controls — this function takes the
    /// signature bytes directly rather than a `RootIdentityKey`
    /// reference, since an issuer signing a claim about a *different*
    /// account than itself is exactly the case §104 exists for, and
    /// requiring the caller to already have done the actual signing
    /// keeps this type from needing to reach across accounts to find a
    /// key.
    pub fn new(
        subject: AccountId,
        claim_type: ClaimType,
        value: ClaimValue,
        issuer: IssuerId,
        expires_at: Option<Timestamp>,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            subject,
            claim_type,
            value,
            issuer,
            expires_at,
            signature,
        }
    }

    fn verify_signature(&self, issuer_public_key: &RootPublicKey) -> Result<(), IdentityError> {
        let payload = Self::signing_payload(
            self.subject,
            &self.claim_type,
            &self.value,
            self.issuer,
            self.expires_at,
        );
        let signature: [u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| IdentityError::MalformedKey)?;
        issuer_public_key.verify(&payload, &signature)
    }

    /// §104's implicit lifecycle: a claim can expire.
    pub fn is_expired(&self, now: Timestamp) -> bool {
        matches!(self.expires_at, Some(expires_at) if expires_at.0 <= now.0)
    }

    /// §102's actual gate, made into one function: "the UI can display
    /// 'Verified Authority' only when cryptographic policy validates
    /// it." Requires all three of: the claim's own signature verifies
    /// under the issuer key the caller supplies, the claim hasn't
    /// expired, AND the caller's own policy actually trusts this
    /// issuer for this claim type — the third condition is why this
    /// takes a `trusted_issuers` slice rather than treating "signature
    /// verifies" alone as sufficient, since §104's "applications
    /// decide which issuers they trust" would otherwise be silently
    /// bypassed by this crate deciding it instead.
    pub fn is_valid(
        &self,
        issuer_public_key: &RootPublicKey,
        trusted_issuers: &[IssuerId],
        now: Timestamp,
    ) -> bool {
        trusted_issuers.contains(&self.issuer)
            && !self.is_expired(now)
            && self.verify_signature(issuer_public_key).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::root_key::RootIdentityKey;

    fn sign_claim(
        issuer_key: &RootIdentityKey,
        subject: AccountId,
        claim_type: ClaimType,
        value: ClaimValue,
        issuer: IssuerId,
        expires_at: Option<Timestamp>,
    ) -> IdentityClaim {
        let payload =
            IdentityClaim::signing_payload(subject, &claim_type, &value, issuer, expires_at);
        let signature = issuer_key.sign(&payload).to_vec();
        IdentityClaim::new(subject, claim_type, value, issuer, expires_at, signature)
    }

    #[test]
    fn authority_claim_is_valid_only_when_issuer_is_trusted() {
        let issuer_key = RootIdentityKey::generate();
        let issuer = AccountId::new();
        let subject = AccountId::new();
        let claim = sign_claim(
            &issuer_key,
            subject,
            ClaimType::EmergencyAuthority,
            ClaimValue("Police Device".to_string()),
            issuer,
            None,
        );

        assert!(claim.is_valid(&issuer_key.root_public_key(), &[issuer], Timestamp(0)));
        // Same signature, same everything — but the application simply
        // doesn't trust this issuer for anything.
        assert!(!claim.is_valid(&issuer_key.root_public_key(), &[], Timestamp(0)));
    }

    #[test]
    fn expired_claim_is_never_valid_even_if_trusted_and_correctly_signed() {
        let issuer_key = RootIdentityKey::generate();
        let issuer = AccountId::new();
        let subject = AccountId::new();
        let claim = sign_claim(
            &issuer_key,
            subject,
            ClaimType::Role,
            ClaimValue("Rescue Coordinator".to_string()),
            issuer,
            Some(Timestamp(100)),
        );

        assert!(claim.is_valid(&issuer_key.root_public_key(), &[issuer], Timestamp(50)));
        assert!(!claim.is_valid(&issuer_key.root_public_key(), &[issuer], Timestamp(200)));
    }

    #[test]
    fn a_forged_claim_never_validates_regardless_of_trust_list() {
        let real_issuer_key = RootIdentityKey::generate();
        let forger_key = RootIdentityKey::generate();
        let issuer = AccountId::new();
        let subject = AccountId::new();

        // Forger signs a claim, dishonestly labeling itself as the
        // real issuer's AccountId.
        let forged = sign_claim(
            &forger_key,
            subject,
            ClaimType::EmergencyAuthority,
            ClaimValue("Hospital Device".to_string()),
            issuer,
            None,
        );

        assert!(!forged.is_valid(&real_issuer_key.root_public_key(), &[issuer], Timestamp(0)));
    }
}
