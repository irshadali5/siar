//! CSS for the whole app, split one file per concern instead of the
//! single giant Rust string this used to be (`ui::theme::css`). Visual
//! language is unchanged: the parts of Signal and WhatsApp that are
//! actually about legibility and hierarchy — narrow fixed sidebar of
//! chat rows, rounded asymmetric message bubbles, slim top bar per
//! conversation — rather than copying either brand's exact palette.
//!
//! ## Why `include_str!` + a `<style>` block, not `asset!` + `document::Stylesheet`
//!
//! Dioxus 0.7's own docs recommend `asset!("/assets/foo.css")` +
//! `document::Stylesheet` for CSS — that's the right call on the
//! webview-backed `desktop`/`web` renderers. But this app is moving its
//! render backend to `dioxus-native` (Blitz + WGPU, see the top-level
//! message this shipped with), and as of this pass there's an open,
//! renderer-tagged upstream bug where `asset!` + `document::Stylesheet`
//! does not actually apply the linked CSS under the native renderer
//! (DioxusLabs/dioxus#4666). Verified by search, not guessed — same
//! standard as the iroh discovery-API detour a few rounds back.
//!
//! `style { { ... } }` with the CSS inlined as a string is the path that
//! already worked in the original single-crate `theme::css`, so this
//! split keeps using it: each file below still gets pulled in via
//! `include_str!` at compile time (so it's still a real, separately
//! edited, separately reviewable module — just not a runtime asset
//! link), concatenated once, and injected the same way `AppRoot` always
//! injected it. If a future `dioxus-native` release fixes #4666, this is
//! the one place that would need to change to switch over.

const TOKENS_DARK: &str = include_str!("tokens.dark.css");
const TOKENS_LIGHT: &str = include_str!("tokens.light.css");
const TOKENS_HACKER_GREEN: &str = include_str!("tokens.hacker-green.css");
const TOKENS_HACKER_RED: &str = include_str!("tokens.hacker-red.css");
const BASE: &str = include_str!("base.css");
const TITLEBAR: &str = include_str!("titlebar.css");
const SIDEBAR: &str = include_str!("sidebar.css");
const CHAT: &str = include_str!("chat.css");
const COMPOSER: &str = include_str!("composer.css");
const ONBOARDING: &str = include_str!("onboarding.css");
const SETTINGS: &str = include_str!("settings.css");
const TOAST: &str = include_str!("toast.css");
const RESPONSIVE: &str = include_str!("responsive.css");
const HACKER: &str = include_str!("hacker.css");

/// Full stylesheet, theme-mode-agnostic — see `store::ThemeMode`. The
/// three modes are entirely a CSS-selector concern now, not a
/// content-generation one:
///
/// - `:root` holds the light tokens as the unconditional default.
/// - `@media (prefers-color-scheme: dark)` overrides them with the dark
///   tokens — this is what makes `ThemeMode::System` *live*-sync with
///   the OS: the webview re-evaluates that media query on its own the
///   moment the OS theme changes, no polling or restart needed. That's
///   also why `stylesheet()` no longer takes a `dark: bool` — there's
///   nothing left for Rust to decide by generating different CSS.
/// - `.app-shell[data-theme="light"|"dark"]` overrides both of the above
///   at higher specificity — this is what `ThemeMode::Light`/`Dark`
///   actually do: `AppRoot` sets (or omits, for `System`) that attribute
///   on the root div based on the persisted setting. See that call site
///   in `lib.rs`.
///
/// `store::ThemeStyle` (`Regular`/`HackerGreen`/`HackerRed`) is a second,
/// independent axis layered on top the same way: `.app-shell
/// [data-theme="hacker-green"|"hacker-red"]` are two more values the same
/// `data-theme` attribute can take, at the same specificity as `"light"`/
/// `"dark"` — `AppRoot` picks whichever single value actually applies
/// (see that call site), so there's never a conflict between the two
/// axes at the CSS level, only in which one Rust decided wins. `HACKER`
/// carries the non-color parts of that look (monospace font, glow,
/// squarer bubble corners) that don't fit the flat token-substitution
/// model the light/dark pair uses.
pub fn stylesheet() -> String {
    format!(
        ":root {{\n{TOKENS_LIGHT}}}\n\
         @media (prefers-color-scheme: dark) {{\n  :root {{\n{TOKENS_DARK}}}\n}}\n\
         .app-shell[data-theme=\"light\"] {{\n{TOKENS_LIGHT}}}\n\
         .app-shell[data-theme=\"dark\"] {{\n{TOKENS_DARK}}}\n\
         .app-shell[data-theme=\"hacker-green\"] {{\n{TOKENS_HACKER_GREEN}}}\n\
         .app-shell[data-theme=\"hacker-red\"] {{\n{TOKENS_HACKER_RED}}}\n\
         {BASE}\n{TITLEBAR}\n{SIDEBAR}\n{CHAT}\n{COMPOSER}\n{ONBOARDING}\n{SETTINGS}\n{TOAST}\n{RESPONSIVE}\n{HACKER}"
    )
}
