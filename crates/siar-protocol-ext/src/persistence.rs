//! spec §69 "Extension Persistence", §70 "Wire vs Database Migration".
//!
//! §70 needs no code here at all — it's already true of this crate by
//! construction: nothing in `siar-protocol-ext` touches a database
//! schema (that's `siar-storage`'s job), so "wire protocol version"
//! and "local database schema version" are already two entirely
//! separate numbers with no code path tying them together. Documented
//! here rather than silently skipped.

/// spec §69: "Extensions may own durable state via dedicated
/// repositories... Protocol handlers should not write arbitrary
/// product database tables directly." A named, closed declaration of
/// which tables belong to which extension — not a schema (this crate
/// has no database dependency and shouldn't gain one just to name
/// tables), just the ownership boundary spec §69 asks for, checkable
/// instead of only asserted in a code comment somewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct OwnedTable(pub &'static str);

/// spec §69's own messaging example, verbatim table names.
pub const MESSAGING_OWNED_TABLES: [OwnedTable; 3] = [
    OwnedTable("messages"),
    OwnedTable("outbox"),
    OwnedTable("receipts"),
];

/// spec §69's own files example, verbatim table names.
pub const FILES_OWNED_TABLES: [OwnedTable; 2] = [
    OwnedTable("transfer journal"),
    OwnedTable("verified chunks"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_69_messaging_and_files_own_disjoint_tables() {
        // The actual content of §69's isolation rule, made checkable:
        // one extension's owned tables never overlap another's, which
        // is what "should not write arbitrary product database tables
        // directly" is protecting against in practice.
        for messaging_table in MESSAGING_OWNED_TABLES {
            assert!(!FILES_OWNED_TABLES.contains(&messaging_table));
        }
    }
}
