# Compilation and Build Status Report

## 1. Cargo (`cargo check`, `cargo test --no-run`)
- **Status:** **SUCCESS (0 Errors)**
- **Exit Code:** `0`
- **Compiler Warnings (Host Targets):**
  - `apps/android/messaging-jni/src/lib.rs`: Unused items (`AppMessaging`, `RUNTIME`, `APP`, `runtime`, `bootstrap_inner`, `my_ticket_inner`, `add_peer_inner`, `send_text_inner`, `check_mailbox_inner`, `send_text_anon_inner`, `check_mailbox_anon_inner`, `poll_next_event_inner`). *(Part of dead code warning when compiled outside of active Android JNI caller).*
  - `apps/desktop/src/main.rs:122`: `key_package_directory` field in `Bootstrapped` never read.
  - `apps/desktop/src/app.rs:110`: `device_id` field in `LocalIdentity` never read.

---

## 2. Cargo-NDK (`cargo ndk -t <targets>`)
- **Targets Verified:** `arm64-v8a`, `armeabi-v7a`, `x86_64`, `x86` (API Level 26)
- **Crates Verified:**
  - `apps/android/rust-jni-glue` (`siar-android-connectivity`)
  - `apps/android/messaging-jni` (`siar-android-messaging`)
  - `crates/siar-media-android`
  - `crates/siar-transport-ble-android`
  - `crates/siar-transport-wifi-aware`
  - `crates/siar-transport-bluetooth-classic`
  - `crates/siar-transport-wifi-direct`
- **Status:** **SUCCESS (0 Errors)**
- **Exit Code:** `0`
- **Warnings:**
  - Unused `mut` warnings on `JNIEnv` parameters in:
    - `crates/siar-media-android/src/jni_bridge.rs:288, 313, 369`
    - `crates/siar-transport-ble-android/src/jni_bridge.rs:112, 152`
    - `crates/siar-transport-bluetooth-classic/src/jni_bridge.rs:90, 134`

---

## 3. Gradle (`compileDebugSources`, `assembleDebug`)
- **Directory:** `apps/android`
- **Gradle Version:** `8.9` (Android Gradle Plugin `8.7.2`, Kotlin `2.0.21`)
- **Status:** **SUCCESS (0 Errors)**
- **Exit Code:** `0`
- **Deprecation Warnings:**
  - `apps/android/app/src/main/kotlin/com/siar/ble/BleGattManager.kt:228, 229`: `BluetoothGattCharacteristic.value` and `BluetoothGatt.writeCharacteristic(BluetoothGattCharacteristic)` deprecated in Android SDK.
  - `apps/android/app/src/main/kotlin/com/siar/wifiaware/WifiAwareManagerBridge.kt:115`: `WifiAwareNetworkSpecifier.Builder.createNetworkSpecifierOpen(PeerHandle)` deprecated.
  - `apps/android/app/src/main/kotlin/com/siar/wifidirect/WifiDirectManager.kt:51, 54`: `Intent.getParcelableExtra` and `NetworkInfo.isConnected` deprecated.

---

## 4. Clippy (`cargo clippy --workspace --all-targets`)
- **Status:** **FAILED (3 Errors)**
- **Exit Code:** `101`
- **Errors:**
  1. `crates/siar-storage/src/message_repo.rs:147`:
     ```text
     error: this loop never actually loops
     --> crates/siar-storage/src/message_repo.rs:147:9
     = note: `#[deny(clippy::never_loop)]` on by default
     help: if you need the first element of the iterator, try writing:
           if let Some(row) = rows.next() { ... }
     ```
  2. `crates/siar-storage/src/blob_repo.rs:73`:
     ```text
     error: this loop never actually loops
     --> crates/siar-storage/src/blob_repo.rs:73:9
     = note: `#[deny(clippy::never_loop)]` on by default
     help: if you need the first element of the iterator, try writing:
           if let Some(row) = rows.next() { ... }
     ```
  3. `crates/siar-storage/src/group_repo.rs:107`:
     ```text
     error: this loop never actually loops
     --> crates/siar-storage/src/group_repo.rs:107:13
     = note: `#[deny(clippy::never_loop)]` on by default
     help: if you need the first element of the iterator, try writing:
           if let Some(row) = rows.next() { ... }
     ```
- **Additional Clippy Warnings:**
  - `crates/siar-crypto/src/identity.rs:31`: `clippy::needless_borrows_for_generic_args` (`&mut OsRng` -> `OsRng`)
  - `crates/siar-media-audio/src/opus_codec.rs:72`: `clippy::manual_is_multiple_of`
  - `crates/siar-transport-ble/src/fragment.rs:24`: `clippy::doc_lazy_continuation`
  - `crates/siar-media-image/src/thumbnail.rs:124`: `clippy::assertions_on_constants`
  - `crates/siar-messaging/src/ticket.rs:50`: `clippy::manual_is_multiple_of`
  - `crates/siar-messaging/src/group_service.rs:181`: `clippy::items_after_test_module`
  - `apps/emergency-node/src/main.rs:524, 637`: `clippy::clone_on_copy`
  - `apps/desktop/src/app.rs:127`: `clippy::too_many_arguments`
