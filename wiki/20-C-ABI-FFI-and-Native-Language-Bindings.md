# 20 — C-ABI FFI & Native Language Bindings

> **Corresponding Specifications:** [`sys-arch/19-c-abi-ffi-architecture.md`](../sys-arch/19-c-abi-ffi-architecture.md), [`sys-arch/27-rust-driven-android-native-build-packaging-automation.md`](../sys-arch/27-rust-driven-android-native-build-packaging-automation.md)  
> **Key Modules:** [`apps/android/rust-jni-glue`](../apps/android/rust-jni-glue), [`apps/android/messaging-jni`](../apps/android/messaging-jni)

---

## 1. Zero-Copy C-ABI Architectural Foundation

To allow integration into Kotlin/Java, Swift/Objective-C, Python, and C/C++ applications without memory leaks or runtime overhead, the SIAR core exposes a pure C-ABI foreign function interface:

```rust
// Opaque handle to the internal Tokio async runtime and state engine
pub struct SiarEngineHandle {
    pub(crate) runtime: tokio::runtime::Runtime,
    pub(crate) inner: Arc<SiarEngine>,
}

#[no_mangle]
pub unsafe extern "C" fn siar_engine_create(
    db_path: *const c_char,
    out_handle: *mut *mut SiarEngineHandle,
) -> i32 {
    std::panic::catch_unwind(|| {
        // Safe string extraction, engine initialization, and raw pointer packaging
    }).unwrap_or(-1)
}

#[no_mangle]
pub unsafe extern "C" fn siar_engine_destroy(handle: *mut SiarEngineHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}
```

---

## 2. Panic Isolation & Memory Safety Boundaries

Crossing the FFI boundary requires strict adherence to safety rules:

```
[Kotlin Android Application (JVM)]
                |
    JNI Call    | (Java Native Interface)
                v
+---------------------------------------------------------------+
|                      JNI C-ABI Layer                          |
|  1. Convert jstring -> CStr -> Rust &str                      |
|  2. std::panic::catch_unwind Barrier (Never unwind past FFI!) |
|  3. Dispatch to internal async Tokio task pool                |
|  4. Convert Rust Result<T, E> -> C-ABI error status code      |
+-------------------------------+-------------------------------+
                                |
                                v
                [Rust Core Engine (siar-messaging)]
```

### Safety Invariants
1. **No Uncaught Panics**: All exported FFI functions wrap logic in `catch_unwind`. A panic in Rust returns an error code rather than crashing the host JVM / iOS process.
2. **Explicit Memory Lifetime**: Any buffer allocated by Rust is freed exclusively via a paired Rust cleanup function.

---

## 3. Multi-Architecture Android Build Automation

The workspace includes automated build scripts to compile and strip native shared libraries (`libsiar_jni.so`) across all standard Android ABIs:

```bash
# Target Android ABIs
aarch64-linux-android    # Modern 64-bit ARM (Primary phones & tablets)
armv7-linux-androideabi  # Legacy 32-bit ARM (Older smartphones)
x86_64-linux-android     # 64-bit Intel/AMD (Android Studio Emulators)
```
