//! Desktop entry point. Everything that used to be in the single
//! crate's `main.rs` and is genuinely desktop-specific stays here: CLI
//! parsing, the single-instance file lock, and handing off to the
//! renderer. Everything else (business logic, the whole component
//! tree) is `siar_core`/`siar_ui`, shared verbatim with the
//! Android crate.

#![deny(unsafe_code)]

use clap::Parser;
use siar_core::config::{default_data_dir, Config, CONFIG};
use std::path::PathBuf;

/// A peer-to-peer desktop messenger built on iroh. No servers, no
/// passwords — your identity is a 24-word seed phrase, and people find
/// you by a unique username (or a pasted ticket, as a fallback).
#[derive(Parser, Debug, Clone)]
#[command(name = "siar", version)]
struct Cli {
    /// Override the data directory (identity keys, sqlite history, docs/blobs stores).
    /// Defaults to the OS-standard app data dir.
    #[arg(long)]
    data_dir: Option<PathBuf>,

    /// How long to wait for a home relay before giving up and continuing in
    /// degraded mode.
    #[arg(long, default_value = "20")]
    relay_timeout_secs: u64,

    /// Show a desktop notification when a DM, room message, or contact
    /// request arrives.
    #[arg(long, default_value_t = true)]
    notify: bool,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let data_dir = cli.data_dir.unwrap_or_else(default_data_dir);

    // Single-instance guard — a real desktop-app edge case, not a nicety:
    // two instances pointed at the same `data_dir` would both try to bind
    // the same iroh endpoint, open the same sqlite files, and race each
    // other on every write. `_instance_lock` is held for the whole
    // process lifetime via the leaked `Box` below (there's no clean
    // shutdown path in this app currently that would make an explicit
    // `unlock()` meaningful — see the Ctrl+C handler in `siar_ui`,
    // which exits the process directly); the OS releases the lock
    // automatically on process exit or crash either way, so this never
    // leaves a stale lock behind the way a PID file would.
    //
    // Desktop-only, on purpose: there's no equivalent multi-process
    // hazard on Android, where the OS itself enforces one running
    // instance of an app's process per launcher icon.
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        eprintln!("couldn't create data directory {}: {e}", data_dir.display());
        std::process::exit(1);
    }
    let lock_path = data_dir.join(".instance.lock");
    let lock_file = match std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "couldn't open instance lock file {}: {e}",
                lock_path.display()
            );
            std::process::exit(1);
        }
    };
    use fs2::FileExt;
    if lock_file.try_lock_exclusive().is_err() {
        eprintln!(
            "another instance of siar is already running against this data directory \
             ({}). If you're sure that's wrong (e.g. it crashed without releasing the lock — \
             unlikely, since the OS releases it automatically on process exit, but possible if \
             the machine itself lost power), delete {} and try again.",
            data_dir.display(),
            lock_path.display(),
        );
        std::process::exit(1);
    }
    Box::leak(Box::new(lock_file));

    CONFIG
        .set(Config {
            data_dir,
            relay_timeout_secs: cli.relay_timeout_secs,
            notify: cli.notify,
        })
        .ok();

    // Renderer choice is a Cargo feature now, not hardcoded — see
    // Cargo.toml's `[features]` block for why `webview` is the default.
    // `dioxus::prelude::*` (rsx!, signals, hooks — everything siar-ui's
    // component tree uses) is identical either way; only the launch
    // entry point differs, so this is the one call site that needs to
    // branch.
    #[cfg(feature = "native")]
    dioxus_native::launch(siar_ui::AppRoot);

    // Undecorated (`with_decorations(false)`) so `siar_ui::TitleBar`'s
    // custom bar replaces the OS one instead of stacking a second bar
    // underneath it — see that component's doc for the real, currently-
    // open trade-off this comes with (no OS-level edge/corner drag-to-
    // resize; DioxusLabs/dioxus#3128) and the one-line revert if that
    // turns out to matter more than the custom look does.
    // `with_menu(None)` is redundant with `with_decorations(false)`
    // (dioxus-desktop's own docs: the menu bar is hidden automatically
    // once decorations are off) but set explicitly anyway — cheap
    // insurance against that specific behavior changing upstream
    // without this file's owner necessarily noticing.
    #[cfg(not(feature = "native"))]
    {
        use dioxus::desktop::{Config, WindowBuilder};
        let window = WindowBuilder::new()
            .with_title("Siar")
            .with_decorations(false)
            .with_resizable(true)
            .with_min_inner_size(dioxus::desktop::LogicalSize::new(480.0, 360.0));
        dioxus::LaunchBuilder::new()
            .with_cfg(Config::new().with_window(window).with_menu(None))
            .launch(siar_ui::AppRoot);
    }
}
