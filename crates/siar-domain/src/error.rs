//! Domain-level errors (plan.md §88: typed errors, no `anyhow` in the
//! domain layer — `anyhow` is fine at the CLI/bootstrap boundary only).

use crate::ConversationId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("conversation {0} not found")]
    UnknownConversation(ConversationId),
    #[error("illegal delivery-state transition")]
    IllegalTransition,
}
