//! Message content types.
//!
//! Phase 1 only needs text messages (plan.md §124: "Alice ↔ Bob text
//! messaging"). Attachments/voice/calls arrive in later phases, but the
//! enum shape is left open (`content_kind` naming, etc.) so those variants
//! slot in without a breaking rewrite of `MessageContent` consumers.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Validated, size-bounded message text (plan.md §61 decode limits, §121
/// validated types).
///
/// Constructing a `MessageText` guarantees every downstream consumer that
/// the text is non-empty and within the wire size limit — they never need
/// to re-check it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageText(String);

/// Hard cap per plan.md §61 ("text message: 64 KiB"). Kept in bytes since
/// that's what actually crosses the wire.
pub const MAX_TEXT_BYTES: usize = 64 * 1024;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TextParseError {
    #[error("message text cannot be empty")]
    Empty,
    #[error("message text is {0} bytes, exceeds the {MAX_TEXT_BYTES}-byte limit")]
    TooLong(usize),
}

impl MessageText {
    pub fn parse(value: String) -> Result<Self, TextParseError> {
        if value.is_empty() {
            return Err(TextParseError::Empty);
        }
        if value.len() > MAX_TEXT_BYTES {
            return Err(TextParseError::TooLong(value.len()));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageContent {
    Text(MessageText),
    Attachment(crate::AttachmentReference),
    // Voice(VoiceMessage)           — later phase
    // Reaction(Reaction)            — later phase
    // System(SystemMessage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_text() {
        assert_eq!(
            MessageText::parse(String::new()),
            Err(TextParseError::Empty)
        );
    }

    #[test]
    fn rejects_oversized_text() {
        let too_big = "a".repeat(MAX_TEXT_BYTES + 1);
        assert_eq!(
            MessageText::parse(too_big),
            Err(TextParseError::TooLong(MAX_TEXT_BYTES + 1))
        );
    }

    #[test]
    fn accepts_ordinary_text() {
        let text = MessageText::parse("hey".to_string()).unwrap();
        assert_eq!(text.as_str(), "hey");
    }
}
