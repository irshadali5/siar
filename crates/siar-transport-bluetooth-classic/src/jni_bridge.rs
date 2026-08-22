//! See this crate's `lib.rs` doc comment for scope and the
//! risk-inversion rationale. JNI naming: package
//! `com.siar.bluetoothclassic`, class `NativeBluetoothClassicBridge` →
//! symbol prefix
//! `Java_com_siar_bluetoothclassic_NativeBluetoothClassicBridge_`, same
//! convention as every other `jni_bridge.rs` in this workspace.
//!
//! `handle` contract is the same one-alloc/one-free pattern used
//! throughout this workspace — see `siar-transport-wifi-direct`'s
//! `jni_bridge.rs` for the full explanation.
//!
//! Deliberately mirrors `siar-transport-ble-android`'s push/pull queue
//! shape (`onFragmentReceived`/`nextReceivedEnvelope`,
//! `queueEnvelopeToSend`/`nextFragmentToSend`) with the method names
//! adjusted for what's actually being pushed here: raw
//! `InputStream.read()` byte chunks in (not pre-parsed fragments — see
//! `framing.rs`'s doc comment on why RFCOMM needs its own incremental
//! decoder rather than BLE's per-write fragment model), complete
//! envelopes out.

use crate::framing::{encode_frame, FrameDecoder};
use jni::objects::{JByteArray, JClass};
use jni::sys::{jbyteArray, jlong};
use jni::JNIEnv;
use std::collections::VecDeque;
use std::sync::Mutex;

pub struct RfcommBridge {
    decoder: FrameDecoder,
    received_envelopes: VecDeque<Vec<u8>>,
    frames_to_send: VecDeque<Vec<u8>>,
}

#[no_mangle]
pub extern "system" fn Java_com_siar_bluetoothclassic_NativeBluetoothClassicBridge_createBridge<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jlong {
    Box::into_raw(Box::new(Mutex::new(RfcommBridge {
        decoder: FrameDecoder::new(),
        received_envelopes: VecDeque::new(),
        frames_to_send: VecDeque::new(),
    }))) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_siar_bluetoothclassic_NativeBluetoothClassicBridge_destroyBridge<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    if handle == 0 {
        return;
    }
    // SAFETY: reclaims exactly the box `createBridge` allocated for this
    // handle — see this file's top doc comment.
    unsafe {
        drop(Box::from_raw(handle as *mut Mutex<RfcommBridge>));
    }
}

/// # Safety
/// `handle` must be zero, or a value `createBridge` returned that
/// `destroyBridge` hasn't been called on yet — see this file's top doc
/// comment.
unsafe fn bridge_from_handle<'a>(handle: jlong) -> Option<&'a Mutex<RfcommBridge>> {
    if handle == 0 {
        return None;
    }
    Some(unsafe { &*(handle as *const Mutex<RfcommBridge>) })
}

fn bytes_to_jbyte_array(env: &mut JNIEnv, data: &[u8]) -> jbyteArray {
    match env.byte_array_from_slice(data) {
        Ok(array) => array.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Kotlin calls this from its `InputStream.read()` loop with whatever
/// bytes just arrived — may be a partial frame, a whole frame, several
/// frames, or any split thereof (see `framing.rs`'s doc comment). On a
/// framing error (oversized length prefix or checksum mismatch), this
/// crate does nothing further to the stream itself — Kotlin should
/// treat that as "this RFCOMM connection is no longer trustworthy" and
/// close the socket, the same "tear it down, don't paper over it"
/// stance `framing.rs`'s own doc comment describes.
#[no_mangle]
pub extern "system" fn Java_com_siar_bluetoothclassic_NativeBluetoothClassicBridge_onBytesReceived<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    data: JByteArray<'local>,
) {
    // SAFETY: see `bridge_from_handle`.
    let Some(bridge) = (unsafe { bridge_from_handle(handle) }) else { return };
    let Ok(bytes) = env.convert_byte_array(&data) else { return };

    let mut bridge = bridge.lock().expect("RfcommBridge lock poisoned");
    if let Ok(envelopes) = bridge.decoder.push(&bytes) {
        bridge.received_envelopes.extend(envelopes);
    }
    // A framing error is intentionally swallowed here rather than
    // propagated as a Kotlin exception — same "no exception crossing
    // the JNI boundary" discipline this workspace's other bridges
    // follow. Kotlin finds out indirectly: no more envelopes will ever
    // come out of `nextReceivedEnvelope` for this handle, which is
    // itself the "something's wrong with this connection" signal for
    // now (see this file's doc comment above).
}

/// Pull side of `onBytesReceived`: `null` if nothing's ready yet.
#[no_mangle]
pub extern "system" fn Java_com_siar_bluetoothclassic_NativeBluetoothClassicBridge_nextReceivedEnvelope<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jbyteArray {
    // SAFETY: see `bridge_from_handle`.
    let Some(bridge) = (unsafe { bridge_from_handle(handle) }) else { return std::ptr::null_mut() };
    let mut bridge = bridge.lock().expect("RfcommBridge lock poisoned");
    match bridge.received_envelopes.pop_front() {
        Some(data) => bytes_to_jbyte_array(&mut env, &data),
        None => std::ptr::null_mut(),
    }
}

/// Frames `data` (already-encrypted envelope bytes — this crate never
/// looks inside it, same rule every other transport crate in this
/// workspace follows) via `framing::encode_frame` and queues the wire
/// bytes for `nextChunkToSend` to pull.
#[no_mangle]
pub extern "system" fn Java_com_siar_bluetoothclassic_NativeBluetoothClassicBridge_queueEnvelopeToSend<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    data: JByteArray<'local>,
) {
    // SAFETY: see `bridge_from_handle`.
    let Some(bridge) = (unsafe { bridge_from_handle(handle) }) else { return };
    let Ok(bytes) = env.convert_byte_array(&data) else { return };

    let mut bridge = bridge.lock().expect("RfcommBridge lock poisoned");
    bridge.frames_to_send.push_back(encode_frame(&bytes));
}

/// Pull side of `queueEnvelopeToSend`: `null` once there's nothing left
/// to write for now. Kotlin is responsible for the actual
/// `OutputStream.write()` call — unlike BLE's per-fragment writes,
/// Kotlin may write this straight through in one call, since RFCOMM has
/// no MTU to pace against.
#[no_mangle]
pub extern "system" fn Java_com_siar_bluetoothclassic_NativeBluetoothClassicBridge_nextChunkToSend<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jbyteArray {
    // SAFETY: see `bridge_from_handle`.
    let Some(bridge) = (unsafe { bridge_from_handle(handle) }) else { return std::ptr::null_mut() };
    let mut bridge = bridge.lock().expect("RfcommBridge lock poisoned");
    match bridge.frames_to_send.pop_front() {
        Some(data) => bytes_to_jbyte_array(&mut env, &data),
        None => std::ptr::null_mut(),
    }
}
