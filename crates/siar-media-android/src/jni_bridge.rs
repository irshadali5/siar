//! The JNI boundary — verified against `jni 0.21.1`'s actual source
//! (`env.convert_byte_array`, `env.byte_array_from_slice`,
//! `env.new_byte_array`, `env.get_string`, exported-function signature
//! shape `extern "system" fn Java_..._<method><'local>(env: JNIEnv<'local>,
//! class: JClass<'local>, ...)`). Pinned to 0.21 rather than the newer
//! 0.22 deliberately — 0.22 redesigned the entry-point API
//! (`EnvUnowned`/`with_env`), and this file doesn't need whatever that
//! redesign is for; 0.21's `JNIEnv` parameter is the long-established,
//! widely-documented convention, which matters more here than being on
//! the latest version.
//!
//! Direction, restated from `NativeMediaBridge.kt`'s doc comment: every
//! `#[no_mangle] extern "system" fn` below is called *by*
//! Kotlin. Nothing in this file calls back into the JVM by method name
//! — the only two touches of JNI types are `JNIEnv`/`JByteArray`
//! themselves (unavoidable to marshal bytes across the boundary), not
//! `env.call_method(...)`.
//!
//! JNI naming: package `com.siar.media`, class `NativeMediaBridge` →
//! symbol prefix `Java_com_siar_media_NativeMediaBridge_`. None of the
//! Kotlin method names contain underscores, so none of JNI's `_1`
//! escaping applies — every symbol below is a direct, unescaped
//! `Java_com_siar_media_NativeMediaBridge_<methodName>`.
//!
//! `handle` (`jlong` on the Kotlin side) is
//! `Box::into_raw(Box::new(Mutex::new(session))) as jlong` — a raw
//! pointer smuggled through a 64-bit integer because JNI has no notion
//! of a Rust type. `session_from_handle` is the one place that pointer
//! gets reconstituted; it's `unsafe` for the reason every raw-pointer
//! deref is. This defends against forgetting to free a session (by
//! making the one alloc/one free pairing explicit in
//! `create_session`/`destroy_session`) but, same as any JNI boundary,
//! does not defend against a forged or reused handle — this is our own
//! app's own Kotlin calling in, not untrusted external input.
//!
//! The `nextRawFrame`/`nextRawFrameTimestampUs` split (and its decode
//! mirror) exists because a `ByteArray?`-returning `external fun` can't
//! also return a `Long` alongside it — Kotlin calls `nextRawFrame`
//! first, and if it got a non-null array back, immediately calls
//! `nextRawFrameTimestampUs` for that same frame's timestamp. `pop_input`
//! below stashes the timestamp of whatever it just popped into
//! `last_input_timestamp_micros` specifically so the second call has
//! something to read without re-dequeuing.

use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jboolean, jbyteArray, jint, jlong};
use jni::JNIEnv;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

use siar_media_core::{RawVideoFrame, Resolution, VideoCodec};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SessionKind {
    Encode,
    Decode,
}

struct QueuedBytes {
    data: Vec<u8>,
    timestamp_micros: i64,
}

pub enum SessionOutput {
    Encoded { codec: VideoCodec, data: Vec<u8>, is_keyframe: bool, timestamp_micros: i64 },
    Decoded { frame: RawVideoFrame, timestamp_micros: i64 },
}

/// Everything one `handle` refers to. A single `Mutex` covering the
/// whole session (rather than one per field) because every access here
/// is a short, non-blocking dequeue/push — no risk of holding the lock
/// across anything slow, so the simplicity of one lock wins over
/// finer-grained locking that would only matter under real contention
/// this session never sees (one Kotlin worker thread per session, one
/// Rust producer/consumer side).
pub struct MediaSession {
    #[allow(dead_code)] // read by future orchestration code that branches encode vs decode handling
    kind: SessionKind,
    #[allow(dead_code)] // read by future orchestration code, not by this file
    codec: VideoCodec,
    #[allow(dead_code)]
    resolution: Resolution,
    /// Encode sessions: raw frames waiting to be encoded. Decode
    /// sessions: encoded packets waiting to be decoded. Only one
    /// direction is ever populated, per `kind`.
    input_queue: VecDeque<QueuedBytes>,
    last_input_timestamp_micros: i64,
    /// Filled by `on_encoded_frame`/`on_decoded_frame`. Draining this
    /// from the Rust side (feeding it into `siar-media-core`'s
    /// `VideoEncoder`/`VideoDecoder`-shaped consumers, and ultimately
    /// the network) is `calls`-crate-level orchestration this file
    /// doesn't implement — same scope boundary as the rest of
    /// `siar-media-android`, see its top-level doc comment.
    output_queue: VecDeque<SessionOutput>,
    /// Signaled by `onEncodedFrame`/`onDecodedFrame` whenever they push
    /// to `output_queue`, so an async consumer in `siar-calls::android`
    /// can `.notified().await` instead of polling this queue on a fixed
    /// interval. `Arc` so a consumer can hold its own clone independent
    /// of this session's `Mutex` guard — `notify_one` only ever needs
    /// `&Notify`, never ownership of the guard, so calling it from
    /// inside an already-held lock (as the two callbacks below do) has
    /// no borrow conflict. That's *not* true of `Condvar`, which needs
    /// to consume the guard it's paired with and so can't usefully live
    /// as a field *inside* the data that guard protects — `Notify` was
    /// chosen specifically to sidestep that.
    output_ready: Arc<Notify>,
    last_error: Option<String>,
}

fn codec_from_wire(value: jint) -> Option<VideoCodec> {
    match value {
        0 => Some(VideoCodec::Av1),
        1 => Some(VideoCodec::H264),
        2 => Some(VideoCodec::H265),
        _ => None,
    }
}

fn codec_to_wire(codec: VideoCodec) -> jint {
    match codec {
        VideoCodec::Av1 => 0,
        VideoCodec::H264 => 1,
        VideoCodec::H265 => 2,
    }
}

/// SAFETY (applies to every call site below): `handle` must be a value
/// `create_session` returned that hasn't since been passed to
/// `destroy_session`. Every call site in this file gets `handle`
/// straight from a Kotlin parameter, which only ever forwards a value
/// this module itself handed out — see this module's top doc comment.
unsafe fn session_from_handle<'a>(handle: jlong) -> Option<&'a Mutex<MediaSession>> {
    if handle == 0 {
        return None;
    }
    Some(unsafe { &*(handle as *const Mutex<MediaSession>) })
}

#[no_mangle]
pub extern "system" fn Java_com_siar_media_NativeMediaBridge_createSession<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    kind: jint,
    codec: jint,
    width: jint,
    height: jint,
) -> jlong {
    let Some(codec) = codec_from_wire(codec) else { return 0 };
    if width <= 0 || height <= 0 {
        return 0;
    }
    let kind = if kind == 0 { SessionKind::Encode } else { SessionKind::Decode };

    let session = MediaSession {
        kind,
        codec,
        resolution: Resolution::new(width as u32, height as u32),
        input_queue: VecDeque::new(),
        last_input_timestamp_micros: 0,
        output_queue: VecDeque::new(),
        output_ready: Arc::new(Notify::new()),
        last_error: None,
    };

    // The only `Box::into_raw` for a `MediaSession` — `destroy_session`
    // is its only `Box::from_raw`, and it's written to be the sole
    // place this pointer is ever reclaimed.
    Box::into_raw(Box::new(Mutex::new(session))) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_siar_media_NativeMediaBridge_destroySession<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    if handle == 0 {
        return;
    }
    // SAFETY: reclaims exactly the box `create_session` allocated for
    // this handle. Correctness here depends on Kotlin calling this
    // exactly once per `createSession` call — a double-`destroySession`
    // would be a use-after-free, same as any Rust `Box::from_raw`
    // misuse; that's a bug to catch in our own Kotlin lifecycle code
    // (pair every `HardwareVideoEncoder`/`Decoder`'s `stop()` with
    // exactly one `destroySession`), not something this function can
    // detect after the fact.
    unsafe {
        drop(Box::from_raw(handle as *mut Mutex<MediaSession>));
    }
}

/// Queues a raw frame for an encode session, or an encoded packet for a
/// decode session, to be pulled by `nextRawFrame`/`nextEncodedFrame`.
/// Not exposed to Kotlin — this is what Rust-side orchestration code
/// (feeding camera/network input in) calls; it's `pub` for that future
/// caller, not part of the JNI surface itself.
pub fn push_input(session: &Mutex<MediaSession>, data: Vec<u8>, timestamp_micros: i64) {
    let mut session = session.lock().expect("MediaSession lock poisoned");
    session.input_queue.push_back(QueuedBytes { data, timestamp_micros });
}

/// A clone of this session's ready-notifier, for an async consumer (see
/// `siar-calls::android`) to await on without holding the session's
/// lock or polling `output_queue` on a fixed interval. Not part of the
/// JNI surface — `pub` for that future in-process Rust caller only,
/// same visibility rationale as `push_input` above.
pub fn output_ready_notifier(session: &Mutex<MediaSession>) -> Arc<Notify> {
    Arc::clone(&session.lock().expect("MediaSession lock poisoned").output_ready)
}

/// Drains everything currently queued in `output_queue`, in arrival
/// order. Pairs with `output_ready_notifier`: a consumer awaits one
/// notification, then drains whatever's there — there may be more than
/// one item per notification, since `notify_one` only guarantees "check
/// again," not "exactly one new item since last time."
pub fn drain_output(session: &Mutex<MediaSession>) -> Vec<SessionOutput> {
    session.lock().expect("MediaSession lock poisoned").output_queue.drain(..).collect()
}

fn pop_input(handle: jlong) -> Option<Vec<u8>> {
    // SAFETY: see `session_from_handle`.
    let session = unsafe { session_from_handle(handle) }?;
    let mut session = session.lock().expect("MediaSession lock poisoned");
    let queued = session.input_queue.pop_front()?;
    session.last_input_timestamp_micros = queued.timestamp_micros;
    Some(queued.data)
}

fn last_input_timestamp(handle: jlong) -> jlong {
    // SAFETY: see `session_from_handle`.
    let Some(session) = (unsafe { session_from_handle(handle) }) else { return 0 };
    session.lock().expect("MediaSession lock poisoned").last_input_timestamp_micros
}

fn bytes_to_jbyte_array(env: &mut JNIEnv, data: &[u8]) -> jbyteArray {
    match env.byte_array_from_slice(data) {
        Ok(array) => array.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_siar_media_NativeMediaBridge_nextRawFrame<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jbyteArray {
    match pop_input(handle) {
        Some(data) => bytes_to_jbyte_array(&mut env, &data),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_siar_media_NativeMediaBridge_nextRawFrameTimestampUs<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jlong {
    last_input_timestamp(handle)
}

#[no_mangle]
pub extern "system" fn Java_com_siar_media_NativeMediaBridge_nextEncodedFrame<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jbyteArray {
    match pop_input(handle) {
        Some(data) => bytes_to_jbyte_array(&mut env, &data),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_siar_media_NativeMediaBridge_nextEncodedFrameTimestampUs<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jlong {
    last_input_timestamp(handle)
}

#[no_mangle]
pub extern "system" fn Java_com_siar_media_NativeMediaBridge_onEncodedFrame<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    codec: jint,
    data: JByteArray<'local>,
    is_key_frame: jboolean,
    presentation_time_us: jlong,
) {
    // SAFETY: see `session_from_handle`.
    let Some(session) = (unsafe { session_from_handle(handle) }) else { return };
    let Some(codec) = codec_from_wire(codec) else { return };
    let Ok(bytes) = env.convert_byte_array(&data) else { return };

    let mut guard = session.lock().expect("MediaSession lock poisoned");
    guard.output_queue.push_back(SessionOutput::Encoded {
        codec,
        data: bytes,
        is_keyframe: is_key_frame != 0,
        timestamp_micros: presentation_time_us,
    });
    guard.output_ready.notify_one();
}

#[no_mangle]
pub extern "system" fn Java_com_siar_media_NativeMediaBridge_onDecodedFrame<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    y: JByteArray<'local>,
    u: JByteArray<'local>,
    v: JByteArray<'local>,
    width: jint,
    height: jint,
    presentation_time_us: jlong,
) {
    // SAFETY: see `session_from_handle`.
    let Some(session) = (unsafe { session_from_handle(handle) }) else { return };
    if width <= 0 || height <= 0 {
        return;
    }
    let (Ok(y_plane), Ok(u_plane), Ok(v_plane)) =
        (env.convert_byte_array(&y), env.convert_byte_array(&u), env.convert_byte_array(&v))
    else {
        return;
    };

    let resolution = Resolution::new(width as u32, height as u32);
    let frame = RawVideoFrame { resolution, y_plane, u_plane, v_plane, timestamp_micros: presentation_time_us as u64 };
    // `is_well_formed` re-checks the same plane-size arithmetic
    // `HardwareVideoDecoder.extractYuv420` (Kotlin side) used to build
    // these arrays — plan.md §68's "treat all remote input as hostile"
    // extends here to "treat all cross-language input as hostile,"
    // even from our own Kotlin: a MediaCodec vendor quirk producing an
    // unexpected buffer size should not silently corrupt whatever
    // consumes this frame downstream.
    if !frame.is_well_formed() {
        session.lock().expect("MediaSession lock poisoned").last_error =
            Some(format!("onDecodedFrame: plane sizes did not match {width}x{height}"));
        return;
    }

    let mut guard = session.lock().expect("MediaSession lock poisoned");
    guard.output_queue.push_back(SessionOutput::Decoded { frame, timestamp_micros: presentation_time_us });
    guard.output_ready.notify_one();
}

#[no_mangle]
pub extern "system" fn Java_com_siar_media_NativeMediaBridge_onCodecError<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    message: JString<'local>,
) {
    // SAFETY: see `session_from_handle`.
    let Some(session) = (unsafe { session_from_handle(handle) }) else { return };
    let message: String = env.get_string(&message).map(String::from).unwrap_or_else(|_| "<unreadable error message>".to_string());
    session.lock().expect("MediaSession lock poisoned").last_error = Some(message);
}

#[no_mangle]
pub extern "system" fn Java_com_siar_media_NativeMediaBridge_reportCapabilities<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    payload: JByteArray<'local>,
) {
    // SAFETY: see `session_from_handle`.
    let Some(session) = (unsafe { session_from_handle(handle) }) else { return };
    let Ok(bytes) = env.convert_byte_array(&payload) else { return };

    match decode_capabilities(&bytes) {
        Ok(_capabilities) => {
            // Handing `_capabilities` to `siar_media_core::negotiate_call`
            // is `calls`-crate-level orchestration, same scope line as
            // `output_queue`'s doc comment above — not implemented in
            // this file. What's real here: the wire format is decoded
            // correctly and safely (bounds-checked, no panics on a
            // malformed payload) or a parse error is recorded, either
            // way.
        }
        Err(e) => {
            session.lock().expect("MediaSession lock poisoned").last_error = Some(format!("reportCapabilities: {e}"));
        }
    }
}

/// Mirrors `CapabilityWireFormat.encode` in `CapabilityWireFormat.kt`
/// byte for byte — both sides were written together in this session
/// specifically so they'd match; changing one without the other breaks
/// this at the first parsed byte, not gracefully.
fn decode_capabilities(bytes: &[u8]) -> Result<Vec<siar_media_core::VideoCodecCapability>, String> {
    use siar_media_core::{BitrateRange, CodecImplementation, FrameRateRange};

    let mut cursor = bytes;
    let take = |cursor: &mut &[u8], n: usize| -> Result<Vec<u8>, String> {
        if cursor.len() < n {
            return Err(format!("payload truncated: needed {n} more bytes, had {}", cursor.len()));
        }
        let (head, tail) = cursor.split_at(n);
        *cursor = tail;
        Ok(head.to_vec())
    };
    let take_u32_le = |cursor: &mut &[u8]| -> Result<u32, String> {
        let bytes = take(cursor, 4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    };

    let entry_count = *take(&mut cursor, 1)?.first().ok_or("empty payload")?;
    let mut entries = Vec::with_capacity(entry_count as usize);

    for _ in 0..entry_count {
        let codec_byte = take(&mut cursor, 1)?[0];
        let codec = codec_from_wire(codec_byte as jint).ok_or_else(|| format!("unknown codec id {codec_byte}"))?;
        let hardware = take(&mut cursor, 1)?[0] != 0;
        let can_encode = take(&mut cursor, 1)?[0] != 0;
        let can_decode = take(&mut cursor, 1)?[0] != 0;
        let max_width = take_u32_le(&mut cursor)?;
        let max_height = take_u32_le(&mut cursor)?;
        let max_fps = take_u32_le(&mut cursor)?;

        entries.push(siar_media_core::VideoCodecCapability {
            codec,
            implementation: if hardware { CodecImplementation::Hardware } else { CodecImplementation::Software },
            can_encode,
            can_decode,
            max_resolution: Resolution::new(max_width, max_height),
            max_fps: max_fps as u16,
            supported_profiles: vec![],
            // Not carried over the wire — the capability probe on the
            // Kotlin side (`MediaCodecCapabilities.kt`) doesn't query
            // per-codec bitrate/frame-rate *ranges* from
            // `VideoCapabilities`, only the max values above. A real
            // negotiation pass that needs the full range would extend
            // both `CapabilityWireFormat.kt` and this function together
            // — flagged here rather than inventing plausible-looking
            // range bounds that were never actually reported.
            bitrate_range: BitrateRange { min_bps: 0, max_bps: 0 },
            frame_rate_range: FrameRateRange { min_fps: 0, max_fps: max_fps as u16 },
        });
    }

    Ok(entries)
}

// `codec_to_wire` exists for future use by whatever orchestration code
// eventually sends codec selections back down to Kotlin (e.g. a
// `selectedCodec` field on a future `startSession`-style call) — not
// called anywhere in this file yet, since that orchestration doesn't
// exist here. Kept instead of deleted so the wire-format mapping stays
// in exactly one place when that caller is written.
#[allow(dead_code)]
fn _unused_codec_to_wire_reference() -> jint {
    codec_to_wire(VideoCodec::Av1)
}
