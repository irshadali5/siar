//! First-run flow: create a brand new identity (show the 24 words once,
//! require an explicit "I've written it down" confirmation) or recover an
//! existing one (paste the 24 words back in), then claim a username
//! against the registry before entering the main app.
//!
//! This component owns its own local signals and doesn't touch `app::App`
//! at all until the very last step (`on_ready`) — identity creation and
//! username claiming both need the derived keys and a running `App`
//! instance, which `ui::mod::AppRoot` constructs once this component
//! reports success.

use dioxus::prelude::*;
use siar_core::identity::seed::Seed;

#[derive(Clone, Copy, PartialEq)]
enum Step {
    Choose,
    ShowSeed,
    RecoverSeed,
    RestoreBackup,
    ClaimUsername,
}

pub struct OnboardingResult {
    pub seed: Seed,
    pub username: String,
    pub display_name: String,
}

#[component]
pub fn Onboarding(
    /// Called once the user has a seed (fresh or recovered) and a chosen
    /// username; `on_check_username` and `on_claim` let the parent do the
    /// actual registry I/O (this component just drives the flow).
    on_check_username: EventHandler<String>,
    on_ready: EventHandler<OnboardingResult>,
    /// Set by the parent after `on_check_username` resolves: `Some(true)`
    /// available, `Some(false)` taken, `None` not yet checked / checking.
    username_available: Signal<Option<bool>>,
) -> Element {
    let mut step = use_signal(|| Step::Choose);
    let mut seed = use_signal(|| None::<std::sync::Arc<Seed>>);
    let mut seed_confirmed = use_signal(|| false);
    let mut recover_input = use_signal(String::new);
    let mut recover_error = use_signal(|| None::<String>);
    let restore_file = use_signal(|| None::<Vec<u8>>);
    let mut restore_passphrase = use_signal(String::new);
    let mut restore_error = use_signal(|| None::<String>);
    let mut restoring = use_signal(|| false);
    let mut username = use_signal(String::new);
    let mut display_name = use_signal(String::new);

    rsx! {
        div { class: "onboarding-shell",
            div { class: "onboarding-card",
                match step() {
                    Step::Choose => rsx! {
                        img {
                            src: "data:image/png;base64,{crate::icon_b64::ICON_B64}",
                            style: "width: 128px; height: 128px; margin: 0 auto; display: block;"
                        }
                        h1 { "Welcome" }
                        p { style: "color: var(--text-muted)",
                            "No accounts, no servers — your identity is a 24-word phrase only you hold."
                        }
                        div { style: "display:flex; gap:10px; margin-top:18px; flex-wrap:wrap;",
                            button {
                                onclick: move |_| {
                                    seed.set(Some(std::sync::Arc::new(
                                        Seed::generate().expect("CSPRNG available"),
                                    )));
                                    seed_confirmed.set(false);
                                    step.set(Step::ShowSeed);
                                },
                                "Create new identity"
                            }
                            button { class: "secondary",
                                onclick: move |_| step.set(Step::RecoverSeed),
                                "I have a recovery phrase"
                            }
                            // Same `rfd` mobile limitation as every other local-file
                            // feature in this codebase (status image/audio attach,
                            // avatar change) — see those call sites' comments.
                            // Restore-from-backup needs a file picker either way, so
                            // it's desktop-only until that gap closes. `#[cfg(...)]`
                            // directly on an rsx element isn't valid syntax — `rsx!`
                            // has its own macro grammar, not plain Rust item syntax —
                            // so this needs `cfg!()` (a compile-time bool literal) used
                            // as an ordinary `if` condition instead, which `rsx!`
                            // does support natively.
                            if cfg!(not(any(target_os = "android", target_os = "ios"))) {
                                button { class: "secondary",
                                    onclick: move |_| step.set(Step::RestoreBackup),
                                    "Restore from encrypted backup"
                                }
                            }
                        }
                    },
                    Step::ShowSeed => {
                        let words: Vec<String> = seed()
                            .as_ref()
                            .map(|s| s.phrase().split(' ').map(str::to_string).collect())
                            .unwrap_or_default();
                        rsx! {
                            h1 { "Your recovery phrase" }
                            p { style: "color: var(--text-muted)",
                                "Write these 24 words down, in order, somewhere safe and offline. "
                                "Anyone with this phrase can access your identity. It is never stored or sent anywhere by this app."
                            }
                            div { class: "seed-grid",
                                for (i, w) in words.iter().enumerate() {
                                    div { class: "seed-word", key: "{i}", span { "{i + 1}." } "{w}" }
                                }
                            }
                            label { style: "display:flex; align-items:center; gap:8px; margin: 10px 0;",
                                input {
                                    r#type: "checkbox",
                                    checked: seed_confirmed(),
                                    onchange: move |e| seed_confirmed.set(e.checked()),
                                }
                                "I've written down my 24 words"
                            }
                            button {
                                disabled: !seed_confirmed(),
                                onclick: move |_| step.set(Step::ClaimUsername),
                                "Continue"
                            }
                        }
                    },
                    Step::RecoverSeed => rsx! {
                        h1 { "Recover your identity" }
                        p { style: "color: var(--text-muted)", "Enter your 24-word phrase, space-separated, in order." }
                        textarea {
                            rows: "4",
                            value: "{recover_input}",
                            oninput: move |e| recover_input.set(e.value()),
                        }
                        if let Some(err) = recover_error() {
                            p { style: "color: var(--danger); font-size: 12px;", "{err}" }
                        }
                        div { style: "display:flex; gap:10px; margin-top:12px;",
                            button { class: "secondary", onclick: move |_| step.set(Step::Choose), "Back" }
                            button {
                                onclick: move |_| {
                                    match Seed::from_phrase(&recover_input.read()) {
                                        Ok(s) => {
                                            seed.set(Some(std::sync::Arc::new(s)));
                                            recover_error.set(None);
                                            step.set(Step::ClaimUsername);
                                        }
                                        Err(e) => recover_error.set(Some(e.to_string())),
                                    }
                                },
                                "Recover"
                            }
                        }
                    },
                    Step::RestoreBackup => rsx! {
                        h1 { "Restore from backup" }
                        p { style: "color: var(--text-muted)",
                            "Pick a Siar backup file and enter the backup passphrase you chose when you "
                            "created it — not your 24-word recovery phrase. This restores your identity, "
                            "messages, and files from that backup onto this device."
                        }
                        div { style: "display:flex; gap:10px; align-items:center; margin: 12px 0;",
                            button {
                                class: "secondary",
                                onclick: move |_| {
                                    #[cfg(not(any(target_os = "android", target_os = "ios")))]
                                    spawn(async move {
                                        let Some(handle) = rfd::AsyncFileDialog::new()
                                            .add_filter("Siar backup", &["siarbackup"])
                                            .pick_file()
                                            .await
                                        else {
                                            return;
                                        };
                                        restore_file.clone().set(Some(handle.read().await));
                                        restore_error.set(None);
                                    });
                                    // Unreachable via the UI on mobile already (see the
                                    // `cfg!()` guard around the button that navigates to
                                    // this step) — this branch exists so the closure still
                                    // has something to do on a target where the branch
                                    // above isn't compiled in, rather than leaving it empty
                                    // and implicit.
                                    #[cfg(any(target_os = "android", target_os = "ios"))]
                                    restore_error.set(Some("Not available on mobile yet".to_string()));
                                },
                                "Choose backup file"
                            }
                            if restore_file.read().is_some() {
                                span { style: "font-size:12px; color:var(--text-muted);", "File selected ✓" }
                            }
                        }
                        input {
                            r#type: "password",
                            class: "onboarding-input",
                            placeholder: "Backup passphrase",
                            value: "{restore_passphrase}",
                            oninput: move |e| restore_passphrase.set(e.value()),
                        }
                        if let Some(err) = restore_error() {
                            p { style: "color: var(--danger); font-size: 12px; margin-top:8px;", "{err}" }
                        }
                        div { style: "display:flex; gap:10px; margin-top:12px;",
                            button { class: "secondary", onclick: move |_| step.set(Step::Choose), "Back" }
                            button {
                                disabled: restore_file.read().is_none() || restoring(),
                                onclick: move |_| {
                                    let Some(bytes) = restore_file.cloned() else { return };
                                    let passphrase = restore_passphrase.cloned();
                                    restoring.set(true);
                                    spawn(async move {
                                        let data_dir = siar_core::CONFIG.get().unwrap().data_dir.clone();
                                        // Argon2id is deliberately slow (that's the whole
                                        // point — see backup.rs's doc) — spawn_blocking so
                                        // it doesn't stall this component's own render loop
                                        // while it runs.
                                        let result = tokio::task::spawn_blocking(move || {
                                            siar_core::backup::restore_backup(&data_dir, &bytes, &passphrase)
                                        })
                                        .await;
                                        restoring.set(false);
                                        match result {
                                            Ok(Ok(phrase)) => match Seed::from_phrase(&phrase) {
                                                Ok(s) => {
                                                    seed.set(Some(std::sync::Arc::new(s)));
                                                    restore_error.set(None);
                                                    step.set(Step::ClaimUsername);
                                                }
                                                Err(e) => restore_error.set(Some(format!(
                                                    "restored, but the recovered phrase didn't parse: {e}"
                                                ))),
                                            },
                                            Ok(Err(e)) => restore_error.set(Some(e.to_string())),
                                            Err(e) => restore_error.set(Some(format!("restore task panicked: {e}"))),
                                        }
                                    });
                                },
                                if restoring() { "Restoring…" } else { "Restore" }
                            }
                        }
                    },
                    Step::ClaimUsername => {
                        // Derived locally from the seed's identity key —
                        // same derivation `identity::create_from_seed` uses,
                        // just without touching disk or the network yet
                        // (see that function and `Seed::derive_identity_key`).
                        // Shown so there's an instant, always-works way to
                        // connect to a friend from minute one: a pasted
                        // ticket connects directly by address, while a
                        // username only becomes findable once its claim has
                        // had a chance to propagate to the person searching
                        // for it (see `net::registry`'s doc comment).
                        let my_ticket = seed().as_ref().and_then(|s| {
                            let secret = iroh::SecretKey::from_bytes(&s.derive_identity_key());
                            // No relay/direct addresses yet — the endpoint
                            // hasn't bound (that's the whole point of
                            // computing this locally, pre-network). It'll
                            // still work, just via discovery like the old
                            // ID-only format did — the richer, instant-
                            // connect version of this ticket lives in
                            // Settings once the app's fully started.
                            siar_core::ticket::encode(iroh::EndpointAddr::from(secret.public())).ok()
                        });
                        rsx! {
                        h1 { "Pick your username" }
                        p { style: "color: var(--text-muted)", "This is how people find and message you." }
                        if let Some(ticket) = my_ticket.clone() {
                            div {
                                style: "background: var(--bg-secondary, rgba(255,255,255,0.04)); \
                                        border: 1px dashed var(--border); border-radius: 8px; \
                                        padding: 10px 12px; margin-bottom: 14px; opacity: 0.85;",
                                p { style: "margin:0 0 6px 0; font-size:12px; color:var(--text-muted);",
                                    strong { style: "color:var(--text);", "Preview ticket — not the reliable one yet. " }
                                    "This is computed from your identity alone, before this device has ever come online, "
                                    "so it carries no address — the other side's app has to fall back entirely on public "
                                    "discovery, which hasn't heard of this brand-new identity yet either. It may still "
                                    "connect, eventually, but don't rely on it. "
                                    "Once setup finishes, open Settings → Profile and copy the ticket shown there instead: "
                                    "the real one, generated from your endpoint's actual live address, that reliably connects."
                                }
                                div { style: "display:flex; gap:8px; align-items:center;",
                                    code { style: "flex:1; font-size:11px; word-break:break-all; opacity:0.7;", "{ticket}" }
                                    button {
                                        class: "secondary",
                                        onclick: move |_| crate::copy_to_clipboard(ticket.clone()),
                                        "Copy anyway"
                                    }
                                }
                            }
                        }
                        label { style: "font-size:12px; color:var(--text-muted); display:block; margin-bottom:4px;", "Display name" }
                        input {
                            class: "onboarding-input",
                            placeholder: "e.g. Ali",
                            value: "{display_name}",
                            oninput: move |e| display_name.set(e.value()),
                        }
                        div { style: "height:10px;" }
                        label { style: "font-size:12px; color:var(--text-muted); display:block; margin-bottom:4px;", "Username" }
                        input {
                            class: "onboarding-input",
                            placeholder: "e.g. ali",
                            value: "{username}",
                            oninput: move |e| {
                                let v = e.value();
                                username.set(v.clone());
                                username_available.set(None);
                                if !v.trim().is_empty() {
                                    on_check_username.call(v);
                                }
                            },
                        }
                        match username_available() {
                            Some(true) => rsx! {
                                div {
                                    p { class: "username-status ok", "✓ available" }
                                    p { style: "font-size:11px; color:var(--text-muted); margin-top:2px;",
                                        "Based on what's synced so far — on a brand-new install this can be "
                                        "nearly nothing yet, so this isn't a hard guarantee someone else "
                                        "hasn't already claimed it elsewhere. If a message from them later "
                                        "shows a different name for the same identity, that's why."
                                    }
                                }
                            },
                            Some(false) => rsx! { p { class: "username-status taken", "✗ already taken — try another" } },
                            None => rsx! { p { class: "username-status", "" } },
                        }
                        div { style: "display:flex; gap:10px; margin-top:12px;",
                            button { class: "secondary", onclick: move |_| step.set(Step::Choose), "Back" }
                            button {
                                disabled: username_available() != Some(true) || display_name.read().trim().is_empty(),
                                onclick: move |_| {
                                    if let Some(s) = seed() {
                                        // `Seed` isn't `Clone` (holds sensitive material) — since
                                        // we only need to move it once, out of the `Arc`, this
                                        // relies on being the sole owner at this point. Calling
                                        // the signal (`seed()`) clones the `Arc` out (bumping the
                                        // strong count to 2: one here, one still held inside the
                                        // signal itself), so `try_unwrap` would otherwise always
                                        // fail — silently, with this whole block just doing
                                        // nothing on every click. Dropping the signal's own copy
                                        // first makes `s` the sole owner again.
                                        seed.set(None);
                                        if let Ok(s) = std::sync::Arc::try_unwrap(s) {
                                            on_ready.call(OnboardingResult {
                                                seed: s,
                                                username: username.read().trim().to_lowercase(),
                                                display_name: display_name.read().trim().to_string(),
                                            });
                                        }
                                    }
                                },
                                "Start messaging"
                            }
                        }
                    }},
                }
            }
        }
    }
}
