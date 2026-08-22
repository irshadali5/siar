package com.siar.media

/**
 * The entire Rust boundary for Android media, and deliberately the
 * *only* file in this module that touches `external fun`.
 *
 * Direction matters here: every function below is Kotlin calling into
 * Rust (`#[no_mangle] extern "system" fn Java_com_siar_media_..._` on
 * the Rust side — see `siar-media-android/src/jni_bridge.rs`). Nothing
 * in this codebase has Rust call *into* Kotlin by method name — that
 * direction requires JNI method-signature strings
 * (`env.call_method(obj, "name", "(Ljava/lang/String;)V", ...)`) that
 * are checked at runtime, not compile time, and a wrong one is a
 * crash on a real device, not a build error. Kotlin calling `external
 * fun` is checked by the Kotlin compiler on this side and by the
 * dynamic linker (symbol name mismatch = `UnsatisfiedLinkError` at
 * `System.loadLibrary` time, immediately, not buried in a call path)
 * on the Rust side. That asymmetry is why the protocol below is
 * "Kotlin pulls work from Rust and pushes results back to Rust,"
 * never "Rust tells Kotlin to do something."
 *
 * `handle` in every function is an opaque session identifier — the
 * `i64` cast of a `Box::into_raw` pointer on the Rust side (see
 * `jni_bridge.rs`'s `SessionHandle`). Kotlin never interprets it,
 * only passes it back unchanged so Rust knows which encode/decode
 * session a call belongs to.
 */
internal object NativeMediaBridge {
    init {
        System.loadLibrary("siar_media_android")
    }

    /**
     * Allocates a Rust-side session for one encoder/decoder instance
     * and returns its `handle`. `kind` is 0=encode, 1=decode. Every
     * other function above takes the `handle` this returns — call
     * [destroySession] exactly once when the corresponding
     * [HardwareVideoEncoder]/[HardwareVideoDecoder] is torn down, or
     * the Rust-side session (and its queued frames) leaks.
     */
    external fun createSession(kind: Int, codec: Int, width: Int, height: Int): Long

    external fun destroySession(handle: Long)

    /**
     * Pulls the next raw YUV 4:2:0 frame to encode, or `null` if none
     * is queued yet (the encoder's input-available callback should
     * just skip this cycle, not treat `null` as an error or an
     * end-of-stream signal). Layout: `width*height` Y bytes, then
     * `ceil(width/2)*ceil(height/2)` U bytes, then the same size of V
     * bytes, concatenated — matching `siar_media_core::RawVideoFrame`'s
     * own documented layout exactly, so no reshaping happens on either
     * side of this call.
     */
    external fun nextRawFrame(handle: Long): ByteArray?

    /** Presentation timestamp, in microseconds, for the frame `nextRawFrame` most recently returned for this handle. */
    external fun nextRawFrameTimestampUs(handle: Long): Long

    /** Hands one encoded access unit back to Rust. `codec` is 0=AV1, 1=H264, 2=H265 — see `CodecId` in both this file and `jni_bridge.rs`; they must stay in sync. */
    external fun onEncodedFrame(handle: Long, codec: Int, data: ByteArray, isKeyFrame: Boolean, presentationTimeUs: Long)

    /** Pulls the next encoded access unit to decode, or `null` if none is queued yet. */
    external fun nextEncodedFrame(handle: Long): ByteArray?

    external fun nextEncodedFrameTimestampUs(handle: Long): Long

    /**
     * Hands one decoded picture back to Rust — `y`/`u`/`v` in the same
     * tightly-packed 4:2:0 layout `nextRawFrame` documents.
     */
    external fun onDecodedFrame(handle: Long, y: ByteArray, u: ByteArray, v: ByteArray, width: Int, height: Int, presentationTimeUs: Long)

    /**
     * Reports what this device's `MediaCodecList` actually supports
     * (architecture doc §3/§4) — `payload` is the fixed binary layout
     * documented in `CapabilityWireFormat.kt` and mirrored exactly in
     * `jni_bridge.rs`'s `decode_capabilities`. A hand-rolled fixed
     * format instead of JSON/protobuf on purpose: it's a handful of
     * fixed-width fields, and matching two independently-written
     * codecs by hand for something this small is lower risk than
     * pulling a JSON parser into the JNI boundary for no real benefit.
     */
    external fun reportCapabilities(handle: Long, payload: ByteArray)

    /** Signals a codec/backend failure Kotlin's MediaCodec calls hit (configure/start/queue exceptions) — see architecture doc §45's "renegotiation, don't just terminate the call" handling, driven from the Rust side once it receives this. */
    external fun onCodecError(handle: Long, message: String)
}
