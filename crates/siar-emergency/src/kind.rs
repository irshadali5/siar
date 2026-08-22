//! Structured emergency message kinds — next.md §45: "Avoid
//! unstructured text only. Structured data can travel in extremely
//! small packets."

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmergencyMessageKind {
    Sos,
    Safe,
    NeedMedicalHelp,
    NeedFood,
    NeedWater,
    NeedShelter,
    Trapped,
    MissingPerson,
    HazardAlert,
    EvacuationNotice,
    LocationBeacon,
}
