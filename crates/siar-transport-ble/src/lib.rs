//! BLE transport, pure-logic half — next.md §22–26, §72, §100. Phase 3
//! of `next.md`'s roadmap ("BLE discovery, BLE direct small-message
//! channel").
//!
//! - [`fragment`]: splitting an encrypted envelope into
//!   [`fragment::BleFragment`]s that fit one GATT write, and the wire
//!   format for one.
//! - [`reassembly`]: bounded reassembly of fragments back into an
//!   envelope, on the receiving side of one connection.
//! - [`discovery`]: the BLE advertisement payload's wire format
//!   ([`discovery::DiscoveryBeacon`]).
//!
//! What's NOT in this crate, matching this workspace's established
//! split between a pure-logic crate and its Android JNI boundary
//! (`siar-media-core`/`siar-media-android`, `siar-calls`/
//! `siar-calls::android`): anything that touches
//! `BluetoothLeScanner`/`BluetoothGattServer`/GATT characteristics
//! themselves. That's next — an Android-only crate (not yet built) that
//! hands raw bytes across the same risk-inversion JNI boundary the
//! media and Wi-Fi Direct crates already use (Kotlin drives the
//! Android Bluetooth APIs directly; Rust only receives calls), feeding
//! this crate's [`fragment::BleFragment::decode`]/
//! [`reassembly::ReassemblyBuffer`] on the way in and
//! [`fragment::fragment_envelope`] on the way out. Scoped out of this
//! pass the same way `siar-calls`'s Android bridge was scoped out until
//! asked for by name, not an oversight.
//!
//! next.md §26's Bluetooth Mesh warning is why none of this reaches for
//! the Bluetooth SIG Mesh standard — this is a from-scratch application
//! overlay mesh instead, per §27, built from these fragments plus a
//! later DTN layer (Phase 4), not from BLE Mesh's lighting/sensor-
//! oriented provisioning model.

pub mod discovery;
pub mod fragment;
pub mod reassembly;
