//! Android entry point. Same shape as `siar-desktop`'s `main.rs` —
//! set up logging, populate `CONFIG`, hand off to the shared
//! `siar_ui::AppRoot` — minus the two things that are genuinely
//! desktop-only (CLI flags: nothing on Android reads argv; the
//! single-instance file lock: Android already enforces one running
//! process per app).
//!
//! Renderer is a Cargo feature (see Cargo.toml), defaulting to
//! `webview`. That default matters here specifically: under `webview`,
//! `dx build --platform android` generates a real `MainActivity.kt`
//! hosting wry's WebView, and that's an established, well-documented
//! bootstrap you can diff `kotlin/`'s two files against directly. The
//! `native` (Blitz/WGPU) feature is still wired up as an opt-in for
//! later, but it's currently blocked upstream (stylo build failure —
//! see `siar-desktop/Cargo.toml`'s `[features]` comment) *and* would
//! reopen a second open question on top of that: the JNI/native-activity
//! bootstrap glue `dx` generates for Android under that renderer isn't
//! the same WebView-hosting path, and isn't something to guess at
//! without a real build to check against — same standard as the title
//! bar/live-theme-sync/context-menu items called out separately.

#![deny(unsafe_code)]

use siar_core::config::{Config, CONFIG};

fn main() {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default().with_max_level(log::LevelFilter::Info),
        );
    }

    let data_dir = siar_core::config::default_data_dir();
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        eprintln!(
            "warning: failed to pre-create data_dir {}: {}",
            data_dir.display(),
            e
        );
    }
    let config = Config {
        data_dir: data_dir.clone(),
        relay_timeout_secs: 20,
        notify: true,
    };

    if CONFIG.set(config).is_err() {
        eprintln!("warning: config already initialized; continuing with existing values");
    }

    #[cfg(feature = "native")]
    dioxus_native::launch(siar_ui::AppRoot);

    #[cfg(not(feature = "native"))]
    dioxus::launch(siar_ui::AppRoot);
}
