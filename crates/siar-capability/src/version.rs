//! §9 "Capability Versioning" and §10 "Protocol Version vs Capability
//! Version".

use serde::{Deserialize, Serialize};
use std::fmt;

/// §9: a capability's own major/minor version, kept deliberately
/// separate from the protocol extension's own version (§10: "This lets
/// a stable protocol major gain backward-compatible features").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CapabilityVersion {
    pub major: u16,
    pub minor: u16,
}

impl CapabilityVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// §19's "range" intersection rule specialized to versions: two
    /// advertisements of the *same* capability are compatible only on
    /// matching major (a major bump is a breaking change by
    /// convention — the spec doesn't restate semver explicitly, but
    /// §10's whole point is that minor-version growth is what stays
    /// backward compatible within one major). The negotiated version
    /// is then the lower minor, i.e. `min(local, remote)` per §19's
    /// "effective max = min(...)" rule.
    pub fn negotiate(self, remote: Self) -> Option<Self> {
        if self.major != remote.major {
            return None;
        }
        Some(Self::new(self.major, self.minor.min(remote.minor)))
    }
}

impl fmt::Display for CapabilityVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negotiate_takes_lower_minor_on_matching_major() {
        let local = CapabilityVersion::new(1, 4);
        let remote = CapabilityVersion::new(1, 2);
        assert_eq!(local.negotiate(remote), Some(CapabilityVersion::new(1, 2)));
        // Symmetric.
        assert_eq!(remote.negotiate(local), Some(CapabilityVersion::new(1, 2)));
    }

    #[test]
    fn negotiate_rejects_major_mismatch() {
        let local = CapabilityVersion::new(2, 0);
        let remote = CapabilityVersion::new(1, 9);
        assert_eq!(local.negotiate(remote), None);
    }
}
