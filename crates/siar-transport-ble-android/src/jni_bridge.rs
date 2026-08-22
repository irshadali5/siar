//! The JNI boundary for BLE GATT — next.md §22–26's Android radio
//! access, the gap `siar-transport-ble`'s (Phase 3) `lib.rs` doc
//! comment flagged as "not yet built."
//!
//! Same risk-inversion pattern as `siar-media-android` and
//! `siar-transport-wifi-direct`: Kotlin's `BleGattManager.kt` drives
//! `BluetoothLeScanner`/`BluetoothGattServer`/`BluetoothGatt` directly —
//! ordinary, statically-typed Kotlin against the Android SDK,
//! compiler-checked on that side. This crate is only ever *called into*
//! from Kotlin, never the reverse.
//!
//! Different shape from both sibling crates, deliberately: a phone can
//! hold several simultaneous BLE connections (multiple peers in range),
//! unlike Wi-Fi Direct's one-radio-one-group model — so `handle` here is
//! **per connection**, not a single app-wide singleton. Kotlin creates
//! one `BleLinkBridge` per `BluetoothGatt`/per connected central, the
//! same way `siar-media-android` creates one `MediaSession` per call.
//!
//! This crate never decodes an *envelope*, only *fragments* — it
//! delegates all of the actual framing/reassembly logic to
//! `siar-transport-ble`'s already-tested pure functions
//! (`BleFragment::decode`, `ReassemblyBuffer::insert`,
//! `fragment_envelope`). Its own job is exactly the JNI marshaling and
//! the two queues (received envelopes, fragments-to-send) Kotlin pulls
//! from — same "hold queues, Kotlin pulls" shape `siar-media-android`'s
//! `pop_input`/`nextRawFrame` pair established.
//!
//! JNI naming: package `com.siar.ble`, class `NativeBleBridge` → symbol
//! prefix `Java_com_siar_ble_NativeBleBridge_`.

use std::collections::VecDeque;
use std::sync::Mutex;

use jni::objects::{JByteArray, JClass};
use jni::sys::{jbyte, jbyteArray, jint, jlong};
use jni::JNIEnv;
use siar_transport_ble::fragment::{fragment_envelope, BleFragment};
use siar_transport_ble::reassembly::{ReassemblyBuffer, ReassemblyOutcome};

pub struct BleLinkBridge {
    reassembly: ReassemblyBuffer,
    /// Complete, reassembled envelope bytes, ready for the app layer to
    /// decode as a `siar_protocol::WireMessage` (almost certainly the
    /// `Mesh` variant — see that type's own doc comment — since a
    /// direct `V1` session over BLE would be unusual, though nothing
    /// here enforces that).
    received_envelopes: VecDeque<Vec<u8>>,
    /// Encoded `BleFragment` bytes Kotlin should write to the peer's
    /// GATT characteristic, in order.
    fragments_to_send: VecDeque<Vec<u8>>,
    next_transfer_id: u32,
}

#[no_mangle]
pub extern "system" fn Java_com_siar_ble_NativeBleBridge_createBridge<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    reassembly_capacity: jint,
) -> jlong {
    let capacity = if reassembly_capacity > 0 { reassembly_capacity as usize } else { 8 };
    let bridge = BleLinkBridge {
        reassembly: ReassemblyBuffer::new(capacity),
        received_envelopes: VecDeque::new(),
        fragments_to_send: VecDeque::new(),
        next_transfer_id: 0,
    };
    Box::into_raw(Box::new(Mutex::new(bridge))) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_siar_ble_NativeBleBridge_destroyBridge<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    if handle == 0 {
        return;
    }
    // SAFETY: reclaims exactly the box `createBridge` allocated for
    // this handle — same one-alloc/one-free contract as
    // `siar-media-android`'s `MediaSession`.
    unsafe {
        drop(Box::from_raw(handle as *mut Mutex<BleLinkBridge>));
    }
}

/// # Safety
/// `handle` must be zero, or a value `createBridge` returned that
/// `destroyBridge` hasn't been called on yet.
unsafe fn bridge_from_handle<'a>(handle: jlong) -> Option<&'a Mutex<BleLinkBridge>> {
    if handle == 0 {
        return None;
    }
    Some(unsafe { &*(handle as *const Mutex<BleLinkBridge>) })
}

fn bytes_to_jbyte_array(env: &mut JNIEnv, data: &[u8]) -> jbyteArray {
    match env.byte_array_from_slice(data) {
        Ok(array) => array.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Kotlin calls this from a GATT characteristic write/notify callback
/// with whatever bytes just arrived over the air. Decodes one
/// `BleFragment` and feeds it to the reassembly buffer — a malformed
/// fragment (bad checksum, too short) is dropped silently, same as a
/// dropped radio packet would be; there's no ACK/retry machinery below
/// this layer yet (next.md §25 flags that as real remaining work).
#[no_mangle]
pub extern "system" fn Java_com_siar_ble_NativeBleBridge_onFragmentReceived<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    data: JByteArray<'local>,
) {
    // SAFETY: see `bridge_from_handle`.
    let Some(bridge) = (unsafe { bridge_from_handle(handle) }) else { return };
    let Ok(bytes) = env.convert_byte_array(&data) else { return };
    let Ok(fragment) = BleFragment::decode(&bytes) else { return };

    let mut bridge = bridge.lock().expect("BleLinkBridge lock poisoned");
    if let Ok(ReassemblyOutcome::Complete(envelope)) = bridge.reassembly.insert(fragment) {
        bridge.received_envelopes.push_back(envelope);
    }
}

/// Pull side of `onFragmentReceived`: `null` if nothing's ready yet.
#[no_mangle]
pub extern "system" fn Java_com_siar_ble_NativeBleBridge_nextReceivedEnvelope<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jbyteArray {
    // SAFETY: see `bridge_from_handle`.
    let Some(bridge) = (unsafe { bridge_from_handle(handle) }) else { return std::ptr::null_mut() };
    let mut bridge = bridge.lock().expect("BleLinkBridge lock poisoned");
    match bridge.received_envelopes.pop_front() {
        Some(data) => bytes_to_jbyte_array(&mut env, &data),
        None => std::ptr::null_mut(),
    }
}

/// Fragments `data` (already-encrypted envelope bytes — this crate
/// never looks inside it, same rule `siar-protocol`'s wire types
/// already follow) via `siar_transport_ble::fragment::fragment_envelope`
/// and queues the encoded fragments for `nextFragmentToSend` to pull.
/// `protocol` is `BleFragment::protocol` — an app-defined tag, not
/// interpreted by this crate.
#[no_mangle]
pub extern "system" fn Java_com_siar_ble_NativeBleBridge_queueEnvelopeToSend<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    data: JByteArray<'local>,
    protocol: jbyte,
    max_fragment_bytes: jint,
) {
    // SAFETY: see `bridge_from_handle`.
    let Some(bridge) = (unsafe { bridge_from_handle(handle) }) else { return };
    let Ok(bytes) = env.convert_byte_array(&data) else { return };
    if max_fragment_bytes <= 0 {
        return;
    }

    let mut bridge = bridge.lock().expect("BleLinkBridge lock poisoned");
    let transfer_id = bridge.next_transfer_id;
    bridge.next_transfer_id = bridge.next_transfer_id.wrapping_add(1);

    let Ok(fragments) = fragment_envelope(protocol as u8, transfer_id, &bytes, max_fragment_bytes as usize) else {
        return;
    };
    for fragment in fragments {
        bridge.fragments_to_send.push_back(fragment.encode());
    }
}

/// Pull side of `queueEnvelopeToSend`: `null` once there's nothing left
/// to write for now. Kotlin is responsible for the actual
/// `BluetoothGatt.writeCharacteristic` call and for pacing writes to
/// however many the peer's negotiated MTU/queue depth allows — this
/// queue has no concept of "in flight," only "not yet handed to
/// Kotlin."
#[no_mangle]
pub extern "system" fn Java_com_siar_ble_NativeBleBridge_nextFragmentToSend<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jbyteArray {
    // SAFETY: see `bridge_from_handle`.
    let Some(bridge) = (unsafe { bridge_from_handle(handle) }) else { return std::ptr::null_mut() };
    let mut bridge = bridge.lock().expect("BleLinkBridge lock poisoned");
    match bridge.fragments_to_send.pop_front() {
        Some(data) => bytes_to_jbyte_array(&mut env, &data),
        None => std::ptr::null_mut(),
    }
}
