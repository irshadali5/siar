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
        android_data_dir_from_environment()
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

/// Resolve Android storage without allowing an environment-variable change
/// between app versions to silently create a second, empty account.
///
/// Android preserves an application's private `files/` directory during a
/// normal signed update. Older Siar builds could nevertheless choose either
/// that directory or `$HOME/.siar`, depending on which variables Wry exposed
/// on that launch. We inspect every historical location and prefer the one
/// containing the richest real account before falling back to the canonical
/// package files directory. This is selection, not migration: existing data
/// is never moved, deleted, or overwritten during startup.
#[cfg(target_os = "android")]
fn android_data_dir_from_environment() -> PathBuf {
    let canonical = PathBuf::from("/data/user/0/dev.irshad.siar/files");
    let mut candidates = vec![
        canonical.clone(),
        PathBuf::from("/data/data/dev.irshad.siar/files"),
    ];
    if let Some(files) = std::env::var_os("FILES_DIR") {
        candidates.push(PathBuf::from(files));
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        candidates.push(home.join(".siar"));
        candidates.push(home.join("files"));
    }
    deduplicate_paths(&mut candidates);
    select_existing_data_dir(&candidates).unwrap_or(canonical)
}

#[cfg(target_os = "android")]
fn deduplicate_paths(paths: &mut Vec<PathBuf>) {
    let mut unique = Vec::with_capacity(paths.len());
    for path in paths.drain(..) {
        if !unique.contains(&path) {
            unique.push(path);
        }
    }
    *paths = unique;
}

/// Pick an existing account rather than merely the first existing directory.
/// `identity.key` is the completion marker for onboarding; database size then
/// favors the location that contains actual history over an accidentally
/// created empty account directory.
#[cfg(any(target_os = "android", test))]
fn select_existing_data_dir(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates
        .iter()
        .filter(|path| path.join("identity.key").is_file())
        .max_by_key(|path| {
            let db_bytes = std::fs::metadata(path.join("messenger.db"))
                .map(|meta| meta.len())
                .unwrap_or(0);
            let has_docs = u64::from(path.join("docs").is_dir());
            let has_blobs = u64::from(path.join("blobs").is_dir());
            (db_bytes, has_docs + has_blobs)
        })
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::select_existing_data_dir;

    #[test]
    fn existing_android_history_wins_over_an_empty_update_directory() {
        let root = std::env::temp_dir().join(format!(
            "siar-config-test-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("android-history")
        ));
        let old = root.join("legacy");
        let accidentally_new = root.join("canonical");
        std::fs::create_dir_all(old.join("docs")).unwrap();
        std::fs::create_dir_all(accidentally_new.join("blobs")).unwrap();
        std::fs::write(old.join("identity.key"), [1_u8; 32]).unwrap();
        std::fs::write(old.join("messenger.db"), [7_u8; 128]).unwrap();
        std::fs::write(accidentally_new.join("identity.key"), [2_u8; 32]).unwrap();
        std::fs::write(accidentally_new.join("messenger.db"), [8_u8; 16]).unwrap();

        let selected = select_existing_data_dir(&[accidentally_new.clone(), old.clone()]);
        assert_eq!(selected.as_deref(), Some(old.as_path()));

        std::fs::remove_dir_all(root).unwrap();
    }
}
