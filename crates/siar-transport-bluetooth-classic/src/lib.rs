//! Bluetooth Classic RFCOMM transport — next.md §21.
//!
//! `framing.rs` is pure, cross-platform logic (real unit tests, no
//! Bluetooth API involved) — see that module's doc comment for why
//! RFCOMM needs stream framing at all, unlike BLE's already-bounded
//! GATT writes.
//!
//! `jni_bridge.rs` is the Android JNI boundary, same risk-inversion
//! pattern and same push/pull queue shape as
//! `siar-transport-ble-android`'s `jni_bridge.rs` (mirrored
//! deliberately, method names included, so anyone who's read one has
//! already read the other): Kotlin's `BluetoothClassicManager.kt`
//! drives `BluetoothSocket`/`BluetoothServerSocket` directly — ordinary
//! Kotlin against the Android SDK — and only ever calls *into* this
//! crate, never the reverse.
//!
//! Per-connection handle model (not a single app-wide singleton, same
//! as BLE and unlike Wi-Fi Direct's one-radio model): next.md §7 and
//! §21 both treat Bluetooth Classic as one RFCOMM socket per peer
//! relationship, and a device may be mid-conversation with more than
//! one nearby peer over Bluetooth Classic at once.
//!
//! What this crate deliberately does NOT do, same honesty as its
//! siblings: retry/ACK above frame-level corruption detection (a
//! checksum failure tears the connection down — see `framing.rs` — but
//! nothing here decides whether the peer relationship as a whole should
//! be retried; that's `siar-dtn`'s job, one layer up), and it does not
//! decide *when* to prefer Bluetooth Classic over BLE or Wi-Fi Direct —
//! next.md §10/§20's transport-scoring routing policy owns that
//! decision, this crate only carries bytes once told to.

pub mod framing;

#[cfg(target_os = "android")]
mod jni_bridge;
