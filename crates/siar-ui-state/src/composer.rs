//! `ComposerState` (plan.md §53). Validation happens here, at the
//! boundary — a component binds this to a text input and only ever gets
//! a `MessageText` (or a `TextParseError` it can show inline) out of it,
//! never a raw unvalidated `String` making it further into the app.

use siar_domain::{MessageText, TextParseError};

#[derive(Debug, Default)]
pub struct ComposerState {
    draft: String,
}

impl ComposerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_draft(&mut self, text: String) {
        self.draft = text;
    }

    pub fn draft(&self) -> &str {
        &self.draft
    }

    pub fn is_empty(&self) -> bool {
        self.draft.is_empty()
    }

    /// Validates the current draft and clears it on success — the
    /// "submit" action a Composer's send button drives. Leaves the draft
    /// untouched on failure, so the user's text isn't silently lost after
    /// (say) hitting the 64 KiB limit.
    pub fn try_submit(&mut self) -> Result<MessageText, TextParseError> {
        let text = MessageText::parse(self.draft.clone())?;
        self.draft.clear();
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submitting_valid_text_clears_the_draft() {
        let mut composer = ComposerState::new();
        composer.set_draft("hello".to_string());
        let text = composer.try_submit().unwrap();
        assert_eq!(text.as_str(), "hello");
        assert!(composer.is_empty());
    }

    #[test]
    fn submitting_empty_draft_fails_and_leaves_draft_alone() {
        let mut composer = ComposerState::new();
        let err = composer.try_submit().unwrap_err();
        assert_eq!(err, TextParseError::Empty);
    }

    #[test]
    fn submitting_oversized_draft_fails_without_losing_the_text() {
        let mut composer = ComposerState::new();
        // 70,000 bytes is comfortably past siar-domain's 64 KiB cap
        // (`MessageText::MAX_TEXT_BYTES`) without this test needing that
        // constant re-exported just for a size comparison.
        let too_big = "a".repeat(70_000);
        composer.set_draft(too_big.clone());
        let err = composer.try_submit().unwrap_err();
        assert!(matches!(err, TextParseError::TooLong(_)));
        assert_eq!(composer.draft(), too_big);
    }
}
