//! Strong protocol identifiers — spec §5 "Protocol Namespace", §6
//! "Strong Protocol Identifiers", §7 "Major and Minor Versions".
//!
//! The spec's own reasoning for why these are newtypes rather than
//! `String`/`u32` scattered through the runtime: "Do not use arbitrary
//! strings throughout the runtime" (§6). A canonical string
//! (`org.example.comm/messaging/1`) is still how a `ProtocolId` prints
//! and how it round-trips at negotiation time — see [`ProtocolId::canonical_name`]
//! — but nothing downstream of parsing compares raw strings.

use std::fmt;

/// `org.example.comm` in `org.example.comm/messaging/1` — spec §5.
/// Validated on construction: non-empty, ASCII, no `/` (the field
/// separator in the canonical form), since a namespace containing `/`
/// would make [`ProtocolId::canonical_name`] ambiguous to parse back.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct NamespaceId(String);

/// `messaging` in `org.example.comm/messaging/1` — spec §5. Same
/// validation reasoning as [`NamespaceId`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProtocolName(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdentifierError {
    #[error("identifier must not be empty")]
    Empty,
    #[error("identifier must not contain '/': {0:?}")]
    ContainsSeparator(String),
    #[error("identifier must be ASCII: {0:?}")]
    NotAscii(String),
    #[error("expected a numeric major version, got {0:?}")]
    InvalidMajorVersion(String),
}

fn validate_segment(s: &str) -> Result<(), IdentifierError> {
    if s.is_empty() {
        return Err(IdentifierError::Empty);
    }
    if s.contains('/') {
        return Err(IdentifierError::ContainsSeparator(s.to_string()));
    }
    if !s.is_ascii() {
        return Err(IdentifierError::NotAscii(s.to_string()));
    }
    Ok(())
}

impl NamespaceId {
    pub fn new(s: impl Into<String>) -> Result<Self, IdentifierError> {
        let s = s.into();
        validate_segment(&s)?;
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ProtocolName {
    pub fn new(s: impl Into<String>) -> Result<Self, IdentifierError> {
        let s = s.into();
        validate_segment(&s)?;
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NamespaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for ProtocolName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Wire-incompatible version number — spec §7 "Major version": "Peers
/// sharing no major version cannot use that extension together."
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProtocolMajor(pub u16);

/// Backward-compatible feature-expansion counter within one
/// [`ProtocolMajor`] — spec §7 "Minor version": "Messaging v1.4 may
/// communicate with Messaging v1.2 using the common feature subset."
/// The "common feature subset" itself is [`crate::capability::CapabilitySet`]
/// intersection (spec §8), not derived from the minor number directly —
/// minor is informational/diagnostic here, matching the spec's own
/// framing that capability negotiation, not the version number alone,
/// determines the effective feature set (§8: "Version alone is
/// insufficient").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProtocolMinor(pub u16);

/// spec §6's `ProtocolId` struct, exactly as given there — the strong
/// identifier every extension is keyed by throughout this crate,
/// instead of an arbitrary string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProtocolId {
    pub namespace: NamespaceId,
    pub protocol: ProtocolName,
    pub major: ProtocolMajor,
}

impl ProtocolId {
    pub fn new(namespace: NamespaceId, protocol: ProtocolName, major: ProtocolMajor) -> Self {
        Self { namespace, protocol, major }
    }

    /// `org.example.comm/messaging/1` — spec §5's canonical form,
    /// spec §6's "bounded canonical string during negotiation" (the
    /// first half of the two-phase identifier strategy §6 recommends;
    /// see [`crate::negotiation`] for the second half, the
    /// session-local numeric ID assigned after negotiation completes).
    pub fn canonical_name(&self) -> String {
        format!("{}/{}/{}", self.namespace, self.protocol, self.major.0)
    }
}

impl fmt::Display for ProtocolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical_name())
    }
}

/// Parses `org.example.comm/messaging/1` back into a [`ProtocolId`] —
/// the receiving side of what [`ProtocolId::canonical_name`] produces,
/// needed since HELLO/HELLO_ACK (spec §10) exchange the canonical
/// string form, not a pre-parsed struct, before any session-local
/// numeric ID exists to negotiate with.
impl std::str::FromStr for ProtocolId {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(3, '/');
        let namespace = parts.next().ok_or(IdentifierError::Empty)?;
        let protocol = parts.next().ok_or(IdentifierError::Empty)?;
        let major = parts.next().ok_or(IdentifierError::Empty)?;
        let major: u16 = major.parse().map_err(|_| IdentifierError::InvalidMajorVersion(major.to_string()))?;
        Ok(Self {
            namespace: NamespaceId::new(namespace)?,
            protocol: ProtocolName::new(protocol)?,
            major: ProtocolMajor(major),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_round_trip() {
        let id = ProtocolId::new(
            NamespaceId::new("org.example.comm").unwrap(),
            ProtocolName::new("messaging").unwrap(),
            ProtocolMajor(1),
        );
        assert_eq!(id.canonical_name(), "org.example.comm/messaging/1");
        let parsed: ProtocolId = id.canonical_name().parse().unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn rejects_separator_in_segment() {
        assert!(matches!(NamespaceId::new("a/b"), Err(IdentifierError::ContainsSeparator(_))));
    }

    #[test]
    fn rejects_empty_segment() {
        assert!(matches!(ProtocolName::new(""), Err(IdentifierError::Empty)));
    }
}
