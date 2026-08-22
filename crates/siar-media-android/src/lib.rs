//! Android hardware codec access — real, not a stub, as of this
//! revision. The design that unblocked it: earlier I held this crate
//! back because "Rust drives JNI calls into MediaCodec by method-
//! signature string" is genuinely high-risk to write without a
//! compiler or device (`env.call_method(obj, "configure",
//! "(Landroid/media/MediaFormat;...)V", ...)` — get one character of
//! that signature string wrong and it's a runtime `NoSuchMethodError`
//! on a real phone, not a build error). The fix wasn't writing that
//! code more carefully — it was inverting the direction:
//!
//! - `platform/android/media/`'s Kotlin (`HardwareVideoEncoder.kt`,
//!   `HardwareVideoDecoder.kt`, `MediaCodecCapabilities.kt`) drives
//!   `android.media.MediaCodec`/`MediaCodecList` directly. That's
//!   ordinary, statically-typed Kotlin against the Android SDK —
//!   compiler-checked on that side, the same confidence level as this
//!   session's `rav1e`/`opus`/`cpal` work.
//! - `jni_bridge.rs` (this crate, Android-only) is the JNI boundary,
//!   and Kotlin only ever *calls into* it (`external fun` matched
//!   against `#[unsafe(no_mangle)] extern "system" fn
//!   Java_com_siar_media_NativeMediaBridge_...`) — never the reverse.
//!   A symbol-name mismatch there is an `UnsatisfiedLinkError` at
//!   `System.loadLibrary` time, immediately and loudly, not a stringly-
//!   typed runtime surprise mid-call.
//!
//! Wiring `MediaSession`'s `output_queue` into `siar-media-core`'s
//! `VideoEncoder`/`VideoDecoder` trait consumers, and feeding
//! `push_input` from a real camera/network source, used to be flagged
//! here as unimplemented "`calls`-crate-level orchestration." That
//! wiring now lives in `siar-calls::android`, which consumes this
//! crate's `push_input`/`output_ready_notifier`/`drain_output` — this
//! crate's own job stays exactly what it was: "Kotlin and Rust can
//! safely hand codec bytes back and forth," nothing more.
//!
//! Architecture doc §43's device-matrix testing requirement doesn't go
//! away just because the JNI-direction risk did, or because the
//! orchestration layer is now written — this still needs real
//! Snapdragon/Dimensity/Exynos/Tensor hardware before it's trustworthy
//! in production, same caveat as always for anything touching vendor
//! MediaCodec implementations.

#[cfg(target_os = "android")]
mod jni_bridge;
#[cfg(target_os = "android")]
pub use jni_bridge::{drain_output, output_ready_notifier, push_input, MediaSession, SessionOutput};
