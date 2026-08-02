//! Persistent node identity.
//!
//! iroh authenticates every connection against an Ed25519 `SecretKey`; the
//! corresponding public key *is* the node's address (`EndpointId`). v2
//! change from the original single-file version: the key is no longer
//! generated randomly and persisted as the root secret — it's *derived*
//! from a 24-word seed phrase (see `seed.rs`), so the same phrase always
//! reproduces the same identity on any device. What's persisted to disk is
//! still just the resulting 32-byte derived key, exactly as before — the
//! mnemonic itself is never written anywhere.

pub mod seed;

use anyhow::{Context, Result};
use hkdf::Hkdf;
use iroh::SecretKey;
use sha2::Sha512;
use std::fs;
use std::path::{Path, PathBuf};

const KEY_FILE: &str = "identity.key";

/// Domain-separation string for `storage_key` below — same HKDF-with-info
/// pattern as `seed::INFO_IDENTITY`/`INFO_DOCS_AUTHOR`, just one more
/// purpose fanned out from a root secret. This one is fanned out from the
/// *persisted identity key* rather than the mnemonic: the mnemonic
/// (`seed::Seed`) only ever exists transiently at onboarding/recovery time
/// (it's never written to disk — see `seed.rs`'s module doc) and is gone
/// by the time `App::start` runs on every later launch, so it isn't
/// available to re-derive a passphrase from at normal boot. The identity
/// key *is* available (it's `identity.key` on disk) on every boot, and
/// it's itself already an HKDF-derived, 256-bit-entropy secret, so
/// deriving one more purpose-specific key from it is just as sound as
/// deriving straight from the mnemonic would be.
const INFO_STORAGE: &[u8] = b"iroh-messenger/sqlcipher-key/v1";

/// Derive the 32-byte key used to encrypt `messenger.db` at rest (see
/// `store::Store::open`). Re-derived from `identity.key` on every launch
/// rather than stored anywhere itself — there is deliberately no
/// `storage.key` file sitting next to the database it unlocks.
pub fn storage_key(data_dir: &Path) -> Result<[u8; 32]> {
    let identity_bytes = read_32(&path_in(data_dir, KEY_FILE)?)?;
    let hk = Hkdf::<Sha512>::new(None, &identity_bytes);
    let mut out = [0u8; 32];
    hk.expand(INFO_STORAGE, &mut out)
        .expect("32 bytes is within HKDF-SHA512's output-length limit");
    Ok(out)
}

/// Returns `<data_dir>/<name>`, creating `data_dir` if needed.
fn path_in(data_dir: &Path, name: &str) -> Result<PathBuf> {
    if !data_dir.exists() {
        fs::create_dir_all(data_dir)
            .with_context(|| format!("creating data dir {}", data_dir.display()))?;
    }
    Ok(data_dir.join(name))
}

/// Whether an identity has already been created/recovered on this machine.
/// Drives the onboarding-vs-main-shell branch in `ui::mod`.
pub fn exists(data_dir: &Path) -> bool {
    data_dir.join(KEY_FILE).exists()
}

/// First-run setup: derive the iroh identity key from a `Seed` (freshly
/// generated or recovered from a typed-in phrase) and persist it.
///
/// Earlier revisions of this module also derived and persisted a second
/// key here (`docs_author.key`, via `Seed::derive_docs_author_key`) meant
/// to seed an `iroh_docs::AuthorId`. That value was never actually read by
/// anything — `App::start` generates/persists its own docs-author identity
/// independently (`net::registry::Registry::new` + the `docs_author_id`
/// row in `store`'s settings table), which is the one that's actually
/// live. Worse than just dead weight: `load` below *required* that second
/// file to exist, so a data dir with `identity.key` but no
/// `docs_author.key` (e.g. one from a build that didn't get around to
/// writing it, or had it deleted) would `exists() == true` and then fail
/// to boot — permanently, with the app never able to construct the file
/// itself since the mnemonic needed to re-derive it is deliberately never
/// persisted (see this module's top doc). Removed rather than "fixed",
/// since there was no real use for it to begin with.
pub fn create_from_seed(data_dir: &Path, seed: &seed::Seed) -> Result<SecretKey> {
    let identity_bytes = seed.derive_identity_key();
    persist(&path_in(data_dir, KEY_FILE)?, &identity_bytes)?;
    Ok(SecretKey::from_bytes(&identity_bytes))
}

/// Every subsequent launch: load the identity key straight from disk. No
/// mnemonic involved — that's the whole point of persisting the derived
/// key instead of asking for the 24 words every time.
pub fn load(data_dir: &Path) -> Result<SecretKey> {
    let identity_bytes = read_32(&path_in(data_dir, KEY_FILE)?)?;
    Ok(SecretKey::from_bytes(&identity_bytes))
}

/// Does re-deriving from `seed` produce the exact identity key already
/// persisted on this device? Read-only — unlike `create_from_seed`,
/// this never writes anything, so it's safe to call speculatively (see
/// `backup::create_backup`, which uses this to refuse backing up a seed
/// phrase that doesn't actually match the identity it would claim to
/// back up — catching a wrong-phrase mistake at backup time, not
/// months later when a restore silently produces a different identity
/// than the one the backup was supposedly of).
pub fn verify_seed_matches_current(data_dir: &Path, seed: &seed::Seed) -> Result<bool> {
    let current = read_32(&path_in(data_dir, KEY_FILE)?)?;
    Ok(seed.derive_identity_key() == current)
}

fn read_32(path: &Path) -> Result<[u8; 32]> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    bytes.try_into().map_err(|_| {
        anyhow::anyhow!(
            "{} is not 32 bytes; delete it to regenerate",
            path.display()
        )
    })
}

fn persist(path: &Path, bytes: &[u8; 32]) -> Result<()> {
    fs::write(path, bytes).with_context(|| format!("writing {}", path.display()))?;

    // Best-effort: lock the key file down on unix so other local users
    // can't read it. Not fatal if it fails (e.g. filesystems without unix
    // perms, which also means Windows — NTFS ACLs default to
    // user-only-readable under %LOCALAPPDATA% already, so this is a
    // unix-specific hardening step, not a cross-platform requirement).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o600);
            let _ = fs::set_permissions(path, perms);
        }
    }

    Ok(())
}
