//! §18 "Negotiation Output" (partial — see the module doc note below),
//! §19 "Intersection Rule", §20 "Policy Filter", §24 "Negotiation
//! State Machine" (the `Validate → Intersect → PolicyFilter →
//! RequiredCheck` portion), §72 "Negotiation Determinism", §73
//! "Selection Function".
//!
//! This module implements the pure core of negotiation — combining
//! two [`CapabilitySet`]s into the mutually-usable, policy-filtered
//! result — as the single [`negotiate`] function. It deliberately
//! returns a plain [`CapabilitySet`], not the spec's full §18
//! `NegotiatedCapabilities` struct: that struct's fields
//! (`extensions: HashMap<ProtocolId, ...>`, `security`, `transport`,
//! `media`, `limits`) each depend on subsystems this crate doesn't
//! reach into yet (Part 01's `ProtocolId`, and the not-yet-built
//! security/transport/media capability schemas from §29 / §38 / §40).
//! A caller that owns those types can build `NegotiatedCapabilities`
//! by calling [`negotiate`] once per domain and assembling the result
//! — that assembly step is left to that future caller rather than
//! guessed at here.

use crate::descriptor::{CapabilityDescriptor, CapabilityParameters, CapabilityRequirement};
use crate::error::CapabilityNegotiationError;
use crate::id::CapabilityId;
use crate::policy::CapabilityPolicy;
use crate::registry::CapabilityRegistry;
use crate::set::CapabilitySet;
use std::collections::BTreeSet;

/// §73: `negotiate(local, remote, policy) -> Result<...>` as a "pure...
/// no network side effects" function, extended with a `registry`
/// argument so the §68/§69 checks already built in `registry.rs` run
/// as this function's own §24 "Validate" step rather than being left
/// for every caller to remember to call separately.
pub fn negotiate(
    local: &CapabilitySet,
    remote: &CapabilitySet,
    registry: &CapabilityRegistry,
    policy: &CapabilityPolicy,
) -> Result<CapabilitySet, CapabilityNegotiationError> {
    // §24 "Validate".
    registry.validate(local)?;
    registry.validate(remote)?;

    // §24 "Intersect" (§19's rules, applied per capability id).
    let mut ids: BTreeSet<CapabilityId> = BTreeSet::new();
    ids.extend(local.iter().map(|d| d.id));
    ids.extend(remote.iter().map(|d| d.id));

    let mut negotiated = CapabilitySet::new();
    for id in ids {
        match (local.get(&id), remote.get(&id)) {
            (Some(l), Some(r)) => {
                if let Some(descriptor) = intersect_descriptor(l, r)? {
                    negotiated
                        .insert(descriptor)
                        .map_err(|_| CapabilityNegotiationError::LimitExceeded)?;
                }
            }
            (Some(only), None) | (None, Some(only)) => {
                // §8: present on only one side. If that side didn't
                // require it, this is the ordinary "not mutually
                // supported, drop it" case (§19's intersection rule).
                // If it *did* require it, the peer that lacks it can
                // never satisfy that requirement — negotiation fails
                // (§8: "Unknown required capability: negotiation
                // failure"). Checking both directions, not just
                // "local requires something remote lacks", is what
                // makes the result the same regardless of which side
                // calls `negotiate` (§72 determinism).
                if only.requirement == CapabilityRequirement::Required {
                    return Err(CapabilityNegotiationError::MissingRequired(id));
                }
            }
            (None, None) => unreachable!("id came from the union of both sets' own ids"),
        }
    }

    // §24 "PolicyFilter" + "RequiredCheck": §20-21, a hard/user/app
    // disabled capability is removed even after successful
    // intersection; §21 "Hard policy cannot be overridden by peer
    // advertisement" is why this runs after intersection rather than
    // only filtering the inputs beforehand (the filter must apply to
    // the negotiated *result*, so it can't be bypassed by combining
    // two sides that individually looked fine).
    let mut filtered = CapabilitySet::new();
    for descriptor in negotiated.iter() {
        if policy.is_disabled(&descriptor.id) {
            if descriptor.requirement == CapabilityRequirement::Required {
                return Err(CapabilityNegotiationError::SecurityPolicy(descriptor.id));
            }
            continue;
        }
        filtered
            .insert(descriptor.clone())
            .map_err(|_| CapabilityNegotiationError::LimitExceeded)?;
    }

    Ok(filtered)
}

/// Combines one capability advertised by both peers into its
/// negotiated form, or `Ok(None)` if the two advertisements are
/// incompatible but neither side required it (§19's per-field
/// combining rules; requirement is escalated to `Required` if either
/// side required it, so a downstream per-operation check (§54-55)
/// still sees it as required).
fn intersect_descriptor(
    local: &CapabilityDescriptor,
    remote: &CapabilityDescriptor,
) -> Result<Option<CapabilityDescriptor>, CapabilityNegotiationError> {
    let required = local.requirement == CapabilityRequirement::Required
        || remote.requirement == CapabilityRequirement::Required;

    let Some(version) = local.version.negotiate(remote.version) else {
        return if required {
            Err(CapabilityNegotiationError::VersionMismatch(local.id))
        } else {
            Ok(None)
        };
    };

    let Some(parameters) = combine_parameters(&local.parameters, &remote.parameters) else {
        return if required {
            Err(CapabilityNegotiationError::Malformed)
        } else {
            Ok(None)
        };
    };

    let requirement = if required {
        CapabilityRequirement::Required
    } else {
        CapabilityRequirement::Optional
    };

    Ok(Some(CapabilityDescriptor::new(
        local.id,
        version,
        requirement,
        parameters,
    )))
}

/// §19's three named combining rules — boolean/bitset intersection,
/// range overlap, and `min()` for maximum limits — applied per
/// [`CapabilityParameters`] variant. `Bytes`/`None` have no combining
/// rule stated anywhere in the spec (they're not booleans, ranges, or
/// limits); treating them as "must match exactly" is the conservative
/// reading consistent with §19's closing line, "Do not always use
/// simple intersection" — for a parameter shape the spec gives no
/// weakening rule for, equality is the only safe interpretation, not
/// a arbitrary pick.
fn combine_parameters(
    local: &CapabilityParameters,
    remote: &CapabilityParameters,
) -> Option<CapabilityParameters> {
    match (local, remote) {
        (CapabilityParameters::None, CapabilityParameters::None) => {
            Some(CapabilityParameters::None)
        }
        (CapabilityParameters::U32(a), CapabilityParameters::U32(b)) => {
            Some(CapabilityParameters::U32(*a.min(b)))
        }
        (CapabilityParameters::U64(a), CapabilityParameters::U64(b)) => {
            Some(CapabilityParameters::U64(*a.min(b)))
        }
        (
            CapabilityParameters::RangeU32 {
                min: a_min,
                max: a_max,
            },
            CapabilityParameters::RangeU32 {
                min: b_min,
                max: b_max,
            },
        ) => {
            let min = *a_min.max(b_min);
            let max = *a_max.min(b_max);
            (min <= max).then_some(CapabilityParameters::RangeU32 { min, max })
        }
        (CapabilityParameters::BitSet(a), CapabilityParameters::BitSet(b)) => {
            Some(CapabilityParameters::BitSet(a.intersect(*b)))
        }
        (CapabilityParameters::Bytes(a), CapabilityParameters::Bytes(b)) => {
            (a == b).then(|| CapabilityParameters::Bytes(a.clone()))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::CapabilityBits;
    use crate::id::CapabilityNamespace;
    use crate::registry::{CapabilityDefinition, ParameterSchema, SecurityClass};
    use crate::version::CapabilityVersion;

    fn cap(code: u32) -> CapabilityId {
        CapabilityId::new(CapabilityNamespace::Files, code)
    }

    fn desc(
        id: CapabilityId,
        version: CapabilityVersion,
        requirement: CapabilityRequirement,
        parameters: CapabilityParameters,
    ) -> CapabilityDescriptor {
        CapabilityDescriptor::new(id, version, requirement, parameters)
    }

    #[test]
    fn optional_capability_only_on_one_side_is_dropped_not_failed() {
        let mut local = CapabilitySet::new();
        local
            .insert(desc(
                cap(1),
                CapabilityVersion::new(1, 0),
                CapabilityRequirement::Optional,
                CapabilityParameters::None,
            ))
            .unwrap();
        let remote = CapabilitySet::new();

        let result = negotiate(
            &local,
            &remote,
            &CapabilityRegistry::new(),
            &CapabilityPolicy::new(),
        )
        .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn required_capability_missing_on_other_side_fails() {
        let mut local = CapabilitySet::new();
        local
            .insert(desc(
                cap(1),
                CapabilityVersion::new(1, 0),
                CapabilityRequirement::Required,
                CapabilityParameters::None,
            ))
            .unwrap();
        let remote = CapabilitySet::new();

        let err = negotiate(
            &local,
            &remote,
            &CapabilityRegistry::new(),
            &CapabilityPolicy::new(),
        )
        .unwrap_err();
        assert_eq!(err, CapabilityNegotiationError::MissingRequired(cap(1)));
    }

    #[test]
    fn max_limit_parameter_takes_the_minimum() {
        // §36's own worked example: sender max chunk 4 MiB, receiver
        // max chunk 1 MiB → effective 1 MiB.
        let mut local = CapabilitySet::new();
        local
            .insert(desc(
                cap(2),
                CapabilityVersion::new(1, 0),
                CapabilityRequirement::Optional,
                CapabilityParameters::U32(4 * 1024 * 1024),
            ))
            .unwrap();
        let mut remote = CapabilitySet::new();
        remote
            .insert(desc(
                cap(2),
                CapabilityVersion::new(1, 0),
                CapabilityRequirement::Optional,
                CapabilityParameters::U32(1024 * 1024),
            ))
            .unwrap();

        let result = negotiate(
            &local,
            &remote,
            &CapabilityRegistry::new(),
            &CapabilityPolicy::new(),
        )
        .unwrap();
        assert_eq!(
            result.get(&cap(2)).unwrap().parameters,
            CapabilityParameters::U32(1024 * 1024)
        );
    }

    #[test]
    fn bitset_negotiates_bitwise_and() {
        let mut local = CapabilitySet::new();
        local
            .insert(desc(
                cap(3),
                CapabilityVersion::new(1, 0),
                CapabilityRequirement::Optional,
                CapabilityParameters::BitSet(CapabilityBits::EMPTY.with_bit(0).with_bit(1)),
            ))
            .unwrap();
        let mut remote = CapabilitySet::new();
        remote
            .insert(desc(
                cap(3),
                CapabilityVersion::new(1, 0),
                CapabilityRequirement::Optional,
                CapabilityParameters::BitSet(CapabilityBits::EMPTY.with_bit(1).with_bit(2)),
            ))
            .unwrap();

        let result = negotiate(
            &local,
            &remote,
            &CapabilityRegistry::new(),
            &CapabilityPolicy::new(),
        )
        .unwrap();
        let CapabilityParameters::BitSet(bits) = result.get(&cap(3)).unwrap().parameters else {
            panic!("expected BitSet parameters");
        };
        assert!(!bits.has_bit(0));
        assert!(bits.has_bit(1));
        assert!(!bits.has_bit(2));
    }

    #[test]
    fn hard_policy_removes_capability_even_when_both_peers_want_it() {
        // §21: "Hard policy cannot be overridden by peer advertisement."
        let mut local = CapabilitySet::new();
        local
            .insert(desc(
                cap(4),
                CapabilityVersion::new(1, 0),
                CapabilityRequirement::Optional,
                CapabilityParameters::None,
            ))
            .unwrap();
        let mut remote = CapabilitySet::new();
        remote
            .insert(desc(
                cap(4),
                CapabilityVersion::new(1, 0),
                CapabilityRequirement::Optional,
                CapabilityParameters::None,
            ))
            .unwrap();

        let mut policy = CapabilityPolicy::new();
        policy.disable_hard(cap(4));

        let result = negotiate(&local, &remote, &CapabilityRegistry::new(), &policy).unwrap();
        assert!(!result.contains(&cap(4)));
    }

    #[test]
    fn policy_disabling_a_required_capability_is_a_hard_failure() {
        let mut local = CapabilitySet::new();
        local
            .insert(desc(
                cap(5),
                CapabilityVersion::new(1, 0),
                CapabilityRequirement::Required,
                CapabilityParameters::None,
            ))
            .unwrap();
        let mut remote = CapabilitySet::new();
        remote
            .insert(desc(
                cap(5),
                CapabilityVersion::new(1, 0),
                CapabilityRequirement::Required,
                CapabilityParameters::None,
            ))
            .unwrap();

        // registry.validate() (§68) rejects a Required capability it
        // doesn't recognize, independent of policy — so this test
        // must register cap(5) as a known capability first, or it
        // would (correctly) fail at the Validate step rather than
        // exercising the PolicyFilter step this test is actually for.
        let mut registry = CapabilityRegistry::new();
        registry.register(CapabilityDefinition {
            id: cap(5),
            name: "files.test5",
            max_version: CapabilityVersion::new(1, 0),
            parameter_schema: ParameterSchema::None,
            security_class: SecurityClass::Functional,
        });

        let mut policy = CapabilityPolicy::new();
        policy.disable_hard(cap(5));

        let err = negotiate(&local, &remote, &registry, &policy).unwrap_err();
        assert_eq!(err, CapabilityNegotiationError::SecurityPolicy(cap(5)));
    }

    #[test]
    fn negotiation_is_symmetric_under_swapped_inputs() {
        // §72 "Negotiation Determinism": given the same local set,
        // remote set, and policy, both peers should derive the same
        // result — which requires `negotiate` to behave symmetrically
        // when the two sides call it with their roles swapped.
        let mut a = CapabilitySet::new();
        a.insert(desc(
            cap(1),
            CapabilityVersion::new(1, 5),
            CapabilityRequirement::Required,
            CapabilityParameters::U32(10),
        ))
        .unwrap();
        a.insert(desc(
            cap(6),
            CapabilityVersion::new(1, 0),
            CapabilityRequirement::Optional,
            CapabilityParameters::None,
        ))
        .unwrap();

        let mut b = CapabilitySet::new();
        b.insert(desc(
            cap(1),
            CapabilityVersion::new(1, 2),
            CapabilityRequirement::Optional,
            CapabilityParameters::U32(7),
        ))
        .unwrap();

        // cap(1) is Required on the `a` side, so registry.validate()
        // (§68) needs it registered regardless of which side is
        // passed as `local` — same reasoning as the test above.
        let mut registry = CapabilityRegistry::new();
        registry.register(CapabilityDefinition {
            id: cap(1),
            name: "files.test1",
            max_version: CapabilityVersion::new(1, 5),
            parameter_schema: ParameterSchema::U32,
            security_class: SecurityClass::Functional,
        });
        let policy = CapabilityPolicy::new();

        let forward = negotiate(&a, &b, &registry, &policy).unwrap();
        let backward = negotiate(&b, &a, &registry, &policy).unwrap();

        let forward_ids: Vec<_> = forward.iter().map(|d| d.id).collect();
        let backward_ids: Vec<_> = backward.iter().map(|d| d.id).collect();
        assert_eq!(forward_ids, backward_ids);
        assert_eq!(
            forward.get(&cap(1)).unwrap().parameters,
            backward.get(&cap(1)).unwrap().parameters
        );
        assert_eq!(
            forward.get(&cap(1)).unwrap().requirement,
            backward.get(&cap(1)).unwrap().requirement
        );
    }
}
