//! §9 "Path Candidate".

use serde::{Deserialize, Serialize};

use crate::metrics::PathMetrics;
use crate::types::{PathCapabilities, PathId, RouteHealth, TransportKind};

/// §9: "Routing works on this generic representation." Deliberately
/// opaque — the actual dial-able address is transport-specific
/// (`iroh::EndpointAddr` for `IrohDirect`/`IrohRelay`, a socket addr for
/// `LocalLan`, a Bluetooth device address for the BT variants) and this
/// crate has no dependency on any transport crate to name a concrete
/// type for all of them (see this crate's own top doc comment: no wire
/// integration). Raw bytes a caller round-trips through whatever the
/// real endpoint type's own serialization already is, not reinterpreted
/// here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransportEndpoint(pub Vec<u8>);

/// §9, plus `underlay` (§74 "Underlay Group") — not one of §9's own
/// listed fields, added this round because §73/§74 need somewhere to
/// put it; see [`crate::diversity`]'s own doc comment for why it's a
/// real newtype rather than the bare unit-struct stub §74's own code
/// block literally shows (`pub struct UnderlayId;`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathCandidate {
    pub path_id: PathId,
    pub transport: TransportKind,
    pub peer: siar_domain::DeviceId,
    pub endpoint: TransportEndpoint,
    pub metrics: PathMetrics,
    pub capabilities: PathCapabilities,
    pub health: RouteHealth,
    pub underlay: Option<crate::diversity::UnderlayId>,
}
