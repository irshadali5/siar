//! Encrypted backup/restore: the recovery seed phrase, the local
//! message database, and media blobs, packed into one file and
//! encrypted with a user-chosen backup passphrase.
//!
//! # Why a *separate* passphrase, not the identity/seed itself
//!
//! Using the seed phrase (or a key derived from it) to encrypt its own
//! backup would be circular: the whole point of a backup is to survive
//! losing access to the thing it protects. A human-memorable backup
//! passphrase — distinct from the 24-word phrase, chosen fresh for this
//! purpose — is what actually makes a backup file safe to put somewhere
//! less trusted than this device (a cloud-synced folder, a USB drive) or
//! to hand to a "keep this somewhere for me" trusted contact.
//!
//! # Format
//!
//! ```text
//! MAGIC (8 bytes: "SIARBKP1")
//! SALT (16 bytes, for Argon2id)
//! NONCE (24 bytes, for XChaCha20-Poly1305)
//! CIPHERTEXT (the rest — an AEAD-sealed, postcard-encoded `BackupPayload`)
//! ```
//!
//! The version is baked into the magic string itself (`SIARBKP1`, not a
//! separate version byte) — same reasoning this codebase already applies
//! to ALPN strings for wire protocols: a format change becomes a new
//! magic string, so an old and new build fail to recognize each other's
//! files cleanly rather than one silently misparsing the other's bytes.
//!
//! Argon2id (memory-hard, resists GPU/ASIC brute-force in a way plain
//! HKDF — used everywhere else in this codebase, for keys derived from
//! an already-high-entropy secret — doesn't need to) stretches the
//! passphrase into a key; XChaCha20-Poly1305 (RustCrypto, audited,
//! extended 192-bit nonce so a randomly generated one is safe to use
//! without a counter) seals everything else.
//!
//! # "Online drive"
//!
//! This module only writes a `Vec<u8>` (`create_backup`) and reads one
//! back (`restore_backup`) — it has no idea where that file ends up.
//! `siar-ui`'s backup flow saves it via the same local file-save dialog
//! (`rfd`) every other local-file feature in this codebase already
//! uses, which means "save to an online drive" in practice means
//! pointing that dialog at a Dropbox/Google Drive/iCloud Drive *sync
//! folder* already on disk — not a real OAuth integration with any
//! specific provider's upload API. That's a deliberate scope decision,
//! not an oversight: a real Google Drive/Dropbox API integration is a
//! credentials-and-token-management undertaking in its own right, and
//! "one encrypted file, save it anywhere including a folder your cloud
//! provider happens to sync" already covers the actual use case (get an
//! encrypted backup off this one device) without that scope. Real apps
//! (Signal Desktop among them) ship exactly this pattern for exactly
//! this reason.
//!
//! # A known real limitation
//!
//! `create_backup`/`restore_backup` hold the entire database and every
//! media blob in memory at once (as one `Vec<u8>` each) — fine for a
//! typical chat history, but a account with a large media library could
//! mean a multi-gigabyte in-memory buffer. Streaming encryption
//! (encrypt-as-you-write directly to the output file/archive) would
//! avoid that at the cost of real additional complexity — not done in
//! this pass; flagged here rather than silently limiting how large a
//! backup can practically be.

use crate::identity::{self, seed::Seed};
use anyhow::{bail, Context, Result};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 8] = b"SIARBKP1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
/// Argon2id output length — 32 bytes, exactly what
/// `XChaCha20Poly1305`'s key needs, no truncation/padding to reason
/// about.
const KEY_LEN: usize = 32;

#[derive(Serialize, Deserialize)]
struct BackupPayload {
    seed_phrase: String,
    /// `messenger.db` (already SQLCipher-encrypted at rest — see
    /// `store.rs` — with a key deterministically derived from the
    /// identity key, which `restore_backup` re-derives from
    /// `seed_phrase` via the exact same path normal seed recovery
    /// already uses. Bundling the raw file works precisely because that
    /// key is reproducible, not because this module does anything
    /// special with it) plus its WAL/SHM sidecar files if SQLite has one
    /// open — skipping those would silently lose whatever was written
    /// since the last checkpoint.
    db_files: Vec<(String, Vec<u8>)>,
    /// Every file under the blob store directory, path relative to that
    /// directory's root (so restore doesn't need to know or care about
    /// the absolute path a *different* device's data dir happens to
    /// use).
    blob_files: Vec<(String, Vec<u8>)>,
}

/// Builds one encrypted backup file's bytes. `seed_phrase` is checked
/// against this device's actual current identity before anything else
/// happens (see `identity::verify_seed_matches_current`'s doc) — this
/// catches a wrong/mistyped phrase at backup time, which is the only
/// time it's actually catchable; a backup of the wrong phrase would
/// otherwise look completely fine until someone tried to restore from
/// it and got a different identity than they expected.
pub fn create_backup(
    data_dir: &Path,
    seed_phrase: &str,
    backup_passphrase: &str,
) -> Result<Vec<u8>> {
    if backup_passphrase.len() < 8 {
        bail!("backup passphrase should be at least 8 characters — this is the only thing standing between the backup file and whoever finds it");
    }
    let seed = Seed::from_phrase(seed_phrase)
        .context("that doesn't look like a valid 24-word recovery phrase")?;
    if !identity::verify_seed_matches_current(data_dir, &seed)? {
        bail!("that recovery phrase doesn't match this device's current identity — double check the words");
    }

    let mut db_files = Vec::new();
    for name in ["messenger.db", "messenger.db-wal", "messenger.db-shm"] {
        let path = data_dir.join(name);
        if path.is_file() {
            db_files.push((
                name.to_string(),
                std::fs::read(&path).with_context(|| format!("reading {name}"))?,
            ));
        }
    }
    if db_files.is_empty() {
        bail!("no messenger.db found in this device's data directory — nothing to back up yet");
    }

    let mut blob_files = Vec::new();
    let blobs_dir = data_dir.join("blobs");
    if blobs_dir.is_dir() {
        collect_files(&blobs_dir, &blobs_dir, &mut blob_files)?;
    }

    let payload = BackupPayload {
        seed_phrase: seed_phrase.to_string(),
        db_files,
        blob_files,
    };
    let plaintext = postcard::to_stdvec(&payload).context("serializing backup payload")?;

    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let key_bytes = derive_key(backup_passphrase, &salt)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_bytes));

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|_| anyhow::anyhow!("encryption failed"))?;

    let mut out = Vec::with_capacity(MAGIC.len() + SALT_LEN + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypts a backup file and restores it into `data_dir` — the
/// identity (re-derived from the recovered seed phrase via the same
/// `identity::create_from_seed` path normal onboarding-by-seed already
/// uses, not a separate implementation), `messenger.db` (+ any WAL/SHM
/// sidecar files that were present at backup time), and every blob
/// file. Returns the recovered seed phrase so the caller (`siar-ui`'s
/// restore flow) can show it to the person once, the same way fresh
/// onboarding does, rather than leaving them with no record of it
/// having come from a backup at all.
///
/// **Overwrites whatever is currently in `data_dir`.** Callers must
/// confirm with the person before calling this — it doesn't ask itself,
/// since a library function isn't the place to own a confirmation
/// dialog.
pub fn restore_backup(
    data_dir: &Path,
    backup_bytes: &[u8],
    backup_passphrase: &str,
) -> Result<String> {
    let min_len = MAGIC.len() + SALT_LEN + NONCE_LEN;
    if backup_bytes.len() < min_len {
        bail!("not a valid Siar backup file (too short)");
    }
    let (magic, rest) = backup_bytes.split_at(MAGIC.len());
    if magic != MAGIC {
        bail!("not a Siar backup file");
    }
    let (salt, rest) = rest.split_at(SALT_LEN);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);

    let key_bytes = derive_key(backup_passphrase, salt)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let nonce = XNonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("wrong backup passphrase, or this file is corrupted"))?;

    let payload: BackupPayload =
        postcard::from_bytes(&plaintext).context("backup file's contents are corrupted")?;

    let seed = Seed::from_phrase(&payload.seed_phrase)
        .context("backup's recovery phrase is invalid — this backup file may be corrupted")?;
    identity::create_from_seed(data_dir, &seed).context("restoring identity from backup")?;

    for (name, bytes) in &payload.db_files {
        std::fs::write(data_dir.join(name), bytes).with_context(|| format!("writing {name}"))?;
    }

    let blobs_dir = data_dir.join("blobs");
    for (rel_path, content) in &payload.blob_files {
        let full = safe_join(&blobs_dir, rel_path)?;
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(&full, content).with_context(|| format!("writing {}", full.display()))?;
    }

    Ok(payload.seed_phrase)
}

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; KEY_LEN]> {
    let mut out = [0u8; KEY_LEN];
    argon2::Argon2::default()
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|e| anyhow::anyhow!("key derivation failed: {e}"))?;
    Ok(out)
}

fn collect_files(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry
            .with_context(|| format!("reading an entry of {}", dir.display()))?
            .path();
        if path.is_dir() {
            collect_files(base, &path, out)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let content =
                std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
            out.push((rel, content));
        }
    }
    Ok(())
}

/// Joins `rel` onto `base` while refusing anything that would land
/// outside `base` (a `..` component, or an absolute path baked into
/// `rel`) — a backup file is untrusted input by the time it reaches
/// `restore_backup` (it could be a corrupted, tampered-with, or
/// maliciously hand-crafted file someone was handed and told to
/// restore), and without this check a crafted `rel_path` could write
/// files anywhere on disk this process has permission to write to, not
/// just inside the blob store directory.
fn safe_join(base: &Path, rel: &str) -> Result<PathBuf> {
    let rel_path = Path::new(rel);
    if rel_path.is_absolute()
        || rel_path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        bail!("backup file contains an unsafe path ({rel}) — refusing to restore it");
    }
    Ok(base.join(rel_path))
}
