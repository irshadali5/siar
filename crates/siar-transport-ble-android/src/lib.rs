//! The Android GATT JNI boundary for `siar-transport-ble` — next.md
//! §22–26. Closes the gap that crate's own `lib.rs` doc comment flagged:
//! "an Android-only crate (not yet built) that hands raw bytes across
//! the same risk-inversion JNI boundary the media and Wi-Fi Direct
//! crates already use."
//!
//! See `jni_bridge.rs`'s doc comment for the full design — per-
//! connection `BleLinkBridge` handles (not a single app-wide singleton,
//! unlike `siar-transport-wifi-direct`'s one-radio model), because BLE
//! genuinely supports several simultaneous peer connections.
//!
//! What this crate still does NOT do, same honesty as every other JNI
//! crate in this workspace: the actual `BluetoothLeScanner`/
//! `BluetoothGattServer` Kotlin side (`BleGattManager.kt`) isn't written
//! here — Kotlin isn't Rust, so it's not this crate's artifact to
//! produce, only its JNI contract to satisfy. And per next.md §25,
//! there's no ACK/retry/timeout layer above raw fragment delivery yet —
//! a dropped fragment on a lossy BLE link currently just means that
//! transfer never completes and eventually falls out of the
//! `ReassemblyBuffer`'s bounded capacity, not a retransmit request.

#[cfg(target_os = "android")]
mod jni_bridge;
