//! §16 "Logical Channel Model": "Large file traffic must not
//! head-of-line-block control or messaging traffic."

/// §16's own tree, as an enum. Not detailed further as a struct in the
/// spec text — the actual head-of-line-blocking prevention this
/// section asks for is [`crate::scheduler::FairScheduler`]'s job
/// (separate queues per [`crate::lifecycle::TrafficPriority`] tier,
/// not per channel — see that module's own doc comment for why
/// priority, not channel identity, is the axis this crate schedules
/// on). This enum exists so an extension can declare which conceptual
/// channel its traffic belongs to for diagnostics/logging, without
/// this crate maintaining a second, redundant scheduling axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelKind {
    CoreControl,
    Messaging,
    FileControl,
    FileData,
    Presence,
    Media,
}
