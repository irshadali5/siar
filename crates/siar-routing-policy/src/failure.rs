//! §36 "Route Failure Classification", §37 "Failure Examples".

/// §36's seven classes. This crate has no live transport error types to
/// classify from (see its top doc comment: no wire integration), so
/// there is no `classify(transport_error) -> RouteFailureClass`
/// function here — a real transport bridge (e.g. `siar-transport`,
/// `apps/android`'s transport-jni crates) is what would produce a value
/// of this type from its own real error, mapping per §37's worked
/// examples (documented on each variant below, not encoded as
/// executable logic since this crate has nothing concrete to run that
/// logic against yet).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteFailureClass {
    /// §37: "Timeout → Temporary."
    Temporary,
    /// §37: "Bluetooth disabled → TransportUnavailable."
    TransportUnavailable,
    /// §37: "Revoked device → AuthenticationFailure." The natural
    /// integration point for `siar-identity-multidevice`'s
    /// [`crate::resolve`] module: a candidate whose device the
    /// `TrustedAccountStore` no longer lists as active should surface
    /// this class, not `Temporary`.
    AuthenticationFailure,
    /// §37: "Metered forbidden → PolicyDenied."
    PolicyDenied,
    /// §37: "Unsupported extension → RemoteRejected." The natural
    /// integration point for Part 01's extension negotiation (§106
    /// "Extension Capability Integration").
    RemoteRejected,
    /// §37: "Invalid destination → Permanent."
    Permanent,
    Unknown,
}

impl RouteFailureClass {
    /// §37: "Do not blindly retry all failures." `Unknown` is treated
    /// as retryable — a conservative default (better to retry an
    /// operation that turns out permanent than to silently drop one
    /// that was actually transient) rather than the spec dictating
    /// either choice explicitly for this variant.
    pub fn is_retryable(self) -> bool {
        matches!(self, Self::Temporary | Self::TransportUnavailable | Self::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporary_and_transport_unavailable_are_retryable() {
        assert!(RouteFailureClass::Temporary.is_retryable());
        assert!(RouteFailureClass::TransportUnavailable.is_retryable());
    }

    #[test]
    fn permanent_authentication_and_policy_failures_are_not_retryable() {
        assert!(!RouteFailureClass::Permanent.is_retryable());
        assert!(!RouteFailureClass::AuthenticationFailure.is_retryable());
        assert!(!RouteFailureClass::PolicyDenied.is_retryable());
        assert!(!RouteFailureClass::RemoteRejected.is_retryable());
    }
}
