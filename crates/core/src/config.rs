//! Runtime configuration, read once by the root UI component.
//!
//! `dioxus::launch` (or, on the native/GPU renderer, its equivalent)
//! takes a plain `fn() -> Element` for the common case, so config is
//! parsed by whichever platform crate's `main` runs first (desktop reads
//! CLI args via `clap`; Android has none to read) and stashed here for
//! `siar-ui` to read from.
//!
//! Note there's no `name`/`username` field: display name and username
//! are chosen during onboarding and persisted in `identity`/`store`, not
//! passed in at startup — a seed-phrase identity isn't something you
//! want overridable by a stray shell flag or Android intent extra.

use std::path::PathBuf;
use std::sync::OnceLock;

pub struct Config {
    pub data_dir: PathBuf,
    pub relay_timeout_secs: u64,
    pub notify: bool,
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();

/// OS-standard app data dir, used when a platform crate doesn't override
/// it (desktop: no `--data-dir` flag given; Android: always, there's no
/// equivalent flag to give).
pub fn default_data_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        if let Ok(files) = std::env::var("FILES_DIR") {
            return PathBuf::from(files);
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".siar");
        }
        if let Ok(data) = std::env::var("ANDROID_DATA") {
            let p = PathBuf::from(data).join("data/dev.irshad.siar/files");
            if p.parent().is_some_and(|parent| parent.exists()) {
                return p;
            }
        }
        PathBuf::from("/data/user/0/dev.irshad.siar/files")
    }

    #[cfg(not(target_os = "android"))]
    {
        directories::ProjectDirs::from("dev", "irshad", "siar")
            .map(|p| p.data_dir().to_path_buf())
            .unwrap_or_else(|| {
                directories::BaseDirs::new()
                    .map(|b| b.data_dir().join("siar"))
                    .unwrap_or_else(|| std::env::temp_dir().join("siar"))
            })
    }
}
