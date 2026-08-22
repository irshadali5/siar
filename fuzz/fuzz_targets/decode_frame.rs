//! plan.md §100: "Remote network bytes must never be assumed valid."
//! This is the highest-value fuzz target in the whole workspace — every
//! byte `decode_frame` sees originated from an untrusted peer, and it's
//! the first thing that runs on them, before any identity/signature
//! check even gets a chance to reject a malicious sender.
//!
//! Run with: cargo fuzz run decode_frame

#![no_main]
use libfuzzer_sys::fuzz_target;
use siar_protocol::decode_frame;

fuzz_target!(|data: &[u8]| {
    // The only property under test: this must never panic or hang,
    // regardless of input. A parse failure (`Err`) is a correct,
    // expected outcome for malformed bytes — a panic is the bug.
    let _ = decode_frame(data);
});
